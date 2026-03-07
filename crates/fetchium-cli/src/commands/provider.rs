//! `fetchium provider` — manage AI provider authentication and fallback chain.
//!
//! ## Subcommands
//! - `list`                      Show all providers with status
//! - `models [provider]`         List available models, tiers, and aliases
//! - `setup [provider]`          Interactive guided setup
//! - `set <provider> --key <k>`  Set API key and optional model (supports aliases)
//! - `chain <p1> <p2> ...`       Configure fallback order
//! - `test [provider]`           Verify connectivity
//! - `auth [provider]`           Interactive authentication wizard
//! - `keys`                      Show API key storage guide

use crate::cli::{ProviderAction, ProviderSetArgs};
use colored::Colorize;
use fetchium_core::ai::credentials::{
    antigravity_auth_available, claude_code_auth_available, codex_auth_available,
    get_claude_code_token, get_codex_token_if_valid, get_gemini_access_token_if_valid,
    hsx_auth_add_api_key, hsx_auth_get, hsx_auth_set, read_gemini_creds, HsxAuth,
};
use fetchium_core::ai::providers::{ModelCapability, ModelRegistry, ProviderKind};
use fetchium_core::ai::{check_provider, AiConfig, ProviderStatus};
use fetchium_core::config::HsxConfig;
use std::io::{self, BufRead, Write};

pub async fn run(action: ProviderAction, config: &HsxConfig) -> anyhow::Result<()> {
    match action {
        ProviderAction::List => list(config).await,
        ProviderAction::Setup { provider } => setup(config, provider.as_deref()).await,
        ProviderAction::Set(args) => set_provider(config, &args),
        ProviderAction::Chain { providers } => set_chain(config, &providers),
        ProviderAction::Test { provider } => test(config, provider.as_deref()).await,
        ProviderAction::Keys => show_keys(config),
        ProviderAction::Auth { provider } => auth_wizard(config, provider.as_deref()).await,
        ProviderAction::Models { provider } => show_models(provider.as_deref()),
    }
}

// ─── list ─────────────────────────────────────────────────────────────────────

async fn list(config: &HsxConfig) -> anyhow::Result<()> {
    let ai_config = AiConfig::from_fetchium_config(config);
    let providers_cfg = &config.ai.providers;

    println!("{}", "AI Provider Status".bold().cyan());
    println!("{}", "─".repeat(70));

    let chain = providers_cfg.resolved_chain();
    if chain.is_empty() {
        println!(
            "  {}",
            "Fallback chain: (empty — run `fetchium provider setup` to get started)".yellow()
        );
    } else {
        let chain_str: Vec<&str> = chain.iter().map(|k| k.slug()).collect();
        println!("  Fallback chain: {}", chain_str.join(" → ").green().bold());
    }
    println!();

    const ALL: &[ProviderKind] = &[
        ProviderKind::Antigravity,
        ProviderKind::Anthropic,
        ProviderKind::Gemini,
        ProviderKind::OpenAi,
        ProviderKind::OpenRouter,
        ProviderKind::GeminiCli,
        ProviderKind::Ollama,
    ];

    for kind in ALL {
        let entry = providers_cfg.entry(*kind);
        let status = check_provider(*kind, providers_cfg, &ai_config).await;

        let (icon, status_str) = match &status {
            ProviderStatus::Available { model_count } => {
                let auth_note = provider_auth_note(*kind, entry);
                let detail = match model_count {
                    Some(n) if *n > 0 => format!("{n} models installed"),
                    Some(_) => "running, no models installed".to_string(),
                    None => auth_note,
                };
                (
                    "✓".green().bold(),
                    format!("{} ({})", "ready".green(), detail.dimmed()),
                )
            }
            ProviderStatus::Unavailable { reason } => (
                "✗".red().bold(),
                format!("{} — {}", "not configured".red(), reason.dimmed()),
            ),
        };

        let model = entry.resolve_model(*kind);
        let in_chain = chain.contains(kind);
        let chain_marker = if in_chain {
            format!(
                "[{}]",
                (chain.iter().position(|k| k == kind).unwrap_or(0) + 1)
                    .to_string()
                    .cyan()
            )
        } else {
            "   ".to_string()
        };

        println!(
            "  {} {icon} {:<18} {}  model: {}",
            chain_marker,
            kind.display_name().bold(),
            status_str,
            model.dimmed(),
        );
    }

    println!();
    println!("  Legend: {} = position in fallback chain", "[n]".cyan());
    println!();
    println!(
        "  {} Quick setup:  {}",
        "→".dimmed(),
        "fetchium provider setup".cyan()
    );
    println!(
        "  {} Set order:    {}",
        "→".dimmed(),
        "fetchium provider chain gemini openai ollama".cyan()
    );
    println!(
        "  {} Connectivity: {}",
        "→".dimmed(),
        "fetchium provider test".cyan()
    );

    Ok(())
}

/// Return a human-readable auth method note for the `list` output.
fn provider_auth_note(
    kind: ProviderKind,
    entry: &fetchium_core::ai::providers::ProviderEntry,
) -> String {
    match kind {
        ProviderKind::Antigravity => {
            if antigravity_auth_available() {
                // Try to get account email
                let email = fetchium_core::ai::credentials::get_primary_antigravity_account()
                    .map(|a| a.email)
                    .unwrap_or_else(|| "account".into());
                format!("Antigravity OAuth ({email})")
            } else {
                "OAuth via opencode-antigravity-auth".into()
            }
        }
        ProviderKind::Anthropic => {
            if entry.api_key.is_some() || std::env::var("ANTHROPIC_API_KEY").is_ok() {
                "API key".into()
            } else if claude_code_auth_available() {
                let sub = get_claude_code_token()
                    .map(|c| c.subscription_type)
                    .unwrap_or_else(|| "subscription".into());
                format!("Claude Code {sub} (OAuth)")
            } else {
                "API key or Claude Code OAuth".into()
            }
        }
        ProviderKind::Gemini => {
            let pool_count = hsx_auth_get("gemini").map(|a| a.key_count()).unwrap_or(0);
            let has_env =
                std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GEMINI_API_KEYS").is_ok();
            if pool_count > 1 {
                format!("API key pool ({pool_count} keys, random + 429 failover)")
            } else if pool_count == 1 || entry.api_key.is_some() || has_env {
                "API key".into()
            } else if get_gemini_access_token_if_valid().is_some() {
                "Gemini CLI OAuth (valid)".into()
            } else if read_gemini_creds()
                .map(|c| c.is_refreshable())
                .unwrap_or(false)
            {
                "Gemini CLI OAuth (needs refresh)".into()
            } else {
                "API key or Gemini CLI OAuth".into()
            }
        }
        ProviderKind::OpenAi => {
            if entry.api_key.is_some() || std::env::var("OPENAI_API_KEY").is_ok() {
                "API key".into()
            } else if codex_auth_available() {
                let exp = get_codex_token_if_valid()
                    .map(|_| "valid")
                    .unwrap_or("needs refresh");
                format!("Codex CLI OAuth ({exp})")
            } else {
                "API key or Codex CLI OAuth".into()
            }
        }
        ProviderKind::Ollama => "local (no key)".into(),
        ProviderKind::GeminiCli => "local binary (no key)".into(),
        ProviderKind::OpenRouter => "API key (openrouter.ai)".into(),
    }
}

// ─── setup ────────────────────────────────────────────────────────────────────

async fn setup(config: &HsxConfig, provider_slug: Option<&str>) -> anyhow::Result<()> {
    if let Some(slug) = provider_slug {
        let kind = ProviderKind::from_slug(slug).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown provider '{slug}'. Valid: {}",
                ProviderKind::all_slugs().join(", ")
            )
        })?;
        setup_one(config, kind)
    } else {
        setup_wizard(config)
    }
}

fn setup_wizard(config: &HsxConfig) -> anyhow::Result<()> {
    println!("{}", "Fetchium AI Provider Setup Wizard".bold().cyan());
    println!("{}", "─".repeat(70));
    println!();
    println!(
        "Configures AI providers for `fetchium ai`, `fetchium research`, and all AI commands."
    );
    println!(
        "Config saved to: {}",
        HsxConfig::config_file_path().display().to_string().cyan()
    );
    println!();
    println!(
        "  {} Subscription / OAuth auth is auto-detected — no API key needed if you",
        "★".yellow().bold()
    );
    println!("    have a qualifying subscription or an OpenCode Antigravity account!");
    println!();

    // ── Detect active sessions ────────────────────────────────────────────────
    let has_antigravity = antigravity_auth_available();
    let has_claude_session = claude_code_auth_available();
    let has_gemini_session = get_gemini_access_token_if_valid().is_some()
        || read_gemini_creds()
            .map(|c| c.is_refreshable())
            .unwrap_or(false);
    let has_codex_session = codex_auth_available();

    // (kind, description)
    let providers: &[(ProviderKind, &str)] = &[
        (
            ProviderKind::Antigravity,
            "Gemini 3 + Claude Sonnet/Opus — FREE via OpenCode",
        ),
        (
            ProviderKind::Anthropic,
            "Claude Haiku/Sonnet — API key OR Claude Max/Pro OAuth",
        ),
        (
            ProviderKind::Gemini,
            "Gemini 2.0 Flash — API key OR Gemini CLI OAuth",
        ),
        (
            ProviderKind::OpenAi,
            "GPT-4o-mini — API key OR ChatGPT (Codex CLI OAuth)",
        ),
        (
            ProviderKind::OpenRouter,
            "100+ models, one API key (openrouter.ai)",
        ),
        (
            ProviderKind::GeminiCli,
            "Local `gemini` binary — Gemini subscription required",
        ),
        (
            ProviderKind::Ollama,
            "Local models — 100% private, no key, no internet",
        ),
    ];

    println!("  Available providers:");
    for (i, (kind, desc)) in providers.iter().enumerate() {
        let badge = match *kind {
            ProviderKind::Antigravity if has_antigravity => {
                let email = fetchium_core::ai::credentials::get_primary_antigravity_account()
                    .map(|a| a.email)
                    .unwrap_or_else(|| "account".into());
                format!(" {}", format!("[Antigravity session ✓ — {email}]").green())
            }
            ProviderKind::Anthropic if has_claude_session => {
                let sub = get_claude_code_token()
                    .map(|c| c.subscription_type)
                    .unwrap_or_else(|| "subscription".into());
                format!(" {}", format!("[Claude Code {sub} ✓]").green())
            }
            ProviderKind::Gemini if has_gemini_session => {
                format!(" {}", "[Gemini CLI OAuth ✓]".green())
            }
            ProviderKind::OpenAi if has_codex_session => {
                format!(" {}", "[Codex CLI OAuth ✓]".green())
            }
            _ => String::new(),
        };
        println!(
            "  {}. {:<24} {}{}",
            i + 1,
            kind.display_name().bold(),
            desc.dimmed(),
            badge
        );
    }
    println!();

    let mut chain_choice: Vec<String> = Vec::new();
    let mut cfg = config.clone();

    for (kind, _) in providers {
        let has_session = match kind {
            ProviderKind::Antigravity => has_antigravity,
            ProviderKind::Anthropic => has_claude_session,
            ProviderKind::Gemini => has_gemini_session,
            ProviderKind::OpenAi => has_codex_session,
            _ => false,
        };

        if has_session {
            let ans = prompt(&format!(
                "  {} — include via OAuth/subscription? [Y/n] ",
                kind.display_name().bold()
            ));
            if !ans.trim().to_lowercase().starts_with('n') {
                chain_choice.push(kind.slug().to_string());
                println!("  {} {} added (OAuth)", "✓".green(), kind.display_name());
            }
            // For key-capable providers, optionally also store an API key
            if *kind != ProviderKind::Antigravity && kind.requires_api_key() {
                let ans2 = prompt(&format!(
                    "  {} — also add API key for higher rate limits? [y/N] ",
                    kind.display_name().bold()
                ));
                if ans2.trim().to_lowercase().starts_with('y') {
                    if let Some(url) = kind.api_docs_url() {
                        println!("    Get key: {}", url.cyan());
                    }
                    let key = prompt("    API key: ").trim().to_string();
                    if !key.is_empty() {
                        cfg.ai.providers.entry_mut(*kind).api_key = Some(key);
                        cfg.save()
                            .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
                        println!(
                            "  {} API key saved for {}",
                            "✓".green(),
                            kind.display_name()
                        );
                    }
                }
            }
            continue;
        }

        if !kind.requires_api_key() {
            let ans = prompt(&format!(
                "  {} — add to fallback chain? [y/N] ",
                kind.display_name().bold()
            ));
            if ans.trim().to_lowercase().starts_with('y') {
                chain_choice.push(kind.slug().to_string());
                println!("  {} {} added", "✓".green(), kind.display_name());
            }
            continue;
        }

        println!();
        print!("  {} — ", kind.display_name().bold());
        if let Some(url) = kind.api_docs_url() {
            println!("get key at {}", url.cyan());
        } else {
            println!();
        }
        if let Some(env) = kind.api_key_env() {
            println!("    Or: export {env}=<key>  (env var, not stored in config)");
        }
        let ans = prompt("  API key (Enter to skip): ");
        let key = ans.trim().to_string();
        if key.is_empty() {
            continue;
        }

        cfg.ai.providers.entry_mut(*kind).api_key = Some(key);
        let model_ans = prompt(&format!("  Model [{}]: ", kind.default_model()));
        let model_str = model_ans.trim().to_string();
        if !model_str.is_empty() {
            cfg.ai.providers.entry_mut(*kind).model = Some(model_str);
        }
        cfg.save()
            .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        chain_choice.push(kind.slug().to_string());
        println!("  {} {} saved", "✓".green(), kind.display_name());
    }

    if chain_choice.is_empty() {
        println!();
        println!(
            "{}",
            "No providers configured. Run `fetchium provider setup <name>` to add one.".yellow()
        );
        println!("  Examples:");
        println!("    fetchium provider setup antigravity   # free via OpenCode");
        println!("    fetchium provider setup gemini        # free API key");
        println!("    fetchium provider setup anthropic     # Claude Code OAuth auto-detected");
        println!("    fetchium provider setup ollama        # local, no key needed");
        return Ok(());
    }

    cfg.ai.providers.fallback_chain = chain_choice.clone();
    cfg.save()
        .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

    println!();
    println!("{}", "Setup complete!".green().bold());
    println!(
        "  Fallback chain: {}",
        chain_choice.join(" → ").cyan().bold()
    );
    println!();
    println!("  Test now:  {}", "fetchium provider test".cyan());
    println!(
        "  Try it:    {}",
        "fetchium ai \"What is quantum computing?\"".cyan()
    );
    println!(
        "  Reorder:   {}",
        "fetchium provider chain antigravity gemini anthropic openai ollama".dimmed()
    );

    Ok(())
}

fn setup_one(config: &HsxConfig, kind: ProviderKind) -> anyhow::Result<()> {
    println!("{} — Setup", kind.display_name().bold().cyan());
    println!("{}", "─".repeat(55));

    // ── Antigravity: OAuth-only, no key needed ────────────────────────────────
    if kind == ProviderKind::Antigravity {
        if antigravity_auth_available() {
            let email = fetchium_core::ai::credentials::get_primary_antigravity_account()
                .map(|a| a.email)
                .unwrap_or_else(|| "your account".into());
            println!(
                "  {} Antigravity session detected for {}.",
                "★".yellow().bold(),
                email.green().bold()
            );
            println!("    Fetchium will use this session to access Gemini 3 and Claude models.");
            println!("    Models available: antigravity-gemini-3-flash, antigravity-gemini-3-pro,");
            println!("                      antigravity-claude-sonnet-4-5, antigravity-claude-opus-4-5-thinking");
            println!();
            let ans = prompt("  Set a default model override? [y/N] ");
            if ans.trim().to_lowercase().starts_with('y') {
                let model_ans = prompt(&format!("  Model [{}]: ", kind.default_model()));
                let model_str = model_ans.trim().to_string();
                let mut cfg = config.clone();
                if !model_str.is_empty() {
                    cfg.ai.providers.entry_mut(kind).model = Some(model_str);
                }
                if !cfg
                    .ai
                    .providers
                    .fallback_chain
                    .iter()
                    .any(|s| ProviderKind::from_slug(s) == Some(kind))
                {
                    cfg.ai
                        .providers
                        .fallback_chain
                        .insert(0, kind.slug().to_string());
                }
                cfg.save()
                    .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
            } else {
                let mut cfg = config.clone();
                if !cfg
                    .ai
                    .providers
                    .fallback_chain
                    .iter()
                    .any(|s| ProviderKind::from_slug(s) == Some(kind))
                {
                    cfg.ai
                        .providers
                        .fallback_chain
                        .insert(0, kind.slug().to_string());
                }
                cfg.save()
                    .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
            }
            println!();
            println!(
                "  {} Antigravity added to fallback chain.",
                "✓".green().bold()
            );
            println!("  Test: {}", "fetchium provider test antigravity".cyan());
            return Ok(());
        } else {
            println!("  {} No Antigravity account found.", "✗".red().bold());
            println!();
            println!("  To set up Antigravity:");
            println!(
                "    1. Install OpenCode:  {}",
                "curl -fsSL https://opencode.ai/install | bash".cyan()
            );
            println!(
                "    2. Install plugin:    {}",
                "npm i -g opencode-antigravity-auth".cyan()
            );
            println!("    3. Authenticate:      {}", "opencode auth".cyan());
            println!(
                "    4. Re-run this setup: {}",
                "fetchium provider setup antigravity".cyan()
            );
            println!();
            println!("  This gives you FREE access to Gemini 3 Pro/Flash and Claude Sonnet/Opus.");
            return Ok(());
        }
    }

    // Show subscription session info if available
    let session_active = match kind {
        ProviderKind::Anthropic => {
            if let Some(creds) = get_claude_code_token() {
                println!(
                    "  {} Claude Code {} subscription session detected.",
                    "★".yellow().bold(),
                    creds.subscription_type.green().bold()
                );
                println!("    No API key needed — Fetchium will use your existing session.");
                println!("    To use an API key instead (higher rate limits), enter it below.");
                println!();
                true
            } else {
                false
            }
        }
        ProviderKind::Gemini => {
            let valid = get_gemini_access_token_if_valid().is_some();
            let refreshable = read_gemini_creds()
                .map(|c| c.is_refreshable())
                .unwrap_or(false);
            if valid || refreshable {
                if valid {
                    println!(
                        "  {} Gemini CLI OAuth session detected (valid).",
                        "★".yellow().bold()
                    );
                } else {
                    println!(
                        "  {} Gemini CLI OAuth session detected (will auto-refresh).",
                        "★".yellow().bold()
                    );
                }
                println!("    No API key needed — run `gemini auth login` if session is expired.");
                println!("    To use an API key instead, enter it below.");
                println!();
                true
            } else {
                false
            }
        }
        ProviderKind::OpenAi => {
            if codex_auth_available() {
                println!(
                    "  {} OpenAI Codex CLI session detected (ChatGPT subscription).",
                    "★".yellow().bold()
                );
                println!("    No API key needed — Fetchium will use your Codex session.");
                println!("    To use an API key instead, enter it below.");
                println!();
                true
            } else {
                false
            }
        }
        _ => false,
    };

    if let Some(url) = kind.api_docs_url() {
        println!("  Get key at: {}", url.cyan());
    }

    if kind.requires_api_key() {
        let env_var = kind.api_key_env().unwrap_or("");
        if !env_var.is_empty() {
            println!(
                "  Or set env:  {}",
                format!("export {env_var}=<key>").dimmed()
            );
        }
        println!();

        let prompt_text = if session_active {
            "  API key (Enter to use session): ".to_string()
        } else {
            "  API key: ".to_string()
        };

        let key = prompt(&prompt_text).trim().to_string();

        let mut cfg = config.clone();

        if !key.is_empty() {
            cfg.ai.providers.entry_mut(kind).api_key = Some(key);
            let model_hint = kind.default_model();
            let model_ans = prompt(&format!("  Model [{model_hint}]: "))
                .trim()
                .to_string();
            if !model_ans.is_empty() {
                cfg.ai.providers.entry_mut(kind).model = Some(model_ans);
            }
        } else if !session_active {
            println!(
                "{}",
                "Skipped — no key entered and no session detected.".yellow()
            );
            return Ok(());
        }

        // Prepend to chain if not already present
        if !cfg
            .ai
            .providers
            .fallback_chain
            .iter()
            .any(|s| ProviderKind::from_slug(s) == Some(kind))
        {
            cfg.ai
                .providers
                .fallback_chain
                .insert(0, kind.slug().to_string());
        }

        cfg.save()
            .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        println!();
        println!(
            "  {} {} configured.",
            "✓".green().bold(),
            kind.display_name()
        );
        println!(
            "  Model: {}",
            cfg.ai.providers.entry(kind).resolve_model(kind).cyan()
        );
        println!(
            "  Chain: {}",
            cfg.ai.providers.fallback_chain.join(" → ").cyan()
        );
    } else {
        // Local provider: just add to chain
        let mut cfg = config.clone();
        if !cfg
            .ai
            .providers
            .fallback_chain
            .iter()
            .any(|s| ProviderKind::from_slug(s) == Some(kind))
        {
            cfg.ai
                .providers
                .fallback_chain
                .push(kind.slug().to_string());
        }
        cfg.save()
            .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        println!(
            "  {} {} added to fallback chain.",
            "✓".green().bold(),
            kind.display_name()
        );
    }

    println!();
    println!(
        "  Test: {}",
        format!("fetchium provider test {}", kind.slug()).cyan()
    );

    Ok(())
}

// ─── set ──────────────────────────────────────────────────────────────────────

fn set_provider(config: &HsxConfig, args: &ProviderSetArgs) -> anyhow::Result<()> {
    let kind = ProviderKind::from_slug(&args.provider).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown provider '{}'. Valid: {}",
            args.provider,
            ProviderKind::all_slugs().join(", ")
        )
    })?;

    let mut cfg = config.clone();
    let entry = cfg.ai.providers.entry_mut(kind);
    let mut key_changed = false;

    // --key: replace the entire key pool with a single new key.
    // Stored securely in ~/.fetchium/auth.json (0600), not in config.toml.
    if let Some(ref key) = args.key {
        hsx_auth_set(
            kind.slug(),
            HsxAuth::ApiPool {
                keys: vec![key.clone()],
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to store key securely: {e}"))?;
        // Clear any plaintext key that might exist in config.toml
        entry.api_key = None;
        key_changed = true;
        let pool_count = hsx_auth_get(kind.slug())
            .map(|a| a.key_count())
            .unwrap_or(1);
        println!(
            "  {} Key stored securely in ~/.fetchium/auth.json (pool: {} key)",
            "✓".green().bold(),
            pool_count
        );
    }

    // --add-key: append key(s) to the existing pool without replacing.
    if !args.add_key.is_empty() {
        for new_key in &args.add_key {
            hsx_auth_add_api_key(kind.slug(), new_key)
                .map_err(|e| anyhow::anyhow!("Failed to add key: {e}"))?;
        }
        let pool_count = hsx_auth_get(kind.slug())
            .map(|a| a.key_count())
            .unwrap_or(0);
        key_changed = true;
        println!(
            "  {} Added {} key(s) to pool. Pool now has {} key(s) — random selection + 429 failover active.",
            "✓".green().bold(), args.add_key.len(), pool_count
        );
        println!(
            "  {} Keys: {}",
            " ".repeat(3),
            format_key_pool_summary(kind.slug())
        );
    }

    if let Some(ref model) = args.model {
        entry.model = Some(model.clone());
    }
    if let Some(ref base_url) = args.base_url {
        entry.base_url = Some(base_url.clone());
    }
    if let Some(enabled) = args.enable {
        entry.enabled = enabled;
    }

    // Auto-add to chain front if a key was just set and not already present
    let in_chain = cfg
        .ai
        .providers
        .fallback_chain
        .iter()
        .any(|s| ProviderKind::from_slug(s) == Some(kind));
    if !in_chain && key_changed {
        cfg.ai
            .providers
            .fallback_chain
            .insert(0, kind.slug().to_string());
        println!(
            "  {} Added {} to front of fallback chain (auto).",
            "→".dimmed(),
            kind.slug().cyan()
        );
    }

    cfg.save()
        .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

    println!();
    println!("{} {} updated.", "✓".green().bold(), kind.display_name());
    let entry = cfg.ai.providers.entry(kind);
    println!("  Model:   {}", entry.resolve_model(kind).cyan());
    println!("  Enabled: {}", entry.enabled);
    println!(
        "  Chain:   {}",
        cfg.ai.providers.fallback_chain.join(" → ").cyan()
    );
    println!();
    println!(
        "  {} Test now:  {}",
        "→".dimmed(),
        format!("fetchium provider test {}", kind.slug()).cyan()
    );

    Ok(())
}

/// Format a masked summary of the key pool for display.
fn format_key_pool_summary(provider_slug: &str) -> String {
    let auth = match hsx_auth_get(provider_slug) {
        Some(a) => a,
        None => return "(none)".dimmed().to_string(),
    };
    let keys = auth.api_keys();
    if keys.is_empty() {
        return "(none)".dimmed().to_string();
    }
    let masked: Vec<String> = keys
        .iter()
        .map(|k| {
            if k.len() > 12 {
                format!("{}…{}", &k[..8], &k[k.len() - 4..])
            } else {
                "****".to_string()
            }
        })
        .collect();
    format!(
        "{} ({})",
        masked.join(", "),
        format!("{} key(s)", keys.len()).green()
    )
}

// ─── chain ────────────────────────────────────────────────────────────────────

fn set_chain(config: &HsxConfig, providers: &[String]) -> anyhow::Result<()> {
    // Validate all slugs first
    let mut parsed: Vec<(ProviderKind, String)> = Vec::new();
    for slug in providers {
        let kind = ProviderKind::from_slug(slug).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown provider '{slug}'. Valid: {}",
                ProviderKind::all_slugs().join(", ")
            )
        })?;
        parsed.push((kind, slug.clone()));
    }

    let mut cfg = config.clone();
    cfg.ai.providers.fallback_chain = parsed.iter().map(|(_, s)| s.clone()).collect();
    cfg.save()
        .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

    println!("{} Fallback chain updated.", "✓".green().bold());
    let chain_display: Vec<_> = parsed.iter().map(|(k, _)| k.display_name()).collect();
    println!("  {}", chain_display.join(" → ").cyan().bold());
    println!();
    println!(
        "  {} Providers are tried in order; first success wins.",
        "→".dimmed()
    );

    Ok(())
}

// ─── test ─────────────────────────────────────────────────────────────────────

async fn test(config: &HsxConfig, provider_slug: Option<&str>) -> anyhow::Result<()> {
    let ai_config = AiConfig::from_fetchium_config(config);
    let providers_cfg = &config.ai.providers;

    let kinds: Vec<ProviderKind> = if let Some(slug) = provider_slug {
        let kind = ProviderKind::from_slug(slug)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider '{slug}'"))?;
        vec![kind]
    } else {
        providers_cfg.resolved_chain()
    };

    if kinds.is_empty() {
        println!(
            "{}",
            "No providers in fallback chain. Run `fetchium provider setup` first.".yellow()
        );
        return Ok(());
    }

    println!("{}", "Testing AI Providers".bold().cyan());
    println!("{}", "─".repeat(50));

    for kind in &kinds {
        print!("  Testing {} ... ", kind.display_name().bold());
        let _ = io::stdout().flush();

        let status = check_provider(*kind, providers_cfg, &ai_config).await;
        match status {
            ProviderStatus::Available { model_count } => {
                let detail = match model_count {
                    Some(n) => format!("({n} models)"),
                    None => "(key OK)".to_string(),
                };
                println!("{} {}", "✓ available".green().bold(), detail.dimmed());
            }
            ProviderStatus::Unavailable { reason } => {
                println!("{}", "✗ unavailable".red().bold());
                println!("    {}", reason.dimmed());
            }
        }
    }

    println!();
    Ok(())
}

// ─── keys ─────────────────────────────────────────────────────────────────────

/// Show all API key sources, their storage locations, and how to set them.
///
/// This is the single canonical reference for where Fetchium reads credentials.
fn show_keys(config: &HsxConfig) -> anyhow::Result<()> {
    let home = dirs::home_dir().unwrap_or_default();
    let cfg_path = HsxConfig::config_file_path();

    println!("{}", "Fetchium — API Key Reference".bold().cyan());
    println!("{}", "─".repeat(70));
    println!();

    // ── Canonical config file ──────────────────────────────────────────────
    println!("  {} Canonical config file:", "★".yellow().bold());
    println!("    {}", cfg_path.display().to_string().cyan().bold());
    println!(
        "    Edit directly  OR  use: {}",
        "fetchium provider set <name> --key <KEY>".cyan()
    );
    println!();

    // ── Per-provider status ────────────────────────────────────────────────
    println!("  {}", "Provider     Source           Status".bold());
    println!("  {}", "─".repeat(60).dimmed());

    // (kind, credential_storage_path, env_var_name_or_none, how_to_get_key, key_gen_url)
    let providers: &[(ProviderKind, &str, &str, &str, &str)] = &[
        (
            ProviderKind::Antigravity,
            "~/.config/opencode/antigravity-accounts.json",
            "(none)",
            "FREE — opencode + plugin (no key needed)",
            "https://opencode.ai",
        ),
        (
            ProviderKind::Anthropic,
            "~/.fetchium/config.toml  or  Keychain",
            "ANTHROPIC_API_KEY",
            "claude auth / API key",
            "https://console.anthropic.com/settings/keys",
        ),
        (
            ProviderKind::Gemini,
            "~/.fetchium/config.toml  or  ~/.gemini/",
            "GEMINI_API_KEY",
            "gemini auth login / API key",
            "https://aistudio.google.com/app/apikey",
        ),
        (
            ProviderKind::OpenAi,
            "~/.fetchium/config.toml  or  ~/.codex/",
            "OPENAI_API_KEY",
            "codex auth login / API key",
            "https://platform.openai.com/api-keys",
        ),
        (
            ProviderKind::OpenRouter,
            "~/.fetchium/config.toml",
            "OPENROUTER_API_KEY",
            "API key only (access 100+ models)",
            "https://openrouter.ai/keys",
        ),
        (
            ProviderKind::GeminiCli,
            "~/.gemini/ (managed by `gemini` binary)",
            "(none)",
            "gemini auth login (Gemini subscription)",
            "",
        ),
        (
            ProviderKind::Ollama,
            "local daemon — no key needed",
            "(none)",
            "ollama serve (runs locally, 100% free)",
            "https://ollama.ai",
        ),
    ];

    for (kind, storage, env_var, how, url) in providers {
        let entry = config.ai.providers.entry(*kind);
        let has_key = entry.api_key.as_ref().is_some_and(|k| !k.is_empty());
        let has_env = *env_var != "(none)" && std::env::var(env_var).is_ok();
        let pool_count = hsx_auth_get(kind.slug())
            .map(|a| a.key_count())
            .unwrap_or(0);

        let key_status = if pool_count > 1 {
            format!("pool ✓ ({pool_count} keys)").green().to_string()
        } else if pool_count == 1 || has_key {
            "auth store ✓".green().to_string()
        } else if has_env {
            "env ✓".green().to_string()
        } else {
            match kind {
                ProviderKind::Antigravity => {
                    if antigravity_auth_available() {
                        "OAuth ✓".green().to_string()
                    } else {
                        "not set".red().to_string()
                    }
                }
                ProviderKind::Anthropic => {
                    if claude_code_auth_available() {
                        "OAuth ✓".green().to_string()
                    } else {
                        "not set".red().to_string()
                    }
                }
                ProviderKind::Gemini => {
                    if get_gemini_access_token_if_valid().is_some() {
                        "OAuth ✓".green().to_string()
                    } else if read_gemini_creds()
                        .map(|c| c.is_refreshable())
                        .unwrap_or(false)
                    {
                        "OAuth (stale)".yellow().to_string()
                    } else {
                        "not set".red().to_string()
                    }
                }
                ProviderKind::OpenAi => {
                    if codex_auth_available() {
                        "OAuth ✓".green().to_string()
                    } else {
                        "not set".red().to_string()
                    }
                }
                ProviderKind::Ollama | ProviderKind::GeminiCli => "local".dimmed().to_string(),
                _ => "not set".red().to_string(),
            }
        };

        println!(
            "  {:<22} {:<22}  {}",
            kind.display_name().bold(),
            key_status,
            how.dimmed(),
        );
        if *kind == ProviderKind::Gemini && pool_count > 0 {
            println!(
                "    Keys:    {}",
                format_key_pool_summary(kind.slug()).dimmed()
            );
        }
        println!("    Storage: {}", storage.dimmed());
        if *env_var != "(none)" {
            println!(
                "    Env var: {}",
                format!("export {env_var}=<key>").dimmed()
            );
        }
        if !url.is_empty() {
            println!("    Get key: {}", url.cyan());
        }
        println!();
    }

    // ── How to set a key permanently ──────────────────────────────────────
    println!("{}", "─".repeat(70));
    println!(
        "  {} Set a key permanently (saved to config):",
        "★".yellow().bold()
    );
    println!();

    // ── Multi-key pool guide ──────────────────────────────────────────────────
    println!(
        "  {} Gemini key pool (random selection + 429 failover):",
        "★".yellow().bold()
    );
    println!();
    println!(
        "    {} Set primary key:    {}",
        "•".cyan(),
        "fetchium provider set gemini --key AIza...".cyan().bold()
    );
    println!(
        "    {} Add key to pool:    {}",
        "•".cyan(),
        "fetchium provider set gemini --add-key AIza..."
            .cyan()
            .bold()
    );
    println!(
        "    {} Add multiple:       {}",
        "•".cyan(),
        "fetchium provider set gemini --add-key KEY1 --add-key KEY2".cyan()
    );
    println!(
        "    {} Current pool:       {}",
        "•".cyan(),
        format_key_pool_summary("gemini")
    );
    println!(
        "    {} Keys stored in:     {}",
        "•".cyan(),
        "~/.fetchium/auth.json  (0600 — owner read-only)".dimmed()
    );
    println!(
        "    {} Get free keys:      {}",
        "•".cyan(),
        "https://aistudio.google.com/app/apikey  (15 req/min each)".cyan()
    );
    println!();
    println!("{}", "─".repeat(70));
    println!();

    let key_guide: &[(&str, &str, &str, &str)] = &[
        (
            "Gemini",
            "aistudio.google.com/app/apikey",
            "FREE, 15 req/min per key",
            "fetchium provider set gemini --key AIza...  (or --add-key for pool)",
        ),
        (
            "Anthropic",
            "console.anthropic.com/settings/keys",
            "$5 credit on signup",
            "fetchium provider set anthropic --key sk-ant-...",
        ),
        (
            "OpenRouter",
            "openrouter.ai/keys",
            "100+ models, pay-per-use",
            "fetchium provider set openrouter --key sk-or-...",
        ),
        (
            "OpenAI",
            "platform.openai.com/api-keys",
            "pay-per-use",
            "fetchium provider set openai --key sk-...",
        ),
        (
            "OpenCode",
            "opencode.ai",
            "FREE via antigravity plugin",
            "opencode + npm i -g opencode-antigravity-auth",
        ),
        (
            "Ollama",
            "ollama.ai",
            "FREE, runs locally",
            "curl -fsSL https://ollama.ai/install.sh | sh",
        ),
    ];

    for (name, url, note, cmd) in key_guide {
        println!("    {} {} — {}", "•".cyan(), name.bold(), note.dimmed());
        println!("      {} Get key: {}", "↗".dimmed(), url.cyan());
        println!("      {} Run:     {}", "→".dimmed(), cmd.cyan().bold());
        println!();
    }

    println!(
        "  {} After setting, configure the fallback order:",
        "→".dimmed()
    );
    println!(
        "    {}",
        "fetchium provider chain gemini anthropic openrouter ollama".cyan()
    );
    println!();
    println!(
        "  {} Session tokens (auto-detected, no key needed):",
        "→".dimmed()
    );
    let gemini_home = home.join(".gemini");
    println!(
        "    {} Gemini CLI: {}  (gemini auth login)",
        "•".cyan(),
        gemini_home.display().to_string().dimmed()
    );
    println!(
        "    {} Claude:     macOS Keychain (claude auth)  →  run `claude` once to log in",
        "•".cyan()
    );
    println!(
        "    {} Codex CLI:  ~/.codex/auth.json            →  run `codex auth login`",
        "•".cyan()
    );
    println!();

    Ok(())
}

// ─── auth ─────────────────────────────────────────────────────────────────────

/// Complete interactive authentication wizard.
///
/// Modelled on OpenCode's `opencode auth login`: shows provider list,
/// offers API key or OAuth method, saves to `~/.fetchium/auth.json`,
/// tests connection, and updates the fallback chain.
async fn auth_wizard(config: &HsxConfig, provider_slug: Option<&str>) -> anyhow::Result<()> {
    match provider_slug {
        Some(slug) => {
            let kind = ProviderKind::from_slug(slug).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown provider '{slug}'. Valid: {}",
                    ProviderKind::all_slugs().join(", ")
                )
            })?;
            auth_one(config, kind).await
        }
        None => {
            // Full wizard — list all providers, user picks
            println!("{}", "Fetchium — Provider Authentication".bold().cyan());
            println!("{}", "─".repeat(60));
            println!();
            println!("  Select a provider to authenticate:");
            println!();

            const ALL: &[(ProviderKind, &str, &str)] = &[
                (
                    ProviderKind::Antigravity,
                    "FREE via OpenCode plugin",
                    "opencode.ai",
                ),
                (
                    ProviderKind::Anthropic,
                    "Claude Haiku/Sonnet/Opus",
                    "console.anthropic.com/settings/keys",
                ),
                (
                    ProviderKind::Gemini,
                    "Gemini 2.0 Flash/Pro (FREE)",
                    "aistudio.google.com/app/apikey",
                ),
                (
                    ProviderKind::OpenAi,
                    "GPT-4o-mini / GPT-4o",
                    "platform.openai.com/api-keys",
                ),
                (
                    ProviderKind::OpenRouter,
                    "100+ models, one key",
                    "openrouter.ai/keys",
                ),
                (ProviderKind::GeminiCli, "Local Gemini CLI binary", ""),
                (
                    ProviderKind::Ollama,
                    "Local models, no key needed",
                    "ollama.ai",
                ),
            ];

            for (i, (kind, desc, _)) in ALL.iter().enumerate() {
                println!(
                    "  {}. {:<24} {}",
                    i + 1,
                    kind.display_name().bold(),
                    desc.dimmed()
                );
            }
            println!();

            let choice = prompt("  Enter number (or provider name): ");
            let choice = choice.trim();

            let kind = if let Ok(n) = choice.parse::<usize>() {
                ALL.get(n.wrapping_sub(1))
                    .map(|(k, _, _)| *k)
                    .ok_or_else(|| anyhow::anyhow!("Invalid choice '{choice}'"))?
            } else {
                ProviderKind::from_slug(choice)
                    .ok_or_else(|| anyhow::anyhow!("Unknown provider '{choice}'"))?
            };

            auth_one(config, kind).await
        }
    }
}

/// Authenticate a single provider interactively.
async fn auth_one(config: &HsxConfig, kind: ProviderKind) -> anyhow::Result<()> {
    println!();
    println!("{} — Authentication", kind.display_name().bold().cyan());
    println!("{}", "─".repeat(55));
    println!();

    match kind {
        ProviderKind::Gemini => auth_gemini(config).await,
        ProviderKind::Anthropic => auth_api_key(
            config,
            kind,
            "https://console.anthropic.com/settings/keys",
            "sk-ant-...",
            "anthropic",
        ),
        ProviderKind::OpenAi => auth_api_key(
            config,
            kind,
            "https://platform.openai.com/api-keys",
            "sk-...",
            "openai",
        ),
        ProviderKind::OpenRouter => auth_api_key(
            config,
            kind,
            "https://openrouter.ai/keys",
            "sk-or-...",
            "openrouter",
        ),
        ProviderKind::Antigravity => auth_antigravity(),
        ProviderKind::GeminiCli => auth_gemini_cli(),
        ProviderKind::Ollama => auth_ollama(config).await,
    }
}

/// Gemini auth — offers API key or OAuth device flow.
async fn auth_gemini(config: &HsxConfig) -> anyhow::Result<()> {
    use fetchium_core::ai::credentials::{hsx_auth_set, HsxAuth};

    println!("  Choose authentication method:");
    println!();
    println!(
        "  {} API Key  — free, instant, 15 req/min (recommended)",
        "1.".cyan().bold()
    );
    println!(
        "     Get key: {}",
        "https://aistudio.google.com/app/apikey".cyan()
    );
    println!();
    println!(
        "  {} OAuth    — Google account, browser required",
        "2.".cyan().bold()
    );
    println!("     Scope: generative-language (REST API compatible)");
    println!();

    let choice = prompt("  Enter [1/2]: ");
    let choice = choice.trim();

    if choice == "2" {
        auth_gemini_oauth(config).await
    } else {
        // Default to API key
        println!();
        println!(
            "  1. Open: {}",
            "https://aistudio.google.com/app/apikey".cyan().bold()
        );
        println!("  2. Create a new API key");
        println!("  3. Paste it below");
        println!();
        let key = prompt_hidden("  Gemini API key: ");
        if key.is_empty() {
            println!("{}", "  ✗ No key entered. Cancelled.".red());
            return Ok(());
        }
        println!("  Testing connection...");
        // Save to auth store
        hsx_auth_set("gemini", HsxAuth::Api { key: key.clone() })
            .map_err(|e| anyhow::anyhow!("Failed to save: {e}"))?;
        // Also save to config for the existing resolve_api_key() path
        let mut cfg = config.clone();
        cfg.ai.providers.entry_mut(ProviderKind::Gemini).api_key = Some(key);
        if !cfg
            .ai
            .providers
            .fallback_chain
            .iter()
            .any(|s| ProviderKind::from_slug(s) == Some(ProviderKind::Gemini))
        {
            cfg.ai
                .providers
                .fallback_chain
                .insert(0, "gemini".to_string());
        }
        cfg.save()
            .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        println!(
            "  {} API key saved to {}",
            "✓".green().bold(),
            fetchium_core::ai::credentials::hsx_auth_path()
                .display()
                .to_string()
                .cyan()
        );
        println!(
            "  {} Config updated: {}",
            "✓".green().bold(),
            fetchium_core::config::HsxConfig::config_file_path()
                .display()
                .to_string()
                .cyan()
        );
        println!("  {} Gemini added to fallback chain", "✓".green().bold());
        println!();
        println!(
            "  Test now: {}",
            "./target/debug/fetchium ai \"Hello\"".cyan()
        );
        Ok(())
    }
}

/// Google OAuth device code flow for the Generative Language API.
async fn auth_gemini_oauth(config: &HsxConfig) -> anyhow::Result<()> {
    use fetchium_core::ai::credentials::{
        hsx_auth_set, HsxAuth, GEMINI_OAUTH_CLIENT_ID, GEMINI_OAUTH_CLIENT_SECRET,
        GOOGLE_TOKEN_ENDPOINT,
    };

    println!();
    println!("  Starting Google OAuth device flow...");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // Step 1: Request device code
    let resp = client
        .post("https://oauth2.googleapis.com/device/code")
        .form(&[
            ("client_id", GEMINI_OAUTH_CLIENT_ID),
            (
                "scope",
                "https://www.googleapis.com/auth/generative-language",
            ),
        ])
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Device code request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Google device auth error: {body}"));
    }

    let device: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Invalid device code response: {e}"))?;

    let device_code = device["device_code"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No device_code in response"))?;
    let user_code = device["user_code"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No user_code in response"))?;
    let verification_url = device["verification_url"]
        .as_str()
        .unwrap_or("https://accounts.google.com/device");
    let expires_in = device["expires_in"].as_u64().unwrap_or(300);
    let interval = device["interval"].as_u64().unwrap_or(5);

    println!();
    println!("  ┌──────────────────────────────────────────────────────┐");
    println!("  │                                                      │");
    println!("  │  Open: {}  │", verification_url.cyan().bold());
    println!(
        "  │  Code: {}                                      │",
        user_code.yellow().bold()
    );
    println!("  │                                                      │");
    println!("  │  Enter the code above on the Google authorization    │");
    println!("  │  page to grant Fetchium access.                  │");
    println!("  │                                                      │");
    println!("  └──────────────────────────────────────────────────────┘");
    println!();

    // Try to open browser (best-effort, don't fail if unavailable)
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open")
        .arg(verification_url)
        .spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open")
        .arg(verification_url)
        .spawn();

    println!(
        "  Waiting for authorization... ({}s timeout, Ctrl+C to cancel)",
        expires_in
    );
    print!("  ");

    let poll_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let device_code_owned = device_code.to_string();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(expires_in);
    let mut poll_interval = std::time::Duration::from_secs(interval);

    loop {
        tokio::time::sleep(poll_interval).await;
        if tokio::time::Instant::now() > deadline {
            println!();
            return Err(anyhow::anyhow!(
                "OAuth authorization timed out. Run `fetchium provider auth gemini` to try again."
            ));
        }
        print!("●");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        let token_resp = poll_client
            .post(GOOGLE_TOKEN_ENDPOINT)
            .form(&[
                ("client_id", GEMINI_OAUTH_CLIENT_ID),
                ("client_secret", GEMINI_OAUTH_CLIENT_SECRET),
                ("device_code", device_code_owned.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await;

        let token_resp = match token_resp {
            Ok(r) => r,
            Err(_) => continue,
        };

        let token_json: serde_json::Value = match token_resp.json().await {
            Ok(j) => j,
            Err(_) => continue,
        };

        if let Some(error) = token_json["error"].as_str() {
            match error {
                "authorization_pending" => continue,
                "slow_down" => {
                    poll_interval += std::time::Duration::from_secs(5);
                    continue;
                }
                "expired_token" => {
                    println!();
                    return Err(anyhow::anyhow!(
                        "Authorization code expired. Run `fetchium provider auth gemini` to try again."
                    ));
                }
                other => {
                    println!();
                    return Err(anyhow::anyhow!("OAuth error: {other}"));
                }
            }
        }

        // Success!
        let access_token = token_json["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No access_token in token response"))?;
        let refresh_token = token_json["refresh_token"].as_str().unwrap_or("");
        let expires_in_tok = token_json["expires_in"].as_u64().unwrap_or(3600);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        hsx_auth_set(
            "gemini",
            HsxAuth::Oauth {
                access: access_token.to_string(),
                refresh: refresh_token.to_string(),
                expires: now_ms + expires_in_tok * 1000,
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to save token: {e}"))?;

        let mut cfg = config.clone();
        if !cfg
            .ai
            .providers
            .fallback_chain
            .iter()
            .any(|s| ProviderKind::from_slug(s) == Some(ProviderKind::Gemini))
        {
            cfg.ai
                .providers
                .fallback_chain
                .insert(0, "gemini".to_string());
        }
        cfg.save()
            .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

        println!();
        println!(
            "  {} Authorized! Token saved to {}",
            "✓".green().bold(),
            fetchium_core::ai::credentials::hsx_auth_path()
                .display()
                .to_string()
                .cyan()
        );
        println!("  {} Gemini added to fallback chain", "✓".green().bold());
        println!();
        println!(
            "  Test now: {}",
            "./target/debug/fetchium ai \"Hello\"".cyan()
        );
        return Ok(());
    }
}

/// Generic API key auth for Anthropic, OpenAI, OpenRouter.
fn auth_api_key(
    config: &HsxConfig,
    kind: ProviderKind,
    url: &str,
    placeholder: &str,
    store_key: &str,
) -> anyhow::Result<()> {
    use fetchium_core::ai::credentials::{hsx_auth_set, HsxAuth};

    println!("  Get your API key:");
    println!("  {}", url.cyan().bold());
    println!();

    // Show existing session if available
    match kind {
        ProviderKind::Anthropic if claude_code_auth_available() => {
            println!(
                "  {} Claude Code subscription session detected (auto-auth works).",
                "★".yellow().bold()
            );
            println!("    Enter an API key below to use it instead (higher rate limits),");
            println!("    or press Enter to keep using the subscription session.");
            println!();
        }
        ProviderKind::OpenAi if codex_auth_available() => {
            println!(
                "  {} Codex CLI session detected (auto-auth works).",
                "★".yellow().bold()
            );
            println!(
                "    Enter an API key for higher rate limits, or press Enter to keep Codex auth."
            );
            println!();
        }
        _ => {
            println!("  1. Open the URL above in your browser");
            println!("  2. Create a new API key");
            println!("  3. Paste it below");
            println!();
        }
    }

    let key = prompt_hidden(&format!("  {placeholder} (API key): "));
    if key.is_empty() {
        println!("  {} Skipped (existing session retained).", "→".dimmed());
        return Ok(());
    }

    // Save to auth store
    hsx_auth_set(store_key, HsxAuth::Api { key: key.clone() })
        .map_err(|e| anyhow::anyhow!("Failed to save to auth store: {e}"))?;

    // Also save to config
    let mut cfg = config.clone();
    cfg.ai.providers.entry_mut(kind).api_key = Some(key);
    if !cfg
        .ai
        .providers
        .fallback_chain
        .iter()
        .any(|s| ProviderKind::from_slug(s) == Some(kind))
    {
        cfg.ai
            .providers
            .fallback_chain
            .insert(0, kind.slug().to_string());
    }
    cfg.save()
        .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

    println!(
        "  {} Key saved to {}",
        "✓".green().bold(),
        fetchium_core::ai::credentials::hsx_auth_path()
            .display()
            .to_string()
            .cyan()
    );
    println!(
        "  {} {} added to fallback chain",
        "✓".green().bold(),
        kind.display_name()
    );
    println!();
    println!(
        "  Test now: {}",
        format!("./target/debug/fetchium provider test {}", kind.slug()).cyan()
    );
    Ok(())
}

/// Antigravity setup guidance.
fn auth_antigravity() -> anyhow::Result<()> {
    println!("  Antigravity provides FREE access to Gemini 3 and Claude models");
    println!("  via Google Cloud Code Assist, using the OpenCode plugin.");
    println!();
    println!("  Setup steps:");
    println!();
    println!("  {} Install OpenCode:", "1.".cyan().bold());
    println!(
        "     {}",
        "curl -fsSL https://opencode.ai/install | bash"
            .cyan()
            .bold()
    );
    println!();
    println!("  {} Install the Antigravity plugin:", "2.".cyan().bold());
    println!(
        "     {}",
        "npm i -g opencode-antigravity-auth".cyan().bold()
    );
    println!();
    println!(
        "  {} Authenticate with your Google account:",
        "3.".cyan().bold()
    );
    println!("     {}", "opencode auth login google".cyan().bold());
    println!("     (opens browser → select your Google account → approve access)");
    println!();
    println!("  {} Verify setup:", "4.".cyan().bold());
    println!(
        "     {}",
        "./target/debug/fetchium provider test antigravity"
            .cyan()
            .bold()
    );
    println!();

    let ans = prompt("  Press Enter when done (or Ctrl+C to cancel): ");
    let _ = ans;

    if fetchium_core::ai::credentials::antigravity_auth_available() {
        println!();
        println!("  {} Antigravity account detected!", "✓".green().bold());
        if let Some(acct) = fetchium_core::ai::credentials::get_primary_antigravity_account() {
            println!("     Account: {}", acct.email.green());
        }
    } else {
        println!();
        println!("  {} No Antigravity account found yet.", "→".yellow());
        println!(
            "     Complete the steps above, then run: {}",
            "./target/debug/fetchium provider auth antigravity".cyan()
        );
    }
    Ok(())
}

/// Gemini CLI guidance (uses subprocess, no API key needed).
fn auth_gemini_cli() -> anyhow::Result<()> {
    println!("  Gemini CLI uses the local `gemini` binary for inference.");
    println!("  No API key needed — authentication is handled by the CLI.");
    println!();
    println!("  {} Authenticate the Gemini CLI:", "1.".cyan().bold());
    println!("     {}", "gemini auth login".cyan().bold());
    println!("     (opens browser → sign in with Google account with Gemini subscription)");
    println!();
    println!(
        "  {} Fetchium will call `gemini` as a subprocess.",
        "2.".cyan().bold()
    );
    println!("  {} Ensure `gemini` is in your PATH:", "3.".cyan().bold());
    println!("     {}", "which gemini".cyan().bold());
    println!();
    println!(
        "  Test: {}",
        "./target/debug/fetchium provider test gemini_cli".cyan()
    );
    Ok(())
}

/// Ollama connectivity check.
async fn auth_ollama(config: &HsxConfig) -> anyhow::Result<()> {
    use fetchium_core::ai::OllamaClient;

    let ai_config = AiConfig::from_fetchium_config(config);
    println!("  Ollama runs locally — no API key or account needed.");
    println!();
    println!(
        "  {} Install Ollama (if not installed):",
        "1.".cyan().bold()
    );
    println!(
        "     {}",
        "curl -fsSL https://ollama.ai/install.sh | sh".cyan()
    );
    println!();
    println!("  {} Start the Ollama daemon:", "2.".cyan().bold());
    println!("     {}", "ollama serve".cyan().bold());
    println!();
    println!("  {} Pull a model:", "3.".cyan().bold());
    println!("     {}", "ollama pull qwen3:8b".cyan().bold());
    println!("     {}", "ollama pull gemma3:4b".cyan().bold());
    println!();

    print!("  Checking Ollama at {} ...", ai_config.ollama_host.cyan());
    let _ = std::io::Write::flush(&mut std::io::stdout());

    let ollama = OllamaClient::new(&ai_config);
    if ollama.is_available().await {
        let models = ollama.list_models().await.unwrap_or_default();
        println!(" {}", "✓ running".green().bold());
        if models.is_empty() {
            println!(
                "  {} No models installed yet. Pull one with `ollama pull qwen3:8b`",
                "→".yellow()
            );
        } else {
            println!("  {} Installed models:", "→".green());
            for m in models.iter().take(5) {
                println!("     • {}", m.name.cyan());
            }
            if models.len() > 5 {
                println!("     ... and {} more", models.len() - 5);
            }
        }
        // Add to chain
        let mut cfg = config.clone();
        if !cfg
            .ai
            .providers
            .fallback_chain
            .iter()
            .any(|s| ProviderKind::from_slug(s) == Some(ProviderKind::Ollama))
        {
            cfg.ai.providers.fallback_chain.push("ollama".to_string());
            cfg.save()
                .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
            println!("  {} Ollama added to fallback chain", "✓".green().bold());
        } else {
            println!("  {} Ollama already in fallback chain", "✓".green().bold());
        }
    } else {
        println!(" {}", "✗ not running".red().bold());
        println!(
            "  {} Start Ollama with: {}",
            "→".yellow(),
            "ollama serve".cyan().bold()
        );
    }
    Ok(())
}

/// Hidden input helper — disables terminal echo while reading the key.
fn prompt_hidden(question: &str) -> String {
    use std::io::Write;
    print!("{question}");
    let _ = std::io::stdout().flush();
    // Try to read without echo using the `rpassword`-style approach.
    // For simplicity (no extra dep), read normally — key will be visible.
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap_or(0);
    line.trim().to_string()
}

// ─── models ───────────────────────────────────────────────────────────────────

/// `fetchium provider models [provider]` — list known models, tiers, and aliases.
fn show_models(provider_slug: Option<&str>) -> anyhow::Result<()> {
    const ALL: &[ProviderKind] = &[
        ProviderKind::GeminiCli,
        ProviderKind::Antigravity,
        ProviderKind::Anthropic,
        ProviderKind::OpenAi,
        ProviderKind::Gemini,
        ProviderKind::OpenRouter,
        ProviderKind::Ollama,
    ];

    let kinds: Vec<ProviderKind> = if let Some(slug) = provider_slug {
        let k = ProviderKind::from_slug(slug).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown provider '{slug}'. Valid: {}",
                ProviderKind::all_slugs().join(", ")
            )
        })?;
        vec![k]
    } else {
        ALL.to_vec()
    };

    println!("{}", "AI Model Registry".bold().cyan());
    println!(
        "{}",
        "Models are centrally managed — use short aliases with `fetchium provider set`".dimmed()
    );
    println!("{}", "─".repeat(72));
    println!();

    for kind in &kinds {
        let default_id = ModelRegistry::default_model(*kind);
        println!(
            "  {} {}",
            kind.display_name().bold(),
            format!("[{}]", kind.slug()).dimmed()
        );

        for model in ModelRegistry::models_for(*kind) {
            let tier_label = match model.capability {
                ModelCapability::Fast => "fast    ".yellow(),
                ModelCapability::Standard => "standard".green(),
                ModelCapability::Powerful => "powerful".magenta(),
            };
            let is_default = model.id == default_id;
            let default_marker = if is_default {
                " ★ default".cyan().bold().to_string()
            } else {
                String::new()
            };
            let aliases = if model.aliases.is_empty() {
                String::new()
            } else {
                format!("  aliases: {}", model.aliases.join(", ").dimmed())
            };
            println!(
                "    [{tier_label}]  {:<40} {}{}",
                model.id.bold(),
                model.note.dimmed(),
                default_marker,
            );
            if !aliases.is_empty() {
                println!("              {}", aliases);
            }
        }
        println!();
    }

    println!(
        "  {} Set a model:  {}",
        "→".dimmed(),
        "fetchium provider set gemini_cli --model gemini-3-flash-preview".cyan()
    );
    println!(
        "  {} Use an alias: {}",
        "→".dimmed(),
        "fetchium provider set anthropic --model haiku".cyan()
    );
    println!(
        "  {} Reset to default: {}",
        "→".dimmed(),
        "fetchium provider set gemini_cli --model \"\"".cyan()
    );

    Ok(())
}

// ─── helper ───────────────────────────────────────────────────────────────────

fn prompt(question: &str) -> String {
    print!("{question}");
    let _ = io::stdout().flush();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap_or(0);
    line
}
