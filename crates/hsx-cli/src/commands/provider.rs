//! `hsx provider` — manage AI provider authentication and fallback chain.
//!
//! ## Subcommands
//! - `list`                      Show all providers with status
//! - `setup [provider]`          Interactive guided setup
//! - `set <provider> --key <k>`  Set API key and optional model
//! - `chain <p1> <p2> ...`       Configure fallback order
//! - `test [provider]`           Verify connectivity

use crate::cli::{ProviderAction, ProviderSetArgs};
use colored::Colorize;
use hsx_core::ai::credentials::{
    antigravity_auth_available, claude_code_auth_available, codex_auth_available,
    get_claude_code_token, get_codex_token_if_valid,
    get_gemini_access_token_if_valid, read_gemini_creds,
};
use hsx_core::ai::providers::ProviderKind;
use hsx_core::ai::{check_provider, AiConfig, ProviderStatus};
use hsx_core::config::HsxConfig;
use std::io::{self, BufRead, Write};

pub async fn run(action: ProviderAction, config: &HsxConfig) -> anyhow::Result<()> {
    match action {
        ProviderAction::List => list(config).await,
        ProviderAction::Setup { provider } => setup(config, provider.as_deref()).await,
        ProviderAction::Set(args) => set_provider(config, &args),
        ProviderAction::Chain { providers } => set_chain(config, &providers),
        ProviderAction::Test { provider } => test(config, provider.as_deref()).await,
        ProviderAction::Keys => show_keys(config),
    }
}

// ─── list ─────────────────────────────────────────────────────────────────────

async fn list(config: &HsxConfig) -> anyhow::Result<()> {
    let ai_config = AiConfig::from_hsx_config(config);
    let providers_cfg = &config.ai.providers;

    println!("{}", "AI Provider Status".bold().cyan());
    println!("{}", "─".repeat(70));

    let chain = providers_cfg.resolved_chain();
    if chain.is_empty() {
        println!("  {}", "Fallback chain: (empty — run `hsx provider setup` to get started)".yellow());
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
                ("✓".green().bold(), format!("{} ({})", "ready".green(), detail.dimmed()))
            }
            ProviderStatus::Unavailable { reason } => {
                ("✗".red().bold(), format!("{} — {}", "not configured".red(), reason.dimmed()))
            }
        };

        let model = entry.resolve_model(*kind);
        let in_chain = chain.contains(kind);
        let chain_marker = if in_chain {
            format!("[{}]", (chain.iter().position(|k| k == kind).unwrap_or(0) + 1).to_string().cyan())
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
    println!("  {} Quick setup:  {}", "→".dimmed(), "hsx provider setup".cyan());
    println!("  {} Set order:    {}", "→".dimmed(), "hsx provider chain gemini openai ollama".cyan());
    println!("  {} Connectivity: {}", "→".dimmed(), "hsx provider test".cyan());

    Ok(())
}

/// Return a human-readable auth method note for the `list` output.
fn provider_auth_note(kind: ProviderKind, entry: &hsx_core::ai::providers::ProviderEntry) -> String {
    match kind {
        ProviderKind::Antigravity => {
            if antigravity_auth_available() {
                // Try to get account email
                let email = hsx_core::ai::credentials::get_primary_antigravity_account()
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
            if entry.api_key.is_some() || std::env::var("GEMINI_API_KEY").is_ok() {
                "API key".into()
            } else if get_gemini_access_token_if_valid().is_some() {
                "Gemini CLI OAuth (valid)".into()
            } else if read_gemini_creds().is_some() {
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
        ProviderKind::Ollama    => "local (no key)".into(),
        ProviderKind::GeminiCli => "local binary (no key)".into(),
        ProviderKind::OpenRouter => "API key (openrouter.ai)".into(),
    }
}

// ─── setup ────────────────────────────────────────────────────────────────────

async fn setup(config: &HsxConfig, provider_slug: Option<&str>) -> anyhow::Result<()> {
    if let Some(slug) = provider_slug {
        let kind = ProviderKind::from_slug(slug)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider '{slug}'. Valid: {}", ProviderKind::all_slugs().join(", ")))?;
        setup_one(config, kind)
    } else {
        setup_wizard(config)
    }
}

fn setup_wizard(config: &HsxConfig) -> anyhow::Result<()> {
    println!("{}", "HyperSearchX AI Provider Setup Wizard".bold().cyan());
    println!("{}", "─".repeat(70));
    println!();
    println!("Configures AI providers for `hsx ai`, `hsx research`, and all AI commands.");
    println!("Config saved to: {}", HsxConfig::config_file_path().display().to_string().cyan());
    println!();
    println!("  {} Subscription / OAuth auth is auto-detected — no API key needed if you", "★".yellow().bold());
    println!("    have a qualifying subscription or an OpenCode Antigravity account!");
    println!();

    // ── Detect active sessions ────────────────────────────────────────────────
    let has_antigravity    = antigravity_auth_available();
    let has_claude_session = claude_code_auth_available();
    let has_gemini_session = get_gemini_access_token_if_valid().is_some()
        || read_gemini_creds().is_some();
    let has_codex_session  = codex_auth_available();

    // (kind, description)
    let providers: &[(ProviderKind, &str)] = &[
        (ProviderKind::Antigravity, "Gemini 3 + Claude Sonnet/Opus — FREE via OpenCode"),
        (ProviderKind::Anthropic,   "Claude Haiku/Sonnet — API key OR Claude Max/Pro OAuth"),
        (ProviderKind::Gemini,      "Gemini 2.0 Flash — API key OR Gemini CLI OAuth"),
        (ProviderKind::OpenAi,      "GPT-4o-mini — API key OR ChatGPT (Codex CLI OAuth)"),
        (ProviderKind::OpenRouter,  "100+ models, one API key (openrouter.ai)"),
        (ProviderKind::GeminiCli,   "Local `gemini` binary — Gemini subscription required"),
        (ProviderKind::Ollama,      "Local models — 100% private, no key, no internet"),
    ];

    println!("  Available providers:");
    for (i, (kind, desc)) in providers.iter().enumerate() {
        let badge = match *kind {
            ProviderKind::Antigravity if has_antigravity => {
                let email = hsx_core::ai::credentials::get_primary_antigravity_account()
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
        println!("  {}. {:<24} {}{}", i + 1, kind.display_name().bold(), desc.dimmed(), badge);
    }
    println!();

    let mut chain_choice: Vec<String> = Vec::new();
    let mut cfg = config.clone();

    for (kind, _) in providers {
        let has_session = match kind {
            ProviderKind::Antigravity => has_antigravity,
            ProviderKind::Anthropic   => has_claude_session,
            ProviderKind::Gemini      => has_gemini_session,
            ProviderKind::OpenAi      => has_codex_session,
            _                         => false,
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
                        cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
                        println!("  {} API key saved for {}", "✓".green(), kind.display_name());
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
        cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        chain_choice.push(kind.slug().to_string());
        println!("  {} {} saved", "✓".green(), kind.display_name());
    }

    if chain_choice.is_empty() {
        println!();
        println!("{}", "No providers configured. Run `hsx provider setup <name>` to add one.".yellow());
        println!("  Examples:");
        println!("    hsx provider setup antigravity   # free via OpenCode");
        println!("    hsx provider setup gemini        # free API key");
        println!("    hsx provider setup anthropic     # Claude Code OAuth auto-detected");
        println!("    hsx provider setup ollama        # local, no key needed");
        return Ok(());
    }

    cfg.ai.providers.fallback_chain = chain_choice.clone();
    cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

    println!();
    println!("{}", "Setup complete!".green().bold());
    println!("  Fallback chain: {}", chain_choice.join(" → ").cyan().bold());
    println!();
    println!("  Test now:  {}", "hsx provider test".cyan());
    println!("  Try it:    {}", "hsx ai \"What is quantum computing?\"".cyan());
    println!("  Reorder:   {}", "hsx provider chain antigravity gemini anthropic openai ollama".dimmed());

    Ok(())
}

fn setup_one(config: &HsxConfig, kind: ProviderKind) -> anyhow::Result<()> {
    println!("{} — Setup", kind.display_name().bold().cyan());
    println!("{}", "─".repeat(55));

    // ── Antigravity: OAuth-only, no key needed ────────────────────────────────
    if kind == ProviderKind::Antigravity {
        if antigravity_auth_available() {
            let email = hsx_core::ai::credentials::get_primary_antigravity_account()
                .map(|a| a.email)
                .unwrap_or_else(|| "your account".into());
            println!(
                "  {} Antigravity session detected for {}.",
                "★".yellow().bold(), email.green().bold()
            );
            println!("    HyperSearchX will use this session to access Gemini 3 and Claude models.");
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
                if !cfg.ai.providers.fallback_chain.iter().any(|s| ProviderKind::from_slug(s) == Some(kind)) {
                    cfg.ai.providers.fallback_chain.insert(0, kind.slug().to_string());
                }
                cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
            } else {
                let mut cfg = config.clone();
                if !cfg.ai.providers.fallback_chain.iter().any(|s| ProviderKind::from_slug(s) == Some(kind)) {
                    cfg.ai.providers.fallback_chain.insert(0, kind.slug().to_string());
                }
                cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
            }
            println!();
            println!("  {} Antigravity added to fallback chain.", "✓".green().bold());
            println!("  Test: {}", "hsx provider test antigravity".cyan());
            return Ok(());
        } else {
            println!("  {} No Antigravity account found.", "✗".red().bold());
            println!();
            println!("  To set up Antigravity:");
            println!("    1. Install OpenCode:  {}", "curl -fsSL https://opencode.ai/install | bash".cyan());
            println!("    2. Install plugin:    {}", "npm i -g opencode-antigravity-auth".cyan());
            println!("    3. Authenticate:      {}", "opencode auth".cyan());
            println!("    4. Re-run this setup: {}", "hsx provider setup antigravity".cyan());
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
                    "★".yellow().bold(), creds.subscription_type.green().bold()
                );
                println!("    No API key needed — HyperSearchX will use your existing session.");
                println!("    To use an API key instead (higher rate limits), enter it below.");
                println!();
                true
            } else {
                false
            }
        }
        ProviderKind::Gemini => {
            let valid = get_gemini_access_token_if_valid().is_some();
            let has_any = read_gemini_creds().is_some();
            if has_any {
                if valid {
                    println!("  {} Gemini CLI OAuth session detected (valid).", "★".yellow().bold());
                } else {
                    println!("  {} Gemini CLI OAuth session detected (will auto-refresh).", "★".yellow().bold());
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
                println!("  {} OpenAI Codex CLI session detected (ChatGPT subscription).", "★".yellow().bold());
                println!("    No API key needed — HyperSearchX will use your Codex session.");
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
            println!("  Or set env:  {}", format!("export {env_var}=<key>").dimmed());
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
            let model_ans = prompt(&format!("  Model [{model_hint}]: ")).trim().to_string();
            if !model_ans.is_empty() {
                cfg.ai.providers.entry_mut(kind).model = Some(model_ans);
            }
        } else if !session_active {
            println!("{}", "Skipped — no key entered and no session detected.".yellow());
            return Ok(());
        }

        // Prepend to chain if not already present
        if !cfg.ai.providers.fallback_chain.iter().any(|s| ProviderKind::from_slug(s) == Some(kind)) {
            cfg.ai.providers.fallback_chain.insert(0, kind.slug().to_string());
        }

        cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        println!();
        println!("  {} {} configured.", "✓".green().bold(), kind.display_name());
        println!("  Model: {}", cfg.ai.providers.entry(kind).resolve_model(kind).cyan());
        println!("  Chain: {}", cfg.ai.providers.fallback_chain.join(" → ").cyan());
    } else {
        // Local provider: just add to chain
        let mut cfg = config.clone();
        if !cfg.ai.providers.fallback_chain.iter().any(|s| ProviderKind::from_slug(s) == Some(kind)) {
            cfg.ai.providers.fallback_chain.push(kind.slug().to_string());
        }
        cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        println!("  {} {} added to fallback chain.", "✓".green().bold(), kind.display_name());
    }

    println!();
    println!("  Test: {}", format!("hsx provider test {}", kind.slug()).cyan());

    Ok(())
}

// ─── set ──────────────────────────────────────────────────────────────────────

fn set_provider(config: &HsxConfig, args: &ProviderSetArgs) -> anyhow::Result<()> {
    let kind = ProviderKind::from_slug(&args.provider)
        .ok_or_else(|| anyhow::anyhow!(
            "Unknown provider '{}'. Valid: {}", args.provider, ProviderKind::all_slugs().join(", ")
        ))?;

    let mut cfg = config.clone();
    let entry = cfg.ai.providers.entry_mut(kind);

    if let Some(ref key) = args.key {
        entry.api_key = Some(key.clone());
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

    // Auto-add to chain if key was just set and not already present
    let in_chain = cfg.ai.providers.fallback_chain.iter().any(|s| ProviderKind::from_slug(s) == Some(kind));
    if !in_chain && args.key.is_some() {
        cfg.ai.providers.fallback_chain.insert(0, kind.slug().to_string());
        println!("  {} Added {} to the front of the fallback chain.", "→".dimmed(), kind.slug().cyan());
    }

    cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

    println!("{} {} updated.", "✓".green().bold(), kind.display_name());
    let entry = cfg.ai.providers.entry(kind);
    println!("  Model:   {}", entry.resolve_model(kind).cyan());
    println!("  Enabled: {}", entry.enabled);
    println!("  Chain:   {}", cfg.ai.providers.fallback_chain.join(" → ").cyan());

    Ok(())
}

// ─── chain ────────────────────────────────────────────────────────────────────

fn set_chain(config: &HsxConfig, providers: &[String]) -> anyhow::Result<()> {
    // Validate all slugs first
    let mut parsed: Vec<(ProviderKind, String)> = Vec::new();
    for slug in providers {
        let kind = ProviderKind::from_slug(slug)
            .ok_or_else(|| anyhow::anyhow!(
                "Unknown provider '{slug}'. Valid: {}", ProviderKind::all_slugs().join(", ")
            ))?;
        parsed.push((kind, slug.clone()));
    }

    let mut cfg = config.clone();
    cfg.ai.providers.fallback_chain = parsed.iter().map(|(_, s)| s.clone()).collect();
    cfg.save().map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;

    println!("{} Fallback chain updated.", "✓".green().bold());
    let chain_display: Vec<_> = parsed.iter().map(|(k, _)| k.display_name()).collect();
    println!("  {}", chain_display.join(" → ").cyan().bold());
    println!();
    println!("  {} Providers are tried in order; first success wins.", "→".dimmed());

    Ok(())
}

// ─── test ─────────────────────────────────────────────────────────────────────

async fn test(config: &HsxConfig, provider_slug: Option<&str>) -> anyhow::Result<()> {
    let ai_config = AiConfig::from_hsx_config(config);
    let providers_cfg = &config.ai.providers;

    let kinds: Vec<ProviderKind> = if let Some(slug) = provider_slug {
        let kind = ProviderKind::from_slug(slug)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider '{slug}'"))?;
        vec![kind]
    } else {
        providers_cfg.resolved_chain()
    };

    if kinds.is_empty() {
        println!("{}", "No providers in fallback chain. Run `hsx provider setup` first.".yellow());
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
/// This is the single canonical reference for where HyperSearchX reads credentials.
fn show_keys(config: &HsxConfig) -> anyhow::Result<()> {
    let home = dirs::home_dir().unwrap_or_default();
    let cfg_path = HsxConfig::config_file_path();

    println!("{}", "HyperSearchX — API Key Reference".bold().cyan());
    println!("{}", "─".repeat(70));
    println!();

    // ── Canonical config file ──────────────────────────────────────────────
    println!("  {} Canonical config file:", "★".yellow().bold());
    println!("    {}", cfg_path.display().to_string().cyan().bold());
    println!("    Edit directly  OR  use: {}", "hsx provider set <name> --key <KEY>".cyan());
    println!();

    // ── Per-provider status ────────────────────────────────────────────────
    println!("  {}", "Provider     Source           Status".bold());
    println!("  {}", "─".repeat(60).dimmed());

    // (kind, credential_storage_path, env_var_name_or_none, how_to_get_key, key_gen_url)
    let providers: &[(ProviderKind, &str, &str, &str, &str)] = &[
        (ProviderKind::Antigravity, "~/.config/opencode/antigravity-accounts.json", "(none)",             "FREE — opencode + plugin (no key needed)", "https://opencode.ai"),
        (ProviderKind::Anthropic,   "~/.hypersearchx/config.toml  or  Keychain",   "ANTHROPIC_API_KEY",  "claude auth / API key",                    "https://console.anthropic.com/settings/keys"),
        (ProviderKind::Gemini,      "~/.hypersearchx/config.toml  or  ~/.gemini/", "GEMINI_API_KEY",     "gemini auth login / API key",               "https://aistudio.google.com/app/apikey"),
        (ProviderKind::OpenAi,      "~/.hypersearchx/config.toml  or  ~/.codex/",  "OPENAI_API_KEY",     "codex auth login / API key",                "https://platform.openai.com/api-keys"),
        (ProviderKind::OpenRouter,  "~/.hypersearchx/config.toml",                 "OPENROUTER_API_KEY", "API key only (access 100+ models)",         "https://openrouter.ai/keys"),
        (ProviderKind::GeminiCli,   "~/.gemini/ (managed by `gemini` binary)",     "(none)",             "gemini auth login (Gemini subscription)",   ""),
        (ProviderKind::Ollama,      "local daemon — no key needed",                "(none)",             "ollama serve (runs locally, 100% free)",    "https://ollama.ai"),
    ];

    for (kind, storage, env_var, how, url) in providers {
        let entry = config.ai.providers.entry(*kind);
        let has_key = entry.api_key.as_ref().is_some_and(|k| !k.is_empty());
        let has_env = *env_var != "(none)" && std::env::var(env_var).is_ok();

        let key_status = if has_key {
            "config ✓".green().to_string()
        } else if has_env {
            "env ✓".green().to_string()
        } else {
            match kind {
                ProviderKind::Antigravity => {
                    if antigravity_auth_available() { "OAuth ✓".green().to_string() }
                    else { "not set".red().to_string() }
                }
                ProviderKind::Anthropic => {
                    if claude_code_auth_available() { "OAuth ✓".green().to_string() }
                    else { "not set".red().to_string() }
                }
                ProviderKind::Gemini => {
                    if get_gemini_access_token_if_valid().is_some() {
                        "OAuth ✓".green().to_string()
                    } else if read_gemini_creds().map(|c| c.is_refreshable()).unwrap_or(false) {
                        "OAuth (stale)".yellow().to_string()
                    } else {
                        "not set".red().to_string()
                    }
                }
                ProviderKind::OpenAi => {
                    if codex_auth_available() { "OAuth ✓".green().to_string() }
                    else { "not set".red().to_string() }
                }
                ProviderKind::Ollama | ProviderKind::GeminiCli => "local".dimmed().to_string(),
                _ => "not set".red().to_string(),
            }
        };

        println!(
            "  {:<22} {:<18}  {}",
            kind.display_name().bold(),
            key_status,
            how.dimmed(),
        );
        println!("    Storage: {}", storage.dimmed());
        if *env_var != "(none)" {
            println!("    Env var: {}", format!("export {env_var}=<key>").dimmed());
        }
        if !url.is_empty() {
            println!("    Get key: {}", url.cyan());
        }
        println!();
    }

    // ── How to set a key permanently ──────────────────────────────────────
    println!("{}", "─".repeat(70));
    println!("  {} Set a key permanently (saved to config):", "★".yellow().bold());
    println!();

    let key_guide: &[(&str, &str, &str, &str)] = &[
        ("Gemini",      "aistudio.google.com/app/apikey",   "FREE, 15 req/min",          "hsx provider set gemini --key AIza..."),
        ("Anthropic",   "console.anthropic.com/settings/keys", "$5 credit on signup",    "hsx provider set anthropic --key sk-ant-..."),
        ("OpenRouter",  "openrouter.ai/keys",               "100+ models, pay-per-use",  "hsx provider set openrouter --key sk-or-..."),
        ("OpenAI",      "platform.openai.com/api-keys",     "pay-per-use",               "hsx provider set openai --key sk-..."),
        ("OpenCode",    "opencode.ai",                      "FREE via antigravity plugin","opencode + npm i -g opencode-antigravity-auth"),
        ("Ollama",      "ollama.ai",                        "FREE, runs locally",        "curl -fsSL https://ollama.ai/install.sh | sh"),
    ];

    for (name, url, note, cmd) in key_guide {
        println!("    {} {} — {}", "•".cyan(), name.bold(), note.dimmed());
        println!("      {} Get key: {}", "↗".dimmed(), url.cyan());
        println!("      {} Run:     {}", "→".dimmed(), cmd.cyan().bold());
        println!();
    }

    println!("  {} After setting, configure the fallback order:", "→".dimmed());
    println!("    {}", "hsx provider chain gemini anthropic openrouter ollama".cyan());
    println!();
    println!("  {} Session tokens (auto-detected, no key needed):", "→".dimmed());
    let gemini_home = home.join(".gemini");
    println!("    {} Gemini CLI: {}  (gemini auth login)", "•".cyan(), gemini_home.display().to_string().dimmed());
    println!("    {} Claude:     macOS Keychain (claude auth)  →  run `claude` once to log in", "•".cyan());
    println!("    {} Codex CLI:  ~/.codex/auth.json            →  run `codex auth login`", "•".cyan());
    println!();

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
