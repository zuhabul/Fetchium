//! `hsx doctor` — system health check (PRD §13).

use colored::Colorize;
use fetchium_core::ai::providers::ProviderKind;
use fetchium_core::ai::setup::{
    format_setup_guide, recommend_models, DeviceSpec, RecommendCategory,
};
use fetchium_core::ai::{check_provider, AiConfig, ProviderStatus};
use fetchium_core::config::HsxConfig;
use sysinfo::System;

pub async fn run(config: &HsxConfig) -> anyhow::Result<()> {
    println!("{}", "Fetchium Doctor".bold().cyan());
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
        fetchium_core::types::ResourceTier::Minimal => "Minimal (< 4 GB RAM)",
        fetchium_core::types::ResourceTier::Standard => "Standard (4-16 GB RAM)",
        fetchium_core::types::ResourceTier::Performance => "Performance (16-32 GB RAM)",
        fetchium_core::types::ResourceTier::Server => "Server (32+ GB RAM, 8+ cores)",
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

    // ---- Search Backends ----
    println!("{}", "Search Backends".bold());

    // Show which backends are active and in which mode.
    // HTTP-mode backends work with zero setup; headless ones need Chrome/Chromium.
    let configured_backends = &config.search.backends;
    let headless_backends = ["google_scholar"];
    let http_backends = [
        "duckduckgo",
        "google",
        "bing",
        "searxng",
        "wikipedia",
        "hackernews",
        "arxiv",
        "github",
        "reddit",
        "stackoverflow",
        "brave",
    ];

    for backend in configured_backends {
        let b = backend.to_lowercase();
        if http_backends.contains(&b.as_str()) {
            let note = if b == "brave" {
                "HTTP (free tier 2000 req/mo — set BRAVE_API_KEY)"
            } else if b == "google" {
                "HTTP scraper (zero setup, CAPTCHA-graceful)"
            } else if b == "bing" {
                "HTTP scraper (zero setup, robust)"
            } else {
                "HTTP (no API key needed)"
            };
            check(backend, true, note);
        } else if headless_backends.contains(&b.as_str()) {
            check(backend, false, "requires --features headless + Chrome");
        } else {
            check(backend, true, "configured");
        }
    }
    println!();

    // ---- External Tools ----
    println!("{}", "External Tools".bold());

    // Chromium / Chrome
    let chromium = which_chromium();
    check(
        "Chromium/Chrome",
        true, // HTTP-mode Google/Bing work without Chrome
        chromium
            .as_deref()
            .unwrap_or("not found — Google/Bing use HTTP scraper mode (zero setup)"),
    );
    println!();

    // ---- AI Providers ----
    println!("{}", "AI Providers".bold());

    let ai_config = AiConfig::from_hsx_config(config);
    let providers_cfg = &config.ai.providers;
    let chain = providers_cfg.resolved_chain();

    let chain_str: Vec<&str> = chain.iter().map(|k| k.slug()).collect();
    println!(
        "  Fallback chain: {}",
        if chain_str.is_empty() {
            "(none configured — run `hsx provider setup`)"
                .yellow()
                .to_string()
        } else {
            chain_str.join(" → ").green().to_string()
        }
    );
    println!();

    const ALL_PROVIDERS: &[ProviderKind] = &[
        ProviderKind::Ollama,
        ProviderKind::OpenAi,
        ProviderKind::Anthropic,
        ProviderKind::Gemini,
        ProviderKind::GeminiCli,
        ProviderKind::OpenRouter,
    ];

    let mut ollama_ok = false;
    let mut ollama_models: Vec<String> = Vec::new();

    for kind in ALL_PROVIDERS {
        let status = check_provider(*kind, providers_cfg, &ai_config).await;
        let in_chain = chain.contains(kind);

        match &status {
            ProviderStatus::Available { model_count } => {
                if *kind == ProviderKind::Ollama {
                    ollama_ok = true;
                }
                let detail = match model_count {
                    Some(n) => {
                        if *kind == ProviderKind::Ollama {
                            // Collect Ollama model list for recommendations below
                            if let Ok(models) = get_ollama_model_names(&config.ai.ollama_host).await
                            {
                                ollama_models = models;
                            }
                            format!("running ({n} models installed)")
                        } else {
                            format!("({n} models)")
                        }
                    }
                    None => "key configured".to_string(),
                };
                let chain_tag = if in_chain {
                    " [in chain]".cyan().to_string()
                } else {
                    String::new()
                };
                check(kind.display_name(), true, &format!("{detail}{chain_tag}"));
            }
            ProviderStatus::Unavailable { reason } => {
                let chain_tag = if in_chain {
                    " [in chain — WILL FAIL]".red().to_string()
                } else {
                    String::new()
                };
                check(kind.display_name(), false, &format!("{reason}{chain_tag}"));
            }
        }
    }

    println!();

    // ---- Ollama Model Recommendations (shown when Ollama is configured) ----
    let spec = DeviceSpec::detect();
    if ollama_ok || chain.contains(&ProviderKind::Ollama) {
        println!("{}", "Ollama Model Recommendations".bold());
        println!(
            "  Device: {:.0} GB RAM, {} cores{}  |  Available for LLM: ~{:.0} GB",
            spec.total_ram_gb,
            spec.cpu_cores,
            if spec.is_apple_silicon {
                " (Apple Silicon)"
            } else {
                ""
            },
            spec.usable_ram_gb,
        );
        println!();

        if ollama_ok && !ollama_models.is_empty() {
            for model in &ollama_models {
                println!("  {} {} (installed)", "✓".green(), model.green().bold());
            }
            println!();
            println!("  {}", "Recommended additions:".dimmed());
        }

        let recs = recommend_models(&spec);
        for rec in &recs {
            let installed = ollama_models
                .iter()
                .any(|m| m.contains(rec.name) || rec.name.contains(m.as_str()));
            if installed {
                continue;
            }

            let (label_color, cmd_note) = match rec.category {
                RecommendCategory::BestForDevice => (
                    format!("{}", "[BEST FOR YOU]".green().bold()),
                    "← run this first",
                ),
                RecommendCategory::FastAndLight => {
                    (format!("{}", "[FAST & LIGHT ]".cyan()), "← quick responses")
                }
                RecommendCategory::MaxQuality => {
                    (format!("{}", "[MAX QUALITY  ]".yellow()), "← best accuracy")
                }
                RecommendCategory::Reasoning => {
                    (format!("{}", "[REASONING    ]".magenta()), "← logic & math")
                }
                RecommendCategory::TooLarge => (
                    format!("{}", "[TOO LARGE    ]".red().dimmed()),
                    "← needs more RAM",
                ),
            };
            println!(
                "  {}  {:<22}  ~{:.0} GB  {}",
                label_color, rec.name, rec.size_gb, cmd_note
            );
            if rec.category != RecommendCategory::TooLarge {
                println!("    {}", format!("ollama pull {}", rec.name).dimmed());
            }
            println!("    {}", rec.description.dimmed());
            println!();
        }

        if !ollama_ok {
            println!("{}", format_setup_guide(&spec).yellow());
        }
    }

    // Quick setup hint if no cloud providers have keys
    let has_cloud = chain
        .iter()
        .any(|k| *k != ProviderKind::Ollama && *k != ProviderKind::GeminiCli);
    if !has_cloud {
        println!(
            "{}",
            "Tip: Configure a cloud provider for instant AI (no GPU needed):".yellow()
        );
        println!("  hsx provider setup gemini    # Free tier, gemini-2.0-flash");
        println!("  hsx provider setup openai    # GPT-4o-mini");
        println!("  hsx provider setup openrouter # 100+ models");
        println!();
    }

    println!();

    // ---- Summary ----
    println!("{}", "Summary".bold());
    let parallel = match tier {
        fetchium_core::types::ResourceTier::Minimal => "2-4",
        fetchium_core::types::ResourceTier::Standard => "8-16",
        fetchium_core::types::ResourceTier::Performance => "16-32",
        fetchium_core::types::ResourceTier::Server => "32-50",
    };
    let browsers = match tier {
        fetchium_core::types::ResourceTier::Minimal => "0-1",
        fetchium_core::types::ResourceTier::Standard => "2-4",
        fetchium_core::types::ResourceTier::Performance => "4-6",
        fetchium_core::types::ResourceTier::Server => "6-8",
    };
    println!("  Recommended parallel fetches: {}", parallel.green());
    println!("  Recommended browser pool:     {}", browsers.green());
    if chromium.is_none() {
        println!(
            "  {}",
            "Google & Bing use HTTP scraper mode (no Chrome needed). \
             Install Chrome + build with --features headless for JS-rendered results."
                .dimmed()
        );
    }
    if !ollama_ok {
        println!(
            "  {}",
            "Install Ollama for AI features: https://ollama.ai".yellow()
        );
    }

    println!();
    Ok(())
}

async fn get_ollama_model_names(host: &str) -> anyhow::Result<Vec<String>> {
    let url = format!("{host}/api/tags");
    let resp = reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await?;
    let body: serde_json::Value = resp.json().await?;
    let models = body
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(models)
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
