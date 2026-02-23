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
    claude_code_auth_available, codex_auth_available, get_claude_code_token,
    get_codex_token_if_valid, get_gemini_access_token_if_valid, read_gemini_creds,
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
        ProviderKind::Ollama,
        ProviderKind::OpenAi,
        ProviderKind::Anthropic,
        ProviderKind::Gemini,
        ProviderKind::GeminiCli,
        ProviderKind::OpenRouter,
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
        ProviderKind::Anthropic => {
            if entry.api_key.is_some() || std::env::var("ANTHROPIC_API_KEY").is_ok() {
                "API key".into()
            } else if claude_code_auth_available() {
                let sub = get_claude_code_token()
                    .map(|c| c.subscription_type)
                    .unwrap_or_else(|| "subscription".into());
                format!("Claude Code {sub} (OAuth)")
            } else {
                "API key".into()
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
                "API key".into()
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
                "API key".into()
            }
        }
        ProviderKind::Ollama => "local (no key)".into(),
        ProviderKind::GeminiCli => "local binary (no key)".into(),
        ProviderKind::OpenRouter => "API key".into(),
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
    println!("{}", "─".repeat(65));
    println!();
    println!("Configures AI providers for `hsx ai`, `hsx research`, and other AI commands.");
    println!("Keys are saved to: {}", HsxConfig::config_file_path().display().to_string().cyan());
    println!();
    println!("  {} Subscription auth is auto-detected (Claude Code, Gemini CLI, Codex CLI).", "★".yellow());
    println!("    No API key needed if you already have a qualifying subscription!");
    println!();

    // Detect which providers already have subscription sessions
    let has_claude_session = claude_code_auth_available();
    let has_gemini_session = get_gemini_access_token_if_valid().is_some()
        || read_gemini_creds().is_some();
    let has_codex_session = codex_auth_available();

    let providers: &[(ProviderKind, &str)] = &[
        (ProviderKind::Gemini,     "Google Gemini 2.0 Flash — fast, generous free tier"),
        (ProviderKind::OpenAi,     "OpenAI — gpt-4o-mini (or ChatGPT subscription via Codex CLI)"),
        (ProviderKind::Anthropic,  "Anthropic — claude-haiku (or Claude Max/Pro subscription)"),
        (ProviderKind::OpenRouter, "OpenRouter — 100+ models, one API key (openrouter.ai)"),
        (ProviderKind::Ollama,     "Ollama — local models, 100% private, no internet"),
        (ProviderKind::GeminiCli,  "Gemini CLI — local `gemini` binary, Gemini subscription"),
    ];

    println!("  Available providers:");
    for (i, (kind, desc)) in providers.iter().enumerate() {
        let session_badge = match *kind {
            ProviderKind::Anthropic if has_claude_session => {
                let sub = get_claude_code_token()
                    .map(|c| c.subscription_type)
                    .unwrap_or_else(|| "subscription".into());
                format!(" {}", format!("[Claude Code {sub} detected ✓]").green())
            }
            ProviderKind::Gemini if has_gemini_session => {
                format!(" {}", "[Gemini CLI session detected ✓]".green())
            }
            ProviderKind::OpenAi if has_codex_session => {
                format!(" {}", "[Codex CLI session detected ✓]".green())
            }
            _ => String::new(),
        };
        println!("  {}. {:<20} {}{}", i + 1, kind.display_name().bold(), desc.dimmed(), session_badge);
    }
    println!();

    let mut chain_choice: Vec<String> = Vec::new();
    let mut cfg = config.clone();

    for (kind, _) in providers {
        // Auto-add if a session is already available
        let has_session = match kind {
            ProviderKind::Anthropic => has_claude_session,
            ProviderKind::Gemini    => has_gemini_session,
            ProviderKind::OpenAi    => has_codex_session,
            _                       => false,
        };

        if has_session {
            let ans = prompt(&format!(
                "  Include {} via subscription session? [Y/n] ",
                kind.display_name().bold()
            ));
            if !ans.trim().to_lowercase().starts_with('n') {
                chain_choice.push(kind.slug().to_string());
                println!("  {} {} added (subscription auth)", "✓".green(), kind.display_name());
            }
            continue;
        }

        if !kind.requires_api_key() {
            let ans = prompt(&format!(
                "  Include {} in fallback chain? [y/N] ",
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
        let ans = prompt("  API key (Enter to skip): ");
        let key = ans.trim().to_string();
        if key.is_empty() {
            continue;
        }

        cfg.ai.providers.entry_mut(*kind).api_key = Some(key);

        let model_ans = prompt(&format!(
            "  Model [{}]: ",
            kind.default_model()
        ));
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
        println!("    hsx provider setup gemini      # free API key");
        println!("    hsx provider setup anthropic   # Claude Code subscription auto-detected");
        println!("    hsx provider setup ollama      # local, no key needed");
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
    println!("  Reorder:   {}", "hsx provider chain gemini openai ollama".dimmed());

    Ok(())
}

fn setup_one(config: &HsxConfig, kind: ProviderKind) -> anyhow::Result<()> {
    println!("{} — Setup", kind.display_name().bold().cyan());
    println!("{}", "─".repeat(55));

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

// ─── helper ───────────────────────────────────────────────────────────────────

fn prompt(question: &str) -> String {
    print!("{question}");
    let _ = io::stdout().flush();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap_or(0);
    line
}
