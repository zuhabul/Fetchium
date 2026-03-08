use crate::types::{JobState, JobStatusResponse};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use tracing::error;

/// Persistent job store backed by SQLite.
pub struct JobStore {
    conn: Mutex<Connection>,
}

impl JobStore {
    /// Open or create the jobs database
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                owner_key_id TEXT NOT NULL,
                payload TEXT NOT NULL,
                webhook_url TEXT
            )",
            [],
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// In-memory database for tests or defaults
    pub fn new_in_memory() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                owner_key_id TEXT NOT NULL,
                payload TEXT NOT NULL,
                webhook_url TEXT
            )",
            [],
        )
        .unwrap();
        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn create(&self, owner_key_id: String, job_id: String, job_type: String, webhook_url: Option<String>) {
        let payload = JobStatusResponse {
            meta: crate::types::ResponseMeta {
                request_id: job_id.clone(),
                status: "queued".into(),
                endpoint: "/v1/jobs/:id".into(),
                duration_ms: 0,
                query: None,
                tier: None,
                tokens_used: None,
                sources_count: None,
                result_id: Some(job_id.clone()),
                intent: None,
                credits_used: None,
            },
            job_id: job_id.clone(),
            job_type,
            status: JobState::Queued,
            created_at: chrono::Utc::now().to_rfc3339(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        };

        if let Ok(json) = serde_json::to_string(&payload) {
            let conn = self.conn.lock();
            let _ = conn.execute(
                "INSERT INTO jobs (id, owner_key_id, payload, webhook_url) VALUES (?1, ?2, ?3, ?4)",
                params![job_id, owner_key_id, json, webhook_url],
            );
        }
    }

    pub fn mark_running(&self, job_id: &str) {
        let mut hook = None;
        let mut payload = None;
        self.update_job(job_id, &mut hook, &mut payload, |job| {
            job.status = JobState::Running;
            job.meta.status = "running".into();
            job.started_at = Some(chrono::Utc::now().to_rfc3339());
        });
    }

    pub fn complete(&self, job_id: &str, result: serde_json::Value) {
        let mut webhook_url: Option<String> = None;
        let mut final_payload: Option<JobStatusResponse> = None;
        self.update_job(job_id, &mut webhook_url, &mut final_payload, |job| {
            job.status = JobState::Completed;
            job.meta.status = "completed".into();
            job.completed_at = Some(chrono::Utc::now().to_rfc3339());
            job.result = Some(result);
            job.error = None;
        });
        Self::fire_webhook(webhook_url, final_payload);
    }

    pub fn fail(&self, job_id: &str, error: String) {
        let mut webhook_url: Option<String> = None;
        let mut final_payload: Option<JobStatusResponse> = None;
        self.update_job(job_id, &mut webhook_url, &mut final_payload, |job| {
            job.status = JobState::Failed;
            job.meta.status = "failed".into();
            job.completed_at = Some(chrono::Utc::now().to_rfc3339());
            job.error = Some(error);
            job.result = None;
        });
        Self::fire_webhook(webhook_url, final_payload);
    }

    pub fn get_owned(&self, job_id: &str, owner_key_id: &str) -> Option<JobStatusResponse> {
        let conn = self.conn.lock();
        let result: Result<Option<String>, rusqlite::Error> = conn
            .query_row(
                "SELECT payload FROM jobs WHERE id = ?1 AND owner_key_id = ?2",
                params![job_id, owner_key_id],
                |row| row.get(0),
            )
            .optional();

        if let Ok(Some(payload_json)) = result {
            if let Ok(payload) = serde_json::from_str::<JobStatusResponse>(&payload_json) {
                return Some(payload);
            }
        }
        None
    }

    fn update_job<F>(&self, job_id: &str, webhook_out: &mut Option<String>, payload_out: &mut Option<JobStatusResponse>, f: F)
    where
        F: FnOnce(&mut JobStatusResponse),
    {
        let conn = self.conn.lock();
        let result: Result<Option<(String, Option<String>)>, rusqlite::Error> = conn
            .query_row(
                "SELECT payload, webhook_url FROM jobs WHERE id = ?1",
                params![job_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional();

        if let Ok(Some((payload_json, hook))) = result {
            *webhook_out = hook;
            if let Ok(mut payload) = serde_json::from_str::<JobStatusResponse>(&payload_json) {
                f(&mut payload);
                *payload_out = Some(payload.clone());
                if let Ok(new_json) = serde_json::to_string(&payload) {
                    let _ = conn.execute(
                        "UPDATE jobs SET payload = ?1 WHERE id = ?2",
                        params![new_json, job_id],
                    );
                }
            }
        }
    }

    fn fire_webhook(webhook_url: Option<String>, payload: Option<JobStatusResponse>) {
        if let (Some(url), Some(data)) = (webhook_url, payload) {
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let _ = client.post(&url).json(&data).send().await;
            });
        }
    }
}
