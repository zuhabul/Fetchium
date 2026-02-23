//! `hsx doctor` — system health check (PRD §13).

use colored::Colorize;
use hsx_core::config::HsxConfig;
use sysinfo::System;

pub async fn run(config: &HsxConfig) -> anyhow::Result<()> {
    println!("{}", "HyperSearchX Doctor".bold().cyan());
    println!("{}", "=".repeat(50));
    println!();

    // ---- System Information ----
    println!("{}", "System Information".bold());
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "unknown".into());
    let cpu_count = sys.cpus().len();
    let total_ram_mb = sys.total_memory() / (1024 * 1024);
    let free_ram_mb = sys.available_memory() / (1024 * 1024);
    let used_pct = if total_ram_mb > 0 {
        ((total_ram_mb - free_ram_mb) as f64 / total_ram_mb as f64 * 100.0) as u64
    } else {
        0
    };

    check("CPU", true, &format!("{cpu_name} ({cpu_count} cores)"));
    check(
        "RAM",
        true,
        &format!("{total_ram_mb} MB total, {free_ram_mb} MB free ({used_pct}% used)"),
    );

    // ---- Resource Tier ----
    let tier = HsxConfig::detect_resource_tier();
    let tier_detail = match tier {
        hsx_core::types::ResourceTier::Minimal => "Minimal (< 4 GB RAM)",
        hsx_core::types::ResourceTier::Standard => "Standard (4-16 GB RAM)",
        hsx_core::types::ResourceTier::Performance => "Performance (16-32 GB RAM)",
        hsx_core::types::ResourceTier::Server => "Server (32+ GB RAM, 8+ cores)",
    };
    check("Resource Tier", true, tier_detail);

    println!();

    // ---- Configuration ----
    println!("{}", "Configuration".bold());
    let data_dir = config.data_dir();
    let dir_exists = data_dir.exists();
    check(
        "Data directory",
        dir_exists,
        &format!(
            "{} {}",
            data_dir.display(),
            if dir_exists {
                "(exists)"
            } else {
                "(will be created)"
            }
        ),
    );

    let config_path = HsxConfig::config_file_path();
    let config_exists = config_path.exists();
    check(
        "Config file",
        config_exists,
        &format!(
            "{} {}",
            config_path.display(),
            if config_exists {
                "(loaded)"
            } else {
                "(using defaults)"
            }
        ),
    );

    check(
        "Default budget",
        true,
        &format!("{} tokens", config.search.default_budget),
    );
    check(
        "Cache",
        config.cache.enabled,
        if config.cache.enabled {
            "enabled"
        } else {
            "disabled"
        },
    );

    println!();

    // ---- External Tools ----
    println!("{}", "External Tools".bold());

    // Chromium / Chrome
    let chromium = which_chromium();
    check(
        "Chromium/Chrome",
        chromium.is_some(),
        chromium
            .as_deref()
            .unwrap_or("not found (headless features unavailable)"),
    );

    // Ollama
    let ollama_status = check_ollama(&config.ai.ollama_host).await;
    check(
        "Ollama",
        ollama_status.is_some(),
        ollama_status
            .as_deref()
            .unwrap_or("not running (AI features unavailable)"),
    );

    println!();

    // ---- Summary ----
    println!("{}", "Summary".bold());
    let parallel = match tier {
        hsx_core::types::ResourceTier::Minimal => "2-4",
        hsx_core::types::ResourceTier::Standard => "8-16",
        hsx_core::types::ResourceTier::Performance => "16-32",
        hsx_core::types::ResourceTier::Server => "32-50",
    };
    let browsers = match tier {
        hsx_core::types::ResourceTier::Minimal => "0-1",
        hsx_core::types::ResourceTier::Standard => "2-4",
        hsx_core::types::ResourceTier::Performance => "4-6",
        hsx_core::types::ResourceTier::Server => "6-8",
    };
    println!("  Recommended parallel fetches: {}", parallel.green());
    println!("  Recommended browser pool:     {}", browsers.green());
    if chromium.is_none() {
        println!(
            "  {}",
            "Install Chrome/Chromium for headless search (Google, Bing, Scholar)".yellow()
        );
    }
    if ollama_status.is_none() {
        println!(
            "  {}",
            "Install Ollama for AI features: https://ollama.ai".yellow()
        );
    }

    println!();
    Ok(())
}

fn check(name: &str, ok: bool, detail: &str) {
    let icon = if ok {
        "OK".green().bold()
    } else {
        "WARN".yellow().bold()
    };
    println!("  [{icon}] {name}: {detail}");
}

fn which_chromium() -> Option<String> {
    let candidates = if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ]
    } else {
        vec![
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ]
    };

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

async fn check_ollama(host: &str) -> Option<String> {
    let url = format!("{host}/api/tags");
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let model_count = body
                        .get("models")
                        .and_then(|m| m.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    Some(format!("running ({model_count} models available)"))
                } else {
                    Some("running".into())
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
