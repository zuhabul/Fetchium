//! Personal Knowledge Graph (PKG) — stores entities and relationships across sessions.
//!
//! Schema:
//! - `entities(id, name, type, source_url, frequency, first_seen, last_seen)`
//! - `relationships(entity_a, entity_b, relation, weight, first_seen, last_seen)`

use rusqlite::Connection;
use std::sync::Mutex;

use crate::error::HsxError;
use crate::intelligence::enable_wal;

/// A node in the personal knowledge graph.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Entity {
    pub id: u64,
    pub name: String,
    pub entity_type: String,
    pub frequency: u64,
    pub first_seen: String,
    pub last_seen: String,
}

/// A directed edge in the personal knowledge graph.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Relationship {
    pub entity_a: String,
    pub entity_b: String,
    pub relation: String,
    pub weight: f64,
}

pub struct PersonalKnowledgeGraph {
    conn: Mutex<Connection>,
}

impl PersonalKnowledgeGraph {
    pub fn new(db_path: &std::path::Path) -> Result<Self, HsxError> {
        let conn = Connection::open(db_path)?;
        enable_wal(&conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entities (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT    NOT NULL UNIQUE,
                type        TEXT    NOT NULL DEFAULT 'concept',
                source_url  TEXT,
                frequency   INTEGER NOT NULL DEFAULT 1,
                first_seen  TEXT    NOT NULL DEFAULT (datetime('now')),
                last_seen   TEXT    NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS relationships (
                entity_a   TEXT NOT NULL,
                entity_b   TEXT NOT NULL,
                relation   TEXT NOT NULL,
                weight     REAL NOT NULL DEFAULT 1.0,
                first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen  TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (entity_a, entity_b, relation)
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Add or increment an entity. Returns entity ID.
    pub fn add_entity(
        &self,
        name: &str,
        entity_type: &str,
        source_url: &str,
    ) -> Result<u64, HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entities (name, type, source_url)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET
                frequency  = frequency + 1,
                last_seen  = datetime('now')",
            rusqlite::params![name, entity_type, source_url],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM entities WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?;
        Ok(id as u64)
    }

    /// Add or update a weighted relationship between two entities.
    pub fn add_relationship(
        &self,
        entity_a: &str,
        entity_b: &str,
        relation: &str,
        weight: f64,
    ) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO relationships (entity_a, entity_b, relation, weight)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(entity_a, entity_b, relation) DO UPDATE SET
                weight    = (weight + ?4) / 2.0,
                last_seen = datetime('now')",
            rusqlite::params![entity_a, entity_b, relation, weight],
        )?;
        Ok(())
    }

    /// Find entities related to `entity`, sorted by weight descending.
    pub fn related_entities(
        &self,
        entity: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, f64)>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT entity_b, relation, weight FROM relationships WHERE entity_a = ?1
             UNION
             SELECT entity_a, relation, weight FROM relationships WHERE entity_b = ?1
             ORDER BY weight DESC
             LIMIT ?2",
        )?;
        let results = stmt
            .query_map(rusqlite::params![entity, limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    /// Top entities by frequency.
    pub fn top_entities(&self, limit: usize) -> Result<Vec<Entity>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, type, frequency, first_seen, last_seen
             FROM entities
             ORDER BY frequency DESC
             LIMIT ?1",
        )?;
        let results = stmt
            .query_map([limit as i64], |row| {
                Ok(Entity {
                    id: row.get::<_, i64>(0)? as u64,
                    name: row.get(1)?,
                    entity_type: row.get(2)?,
                    frequency: row.get::<_, i64>(3)? as u64,
                    first_seen: row.get(4)?,
                    last_seen: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    /// Total entity count.
    pub fn entity_count(&self) -> Result<u64, HsxError> {
        let conn = self.conn.lock().unwrap();
        let n: i64 =
            conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    /// Total relationship count.
    pub fn relationship_count(&self) -> Result<u64, HsxError> {
        let conn = self.conn.lock().unwrap();
        let n: i64 =
            conn.query_row("SELECT COUNT(*) FROM relationships", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    /// Reset all knowledge graph data.
    pub fn reset(&self) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM entities;
             DELETE FROM relationships;
             VACUUM;",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn entity_frequency_increments_on_repeat() {
        let tmp = NamedTempFile::new().unwrap();
        let pkg = PersonalKnowledgeGraph::new(tmp.path()).unwrap();
        pkg.add_entity("Rust", "language", "https://rust-lang.org").unwrap();
        pkg.add_entity("Rust", "language", "https://doc.rust-lang.org").unwrap();
        let entities = pkg.top_entities(5).unwrap();
        assert_eq!(entities[0].name, "Rust");
        assert_eq!(entities[0].frequency, 2);
    }

    #[test]
    fn relationships_are_bidirectional_queryable() {
        let tmp = NamedTempFile::new().unwrap();
        let pkg = PersonalKnowledgeGraph::new(tmp.path()).unwrap();
        pkg.add_relationship("Rust", "WebAssembly", "compiles_to", 0.8).unwrap();
        let related = pkg.related_entities("WebAssembly", 10).unwrap();
        assert!(related.iter().any(|(e, _, _)| e == "Rust"));
    }
}
