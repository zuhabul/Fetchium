//! PDF and DOCX export via Pandoc subprocess (Phase 5, PRD §26).
//!
//! PDF engine priority: **typst** (fastest, ~1s) → **xelatex** (Unicode) → pandoc default.
//! Typst produces publication-quality PDFs in under a second; install via `brew install typst`.
//!
//! Pandoc is required for all export formats. A clear error message with
//! install instructions is shown when Pandoc is not found.

use crate::error::HsxError;
use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Command;
use tracing::info;

const PANDOC_INSTALL_HELP: &str =
    "Install Pandoc from https://pandoc.org/installing.html or via your package manager \
     (brew install pandoc / apt install pandoc / choco install pandoc)";

const TYPST_INSTALL_HELP: &str =
    "Install Typst for fast PDF generation: brew install typst / cargo install typst-cli";

/// Check whether Typst is installed on this system.
fn typst_available() -> bool {
    Command::new("typst")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check that Typst is installed and return its version string.
pub fn check_typst() -> Result<String, HsxError> {
    let output = Command::new("typst")
        .arg("--version")
        .output()
        .map_err(|_| HsxError::ExternalTool(format!("Typst not found. {TYPST_INSTALL_HELP}")))?;

    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version
        .lines()
        .next()
        .unwrap_or("unknown version")
        .to_string();
    Ok(first_line)
}

/// Check that Pandoc is installed and return its version string.
pub fn check_pandoc() -> Result<String, HsxError> {
    let output = Command::new("pandoc")
        .arg("--version")
        .output()
        .map_err(|_| HsxError::ExternalTool(format!("Pandoc not found. {PANDOC_INSTALL_HELP}")))?;

    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version
        .lines()
        .next()
        .unwrap_or("unknown version")
        .to_string();
    Ok(first_line)
}

/// Export a markdown string to PDF via Pandoc.
///
/// Engine priority: **typst** (~1s) → **xelatex** (Unicode) → pandoc default engine.
/// Install typst with `brew install typst` for the fastest export experience.
pub fn export_pdf(markdown: &str, output_path: &Path, title: Option<&str>) -> Result<(), HsxError> {
    check_pandoc()?;

    let tmp_path = std::env::temp_dir().join(format!("hsx-export-{}.md", uuid_v4()));

    {
        let mut f = std::fs::File::create(&tmp_path)?;
        // YAML frontmatter for Pandoc metadata
        if let Some(t) = title {
            writeln!(f, "---")?;
            writeln!(f, "title: \"{}\"", t.replace('"', "'"))?;
            writeln!(f, "date: \"{}\"", chrono::Utc::now().format("%Y-%m-%d"))?;
            writeln!(f, "geometry: margin=1in")?;
            writeln!(f, "---\n")?;
        }
        write!(f, "{}", markdown)?;
    }

    info!("Exporting PDF to {}", output_path.display());

    // Priority: typst (fastest, ~1s) → xelatex (Unicode) → pandoc default
    let success = (typst_available() && try_pandoc_pdf(&tmp_path, output_path, Some("typst")))
        || try_pandoc_pdf(&tmp_path, output_path, Some("xelatex"))
        || try_pandoc_pdf(&tmp_path, output_path, None);

    let _ = std::fs::remove_file(&tmp_path);

    if success {
        Ok(())
    } else {
        Err(HsxError::ExternalTool(
            "Pandoc PDF generation failed. Install Typst (brew install typst) for fast PDF export, \
             or a LaTeX engine (xelatex, pdflatex, or tectonic) as fallback."
                .into(),
        ))
    }
}

fn try_pandoc_pdf(src: &Path, dst: &Path, engine: Option<&str>) -> bool {
    let mut cmd = Command::new("pandoc");
    cmd.arg(src)
        .args(["-o", &dst.to_string_lossy()])
        .arg("--standalone")
        .arg("--toc");
    if let Some(e) = engine {
        cmd.args(["--pdf-engine", e]);
    }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

/// Export a markdown string to DOCX via Pandoc.
pub fn export_docx(markdown: &str, output_path: &Path) -> Result<(), HsxError> {
    check_pandoc()?;

    let tmp_path = std::env::temp_dir().join(format!("hsx-export-{}.md", uuid_v4()));

    {
        let mut f = std::fs::File::create(&tmp_path)?;
        write!(f, "{}", markdown)?;
    }

    info!("Exporting DOCX to {}", output_path.display());

    let status = Command::new("pandoc")
        .arg(&tmp_path)
        .args(["-o", &output_path.to_string_lossy()])
        .arg("--standalone")
        .status()?;

    let _ = std::fs::remove_file(&tmp_path);

    if status.success() {
        Ok(())
    } else {
        Err(HsxError::ExternalTool(
            "Pandoc DOCX generation failed.".into(),
        ))
    }
}

/// Simple UUID v4-like random string for temp file naming (no uuid dep in core).
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:x}-{:x}", ts, std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_pandoc_returns_version_or_tool_error() {
        // Either Pandoc is installed (Ok) or not (ExternalTool error).
        match check_pandoc() {
            Ok(version) => assert!(version.to_lowercase().contains("pandoc")),
            Err(HsxError::ExternalTool(msg)) => assert!(msg.contains("Pandoc")),
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    fn check_typst_returns_version_or_tool_error() {
        // Either Typst is installed (Ok) or not (ExternalTool error).
        match check_typst() {
            Ok(version) => assert!(!version.is_empty()),
            Err(HsxError::ExternalTool(msg)) => assert!(msg.contains("Typst")),
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    fn typst_available_returns_bool() {
        // Smoke test: returns a bool without panicking.
        let _ = typst_available();
    }
}
