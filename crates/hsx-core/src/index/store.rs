//! SQLite metadata store for the local index (Phase 5, PRD §28).
//!
//! Stores document metadata alongside the HNSW vector index.
//! The SQLite DB is the source of truth for document content;
//! the HNSW index is a search accelerator built on top.

use crate::error::HsxError;
use crate::index::document::{IndexedDocument, IndexStats};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use tracing::debug;

/// SQLite-backed document metadata store.
pub struct DocumentStore {
    conn: Mutex<Connection>,
}

impl DocumentStore {
    /// Open or create the store at `db_path`.
    pub fn new(db_path: &Path) -> Result<Self, HsxError> {
        if let Some(p) = db_path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS documents (
                 id           INTEGER PRIMARY KEY AUTOINCREMENT,
                 url          TEXT    NOT NULL UNIQUE,
                 title        TEXT    NOT NULL DEFAULT '',
                 content      TEXT    NOT NULL DEFAULT '',
                 domain       TEXT    NOT NULL DEFAULT '',
                 fetched_at   TEXT    NOT NULL,
                 content_hash TEXT    NOT NULL,
                 has_embedding INTEGER NOT NULL DEFAULT 0
             );
             CREATE INDEX IF NOT EXISTS idx_docs_domain
                 ON documents(domain);
             CREATE INDEX IF NOT EXISTS idx_docs_hash
                 ON documents(content_hash);",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Insert or update a document. Returns `(id, is_new_or_changed)`.
    pub fn upsert(&self, doc: &IndexedDocument) -> Result<(u64, bool), HsxError> {
        let hash = &doc.content_hash;
        let conn = self.conn.lock().unwrap();

        // Check existing content hash
        let existing_hash: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM documents WHERE url = ?1",
                [&doc.url],
                |r| r.get(0),
            )
            .optional()?;

        let changed = existing_hash.as_deref() != Some(hash.as_str());

        conn.execute(
            "INSERT INTO documents (url, title, content, domain, fetched_at, content_hash, has_embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(url) DO UPDATE SET
                 title        = excluded.title,
                 content      = excluded.content,
                 domain       = excluded.domain,
                 fetched_at   = excluded.fetched_at,
                 content_hash = excluded.content_hash,
                 has_embedding = CASE
                     WHEN excluded.content_hash != documents.content_hash THEN 0
                     ELSE documents.has_embedding
                 END",
            rusqlite::params![
                doc.url,
                doc.title,
                doc.content,
                doc.domain,
                doc.fetched_at.to_rfc3339(),
                hash,
                doc.embedding.is_some() as i64,
            ],
        )?;

        let id: i64 =
            conn.query_row("SELECT id FROM documents WHERE url = ?1", [&doc.url], |r| {
                r.get(0)
            })?;

        debug!("Document upserted: id={id}, url={}, changed={changed}", doc.url);
        Ok((id as u64, changed))
    }

    /// Load a document by ID.
    pub fn get_by_id(&self, id: u64) -> Result<Option<IndexedDocument>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT id, url, title, content, domain, fetched_at, content_hash
                 FROM documents WHERE id = ?1",
                [id],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()?;

        Ok(result.map(|(id, url, title, content, domain, fetched_at, content_hash)| {
            IndexedDocument {
                id: id as u64,
                url,
                title,
                content,
                domain,
                fetched_at: fetched_at
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
                content_hash,
                embedding: None,
            }
        }))
    }

    /// Load multiple documents by their IDs.
    pub fn get_by_ids(&self, ids: &[u64]) -> Result<Vec<IndexedDocument>, HsxError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut docs = Vec::with_capacity(ids.len());
        for &id in ids {
            if let Some(doc) = self.get_by_id(id)? {
                docs.push(doc);
            }
        }
        Ok(docs)
    }

    /// Return all documents that don't yet have embeddings.
    pub fn documents_without_embeddings(&self) -> Result<Vec<IndexedDocument>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, title, content, domain, fetched_at, content_hash
             FROM documents WHERE has_embedding = 0",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })?;

        let mut docs = Vec::new();
        for row in rows {
            let (id, url, title, content, domain, fetched_at, content_hash) = row?;
            docs.push(IndexedDocument {
                id: id as u64,
                url,
                title,
                content,
                domain,
                fetched_at: fetched_at.parse().unwrap_or_else(|_| Utc::now()),
                content_hash,
                embedding: None,
            });
        }
        Ok(docs)
    }

    /// Mark a document as having an embedding.
    pub fn mark_embedded(&self, id: u64) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE documents SET has_embedding = 1 WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    /// Return index statistics.
    pub fn stats(&self) -> Result<IndexStats, HsxError> {
        let conn = self.conn.lock().unwrap();
        let doc_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))?;
        let embedded_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE has_embedding = 1",
            [],
            |r| r.get(0),
        )?;
        Ok(IndexStats {
            document_count: doc_count as usize,
            embedded_count: embedded_count as usize,
            index_size_bytes: 0,       // caller fills this in from disk stat
            vector_index_ready: false, // caller fills this in
        })
    }

    /// Full-text search using SQL LIKE against title and content.
    ///
    /// Returns up to `limit` documents ordered by insertion time (newest first).
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<IndexedDocument>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query.replace('%', r"\%").replace('_', r"\_"));
        let mut stmt = conn.prepare(
            "SELECT id, url, title, content, domain, fetched_at, content_hash
             FROM documents
             WHERE title LIKE ?1 ESCAPE '\\' OR content LIKE ?1 ESCAPE '\\'
             ORDER BY id DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern, limit as i64], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })?;

        let mut docs = Vec::new();
        for row in rows {
            let (id, url, title, content, domain, fetched_at, content_hash) = row?;
            docs.push(IndexedDocument {
                id: id as u64,
                url,
                title,
                content,
                domain,
                fetched_at: fetched_at.parse().unwrap_or_else(|_| Utc::now()),
                content_hash,
                embedding: None,
            });
        }
        Ok(docs)
    }

    /// Delete all documents from the store. Returns the number of rows removed.
    pub fn clear(&self) -> Result<usize, HsxError> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute("DELETE FROM documents", [])?;
        debug!("DocumentStore cleared: {n} documents removed");
        Ok(n)
    }

    /// Compute SHA-256 hex digest of content.
    pub fn content_hash(content: &str) -> String {
        format!("{:x}", Sha256::digest(content.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_store() -> (DocumentStore, NamedTempFile) {
        let tmp = NamedTempFile::new().unwrap();
        let store = DocumentStore::new(tmp.path()).unwrap();
        (store, tmp)
    }

    fn make_doc(n: u64) -> IndexedDocument {
        IndexedDocument {
            id: n,
            url: format!("https://example.com/{n}"),
            title: format!("Doc {n}"),
            content: format!("Content for document {n}"),
            domain: "example.com".into(),
            fetched_at: Utc::now(),
            content_hash: format!("hash{n}"),
            embedding: None,
        }
    }

    #[test]
    fn upsert_and_get() {
        let (store, _tmp) = make_store();
        let doc = make_doc(1);
        let (id, new) = store.upsert(&doc).unwrap();
        assert!(new);
        let loaded = store.get_by_id(id).unwrap().unwrap();
        assert_eq!(loaded.url, "https://example.com/1");
    }

    #[test]
    fn upsert_same_hash_not_changed() {
        let (store, _tmp) = make_store();
        let doc = make_doc(2);
        store.upsert(&doc).unwrap();
        let (_, changed) = store.upsert(&doc).unwrap();
        // Same hash → not changed
        assert!(!changed);
    }

    #[test]
    fn stats_count() {
        let (store, _tmp) = make_store();
        store.upsert(&make_doc(1)).unwrap();
        store.upsert(&make_doc(2)).unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.document_count, 2);
        assert_eq!(stats.embedded_count, 0);
    }

    #[test]
    fn mark_embedded_updates_flag() {
        let (store, _tmp) = make_store();
        let (id, _) = store.upsert(&make_doc(3)).unwrap();
        store.mark_embedded(id).unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.embedded_count, 1);
    }

    #[test]
    fn content_hash_deterministic() {
        let h1 = DocumentStore::content_hash("hello");
        let h2 = DocumentStore::content_hash("hello");
        assert_eq!(h1, h2);
        assert_ne!(h1, DocumentStore::content_hash("world"));
    }
}
