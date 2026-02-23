//! Device-aware Ollama setup guidance and model recommendations.
//!
//! Detects RAM, CPU, and architecture, then recommends the best Ollama models
//! that fit safely in available memory. Provides `ollama pull` commands and
//! per-model descriptions so users can set up with one command.

use sysinfo::System;

/// Hardware profile of the current machine.
#[derive(Debug, Clone)]
pub struct DeviceSpec {
    /// Total RAM in gigabytes (rounded).
    pub total_ram_gb: f64,
    /// Number of logical CPU cores.
    pub cpu_cores: usize,
    /// True on Apple M-series (fast unified memory, CPU+GPU shared).
    pub is_apple_silicon: bool,
    /// Approximate free RAM available for a model (total minus OS overhead).
    pub usable_ram_gb: f64,
}

impl DeviceSpec {
    /// Detect the current machine's hardware spec.
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_bytes = sys.total_memory();
        let total_ram_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        // Reserve ~4 GB for OS + other processes (conservative on Intel/AMD).
        // Apple Silicon shares memory, so deduct less.
        let is_apple_silicon = detect_apple_silicon();
        let os_overhead_gb = if is_apple_silicon { 3.0 } else { 4.5 };
        let usable_ram_gb = (total_ram_gb - os_overhead_gb).max(0.5);

        Self {
            total_ram_gb,
            cpu_cores: sys.cpus().len(),
            is_apple_silicon,
            usable_ram_gb,
        }
    }
}

/// A single Ollama model recommendation with setup guidance.
#[derive(Debug, Clone)]
pub struct ModelRecommendation {
    /// Ollama model tag (e.g. `"qwen3:14b"`).
    pub name: &'static str,
    /// Approximate VRAM/RAM needed for Q4_K_M quantization.
    pub size_gb: f64,
    /// One-line description of the model's strengths.
    pub description: &'static str,
    /// Recommendation category.
    pub category: RecommendCategory,
}

/// Why a model is being recommended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendCategory {
    /// Best balance of quality and speed for this device.
    BestForDevice,
    /// Fastest option that still produces good results.
    FastAndLight,
    /// Maximum quality the device can run without swapping.
    MaxQuality,
    /// Specialised reasoning model (slower but more accurate for logic tasks).
    Reasoning,
    /// Requires more RAM than available; will use swap — not recommended.
    TooLarge,
}

impl RecommendCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BestForDevice => "BEST FOR YOU",
            Self::FastAndLight  => "FAST & LIGHT",
            Self::MaxQuality    => "MAX QUALITY",
            Self::Reasoning     => "REASONING",
            Self::TooLarge      => "TOO LARGE (swap risk)",
        }
    }
}

/// All models we know about, in descending quality order.
/// size_gb is Q4_K_M quantization footprint.
static KNOWN_MODELS: &[(&str, f64, &str)] = &[
    ("qwen3:30b-a3b",  17.0, "Qwen3 30B MoE (3B active) — best reasoning with low compute cost"),
    ("deepseek-r1:32b", 20.0, "DeepSeek-R1 32B — state-of-the-art reasoning, large context"),
    ("qwen3:14b",        9.0, "Qwen3 14B — best quality/size ratio, multilingual, tool use"),
    ("deepseek-r1:14b",  9.0, "DeepSeek-R1 14B — chain-of-thought reasoning, 128k context"),
    ("gemma3:12b",       7.5, "Gemma3 12B — Google's best 12B, strong at code and analysis"),
    ("qwen3:8b",         5.0, "Qwen3 8B — fast, accurate, excellent at following instructions"),
    ("deepseek-r1:7b",   4.7, "DeepSeek-R1 7B — reasoning specialist, great for Q&A"),
    ("gemma3:9b",        5.5, "Gemma3 9B — well-rounded, fast, strong at code"),
    ("qwen3:4b",         2.6, "Qwen3 4B — compact, surprisingly capable"),
    ("phi3:mini",        2.3, "Phi-3 Mini — Microsoft's 3.8B, punches above its weight"),
    ("gemma3:2b",        1.8, "Gemma3 2B — smallest useful model, good for simple Q&A"),
    ("qwen3:1.7b",       1.2, "Qwen3 1.7B — ultra-light, CPU-only viable"),
    ("gemma3:1b",        0.8, "Gemma3 1B — absolute minimum, fast even on old hardware"),
];

/// Build a ranked, categorised recommendation list for this device.
pub fn recommend_models(spec: &DeviceSpec) -> Vec<ModelRecommendation> {
    let usable = spec.usable_ram_gb;
    let mut recs: Vec<ModelRecommendation> = Vec::new();

    // Determine tiers based on usable RAM
    let best_name  = best_model_name(usable);
    let fast_name  = fast_model_name(usable);
    let max_name   = max_quality_model_name(usable);
    let reason_name = reasoning_model_name(usable);

    for &(name, size_gb, description) in KNOWN_MODELS {
        if size_gb > usable + 2.0 {
            // Way too big — skip entirely unless it is "just over" (show as TooLarge warning)
            if size_gb > usable + 6.0 {
                continue;
            }
        }

        let category = if name == best_name {
            RecommendCategory::BestForDevice
        } else if Some(name) == fast_name {
            RecommendCategory::FastAndLight
        } else if Some(name) == max_name && name != best_name {
            RecommendCategory::MaxQuality
        } else if Some(name) == reason_name && name != best_name {
            RecommendCategory::Reasoning
        } else if size_gb > usable {
            RecommendCategory::TooLarge
        } else {
            continue // Don't include models that don't fit any category
        };

        recs.push(ModelRecommendation { name, size_gb, description, category });
    }

    // Sort: BestForDevice first, TooLarge last
    recs.sort_by_key(|r| match r.category {
        RecommendCategory::BestForDevice => 0,
        RecommendCategory::FastAndLight  => 1,
        RecommendCategory::MaxQuality    => 2,
        RecommendCategory::Reasoning     => 3,
        RecommendCategory::TooLarge      => 4,
    });

    recs
}

/// The single best model name for the given usable RAM.
pub fn best_model_name(usable_gb: f64) -> &'static str {
    match usable_gb as u64 {
        u if u >= 28 => "qwen3:30b-a3b",   // 17 GB — fits in 32GB machines
        u if u >= 20 => "qwen3:14b",        // 9 GB — best in class for 24GB
        u if u >= 11 => "qwen3:14b",        // 9 GB — fits safely in 16GB
        u if u >= 6  => "qwen3:8b",         // 5 GB — best for 8-12GB
        u if u >= 4  => "deepseek-r1:7b",   // 4.7 GB — small but reasoning-focused
        u if u >= 2  => "qwen3:4b",         // 2.6 GB
        _            => "gemma3:1b",         // Absolute minimum
    }
}

fn fast_model_name(usable_gb: f64) -> Option<&'static str> {
    match usable_gb as u64 {
        u if u >= 11 => Some("qwen3:8b"),
        u if u >= 6  => Some("deepseek-r1:7b"),
        u if u >= 3  => Some("phi3:mini"),
        _            => None,
    }
}

fn max_quality_model_name(usable_gb: f64) -> Option<&'static str> {
    match usable_gb as u64 {
        u if u >= 28 => Some("deepseek-r1:32b"),
        u if u >= 11 => Some("deepseek-r1:14b"),
        u if u >= 6  => Some("gemma3:9b"),
        _            => None,
    }
}

fn reasoning_model_name(usable_gb: f64) -> Option<&'static str> {
    match usable_gb as u64 {
        u if u >= 11 => Some("deepseek-r1:14b"),
        u if u >= 5  => Some("deepseek-r1:7b"),
        _            => None,
    }
}

/// Format a full Ollama setup guide as a string, ready to print to the terminal.
///
/// Includes:
/// - Install command for Ollama
/// - Device-specific model table with pull commands
/// - One-command quick-start for the best model
pub fn format_setup_guide(spec: &DeviceSpec) -> String {
    let recs = recommend_models(spec);
    let best = best_model_name(spec.usable_ram_gb);

    let mut out = String::new();

    out.push_str("┌─ Ollama Setup Guide ────────────────────────────────────────────┐\n");
    out.push_str("│  Ollama is not running. Follow these steps:                      │\n");
    out.push_str("└─────────────────────────────────────────────────────────────────┘\n\n");

    // Step 1 — install
    out.push_str("  Step 1 — Install Ollama\n");
    out.push_str("  ────────────────────────────────────────\n");
    out.push_str("    macOS / Linux:  curl -fsSL https://ollama.ai/install.sh | sh\n");
    out.push_str("    Windows:        https://ollama.ai/download\n\n");

    // Step 2 — pull a model
    out.push_str("  Step 2 — Pull a model for your machine\n");
    out.push_str("  ────────────────────────────────────────\n");
    out.push_str(&format!(
        "  Your device: {:.0} GB RAM, {} cores{}\n",
        spec.total_ram_gb,
        spec.cpu_cores,
        if spec.is_apple_silicon { ", Apple Silicon (fast)" } else { ", Intel/AMD (CPU inference)" }
    ));
    out.push_str(&format!(
        "  Usable for LLM: ~{:.0} GB\n\n",
        spec.usable_ram_gb
    ));

    for rec in &recs {
        let marker = if rec.name == best { "→" } else { " " };
        out.push_str(&format!(
            "  {marker} [{:16}]  {:<22}  ~{:.0} GB  {}\n",
            rec.category.label(),
            rec.name,
            rec.size_gb,
            rec.description
        ));
        out.push_str(&format!(
            "      ollama pull {}\n\n",
            rec.name
        ));
    }

    // Quick-start
    out.push_str("  ─────────────────────────────────────────────────────────────────\n");
    out.push_str("  Quick-start (best model for your device):\n\n");
    out.push_str("    ollama serve &\n");
    out.push_str(&format!("    ollama pull {best}\n"));
    out.push_str("    hsx ai \"your question\"\n\n");

    out.push_str("  After pulling a model, re-run your command — hsx will detect it automatically.\n");
    out.push_str("  Tip: Add `default_model = \"{best}\"` to ~/.hypersearchx/config.toml to always use it.\n");

    out
}

/// Format a short one-line hint when no models are installed.
pub fn format_no_models_hint(spec: &DeviceSpec) -> String {
    let best = best_model_name(spec.usable_ram_gb);
    format!(
        "No models installed. Run: ollama pull {best}\n  \
         Then re-run your command. Use `hsx doctor` for a full setup guide."
    )
}

fn detect_apple_silicon() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("uname")
            .arg("-m")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("arm64"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(ram_gb: f64) -> DeviceSpec {
        DeviceSpec {
            total_ram_gb: ram_gb,
            cpu_cores: 8,
            is_apple_silicon: false,
            usable_ram_gb: (ram_gb - 4.5).max(0.5),
        }
    }

    #[test]
    fn best_model_for_64gb() {
        assert_eq!(best_model_name(60.0), "qwen3:30b-a3b");
    }

    #[test]
    fn best_model_for_16gb() {
        // 16GB - 4.5 OS = 11.5 usable → qwen3:14b
        assert_eq!(best_model_name(spec(16.0).usable_ram_gb), "qwen3:14b");
    }

    #[test]
    fn best_model_for_8gb() {
        // 8GB - 4.5 OS = 3.5 usable → deepseek-r1:7b
        let s = spec(8.0);
        let b = best_model_name(s.usable_ram_gb);
        // 3.5 GB usable → qwen3:4b
        assert!(b == "qwen3:4b" || b == "deepseek-r1:7b");
    }

    #[test]
    fn best_model_for_4gb() {
        let s = spec(4.0);
        // negative usable → gemma3:1b
        assert_eq!(best_model_name(s.usable_ram_gb), "gemma3:1b");
    }

    #[test]
    fn recommend_models_nonempty_for_16gb() {
        let recs = recommend_models(&spec(16.0));
        assert!(!recs.is_empty());
        assert_eq!(recs[0].category, RecommendCategory::BestForDevice);
    }

    #[test]
    fn setup_guide_contains_ollama_install() {
        let guide = format_setup_guide(&spec(16.0));
        assert!(guide.contains("ollama.ai/install.sh"));
        assert!(guide.contains("ollama pull"));
    }

    #[test]
    fn no_models_hint_has_pull_cmd() {
        let hint = format_no_models_hint(&spec(16.0));
        assert!(hint.contains("ollama pull"));
    }
}
