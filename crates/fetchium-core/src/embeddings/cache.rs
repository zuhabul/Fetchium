//! SQLite-backed embedding cache (Phase 5, PRD §21).
//!
//! Avoids re-computing embeddings for frequently queried texts.
//! Embeddings are stored as raw little-endian f32 BLOB (384 × 4 = 1536 bytes each).

use crate::error::FetchiumError;
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use tracing::debug;

/// SQLite-backed embedding cache.
pub struct EmbeddingCache {
    conn: Mutex<Connection>,
}

impl EmbeddingCache {
    /// Open (or create) the cache at `db_path`.
    pub fn new(db_path: &Path) -> Result<Self, FetchiumError> {
        let conn = Connection::open(db_path)?;
        // WAL mode for concurrent access
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS embedding_cache (
                 text_hash TEXT PRIMARY KEY,
                 embedding  BLOB    NOT NULL,
                 created_at INTEGER DEFAULT (strftime('%s', 'now'))
             );
             CREATE INDEX IF NOT EXISTS idx_cache_created
                 ON embedding_cache(created_at);",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// SHA-256 hex digest of `text`.
    fn hash_text(text: &str) -> String {
        format!("{:x}", Sha256::digest(text.as_bytes()))
    }

    /// Look up a cached embedding. Returns `None` on cache miss.
    pub fn get(&self, text: &str) -> Result<Option<Vec<f32>>, FetchiumError> {
        let hash = Self::hash_text(text);
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare_cached("SELECT embedding FROM embedding_cache WHERE text_hash = ?1")?;
        let blob: Option<Vec<u8>> = stmt.query_row([&hash], |row| row.get(0)).optional()?;

        match blob {
            Some(b) => {
                let floats: Vec<f32> = b
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                    .collect();
                debug!("Embedding cache hit for hash {}", &hash[..8]);
                Ok(Some(floats))
            }
            None => Ok(None),
        }
    }

    /// Store an embedding in the cache.
    pub fn put(&self, text: &str, embedding: &[f32]) -> Result<(), FetchiumError> {
        let hash = Self::hash_text(text);
        let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO embedding_cache (text_hash, embedding) VALUES (?1, ?2)",
            rusqlite::params![hash, blob],
        )?;
        Ok(())
    }

    /// Delete cache entries older than `max_age_secs` seconds.
    pub fn evict_older_than(&self, max_age_secs: u64) -> Result<usize, FetchiumError> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM embedding_cache \
             WHERE created_at < strftime('%s', 'now') - ?1",
            [max_age_secs],
        )?;
        debug!("Evicted {} stale embedding cache entries", deleted);
        Ok(deleted)
    }

    /// Number of cached embeddings.
    pub fn len(&self) -> Result<usize, FetchiumError> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM embedding_cache", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    /// Returns `true` if the cache contains no entries.
    pub fn is_empty(&self) -> Result<bool, FetchiumError> {
        Ok(self.len()? == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_cache() -> (EmbeddingCache, NamedTempFile) {
        let tmp = NamedTempFile::new().unwrap();
        let cache = EmbeddingCache::new(tmp.path()).unwrap();
        (cache, tmp)
    }

    #[test]
    fn cache_miss_returns_none() {
        let (cache, _tmp) = make_cache();
        assert!(cache.get("missing text").unwrap().is_none());
    }

    #[test]
    fn put_and_get_roundtrip() {
        let (cache, _tmp) = make_cache();
        let emb: Vec<f32> = (0..crate::embeddings::EMBEDDING_DIM)
            .map(|i| i as f32 / crate::embeddings::EMBEDDING_DIM as f32)
            .collect();
        cache.put("hello world", &emb).unwrap();

        let retrieved = cache.get("hello world").unwrap().unwrap();
        assert_eq!(retrieved.len(), crate::embeddings::EMBEDDING_DIM);
        for (a, b) in emb.iter().zip(retrieved.iter()) {
            assert!((a - b).abs() < 1e-7, "mismatch at a={a}, b={b}");
        }
    }

    #[test]
    fn len_counts_entries() {
        let (cache, _tmp) = make_cache();
        assert_eq!(cache.len().unwrap(), 0);
        let emb = vec![0.0_f32; crate::embeddings::EMBEDDING_DIM];
        cache.put("text1", &emb).unwrap();
        cache.put("text2", &emb).unwrap();
        assert_eq!(cache.len().unwrap(), 2);
    }

    #[test]
    fn evict_removes_old_entries() {
        let (cache, _tmp) = make_cache();
        let emb = vec![0.1_f32; crate::embeddings::EMBEDDING_DIM];
        cache.put("old entry", &emb).unwrap();
        // Wait 1s so that SQLite `strftime('%s', 'now')` is strictly greater
        std::thread::sleep(std::time::Duration::from_secs(1));
        let deleted = cache.evict_older_than(0).unwrap();
        assert!(deleted >= 1);
    }
}
