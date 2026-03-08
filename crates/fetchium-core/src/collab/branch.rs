//! Research session branching (PRD §37).

use crate::error::FetchiumError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub forked_from: Option<String>,
    pub query: String,
}

impl SessionMeta {
    pub fn new(name: &str, query: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            forked_from: None,
            query: query.to_string(),
        }
    }

    pub fn save(&self, sessions_dir: &std::path::Path) -> Result<(), FetchiumError> {
        let dir = sessions_dir.join(&self.id);
        std::fs::create_dir_all(&dir)?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join("session.json"), content)?;
        Ok(())
    }

    pub fn load(sessions_dir: &std::path::Path, id: &str) -> Result<Self, FetchiumError> {
        let content = std::fs::read_to_string(sessions_dir.join(id).join("session.json"))?;
        Ok(serde_json::from_str(&content)?)
    }
}

/// Fork an existing session to create a new branch.
///
/// Performs a deep copy of `session_id` directory, then updates metadata.
pub fn fork_session(
    sessions_dir: &std::path::Path,
    session_id: &str,
    new_name: &str,
) -> Result<SessionMeta, FetchiumError> {
    let source = sessions_dir.join(session_id);
    if !source.exists() {
        return Err(FetchiumError::Config(format!(
            "Session '{session_id}' not found"
        )));
    }

    let mut meta = SessionMeta::load(sessions_dir, session_id)?;
    let new_id = uuid::Uuid::new_v4().to_string();
    let dest = sessions_dir.join(&new_id);

    copy_dir(&source, &dest)?;

    meta.id = new_id;
    meta.name = new_name.to_string();
    meta.forked_from = Some(session_id.to_string());
    meta.created_at = chrono::Utc::now().to_rfc3339();
    meta.save(sessions_dir)?;

    tracing::info!(name = new_name, from = session_id, "Session forked");
    Ok(meta)
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> Result<(), FetchiumError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
