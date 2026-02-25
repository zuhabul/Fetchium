//! Plugin loader — installs plugins from filesystem paths.

use crate::error::HsxError;
use crate::plugin::manifest::PluginManifest;

/// Install a plugin from a source directory into the plugin directory.
///
/// Copies the entire source directory (which must contain `plugin.toml`) into
/// `~/.hypersearchx/plugins/<plugin-name>/`.
pub fn install_plugin(
    source_path: &std::path::Path,
    plugin_dir: &std::path::Path,
) -> Result<PluginManifest, HsxError> {
    let manifest_path = source_path.join("plugin.toml");
    if !manifest_path.exists() {
        return Err(HsxError::Config(format!(
            "No plugin.toml found at {:?}",
            source_path
        )));
    }
    let manifest = PluginManifest::load(&manifest_path)?;
    let dest = plugin_dir.join(&manifest.name);
    if dest.exists() {
        return Err(HsxError::Config(format!(
            "Plugin '{}' is already installed at {:?}. Remove it first.",
            manifest.name, dest
        )));
    }
    copy_dir(source_path, &dest)?;
    tracing::info!(plugin = %manifest.name, "Plugin installed");
    Ok(manifest)
}

/// Remove an installed plugin.
pub fn remove_plugin(name: &str, plugin_dir: &std::path::Path) -> Result<(), HsxError> {
    let dest = plugin_dir.join(name);
    if !dest.exists() {
        return Err(HsxError::Config(format!(
            "Plugin '{name}' is not installed"
        )));
    }
    std::fs::remove_dir_all(&dest)?;
    tracing::info!(plugin = name, "Plugin removed");
    Ok(())
}

/// Scaffold a new plugin project at `dest_path`.
pub fn create_plugin_scaffold(
    name: &str,
    plugin_type: &str,
    dest_path: &std::path::Path,
) -> Result<(), HsxError> {
    std::fs::create_dir_all(dest_path)?;
    PluginManifest::scaffold(name, plugin_type, &dest_path.join("plugin.toml"))?;
    // Write a minimal README
    std::fs::write(
        dest_path.join("README.md"),
        format!("# {name}\n\nA HyperSearchX `{plugin_type}` plugin.\n"),
    )?;
    tracing::info!(name, plugin_type, "Plugin scaffold created");
    Ok(())
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> Result<(), HsxError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
