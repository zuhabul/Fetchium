//! `hsx plugin` — plugin management (PRD §29).

use clap::Subcommand;
use colored::Colorize;
use fetchium_core::plugin::{loader, PluginRegistry};

#[derive(Debug, Subcommand)]
pub enum PluginCommand {
    /// List installed plugins.
    List,
    /// Install a plugin from a local path.
    Install {
        /// Path to the plugin directory (must contain plugin.toml).
        source: String,
    },
    /// Remove an installed plugin.
    Remove {
        /// Plugin name to remove.
        name: String,
    },
    /// Scaffold a new plugin project.
    Create {
        /// Plugin name.
        name: String,
        /// Plugin type (backend, extractor, ranker, formatter, validator, ai_provider).
        #[arg(long, default_value = "backend")]
        plugin_type: String,
        /// Output directory (defaults to ./<name>).
        #[arg(long)]
        output: Option<String>,
    },
    /// Show info about an installed plugin.
    Info {
        /// Plugin name.
        name: String,
    },
}

pub fn run(cmd: PluginCommand) -> anyhow::Result<()> {
    let plugin_dir = PluginRegistry::default_dir();
    std::fs::create_dir_all(&plugin_dir)?;
    match cmd {
        PluginCommand::List => {
            let mut reg = PluginRegistry::new();
            let count = reg.discover(&plugin_dir)?;
            if count == 0 {
                println!(
                    "{} No plugins installed. Use `hsx plugin install <path>` to add one.",
                    "i".blue()
                );
                return Ok(());
            }
            println!("{}", "Installed Plugins".bold().cyan());
            println!("{}", "\u{2500}".repeat(40));
            for entry in reg.all() {
                println!(
                    "  {:.<25} {} [{}]",
                    entry.manifest.name,
                    entry.manifest.version.yellow(),
                    entry.manifest.plugin_type.dimmed(),
                );
            }
        }
        PluginCommand::Install { source } => {
            let src = std::path::Path::new(&source);
            let manifest = loader::install_plugin(src, &plugin_dir)?;
            println!(
                "{} Plugin '{}' v{} installed.",
                "OK".green(),
                manifest.name,
                manifest.version
            );
        }
        PluginCommand::Remove { name } => {
            loader::remove_plugin(&name, &plugin_dir)?;
            println!("{} Plugin '{}' removed.", "OK".green(), name);
        }
        PluginCommand::Create {
            name,
            plugin_type,
            output,
        } => {
            let dest = output
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(&name));
            loader::create_plugin_scaffold(&name, &plugin_type, &dest)?;
            println!("{} Plugin scaffold created at {:?}", "OK".green(), dest);
            println!(
                "  Edit {} to configure your plugin.",
                dest.join("plugin.toml").display()
            );
        }
        PluginCommand::Info { name } => {
            let mut reg = PluginRegistry::new();
            reg.discover(&plugin_dir)?;
            match reg.get(&name) {
                Some(entry) => {
                    println!("{}", entry.manifest.name.bold().cyan());
                    println!("  Version:  {}", entry.manifest.version);
                    println!("  Type:     {}", entry.manifest.plugin_type);
                    println!("  Runtime:  {}", entry.manifest.runtime);
                    println!("  Path:     {:?}", entry.path);
                    if !entry.manifest.description.is_empty() {
                        println!("  Desc:     {}", entry.manifest.description);
                    }
                }
                None => eprintln!("Plugin '{}' not found.", name),
            }
        }
    }
    Ok(())
}
