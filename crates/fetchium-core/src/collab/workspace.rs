//! Workspace CRUD (PRD §37).

use crate::error::FetchiumError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub members: Vec<String>,
    pub sync_method: SyncMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncMethod {
    Local { shared_dir: std::path::PathBuf },
    Git { remote_url: String },
}

impl Workspace {
    /// Create a new workspace directory at `path`.
    pub fn create(name: &str, path: &std::path::Path) -> Result<Self, FetchiumError> {
        let ws = Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            members: vec![],
            sync_method: SyncMethod::Local {
                shared_dir: path.to_path_buf(),
            },
        };
        std::fs::create_dir_all(path.join("sessions"))?;
        std::fs::create_dir_all(path.join("knowledge_graph"))?;
        std::fs::create_dir_all(path.join("evidence"))?;
        let manifest = serde_json::to_string_pretty(&ws)?;
        std::fs::write(path.join("workspace.json"), manifest)?;
        tracing::info!(name, "Workspace created at {:?}", path);
        Ok(ws)
    }

    /// Load a workspace from a directory.
    pub fn load(path: &std::path::Path) -> Result<Self, FetchiumError> {
        let content = std::fs::read_to_string(path.join("workspace.json"))?;
        Ok(serde_json::from_str(&content)?)
    }

    /// List all workspaces in `base_dir`.
    pub fn list(base_dir: &std::path::Path) -> Result<Vec<Self>, FetchiumError> {
        if !base_dir.exists() {
            return Ok(vec![]);
        }
        let mut workspaces = Vec::new();
        for entry in std::fs::read_dir(base_dir)? {
            let path = entry?.path();
            if path.is_dir() && path.join("workspace.json").exists() {
                if let Ok(ws) = Self::load(&path) {
                    workspaces.push(ws);
                }
            }
        }
        workspaces.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(workspaces)
    }

    /// Default workspace base directory: `~/.fetchium/workspaces/`.
    pub fn default_base_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".fetchium")
            .join("workspaces")
    }

    /// Path to a specific workspace by name.
    pub fn path_for(name: &str) -> std::path::PathBuf {
        Self::default_base_dir().join(name)
    }
}
