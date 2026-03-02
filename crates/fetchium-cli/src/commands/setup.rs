//! `fetchium setup` — one-command full environment setup for Fetchium.
//!
//! | Invocation             | What happens                                         |
//! |------------------------|------------------------------------------------------|
//! | `fetchium setup`            | Full setup: check + Chrome + SearXNG (all-in-one)    |
//! | `fetchium setup --check`    | Print dependency status only, exit                   |
//! | `fetchium setup --headless` | Download Chrome for Testing only (~200 MB)           |
//! | `fetchium setup --searxng`  | Pull Docker image + start SearXNG on port 4040 only  |

use colored::Colorize;
use fetchium_core::{
    config::HsxConfig,
    setup::{checker, chromium, searxng},
};

use crate::cli::SetupArgs;

const SEARXNG_PORT: u16 = 4040;

pub async fn run(args: SetupArgs, config: &HsxConfig) -> anyhow::Result<()> {
    // No flags → full auto-setup (the "just works" path)
    let run_all = !args.headless && !args.searxng && !args.check;

    if run_all {
        println!("{}", "Fetchium — Full Environment Setup".bold().cyan());
        println!("{}", "═".repeat(50).dimmed());
        println!();
    }

    // ── 1. Environment check ──────────────────────────────────────────────
    if args.check || run_all {
        print_section("Environment Check");

        let items = checker::run_checks(config);
        let mut all_ok = true;

        for item in &items {
            let (icon, detail) = match item.status {
                checker::CheckStatus::Ok => (item.status.icon().green(), item.detail.normal()),
                checker::CheckStatus::Warning => {
                    all_ok = false;
                    (item.status.icon().yellow(), item.detail.yellow())
                }
                checker::CheckStatus::Missing => {
                    all_ok = false;
                    (item.status.icon().red(), item.detail.red())
                }
            };
            println!("  {} {:<22} {}", icon, item.name, detail.dimmed());
        }

        // Async: Docker container status
        if searxng::docker_available().await {
            let docker_st = searxng::check_status(SEARXNG_PORT).await;
            let (icon, detail) = match &docker_st {
                searxng::SearxngStatus::Running { url } => {
                    ("✓".green(), format!("running at {url}").normal())
                }
                searxng::SearxngStatus::Stopped => {
                    all_ok = false;
                    ("⚠".yellow(), "container stopped — will restart".yellow())
                }
                searxng::SearxngStatus::NotCreated => {
                    all_ok = false;
                    ("✗".red(), "not installed — will set up".normal())
                }
                searxng::SearxngStatus::DockerUnavailable => {
                    ("⚠".yellow(), "Docker unavailable".yellow())
                }
            };
            println!("  {} {:<22} {}", icon, "SearXNG container", detail.dimmed());
        }

        println!();

        if all_ok {
            println!("  {} All checks passed.", "✓".green().bold());
        } else if args.check {
            println!(
                "  {} Some items need attention (see above).",
                "⚠".yellow().bold()
            );
        }
        println!();

        if args.check {
            return Ok(());
        }
    }

    // ── 2. Headless Chromium ──────────────────────────────────────────────
    if args.headless || run_all {
        print_section("Headless Chromium (Chrome for Testing)");

        if let Some(existing) = chromium::resolve_chrome_path(config) {
            let is_managed = existing.starts_with(config.data_dir().join("chromium"));
            println!("  {} Already installed:", "✓".green().bold());
            println!("    {}", existing.display().to_string().cyan());
            println!(
                "    Source: {}",
                if is_managed {
                    "fetchium-managed"
                } else {
                    "system Chrome/Chromium"
                }
            );
            println!();
            println!("  {} Nothing to do.", "✓".green().bold());
            if args.headless {
                println!(
                    "    Force re-download: rm -rf ~/.fetchium/chromium && fetchium setup --headless"
                );
            }
        } else {
            println!("  {} Downloading Chrome for Testing...", "→".cyan().bold());
            println!("  One-time ~200 MB download. No root required.");
            println!();

            let data_dir = config
                .ensure_data_dir()
                .map_err(|e| anyhow::anyhow!("Failed to create data dir: {e}"))?;

            println!("  Target: {}/chromium/", data_dir.display());
            println!();

            match chromium::download_chromium(&data_dir, false).await {
                Ok(binary) => {
                    println!();
                    println!("  {} Chrome installed!", "✓".green().bold());
                    println!("  Binary : {}", binary.display().to_string().cyan());
                    println!();
                    println!("  Auto-resolved via priority chain:");
                    println!("    1. $FETCHIUM_CHROME_PATH env var");
                    println!("    2. headless.chrome_path in config.toml");
                    println!("    3. {}/chromium/  ← just installed", data_dir.display());
                    println!("    4. System /usr/bin/chromium-browser");
                }
                Err(e) => {
                    println!("  {} Download failed: {e}", "✗".red().bold());
                    println!();
                    println!("  Manual alternatives:");
                    println!("    sudo apt-get install chromium-browser");
                    println!("    export FETCHIUM_CHROME_PATH=/path/to/chrome");
                    if args.headless {
                        return Err(e);
                    }
                    // In run_all mode: warn and continue
                    println!("  Continuing with other setup steps...");
                }
            }
        }
        println!();
    }

    // ── 3. SearXNG via Docker ─────────────────────────────────────────────
    if args.searxng || run_all {
        print_section("SearXNG (federated search backbone)");

        if !searxng::docker_available().await {
            println!("  {} Docker not found.", "✗".red().bold());
            println!();
            println!("  Install Docker to enable SearXNG:");
            println!("    sudo apt-get install docker.io");
            println!("    sudo usermod -aG docker $USER && newgrp docker");
            println!("    https://docs.docker.com/engine/install/ubuntu/");
            if args.searxng {
                return Err(anyhow::anyhow!(
                    "Docker is required for SearXNG setup but is not available"
                ));
            }
            println!("  Skipping SearXNG (Docker unavailable).");
            println!();
            return Ok(());
        }

        // Already running? Just confirm
        let current = searxng::check_status(SEARXNG_PORT).await;
        match current {
            searxng::SearxngStatus::Running { ref url } => {
                println!("  {} Already running at {}", "✓".green().bold(), url.cyan());
                println!();
                println!("  {} Nothing to do.", "✓".green().bold());
                println!("    Logs:    docker logs fetchium-searxng -f");
                println!("    Restart: docker restart fetchium-searxng");
                println!("    Stop:    docker stop fetchium-searxng");
            }
            _ => {
                let data_dir = config
                    .ensure_data_dir()
                    .map_err(|e| anyhow::anyhow!("Failed to create data dir: {e}"))?;

                println!("  Config: {}/searxng/settings.yml", data_dir.display());
                println!("  Port  : {SEARXNG_PORT} (container port 8080 → host {SEARXNG_PORT})");
                println!();

                match searxng::setup_searxng(&data_dir, SEARXNG_PORT, false).await {
                    Ok(url) => {
                        println!();
                        println!("  {} SearXNG running at {}", "✓".green().bold(), url.cyan());
                        println!();
                        println!("  Engines: Google, Bing, DuckDuckGo, Brave, Wikipedia,");
                        println!("           StackOverflow, GitHub, arXiv, Reddit, HackerNews");
                        println!();
                        println!(
                            "  fetchium already configured to use this instance (search.searxng_url)."
                        );
                        println!("    Logs:    docker logs fetchium-searxng -f");
                        println!("    Config:  {}/searxng/settings.yml", data_dir.display());
                    }
                    Err(e) => {
                        println!("  {} SearXNG setup failed: {e}", "✗".red().bold());
                        if args.searxng {
                            return Err(e);
                        }
                        println!("  Continuing...");
                    }
                }
            }
        }
        println!();
    }

    // ── Final summary (run_all only) ──────────────────────────────────────
    if run_all {
        println!("{}", "═".repeat(50).dimmed());
        println!("{}", "  Setup complete!".green().bold());
        println!();
        println!("  Try it:");
        println!("    {}", "fetchium search \"rust programming\"".cyan());
        println!("    {}", "fetchium fetch https://example.com".cyan());
        println!(
            "    {}",
            "fetchium setup --check   # verify everything".cyan()
        );
        println!();
    }

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn print_section(title: &str) {
    let pad = 48usize.saturating_sub(title.len() + 4);
    println!("{}", format!("── {title} {}", "─".repeat(pad)).bold());
    println!();
}
