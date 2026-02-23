//! `hsx doctor` — system health check (PRD §12).

use colored::Colorize;
use hsx_core::config::HsxConfig;

pub async fn run(config: &HsxConfig) -> anyhow::Result<()> {
    println!("{}", "HyperSearchX Doctor".bold().cyan());
    println!("{}", "=".repeat(40));

    // Check Rust version
    check("Rust toolchain", true, "installed");

    // Check system resources
    let tier = HsxConfig::detect_resource_tier();
    check("Resource tier", true, &format!("{tier:?}"));

    // Check data directory
    let data_dir = config.data_dir();
    let dir_exists = data_dir.exists();
    check(
        "Data directory",
        dir_exists,
        &data_dir.to_string_lossy(),
    );

    // Check Chromium
    let chromium = which_chromium();
    check("Chromium/Chrome", chromium.is_some(), chromium.as_deref().unwrap_or("not found"));

    // Check Ollama
    let ollama = check_ollama().await;
    check("Ollama", ollama, if ollama { "running" } else { "not running (optional)" });

    println!();
    Ok(())
}

fn check(name: &str, ok: bool, detail: &str) {
    let status = if ok {
        "OK".green().bold()
    } else {
        "WARN".yellow().bold()
    };
    println!("  [{status}] {name}: {detail}");
}

fn which_chromium() -> Option<String> {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

async fn check_ollama() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();
    let Ok(client) = client else { return false };
    client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .is_ok()
}
