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
            let toml_val = toml::Value::try_from(config)?;
            if let Some(val) = lookup_key(&toml_val, &key) {
                // Remove quotes from strings for cleaner output
                if let Some(s) = val.as_str() {
                    println!("{s}");
                } else {
                    println!("{val}");
                }
            } else {
                anyhow::bail!("Key not found: {key}");
            }
        }
        ConfigAction::Set { key, value } => {
            let mut toml_val = toml::Value::try_from(config)?;
            set_key(&mut toml_val, &key, &value)?;
            let new_config: HsxConfig = toml::from_str(&toml::to_string(&toml_val)?)?;
            new_config.save()?;
            println!("Config set: {key} = {value}");
        }
        ConfigAction::Reset => {
            let default_config = HsxConfig::default();
            default_config.save()?;
            println!("Config reset to defaults");
        }
        ConfigAction::Edit => {
            let config_path = HsxConfig::config_file_path();
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
            std::process::Command::new(editor)
                .arg(&config_path)
                .status()?;
        }
    }
    Ok(())
}

fn lookup_key<'a>(val: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    let mut current = val;
    for part in key.split('.') {
        match current.get(part) {
            Some(v) => current = v,
            None => return None,
        }
    }
    Some(current)
}

fn set_key(val: &mut toml::Value, key: &str, new_value: &str) -> anyhow::Result<()> {
    let mut current = val;
    let parts: Vec<&str> = key.split('.').collect();

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Leaf node: parse the new_value and set it
            if let Some(table) = current.as_table_mut() {
                let parsed_val = if let Ok(b) = new_value.parse::<bool>() {
                    toml::Value::Boolean(b)
                } else if let Ok(i) = new_value.parse::<i64>() {
                    toml::Value::Integer(i)
                } else if let Ok(f) = new_value.parse::<f64>() {
                    toml::Value::Float(f)
                } else {
                    toml::Value::String(new_value.to_string())
                };
                table.insert(part.to_string(), parsed_val);
                return Ok(());
            } else {
                anyhow::bail!("Cannot set key in non-table structure");
            }
        } else {
            // Traverse down
            if let Some(table) = current.as_table_mut() {
                if !table.contains_key(*part) {
                    table.insert(part.to_string(), toml::Value::Table(toml::map::Map::new()));
                }
                current = table.get_mut(*part).unwrap();
            } else {
                anyhow::bail!("Path contains non-table element");
            }
        }
    }
    Ok(())
}
