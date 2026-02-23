//! `hsx config` — configuration management.

use crate::cli::{ConfigAction, ConfigArgs};
use hsx_core::config::HsxConfig;

pub async fn run(args: ConfigArgs, config: &HsxConfig) -> anyhow::Result<()> {
    match args.action {
        ConfigAction::Show => {
            let toml_str = toml::to_string_pretty(config)?;
            println!("{toml_str}");
        }
        ConfigAction::Get { key } => {
            println!("Config get: {key}");
            // TODO: Implement key lookup
        }
        ConfigAction::Set { key, value } => {
            println!("Config set: {key} = {value}");
            // TODO: Implement config persistence
        }
        ConfigAction::Reset => {
            println!("Config reset to defaults");
            // TODO: Implement config reset
        }
        ConfigAction::Edit => {
            let config_path = config.data_dir().join("config.toml");
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
            std::process::Command::new(editor)
                .arg(&config_path)
                .status()?;
        }
    }
    Ok(())
}
