//! Chrome for Testing download and binary resolution.
//!
//! ## Priority chain for Chrome binary resolution
//!
//! 1. `FETCHIUM_CHROME_PATH` env var
//! 2. `config.headless.chrome_path` (config.toml `[headless]` section)
//! 3. fetchium-managed: `~/.fetchium/chromium/<platform>/chrome`
//! 4. System paths: `/usr/bin/chromium-browser`, `/usr/bin/chromium`, etc.
//!
//! ## Download
//!
//! `download_chromium()` fetches from Google's Chrome for Testing CDN:
//! - Manifest: <https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json>
//! - Downloads the Stable channel ZIP for the current platform
//! - Extracts to `data_dir/chromium/`
//! - Makes the binary executable

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::config::HsxConfig;

const MANIFEST_URL: &str = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";

// ─── Path resolution ──────────────────────────────────────────────────────────

/// Resolve Chrome binary using the priority chain described in the module docs.
///
/// Returns `None` if no Chrome/Chromium is found anywhere on the system.
pub fn resolve_chrome_path(config: &HsxConfig) -> Option<PathBuf> {
    // 1. Env var override
    if let Ok(val) = std::env::var("FETCHIUM_CHROME_PATH") {
        let p = PathBuf::from(&val);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Config file override
    if let Some(ref p) = config.headless.chrome_path {
        if p.exists() {
            return Some(p.clone());
        }
    }

    // 3a. fetchium-managed download (new canonical location: `~/.fetchium/chromium/`)
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let new_managed = chrome_binary_in(&home.join(".fetchium").join("chromium"));
    if new_managed.exists() {
        return Some(new_managed);
    }

    // 4. System-installed Chrome/Chromium
    for s in &[
        "/usr/bin/chromium-browser",
        "/usr/bin/chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/snap/bin/chromium",
    ] {
        let p = Path::new(s);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }

    None
}

/// Returns the expected path to the Chrome binary within an extraction root.
///
/// `extract_root` is the directory into which the Chrome for Testing ZIP was
/// extracted (e.g. `~/.fetchium/chromium/`).
pub fn chrome_binary_in(extract_root: &Path) -> PathBuf {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", _) => extract_root.join("chrome-linux64").join("chrome"),
        ("macos", "aarch64") => extract_root
            .join("chrome-mac-arm64")
            .join("Google Chrome for Testing.app")
            .join("Contents")
            .join("MacOS")
            .join("Google Chrome for Testing"),
        ("macos", _) => extract_root
            .join("chrome-mac-x64")
            .join("Google Chrome for Testing.app")
            .join("Contents")
            .join("MacOS")
            .join("Google Chrome for Testing"),
        ("windows", _) => extract_root.join("chrome-win64").join("chrome.exe"),
        _ => extract_root.join("chrome"), // unsupported platform fallback
    }
}

/// Returns the Chrome for Testing platform string for the current OS/arch.
///
/// Used when selecting a download from the manifest.
pub fn current_platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", _) => "linux64",
        ("macos", "aarch64") => "mac-arm64",
        ("macos", _) => "mac-x64",
        ("windows", _) => "win64",
        _ => "linux64",
    }
}

// ─── Manifest types ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Manifest {
    channels: std::collections::HashMap<String, Channel>,
}

#[derive(Deserialize)]
struct Channel {
    version: String,
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    chrome: Vec<PlatformDownload>,
}

#[derive(Deserialize)]
struct PlatformDownload {
    platform: String,
    url: String,
}

// ─── Download ─────────────────────────────────────────────────────────────────

/// Download Chrome for Testing (Stable channel) to `<data_dir>/chromium/`.
///
/// Shows a progress bar unless `quiet` is true.
/// Returns the path to the installed Chrome binary.
pub async fn download_chromium(data_dir: &Path, quiet: bool) -> Result<PathBuf> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("Fetchium/", env!("CARGO_PKG_VERSION"), " (setup)"))
        .build()
        .context("Failed to build HTTP client")?;

    // ── 1. Fetch manifest ─────────────────────────────────────────────────
    if !quiet {
        println!("  Fetching Chrome for Testing manifest...");
    }

    let manifest: Manifest = client
        .get(MANIFEST_URL)
        .send()
        .await
        .context("Failed to fetch Chrome for Testing manifest")?
        .json()
        .await
        .context("Failed to parse Chrome for Testing manifest")?;

    let channel = manifest
        .channels
        .get("Stable")
        .context("Stable channel not found in manifest")?;

    let platform = current_platform();
    let dl = channel
        .downloads
        .chrome
        .iter()
        .find(|d| d.platform == platform)
        .with_context(|| format!("No Chrome download found for platform '{platform}'"))?;

    if !quiet {
        println!("  Version : {}", channel.version);
        println!("  Platform: {platform}");
        println!("  URL     : {}", dl.url);
        println!();
    }

    // ── 2. Stream-download the ZIP ────────────────────────────────────────
    let response = client
        .get(&dl.url)
        .send()
        .await
        .context("Failed to start Chrome download")?;

    if !response.status().is_success() {
        anyhow::bail!("Chrome download failed with HTTP {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = if !quiet {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        Some(pb)
    } else {
        None
    };

    let zip_path = std::env::temp_dir().join("fetchium-chrome-for-testing.zip");
    let mut zip_file = tokio::fs::File::create(&zip_path)
        .await
        .context("Failed to create temporary ZIP file")?;

    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error reading download stream")?;
        downloaded += chunk.len() as u64;
        zip_file
            .write_all(&chunk)
            .await
            .context("Failed to write ZIP chunk to disk")?;
        if let Some(ref pb) = pb {
            pb.set_position(downloaded);
        }
    }
    zip_file.flush().await.context("Failed to flush ZIP file")?;
    drop(zip_file); // close before handing to zip extractor

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }
    if !quiet {
        println!("  Downloaded {downloaded} bytes");
        println!("  Extracting...");
    }

    // ── 3. Extract ZIP in blocking thread ─────────────────────────────────
    let extract_root = data_dir.join("chromium");
    let extract_root_clone = extract_root.clone();
    let zip_path_clone = zip_path.clone();

    tokio::task::spawn_blocking(move || -> Result<()> {
        std::fs::create_dir_all(&extract_root_clone)
            .context("Failed to create chromium directory")?;

        let zf =
            std::fs::File::open(&zip_path_clone).context("Failed to open ZIP for extraction")?;
        let mut archive = zip::ZipArchive::new(zf).context("Failed to read ZIP archive")?;
        archive
            .extract(&extract_root_clone)
            .context("Failed to extract Chrome ZIP")?;

        let _ = std::fs::remove_file(&zip_path_clone);
        Ok(())
    })
    .await
    .context("ZIP extraction task panicked")?
    .context("ZIP extraction failed")?;

    // ── 4. Make binary executable (Unix only) ─────────────────────────────
    let binary = chrome_binary_in(&extract_root);

    #[cfg(unix)]
    if binary.exists() {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(&binary).context("Failed to stat Chrome binary")?;
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary, perms)
            .context("Failed to set Chrome binary executable")?;
    }

    if !binary.exists() {
        anyhow::bail!(
            "Chrome binary not found at expected path after extraction: {}\n\
             The ZIP may have a different internal layout. Please report this.",
            binary.display()
        );
    }

    Ok(binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_string_is_known() {
        let p = current_platform();
        assert!(["linux64", "mac-x64", "mac-arm64", "win64"].contains(&p));
    }

    #[test]
    fn chrome_binary_in_has_correct_subdir() {
        let root = Path::new("/tmp/chromium");
        let binary = chrome_binary_in(root);
        // On Linux, path must contain chrome-linux64
        #[cfg(target_os = "linux")]
        assert!(binary.to_string_lossy().contains("chrome-linux64"));
    }
}
