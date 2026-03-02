//! SearXNG Docker setup utilities.
//!
//! Powers `hsx setup --searxng` — generates an optimised `settings.yml`,
//! pulls the official SearXNG Docker image, and starts the container on a
//! configurable port. Idempotent: re-running never destroys existing data.
//!
//! ## What it does
//!
//! 1. Create `<data_dir>/searxng/settings.yml` (skip if already present)
//! 2. Pull `searxng/searxng:latest` (skip if image already cached)
//! 3. `docker run` or `docker start` the container
//! 4. Poll the JSON search endpoint until healthy

use std::io::Write as _;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use rand::Rng as _;

const CONTAINER_NAME: &str = "fetchium-searxng";
const IMAGE: &str = "docker.io/searxng/searxng:latest";

// ─── Status ───────────────────────────────────────────────────────────────────

/// Current state of the fetchium-searxng Docker container.
#[derive(Debug, PartialEq, Eq)]
pub enum SearxngStatus {
    /// Container is running and reachable.
    Running { url: String },
    /// Container exists but is stopped.
    Stopped,
    /// Container has never been created.
    NotCreated,
    /// Docker is not installed or not accessible.
    DockerUnavailable,
}

/// Check whether Docker is available on this system.
pub async fn docker_available() -> bool {
    tokio::process::Command::new("docker")
        .arg("version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Return the current status of the hsx-searxng container.
pub async fn check_status(port: u16) -> SearxngStatus {
    if !docker_available().await {
        return SearxngStatus::DockerUnavailable;
    }

    let output = tokio::process::Command::new("docker")
        .args(["inspect", CONTAINER_NAME, "--format", "{{.State.Status}}"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if state == "running" {
                SearxngStatus::Running {
                    url: format!("http://localhost:{port}"),
                }
            } else {
                SearxngStatus::Stopped
            }
        }
        _ => SearxngStatus::NotCreated,
    }
}

// ─── Setup ────────────────────────────────────────────────────────────────────

/// Set up SearXNG — idempotent. Returns the URL where SearXNG is accessible.
///
/// Steps performed:
/// 1. Write `settings.yml` to `<data_dir>/searxng/` (skipped if file exists)
/// 2. Pull `searxng/searxng:latest` via `docker pull`
/// 3. `docker run` (new) or `docker start` (existing-but-stopped) the container
/// 4. Poll the JSON endpoint until healthy (up to 30 s)
pub async fn setup_searxng(data_dir: &Path, port: u16, quiet: bool) -> Result<String> {
    // ── 1. Config directory + settings.yml ───────────────────────────────
    let config_dir = data_dir.join("searxng");
    std::fs::create_dir_all(&config_dir).context("Failed to create searxng config dir")?;

    let settings_path = config_dir.join("settings.yml");
    if settings_path.exists() {
        if !quiet {
            println!("  Config: {} (existing)", settings_path.display());
        }
    } else {
        let secret = generate_secret_key();
        let yml = settings_yml(&secret, port);
        std::fs::write(&settings_path, yml).context("Failed to write settings.yml")?;
        if !quiet {
            println!("  Config: {} (created)", settings_path.display());
        }
    }

    // ── 2. Inspect existing container ────────────────────────────────────
    let inspect = tokio::process::Command::new("docker")
        .args(["inspect", CONTAINER_NAME, "--format", "{{.State.Status}}"])
        .output()
        .await
        .ok();

    let container_exists = inspect
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let container_state = inspect
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    if container_exists && container_state == "running" {
        if !quiet {
            println!("  Container: already running");
        }
        let url = format!("http://localhost:{port}");
        // Verify it actually responds
        wait_for_searxng(&url, quiet, 5).await.ok();
        return Ok(url);
    }

    if container_exists && !container_state.is_empty() {
        // Stopped/exited — just start it
        if !quiet {
            println!("  Container: starting (was {container_state})...");
        }
        let st = tokio::process::Command::new("docker")
            .args(["start", CONTAINER_NAME])
            .status()
            .await
            .context("docker start failed")?;
        if !st.success() {
            anyhow::bail!("docker start {CONTAINER_NAME} exited non-zero");
        }
    } else {
        // ── 3. Pull image ─────────────────────────────────────────────────
        if !quiet {
            println!("  Image: pulling {IMAGE}");
            println!("  (First install — may take a few minutes)");
            println!();
        }
        let pull = tokio::process::Command::new("docker")
            .args(["pull", IMAGE])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .await
            .context("docker pull command failed")?;
        if !pull.success() {
            anyhow::bail!("Failed to pull SearXNG Docker image. Is Docker running?");
        }
        if !quiet {
            println!();
        }

        // ── 4. Create and start container ─────────────────────────────────
        if !quiet {
            println!("  Container: creating on port {port}...");
        }
        let run = tokio::process::Command::new("docker")
            .args([
                "run",
                "-d",
                "--name",
                CONTAINER_NAME,
                "--restart",
                "unless-stopped",
                "-p",
                &format!("{port}:8080"),
                "-v",
                &format!("{}:/etc/searxng:rw", config_dir.display()),
                IMAGE,
            ])
            .output()
            .await
            .context("docker run command failed")?;

        if !run.status.success() {
            let err = String::from_utf8_lossy(&run.stderr);
            anyhow::bail!("docker run failed: {err}");
        }
    }

    // ── 5. Wait for healthy ───────────────────────────────────────────────
    let url = format!("http://localhost:{port}");
    if !quiet {
        print!("  Health: waiting");
        std::io::stdout().flush().ok();
    }
    wait_for_searxng(&url, quiet, 30).await?;

    Ok(url)
}

/// Poll `{base_url}/search?q=test&format=json` until 200 or `max_tries` exceeded.
async fn wait_for_searxng(base_url: &str, quiet: bool, max_tries: u32) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .context("Failed to build HTTP client")?;

    for attempt in 0..max_tries {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if !quiet {
            print!(".");
            std::io::stdout().flush().ok();
        }
        let ok = client
            .get(format!("{base_url}/search?q=test&format=json"))
            .send()
            .await
            .map(|r| r.status().is_success() || r.status().as_u16() == 200)
            .unwrap_or(false);
        if ok {
            if !quiet {
                println!(" ready! ({}s)", attempt + 1);
            }
            return Ok(());
        }
    }
    anyhow::bail!("SearXNG did not respond after {max_tries} seconds")
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Generate a cryptographically random secret key (64 hex chars = 32 random bytes).
pub fn generate_secret_key() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Build a fully-tuned `settings.yml` for Fetchium.
///
/// Enables JSON API output, disables the rate limiter (local use),
/// and activates the 10 most reliable engines.
pub fn settings_yml(secret_key: &str, _port: u16) -> String {
    format!(
        r#"# SearXNG settings for Fetchium
# Generated by `hsx setup --searxng` — edit freely
use_default_settings:
  engines:
    keep_only:
      - google
      - bing
      - duckduckgo
      - brave
      - wikipedia
      - stackoverflow
      - github
      - arxiv
      - reddit
      - hackernews

general:
  instance_name: Fetchium Search
  privacypolicy_url: false
  donation_url: false
  contact_url: false
  enable_metrics: false

server:
  # Regenerate with: openssl rand -hex 32
  secret_key: "{secret_key}"
  limiter: false
  image_proxy: false
  http_protocol_version: "1.0"
  method: GET

ui:
  static_use_hash: true
  default_locale: en
  query_in_title: false
  infinite_scroll: false
  default_theme: simple
  simple_style: dark

search:
  safe_search: 0
  autocomplete: ""
  default_lang: en
  ban_time_on_fail: 5
  max_ban_time_on_fail: 120
  suspended_times:
    SearxEngineAccessDenied: 86400
    SearxEngineCaptcha: 86400
    SearxEngineTooManyRequests: 3600
    cf_SearxEngineCaptcha: 1296000
    cf_SearxEngineAccessDenied: 86400
    recaptcha_SearxEngineCaptcha: 604800
  formats:
    - html
    - json

engines:
  - name: google
    timeout: 4.0

  - name: bing
    timeout: 4.0

  - name: duckduckgo
    timeout: 4.0

  - name: brave
    timeout: 4.0

  - name: wikipedia
    timeout: 4.0

  - name: stackoverflow
    timeout: 4.0

  - name: github
    timeout: 4.0

  - name: arxiv
    timeout: 5.0

  - name: reddit
    timeout: 4.0

  - name: hackernews
    timeout: 4.0

outgoing:
  request_timeout: 4.0
  max_request_timeout: 7.0
  useragent_suffix: ""
  pool_connections: 200
  pool_maxsize: 64
  keepalive_expiry: 15.0
  retries: 1
  enable_http2: true
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_key_is_64_hex_chars() {
        let key = generate_secret_key();
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn settings_yml_contains_secret_key() {
        let key = generate_secret_key();
        let yml = settings_yml(&key, 4040);
        assert!(yml.contains(&key));
        assert!(yml.contains("format=json") || yml.contains("- json"));
        assert!(yml.contains("limiter: false"));
        assert!(yml.contains("keep_only:"));
        assert!(yml.contains("- google"));
    }
}
