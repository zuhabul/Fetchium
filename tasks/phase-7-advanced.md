# Phase 7: Advanced Features & Polish

> **Duration:** Weeks 37-48 | **Priority:** P3
> **Depends On:** Phase 6 complete (Intelligence Algorithms)
> **PRD Sections:** 29, 33, 34, 36, 37, 38, 39, 42

---

## Overview

Phase 7 implements the advanced features that differentiate Fetchium from a research tool into a platform. Each epic is relatively self-contained and can be developed in parallel by different contributors.

---

## Epic 7.1: Plugin System

### P7-E1-T1: Plugin System with Dynamic Libraries and WASM

| Field | Value |
|-------|-------|
| **ID** | `P7-E1-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Build an extensible plugin system supporting both native dynamic libraries (via `libloading`) and WASM modules (via `wasmtime`). Define plugin traits for 6 plugin types: Backend, Extractor, Ranker, Formatter, Validator, AiProvider. Implement plugin registry, discovery, loading, and lifecycle management. |
| **PRD Ref** | 29 (Plugin & Extension System), 48 (libloading / wasmtime) |
| **Depends On** | Phase 4 complete (stable core API surface) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-core/src/plugin/mod.rs` | Plugin module root |
| `crates/fetchium-core/src/plugin/traits.rs` | Plugin trait definitions for all 6 types |
| `crates/fetchium-core/src/plugin/registry.rs` | Plugin registry |
| `crates/fetchium-core/src/plugin/loader.rs` | Dynamic library + WASM loader |
| `crates/fetchium-core/src/plugin/lifecycle.rs` | Init, start, stop, unload |
| `crates/fetchium-core/src/plugin/manifest.rs` | Plugin manifest (plugin.toml) parser |
| `crates/fetchium-cli/src/commands/plugin.rs` | CLI: install, list, create, remove |

#### Step-by-Step Implementation Guide

**Step 1: Define plugin traits**

```rust
// crates/fetchium-core/src/plugin/traits.rs

use async_trait::async_trait;

/// Base trait all plugins must implement.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn init(&mut self, config: &serde_json::Value) -> Result<(), crate::Error>;
    fn shutdown(&mut self) -> Result<(), crate::Error>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginType {
    Backend,
    Extractor,
    Ranker,
    Formatter,
    Validator,
    AiProvider,
}

/// Plugin that provides a search backend.
#[async_trait]
pub trait BackendPlugin: Plugin {
    async fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<crate::types::SearchResult>, crate::Error>;

    fn supported_features(&self) -> BackendFeatures;
}

#[derive(Debug, Default)]
pub struct BackendFeatures {
    pub supports_date_filter: bool,
    pub supports_language_filter: bool,
    pub supports_region_filter: bool,
    pub max_results_per_query: usize,
}

/// Plugin that provides content extraction.
#[async_trait]
pub trait ExtractorPlugin: Plugin {
    async fn extract(
        &self,
        url: &str,
        raw_html: &str,
    ) -> Result<crate::types::ExtractedContent, crate::Error>;

    fn supported_content_types(&self) -> Vec<String>;
}

/// Plugin that provides a ranking algorithm.
pub trait RankerPlugin: Plugin {
    fn rank(
        &self,
        query: &str,
        results: &mut [crate::types::SearchResult],
    ) -> Result<(), crate::Error>;
}

/// Plugin that provides an output format.
pub trait FormatterPlugin: Plugin {
    fn format(
        &self,
        data: &crate::types::AgentSearchResult,
    ) -> Result<String, crate::Error>;

    fn file_extension(&self) -> &str;
    fn mime_type(&self) -> &str;
}

/// Plugin that provides content validation.
pub trait ValidatorPlugin: Plugin {
    fn validate(
        &self,
        content: &crate::types::ExtractedContent,
        source: &crate::types::Source,
    ) -> Result<ValidationResult, crate::Error>;
}

/// Plugin that provides AI model access.
#[async_trait]
pub trait AiProviderPlugin: Plugin {
    async fn complete(
        &self,
        prompt: &str,
        params: &crate::ai::CompletionParams,
    ) -> Result<crate::ai::CompletionResult, crate::Error>;

    fn available_models(&self) -> Vec<String>;
}
```

**Step 2: Plugin registry and loader**

```rust
// crates/fetchium-core/src/plugin/registry.rs
use std::collections::HashMap;
use std::sync::Arc;

pub struct PluginRegistry {
    backends: HashMap<String, Arc<dyn BackendPlugin>>,
    extractors: HashMap<String, Arc<dyn ExtractorPlugin>>,
    rankers: HashMap<String, Arc<dyn RankerPlugin>>,
    formatters: HashMap<String, Arc<dyn FormatterPlugin>>,
    validators: HashMap<String, Arc<dyn ValidatorPlugin>>,
    ai_providers: HashMap<String, Arc<dyn AiProviderPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
            extractors: HashMap::new(),
            rankers: HashMap::new(),
            formatters: HashMap::new(),
            validators: HashMap::new(),
            ai_providers: HashMap::new(),
        }
    }

    /// Load all plugins from the plugin directory.
    pub fn load_all(&mut self) -> Result<(), crate::Error> {
        let plugin_dir = crate::config::data_dir().join("plugins");
        if !plugin_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.load_plugin(&path)?;
            }
        }
        Ok(())
    }

    fn load_plugin(&mut self, path: &std::path::Path) -> Result<(), crate::Error> {
        let manifest_path = path.join("plugin.toml");
        let manifest = super::manifest::PluginManifest::load(&manifest_path)?;

        match manifest.runtime.as_str() {
            "native" => {
                let lib_name = if cfg!(target_os = "macos") {
                    format!("lib{}.dylib", manifest.name)
                } else if cfg!(target_os = "linux") {
                    format!("lib{}.so", manifest.name)
                } else {
                    format!("{}.dll", manifest.name)
                };
                let lib_path = path.join(&lib_name);
                super::loader::load_native(&lib_path, &manifest, self)?;
            }
            "wasm" => {
                let wasm_path = path.join(format!("{}.wasm", manifest.name));
                super::loader::load_wasm(&wasm_path, &manifest, self)?;
            }
            other => {
                tracing::warn!(runtime = other, "Unknown plugin runtime, skipping");
            }
        }
        Ok(())
    }

    pub fn get_backend(&self, name: &str) -> Option<Arc<dyn BackendPlugin>> {
        self.backends.get(name).cloned()
    }

    pub fn all_backends(&self) -> Vec<Arc<dyn BackendPlugin>> {
        self.backends.values().cloned().collect()
    }
}

// crates/fetchium-core/src/plugin/manifest.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub plugin_type: String,  // backend, extractor, ranker, formatter, validator, ai_provider
    pub runtime: String,      // native, wasm
    pub description: String,
    pub author: Option<String>,
    pub config_schema: Option<serde_json::Value>,
}
```

**Step 3: CLI commands**

```rust
// crates/fetchium-cli/src/commands/plugin.rs
use clap::Subcommand;

#[derive(Subcommand)]
pub enum PluginCommand {
    /// Install a plugin from a path or URL
    Install { source: String },
    /// List installed plugins
    List,
    /// Create a new plugin scaffold
    Create {
        name: String,
        #[arg(long, default_value = "backend")]
        plugin_type: String,
    },
    /// Remove an installed plugin
    Remove { name: String },
    /// Show plugin info
    Info { name: String },
}
```

#### Acceptance Criteria

- [x] `fetchium plugin create my-arxiv-backend --type backend` scaffolds a new plugin project
- [x] Native plugins load via `libloading` on macOS/Linux/Windows
- [x] WASM plugins load via `wasmtime` (feature-gated)
- [x] Plugin manifest (`plugin.toml`) defines name, version, type, and config schema
- [x] `fetchium plugin list` shows installed plugins with status
- [x] `fetchium plugin install ./my-plugin` copies plugin to `~/.fetchium/plugins/`
- [x] Backend plugins integrate with the search orchestrator
- [x] Extractor plugins integrate with the CEP pipeline
- [x] Plugins are isolated: a crashing plugin does not crash the host

#### Pitfalls

- **ABI stability**: Native Rust plugins require matching ABI. Use `extern "C"` functions and C-compatible types at the FFI boundary, or define a stable vtable.
- **WASM limitations**: WASM plugins cannot make network requests directly. Provide host functions (imports) for HTTP, logging, and storage.
- **Security**: Untrusted plugins could access the filesystem. Consider sandboxing native plugins and using WASM's built-in sandboxing for untrusted code.
- **Version compatibility**: Plugin trait changes will break existing plugins. Version the plugin API and provide compatibility shims.

---

## Epic 7.2: Privacy Modes

### P7-E2-T1: Privacy Modes (Private, Tor, Air-Gap, Redact-PII, Auto-Expire, Cache Encryption)

| Field | Value |
|-------|-------|
| **ID** | `P7-E2-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Implement the 4 privacy modes (standard, private, tor, air-gap) plus PII redaction, auto-expire, and cache encryption features. |
| **PRD Ref** | 36 (Privacy-First Architecture), 36.1-36.3 |
| **Depends On** | `P1-E6` (cache), `P5-E1-T3` (local index for air-gap mode) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-core/src/privacy/mod.rs` | Privacy module root |
| `crates/fetchium-core/src/privacy/modes.rs` | Mode definitions and enforcement |
| `crates/fetchium-core/src/privacy/redact.rs` | PII redaction engine |
| `crates/fetchium-core/src/privacy/expiry.rs` | Auto-expiring research artifacts |
| `crates/fetchium-core/src/privacy/encryption.rs` | Cache encryption at rest |
| `crates/fetchium-core/src/privacy/tor.rs` | Tor SOCKS5 proxy integration |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-core/src/privacy/modes.rs

#[derive(Debug, Clone, PartialEq)]
pub enum PrivacyMode {
    Standard,  // Normal operation
    Private,   // No cache, no history, no learning
    Tor,       // Route through Tor network
    AirGap,    // Local index only, zero network
}

/// Apply privacy mode to the HTTP client and pipeline config.
pub fn apply_mode(
    mode: &PrivacyMode,
    config: &mut crate::config::RuntimeConfig,
) -> Result<(), crate::Error> {
    match mode {
        PrivacyMode::Standard => {}
        PrivacyMode::Private => {
            config.cache_enabled = false;
            config.pie_enabled = false;
            config.history_enabled = false;
            config.embedding_cache_enabled = false;
            tracing::info!("Private mode: cache, history, and learning disabled");
        }
        PrivacyMode::Tor => {
            config.proxy = Some("socks5://127.0.0.1:9050".to_string());
            config.user_agent = "Mozilla/5.0 (Windows NT 10.0; rv:128.0) Gecko/20100101 Firefox/128.0".to_string();
            tracing::info!("Tor mode: routing through SOCKS5 proxy");
        }
        PrivacyMode::AirGap => {
            config.network_enabled = false;
            config.backends = vec![]; // No remote backends
            config.local_index_only = true;
            tracing::info!("Air-gap mode: network disabled, local index only");
        }
    }
    Ok(())
}

// crates/fetchium-core/src/privacy/redact.rs

/// PII patterns to detect and redact.
const PII_PATTERNS: &[(&str, &str)] = &[
    (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL REDACTED]"),
    (r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b", "[PHONE REDACTED]"),
    (r"\b\d{3}-\d{2}-\d{4}\b", "[SSN REDACTED]"),
    (r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b", "[CC REDACTED]"),
    (r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "[IP REDACTED]"),
];

pub fn redact_pii(text: &str) -> String {
    let mut result = text.to_string();
    for (pattern, replacement) in PII_PATTERNS {
        let re = regex::Regex::new(pattern).unwrap();
        result = re.replace_all(&result, *replacement).to_string();
    }
    result
}

// crates/fetchium-core/src/privacy/encryption.rs

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct CacheEncryption {
    cipher: Aes256Gcm,
}

impl CacheEncryption {
    /// Create encryption engine from a user-provided passphrase.
    /// Derives a 256-bit key via Argon2.
    pub fn new(passphrase: &str) -> Result<Self, crate::Error> {
        let salt = b"fetchium-cache-v1";
        let config = argon2::Config::default();
        let hash = argon2::hash_raw(passphrase.as_bytes(), salt, &config)?;
        let key = Key::from_slice(&hash[..32]);
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, crate::Error> {
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self.cipher.encrypt(nonce, data)
            .map_err(|e| crate::Error::Encryption(e.to_string()))?;
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, crate::Error> {
        if data.len() < 12 {
            return Err(crate::Error::Encryption("Data too short".into()));
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| crate::Error::Encryption(e.to_string()))
    }
}

// crates/fetchium-core/src/privacy/expiry.rs

/// Schedule auto-expiration of research artifacts.
pub fn schedule_expiry(
    result_id: &str,
    expire_after: std::time::Duration,
    db: &rusqlite::Connection,
) -> Result<(), crate::Error> {
    let expire_at = chrono::Utc::now() + chrono::Duration::from_std(expire_after)?;
    db.execute(
        "INSERT INTO expiry_schedule (result_id, expire_at) VALUES (?1, ?2)",
        rusqlite::params![result_id, expire_at.to_rfc3339()],
    )?;
    Ok(())
}

/// Purge all expired research artifacts.
pub fn purge_expired(db: &rusqlite::Connection) -> Result<usize, crate::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    // Delete from all related tables
    let count = db.execute(
        "DELETE FROM pds_cache WHERE result_id IN
         (SELECT result_id FROM expiry_schedule WHERE expire_at < ?1)",
        [&now],
    )?;
    db.execute(
        "DELETE FROM expiry_schedule WHERE expire_at < ?1",
        [&now],
    )?;
    Ok(count)
}
```

#### Acceptance Criteria

- [x] `fetchium search "query" --private` leaves no traces (no cache, no history, no PIE updates)
- [x] `fetchium search "query" --tor` routes requests through Tor SOCKS5 proxy
- [x] `fetchium search "query" --air-gap` uses only local index (clear error if no local index)
- [x] `fetchium research "topic" --redact-pii` strips emails, phones, SSNs, IPs from output
- [x] `fetchium research "topic" --auto-expire 24h` schedules artifact deletion
- [x] `fetchium config set privacy.cache_encryption_key <passphrase>` enables encrypted cache
- [x] Encrypted cache data is unreadable without the passphrase
- [x] `purge_expired()` runs on startup and cleans up expired artifacts
- [x] Tor mode requires Tor service running locally (clear error message if not)

#### Pitfalls

- **Tor availability**: Tor must be installed and running. Detect with a socket connection test and provide install instructions.
- **Encryption performance**: AES-GCM encryption of cache entries adds ~1ms per entry. Acceptable for disk cache, not for in-memory LRU.
- **PII false positives**: The regex patterns may flag non-PII numbers. Use conservative patterns and allow user overrides.

---

## Epic 7.3: Collaborative Research

### P7-E3-T1: Collaborative Research Workspaces

| Field | Value |
|-------|-------|
| **ID** | `P7-E3-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Implement shared research workspaces with branching, merging, and shared knowledge graphs. Workspaces sync via local filesystem (NFS/SMB) or Git-managed files. |
| **PRD Ref** | 37 (Collaborative Research Protocol), 37.1-37.2 |
| **Depends On** | `P6-E1-T1` (PIE knowledge graph), `P3-E2` (EGP evidence graphs) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-core/src/collab/mod.rs` | Collaboration module |
| `crates/fetchium-core/src/collab/workspace.rs` | Workspace CRUD |
| `crates/fetchium-core/src/collab/branch.rs` | Research session branching |
| `crates/fetchium-core/src/collab/merge.rs` | Findings merging with dedup |
| `crates/fetchium-core/src/collab/sync.rs` | Filesystem/Git sync |
| `crates/fetchium-cli/src/commands/workspace.rs` | CLI commands |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-core/src/collab/workspace.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub members: Vec<String>,  // user identifiers
    pub sync_method: SyncMethod,
    pub path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMethod {
    Local { shared_dir: std::path::PathBuf },
    Git { remote_url: String },
}

impl Workspace {
    pub fn create(name: &str, path: &std::path::Path) -> Result<Self, crate::Error> {
        let workspace = Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            members: vec![],
            sync_method: SyncMethod::Local { shared_dir: path.to_path_buf() },
            path: path.to_path_buf(),
        };

        // Create workspace directory structure
        std::fs::create_dir_all(path.join("sessions"))?;
        std::fs::create_dir_all(path.join("knowledge_graph"))?;
        std::fs::create_dir_all(path.join("evidence"))?;

        // Write workspace manifest
        let manifest = serde_json::to_string_pretty(&workspace)?;
        std::fs::write(path.join("workspace.json"), manifest)?;

        Ok(workspace)
    }
}

// crates/fetchium-core/src/collab/branch.rs

/// Fork a research session to explore an alternative approach.
pub fn fork_session(
    workspace: &Workspace,
    session_id: &str,
    new_name: &str,
) -> Result<String, crate::Error> {
    let source_dir = workspace.path.join("sessions").join(session_id);
    let new_id = uuid::Uuid::new_v4().to_string();
    let target_dir = workspace.path.join("sessions").join(&new_id);

    // Deep copy the session directory
    copy_dir_recursive(&source_dir, &target_dir)?;

    // Update session metadata
    let mut meta: SessionMeta = serde_json::from_str(
        &std::fs::read_to_string(target_dir.join("session.json"))?
    )?;
    meta.id = new_id.clone();
    meta.name = new_name.to_string();
    meta.forked_from = Some(session_id.to_string());
    meta.created_at = chrono::Utc::now();
    std::fs::write(
        target_dir.join("session.json"),
        serde_json::to_string_pretty(&meta)?,
    )?;

    Ok(new_id)
}

// crates/fetchium-core/src/collab/merge.rs

/// Merge findings from two research sessions with deduplication.
pub fn merge_sessions(
    workspace: &Workspace,
    session_a: &str,
    session_b: &str,
) -> Result<MergeResult, crate::Error> {
    let findings_a = load_findings(workspace, session_a)?;
    let findings_b = load_findings(workspace, session_b)?;

    let mut merged = findings_a.clone();
    let mut added = 0;
    let mut deduplicated = 0;

    for finding in &findings_b {
        let is_duplicate = merged.iter().any(|existing| {
            // Use SimHash or semantic similarity for dedup
            similarity(&existing.content, &finding.content) > 0.85
        });

        if is_duplicate {
            deduplicated += 1;
        } else {
            merged.push(finding.clone());
            added += 1;
        }
    }

    Ok(MergeResult { merged, added, deduplicated })
}
```

#### Acceptance Criteria

- [x] `fetchium workspace create "project-alpha"` creates a workspace directory
- [x] `fetchium research "topic" --workspace project-alpha` stores results in the workspace
- [x] `fetchium research fork <session-id> --name "alt-approach"` creates a branch
- [x] `fetchium research merge <session-a> <session-b> --deduplicate` combines findings
- [x] Workspace manifest (`workspace.json`) tracks metadata and members
- [x] Git-based sync: `fetchium workspace sync` commits and pushes changes
- [x] Merged sessions preserve citation integrity across sources

#### Pitfalls

- **Merge conflicts**: Two researchers may find contradictory evidence. Flag conflicts rather than silently merging.
- **File locking**: If two processes write to the same workspace simultaneously, data corruption can occur. Use file locks or SQLite.
- **Large workspaces**: Research workspaces can grow large. Implement periodic cleanup and archiving.

---

## Epic 7.4: Domain-Specific Intelligence Modes

### P7-E4-T1: Domain-Specific Modes (Academic, Code, Legal, Financial, Medical, Security)

| Field | Value |
|-------|-------|
| **ID** | `P7-E4-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Implement 6 pre-configured domain-specific intelligence modes that adjust backend priorities, ranking weights, special features, and output formatting per domain. |
| **PRD Ref** | 38 (Domain-Specific Intelligence Modes) |
| **Depends On** | `P2-E4` (HyperFusion ranking), `P2-E2` (multiple backends) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-core/src/domains/mod.rs` | Domain mode module |
| `crates/fetchium-core/src/domains/academic.rs` | Academic mode config |
| `crates/fetchium-core/src/domains/code.rs` | Code intelligence mode |
| `crates/fetchium-core/src/domains/legal.rs` | Legal research mode |
| `crates/fetchium-core/src/domains/financial.rs` | Financial analysis mode |
| `crates/fetchium-core/src/domains/medical.rs` | Medical/scientific mode |
| `crates/fetchium-core/src/domains/security.rs` | Cybersecurity mode |
| `data/domain_configs/` | TOML config files per domain |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-core/src/domains/mod.rs

#[derive(Debug, Clone)]
pub struct DomainMode {
    pub name: String,
    pub backends_priority: Vec<String>,       // ordered backend list
    pub ranking_overrides: RankingOverrides,
    pub special_features: Vec<SpecialFeature>,
    pub output_tweaks: OutputTweaks,
}

#[derive(Debug, Clone)]
pub struct RankingOverrides {
    pub authority_weight_multiplier: f64,
    pub temporal_weight_multiplier: f64,
    pub evidence_weight_multiplier: f64,
    pub extra_signals: Vec<ExtraSignal>,
}

#[derive(Debug, Clone)]
pub enum SpecialFeature {
    CitationGraph,           // academic
    DependencyAnalysis,      // code
    PrecedentMapping,        // legal
    TrendAnalysis,           // financial
    EvidenceGrading,         // medical
    CvssScoring,             // security
    BibTexExport,            // academic
    LicenseCheck,            // code
}

pub fn get_mode(name: &str) -> Result<DomainMode, crate::Error> {
    match name {
        "academic" => Ok(academic::mode()),
        "code" => Ok(code::mode()),
        "legal" => Ok(legal::mode()),
        "financial" => Ok(financial::mode()),
        "medical" => Ok(medical::mode()),
        "security" => Ok(security::mode()),
        _ => Err(crate::Error::Config(format!("Unknown domain mode: {name}"))),
    }
}

// crates/fetchium-core/src/domains/academic.rs
pub fn mode() -> DomainMode {
    DomainMode {
        name: "academic".into(),
        backends_priority: vec![
            "arxiv".into(), "scholar".into(), "semantic_scholar".into(),
            "pubmed".into(), "duckduckgo".into(),
        ],
        ranking_overrides: RankingOverrides {
            authority_weight_multiplier: 2.0,  // Peer review matters
            temporal_weight_multiplier: 0.5,   // Older papers can be seminal
            evidence_weight_multiplier: 1.5,
            extra_signals: vec![
                ExtraSignal::CitationCount,
                ExtraSignal::PeerReviewStatus,
                ExtraSignal::ImpactFactor,
            ],
        },
        special_features: vec![
            SpecialFeature::CitationGraph,
            SpecialFeature::BibTexExport,
        ],
        output_tweaks: OutputTweaks {
            default_citation_style: "apa".into(),
            include_methodology: true,
            include_replication_status: true,
        },
    }
}

// crates/fetchium-core/src/domains/security.rs
pub fn mode() -> DomainMode {
    DomainMode {
        name: "security".into(),
        backends_priority: vec![
            "nvd".into(), "cve".into(), "github_advisories".into(),
            "duckduckgo".into(), "stackoverflow".into(),
        ],
        ranking_overrides: RankingOverrides {
            authority_weight_multiplier: 1.5,
            temporal_weight_multiplier: 3.0,  // Recency critical for CVEs
            evidence_weight_multiplier: 1.0,
            extra_signals: vec![
                ExtraSignal::CvssSeverity,
                ExtraSignal::ExploitAvailability,
                ExtraSignal::PatchStatus,
            ],
        },
        special_features: vec![
            SpecialFeature::CvssScoring,
        ],
        output_tweaks: OutputTweaks {
            default_citation_style: "inline".into(),
            include_affected_versions: true,
            include_patch_links: true,
        },
    }
}
```

#### Acceptance Criteria

- [x] `fetchium research "topic" --mode academic` uses ArXiv/Scholar backends with citation-heavy ranking
- [x] `fetchium research "topic" --mode code` prioritizes GitHub/StackOverflow with code extraction
- [x] `fetchium research "topic" --mode security` prioritizes NVD/CVE with CVSS scoring
- [x] Each mode adjusts HyperFusion weights per its domain requirements
- [x] Domain-specific output features (BibTeX for academic, CVSS for security) are enabled
- [x] `fetchium research --mode medical` enables evidence grading (I-V)
- [x] Modes are extensible via TOML config files in `data/domain_configs/`

#### Pitfalls

- **Backend availability**: Not all domain-specific backends may be implemented. Gracefully fall back to general backends.
- **Over-tuning**: Domain-specific weight multipliers can produce extreme rankings. Clamp final weights to reasonable ranges.

---

## Epic 7.5: Proactive Intelligence

### P7-E5-T1: Subscriptions, Radar, Digest, Predictive Prefetch, Anomaly Detection

| Field | Value |
|-------|-------|
| **ID** | `P7-E5-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Implement proactive intelligence features: topic subscriptions with alerts, research radar (suggestions from history), intelligent digests, predictive prefetching, and anomaly detection on monitored sources. |
| **PRD Ref** | 33 (Proactive Intelligence & Anticipatory Search), 33.1-33.3 |
| **Depends On** | `P5-E5-T2` (monitor), `P6-E1-T1` (PIE QPM for predictions) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-core/src/proactive/mod.rs` | Proactive intelligence module |
| `crates/fetchium-core/src/proactive/subscription.rs` | Topic subscriptions |
| `crates/fetchium-core/src/proactive/radar.rs` | Research radar |
| `crates/fetchium-core/src/proactive/digest.rs` | Intelligent digests |
| `crates/fetchium-core/src/proactive/prefetch.rs` | Predictive prefetching |
| `crates/fetchium-core/src/proactive/anomaly.rs` | Anomaly detection |
| `crates/fetchium-cli/src/commands/subscribe.rs` | Subscription CLI |
| `crates/fetchium-cli/src/commands/radar.rs` | Radar CLI |
| `crates/fetchium-cli/src/commands/digest.rs` | Digest CLI |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-core/src/proactive/subscription.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub topic: String,
    pub interval: std::time::Duration,
    pub notify_method: NotifyMethod,
    pub threshold: Option<String>,   // e.g., "critical" for severity filtering
    pub last_checked: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotifyMethod {
    Stdout,
    Webhook { url: String },
    File { path: std::path::PathBuf },
}

/// Check a subscription for new findings.
pub async fn check_subscription(
    sub: &Subscription,
) -> Result<Vec<NewFinding>, crate::Error> {
    let since = sub.last_checked.unwrap_or(chrono::Utc::now() - chrono::Duration::days(1));

    let results = crate::search::search(
        &sub.topic,
        &crate::search::SearchConfig {
            max_sources: 10,
            date_after: Some(since.date_naive()),
            ..Default::default()
        },
    ).await?;

    // Filter to truly new findings (not seen before)
    let new_findings: Vec<NewFinding> = results.into_iter()
        .filter(|r| {
            r.published_date
                .map(|d| d > since.date_naive())
                .unwrap_or(true)
        })
        .map(|r| NewFinding {
            title: r.title,
            url: r.url,
            snippet: r.snippet,
            published: r.published_date,
        })
        .collect();

    Ok(new_findings)
}

// crates/fetchium-core/src/proactive/radar.rs

/// Generate research suggestions based on user's history and patterns.
pub async fn generate_radar(
    qpm: &crate::intelligence::pie::qpm::QueryPredictionModel,
    limit: usize,
) -> Result<Vec<RadarItem>, crate::Error> {
    // Get user's top topics
    let topics = qpm.top_topics(10)?;
    let mut radar_items = Vec::new();

    for (topic, _frequency) in &topics {
        // Search for recent developments in each topic
        let query = format!("{} latest news developments", topic);
        let results = crate::search::search(&query, &crate::search::SearchConfig {
            max_sources: 3,
            ..Default::default()
        }).await?;

        for result in results {
            radar_items.push(RadarItem {
                topic: topic.clone(),
                title: result.title,
                url: result.url,
                snippet: result.snippet,
                relevance_to_user: 0.8, // based on topic frequency
            });
        }
    }

    radar_items.sort_by(|a, b| b.relevance_to_user.partial_cmp(&a.relevance_to_user).unwrap());
    radar_items.truncate(limit);
    Ok(radar_items)
}
```

#### Acceptance Criteria

- [x] `fetchium subscribe "TypeScript breaking changes" --interval weekly` registers a subscription
- [x] `fetchium subscribe list` shows active subscriptions with next check time
- [x] `fetchium radar --limit 10` shows personalized research suggestions
- [x] `fetchium digest --period weekly --topics "rust,ai"` generates a digest of recent findings
- [x] Predictive prefetching pre-caches likely follow-up results
- [x] Anomaly detection flags significant content changes on monitored URLs
- [x] Webhook notifications send POST requests with structured JSON payloads

#### Pitfalls

- **Background execution**: Subscriptions require periodic checking. Either implement a daemon mode or trigger checks on each CLI invocation.
- **Notification reliability**: Webhook endpoints may be down. Implement retry with exponential backoff.
- **Radar noise**: If the user has diverse interests, radar items may be too scattered. Focus on the top 3-5 topics.

---

## Epic 7.6: Multimodal Content

### P7-E6-T1: Images/OCR, Video Transcripts, PDF Extraction, Chart-to-JSON

| Field | Value |
|-------|-------|
| **ID** | `P7-E6-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Add multimodal content understanding: image alt-text and OCR extraction, YouTube video transcript fetching, enhanced PDF extraction with layout awareness, and chart/graph interpretation to structured JSON data. |
| **PRD Ref** | 34 (Multimodal Content Understanding), 34.1-34.2 |
| **Depends On** | `P1-E1` (HTTP fetch), `P5-E6-T1` (PDF export) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-core/src/multimodal/mod.rs` | Multimodal module |
| `crates/fetchium-core/src/multimodal/ocr.rs` | OCR via Tesseract or vision model |
| `crates/fetchium-core/src/multimodal/video.rs` | YouTube transcript extraction |
| `crates/fetchium-core/src/multimodal/pdf.rs` | Enhanced PDF extraction |
| `crates/fetchium-core/src/multimodal/chart.rs` | Chart data extraction |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-core/src/multimodal/video.rs

/// Extract transcript from a YouTube video using the public transcript API.
pub async fn extract_youtube_transcript(
    video_url: &str,
) -> Result<Transcript, crate::Error> {
    let video_id = extract_youtube_id(video_url)?;

    // Try YouTube's timedtext API (no API key needed)
    let transcript_url = format!(
        "https://www.youtube.com/api/timedtext?v={}&lang=en&fmt=json3",
        video_id
    );

    let client = reqwest::Client::new();
    let resp = client.get(&transcript_url).send().await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        let events = data["events"].as_array()
            .ok_or_else(|| crate::Error::Extraction("No transcript events".into()))?;

        let segments: Vec<TranscriptSegment> = events.iter()
            .filter_map(|event| {
                let start_ms = event["tStartMs"].as_u64()?;
                let text = event["segs"].as_array()?
                    .iter()
                    .filter_map(|seg| seg["utf8"].as_str())
                    .collect::<Vec<_>>()
                    .join("");
                Some(TranscriptSegment {
                    start_ms,
                    text: text.trim().to_string(),
                })
            })
            .filter(|s| !s.text.is_empty())
            .collect();

        Ok(Transcript {
            video_id: video_id.to_string(),
            segments,
            full_text: segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" "),
        })
    } else {
        Err(crate::Error::Extraction("Transcript not available for this video".into()))
    }
}

fn extract_youtube_id(url: &str) -> Result<&str, crate::Error> {
    let parsed = url::Url::parse(url)?;
    parsed.query_pairs()
        .find(|(key, _)| key == "v")
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| crate::Error::Extraction("Invalid YouTube URL".into()))
        // Note: simplified; handle youtu.be short URLs too
}

// crates/fetchium-core/src/multimodal/pdf.rs

/// Extract text from PDF with layout awareness.
pub fn extract_pdf(path: &std::path::Path) -> Result<PdfContent, crate::Error> {
    let bytes = std::fs::read(path)?;
    let doc = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| crate::Error::Extraction(format!("PDF extraction failed: {e}")))?;

    Ok(PdfContent {
        text: doc,
        page_count: count_pages(&bytes)?,
        metadata: extract_pdf_metadata(&bytes)?,
    })
}
```

#### Acceptance Criteria

- [x] `fetchium fetch <youtube-url> --transcript` extracts video transcript with timestamps
- [x] `fetchium fetch <image-url> --ocr` extracts text from images (requires Tesseract)
- [x] `fetchium fetch <pdf-url>` extracts text from PDF files with layout preservation
- [x] `fetchium fetch <page> --multimodal` also extracts alt-text from images on the page
- [x] YouTube transcript extraction works without API keys
- [x] PDF metadata (title, author, page count) included in output

#### Pitfalls

- **YouTube API changes**: The timedtext API is undocumented and may change. Have fallback to page scraping for captions.
- **Tesseract dependency**: OCR requires Tesseract installed. Feature-gate this and provide clear error if missing.
- **PDF complexity**: Scanned PDFs need OCR; vector PDFs need text extraction. Detect and route appropriately.

---

## Epic 7.7: Self-Evolving Architecture

### P7-E7-T1: HyperFusion Auto-Tuning, CEP Retraining, A/B Testing

| Field | Value |
|-------|-------|
| **ID** | `P7-E7-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Implement self-evolving capabilities: automatic HyperFusion weight optimization based on user feedback, CEP predictor retraining from accumulated extraction data, and an A/B testing framework for algorithm variants. |
| **PRD Ref** | 39 (Self-Evolving Architecture), 39.1-39.4 |
| **Depends On** | `P6-E1-T1` (PIE for feedback storage), `P2-E4` (HyperFusion), `P5-E3-T1` (CEP predictor) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-core/src/evolve/mod.rs` | Self-evolving module |
| `crates/fetchium-core/src/evolve/automl.rs` | HyperFusion weight auto-tuning |
| `crates/fetchium-core/src/evolve/retrain.rs` | CEP predictor retraining |
| `crates/fetchium-core/src/evolve/ab_test.rs` | A/B testing framework |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-core/src/evolve/automl.rs

/// Auto-tune HyperFusion weights based on implicit user feedback.
/// When a user clicks result #3 instead of #1, result #3 was more relevant.
pub fn optimize_weights(
    feedback_events: &[FeedbackEvent],
    current_weights: &HyperFusionWeights,
) -> HyperFusionWeights {
    let learning_rate = 0.01;
    let mut weights = current_weights.clone();

    for event in feedback_events {
        // Compute gradient: which signals predicted the preferred result better?
        let preferred_signals = &event.preferred_result_signals;
        let demoted_signals = &event.demoted_result_signals;

        // Increase weight of signals where preferred > demoted
        // Decrease weight of signals where preferred < demoted
        weights.bm25 += learning_rate * (preferred_signals.bm25 - demoted_signals.bm25);
        weights.semantic += learning_rate * (preferred_signals.semantic - demoted_signals.semantic);
        weights.temporal += learning_rate * (preferred_signals.temporal - demoted_signals.temporal);
        weights.authority += learning_rate * (preferred_signals.authority - demoted_signals.authority);
        // ... etc for all 8 signals
    }

    // Normalize weights to sum to 1.0
    weights.normalize();

    // Clamp individual weights to [0.01, 0.5] to prevent degenerate solutions
    weights.clamp(0.01, 0.5);

    weights
}

// crates/fetchium-core/src/evolve/ab_test.rs

/// A/B testing framework for algorithm variants.
pub struct AbTest {
    pub name: String,
    pub variant_a: String,
    pub variant_b: String,
    pub allocation_ratio: f64,  // 0.5 = 50/50 split
    pub metrics: AbTestMetrics,
}

impl AbTest {
    /// Assign a request to variant A or B based on hash of query.
    pub fn assign(&self, query: &str) -> Variant {
        let hash = crc32fast::hash(query.as_bytes());
        let bucket = (hash as f64) / (u32::MAX as f64);
        if bucket < self.allocation_ratio {
            Variant::A
        } else {
            Variant::B
        }
    }

    /// Record a metric observation for a variant.
    pub fn record(&mut self, variant: Variant, metric: &str, value: f64) {
        match variant {
            Variant::A => self.metrics.a.entry(metric.to_string()).or_default().push(value),
            Variant::B => self.metrics.b.entry(metric.to_string()).or_default().push(value),
        }
    }

    /// Check if the test has statistical significance.
    pub fn is_significant(&self, metric: &str, confidence: f64) -> Option<Variant> {
        let a_values = self.metrics.a.get(metric)?;
        let b_values = self.metrics.b.get(metric)?;

        if a_values.len() < 30 || b_values.len() < 30 {
            return None; // Not enough data
        }

        let a_mean = a_values.iter().sum::<f64>() / a_values.len() as f64;
        let b_mean = b_values.iter().sum::<f64>() / b_values.len() as f64;

        // Simplified: compare means with threshold
        let diff_ratio = (a_mean - b_mean).abs() / a_mean.max(b_mean).max(0.001);
        if diff_ratio > (1.0 - confidence) {
            if a_mean > b_mean { Some(Variant::A) } else { Some(Variant::B) }
        } else {
            None
        }
    }
}
```

#### Acceptance Criteria

- [x] HyperFusion weights auto-adjust after 50+ implicit feedback events
- [x] Adjusted weights persist across sessions in PIE storage
- [x] CEP predictor retrains from accumulated extraction data periodically
- [x] A/B tests can be defined for ranking algorithms, extraction methods, etc.
- [x] `fetchium config show ranking-weights` displays current (auto-tuned) weights
- [x] A/B test results include sample sizes and significance levels

#### Pitfalls

- **Feedback sparsity**: Implicit feedback (which result the user opens) is noisy. Require many events before adjusting.
- **Catastrophic tuning**: Auto-tuning could make things worse. Keep a "last known good" checkpoint and revert if metrics degrade.
- **A/B test interference**: Running multiple A/B tests simultaneously can confound results. Run one test at a time.

---

## Epic 7.8: Interactive TUI

### P7-E8-T1: Interactive TUI with ratatui

| Field | Value |
|-------|-------|
| **ID** | `P7-E8-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Build an interactive terminal UI using `ratatui` with search, browse, preview, and evidence graph viewer panels. |
| **PRD Ref** | 42 Feature #225 (Interactive TUI for deep research) |
| **Depends On** | All Phase 1-4 commands, `P3-E2` (EGP) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-cli/src/tui/mod.rs` | TUI module |
| `crates/fetchium-cli/src/tui/app.rs` | Application state |
| `crates/fetchium-cli/src/tui/views/search.rs` | Search panel |
| `crates/fetchium-cli/src/tui/views/results.rs` | Results list |
| `crates/fetchium-cli/src/tui/views/preview.rs` | Content preview |
| `crates/fetchium-cli/src/tui/views/evidence.rs` | Evidence graph viewer |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-cli/src/tui/app.rs
use ratatui::prelude::*;
use crossterm::event::{self, Event, KeyCode};

pub struct App {
    pub state: AppState,
    pub search_input: String,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub preview_content: Option<String>,
    pub active_panel: Panel,
    pub should_quit: bool,
}

#[derive(Debug, PartialEq)]
pub enum Panel {
    Search,
    Results,
    Preview,
    EvidenceGraph,
}

impl App {
    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), crate::Error> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.active_panel {
                    Panel::Search => self.handle_search_input(key),
                    Panel::Results => self.handle_results_input(key).await,
                    Panel::Preview => self.handle_preview_input(key),
                    Panel::EvidenceGraph => self.handle_graph_input(key),
                }

                if key.code == KeyCode::Char('q') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    self.should_quit = true;
                }
            }

            if self.should_quit { break; }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // Search bar
                Constraint::Min(10),      // Main content
                Constraint::Length(1),    // Status bar
            ])
            .split(frame.area());

        // Search bar
        let search = Paragraph::new(self.search_input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Search"));
        frame.render_widget(search, layout[0]);

        // Main content: split horizontally
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),  // Results list
                Constraint::Percentage(65),  // Preview
            ])
            .split(layout[1]);

        // Results list
        let items: Vec<ListItem> = self.results.iter().enumerate()
            .map(|(i, r)| {
                let style = if i == self.selected_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("[{}] {}", i + 1, r.title)).style(style)
            })
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Results"));
        frame.render_widget(list, main_layout[0]);

        // Preview
        let preview_text = self.preview_content.as_deref().unwrap_or("Select a result to preview");
        let preview = Paragraph::new(preview_text)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Preview"));
        frame.render_widget(preview, main_layout[1]);

        // Status bar
        let status = Paragraph::new(format!(
            " {} results | Tab: switch panel | Enter: preview | q: quit",
            self.results.len()
        ));
        frame.render_widget(status, layout[2]);
    }
}
```

#### Acceptance Criteria

- [x] `fetchium tui` launches an interactive terminal UI
- [x] Search bar accepts input and triggers search on Enter
- [x] Results panel shows ranked results with scores
- [x] Preview panel shows extracted content for the selected result
- [x] Tab key switches between panels
- [x] Arrow keys navigate the results list
- [x] `e` key opens the evidence graph viewer for the current result
- [x] `q` or Ctrl+C exits the TUI
- [x] TUI handles terminal resize gracefully

#### Pitfalls

- **Async in TUI**: Running async searches inside a TUI event loop requires careful coordination. Use `tokio::spawn` for search operations and poll results in the render loop.
- **Terminal cleanup**: Always restore terminal state on exit (including panics). Use `std::panic::set_hook` to ensure cleanup.
- **Unicode**: Some terminal emulators handle Unicode width incorrectly. Use `unicode-width` crate for accurate column calculations.

---

## Epic 7.9: Framework Adapters

### P7-E9-T1: LangChain and CrewAI Python Package Adapters

| Field | Value |
|-------|-------|
| **ID** | `P7-E9-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Create Python packages that wrap the Fetchium CLI/REST API as LangChain Retriever and CrewAI Tool objects. Published as `fetchium-langchain` and `fetchium-crewai` on PyPI. |
| **PRD Ref** | 25 (Agent Framework Integration), 9 (Framework Adapters) |
| **Depends On** | `P4-E3` (REST API server) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `adapters/langchain/pyproject.toml` | LangChain adapter package |
| `adapters/langchain/fetchium_langchain/retriever.py` | LangChain Retriever class |
| `adapters/crewai/pyproject.toml` | CrewAI adapter package |
| `adapters/crewai/fetchium_crewai/tool.py` | CrewAI Tool class |

#### Step-by-Step Implementation Guide

```python
# adapters/langchain/fetchium_langchain/retriever.py
from typing import List, Optional
from langchain_core.documents import Document
from langchain_core.retrievers import BaseRetriever
import subprocess
import json

class FetchiumRetriever(BaseRetriever):
    """LangChain Retriever that uses Fetchium for token-efficient web search."""

    token_budget: int = 3000
    tier: str = "detailed"
    validate: bool = True
    max_sources: int = 10
    hsx_binary: str = "fetchium"  # or full path

    def _get_relevant_documents(self, query: str) -> List[Document]:
        cmd = [
            self.hsx_binary, "agent-search", query,
            "--budget", str(self.token_budget),
            "--tier", self.tier,
            "--format", "json",
            "--max-sources", str(self.max_sources),
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)

        if result.returncode != 0:
            raise RuntimeError(f"Fetchium failed: {result.stderr}")

        data = json.loads(result.stdout)
        documents = []

        for segment in data.get("segments", []):
            doc = Document(
                page_content=segment.get("content", ""),
                metadata={
                    "source": segment.get("source_url", ""),
                    "relevance": segment.get("relevance", 0),
                    "type": segment.get("type", "paragraph"),
                    "tokens": segment.get("tokens", 0),
                },
            )
            documents.append(doc)

        return documents


# adapters/crewai/fetchium_crewai/tool.py
from crewai_tools import BaseTool
import subprocess
import json

class FetchiumTool(BaseTool):
    """CrewAI Tool that uses Fetchium for web search and research."""

    name: str = "Fetchium Web Search"
    description: str = (
        "Search the web using Fetchium. Returns token-efficient, "
        "validated results with citations. Input should be a search query string."
    )
    token_budget: int = 2000
    tier: str = "summary"

    def _run(self, query: str) -> str:
        cmd = [
            "fetchium", "agent-search", query,
            "--budget", str(self.token_budget),
            "--tier", self.tier,
            "--format", "json",
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)

        if result.returncode != 0:
            return f"Search failed: {result.stderr}"

        data = json.loads(result.stdout)

        # CrewAI expects a string output
        output_parts = []
        for finding in data.get("findings", []):
            output_parts.append(
                f"- {finding['claim']} [Source: {finding.get('source_url', 'N/A')}]"
            )

        return "\n".join(output_parts) if output_parts else "No results found."
```

#### Acceptance Criteria

- [x] `pip install fetchium-langchain` installs the LangChain adapter
- [x] `FetchiumRetriever(token_budget=3000).invoke("query")` returns LangChain `Document` objects
- [x] `pip install fetchium-crewai` installs the CrewAI adapter
- [x] `FetchiumTool().run("query")` returns string output suitable for CrewAI agents
- [x] Both adapters work via CLI subprocess (no network dependency beyond fetchium binary)
- [x] Both adapters also support REST API mode when `fetchium serve --api` is running
- [x] Error handling: clear messages when fetchium binary is not found

#### Pitfalls

- **Binary path**: The `fetchium` binary must be in PATH. Provide configuration for custom paths.
- **Subprocess timeout**: Long research queries may exceed the default timeout. Make it configurable.
- **REST vs CLI**: Subprocess invocation has ~200ms overhead per call. For production use, recommend REST API mode.

---

## Epic 7.10: Shell Completions

### P7-E10-T1: Shell Completions for bash, zsh, and fish

| Field | Value |
|-------|-------|
| **ID** | `P7-E10-T1` |
| **Status** | `TODO` |
| **Priority** | P3 |
| **Description** | Generate shell completions for bash, zsh, and fish using `clap_complete`. Add a `fetchium completions` command that outputs the completion script. |
| **PRD Ref** | 42 Feature #224 (Shell completions), 48 (clap_complete) |
| **Depends On** | `P0-E3` (CLI skeleton with clap) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/fetchium-cli/src/commands/completions.rs` | Completions command |
| `crates/fetchium-cli/build.rs` | Generate completions at build time (optional) |

#### Step-by-Step Implementation Guide

```rust
// crates/fetchium-cli/src/commands/completions.rs
use clap::CommandFactory;
use clap_complete::{Shell, generate};

/// Generate shell completions for the specified shell.
pub fn generate_completions(shell: Shell) {
    let mut cmd = crate::Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, &name, &mut std::io::stdout());
}

// In the CLI definition:
/// Generate shell completions
#[derive(clap::Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

// Usage:
// fetchium completions bash > ~/.bash_completion.d/fetchium
// fetchium completions zsh > ~/.zsh/completions/_hsx
// fetchium completions fish > ~/.config/fish/completions/fetchium.fish
```

#### Acceptance Criteria

- [x] `fetchium completions bash` outputs valid bash completion script
- [x] `fetchium completions zsh` outputs valid zsh completion script
- [x] `fetchium completions fish` outputs valid fish completion script
- [x] Tab completion works for all commands: search, research, deep, fetch, etc.
- [x] Tab completion works for flags: --format, --tier, --mode, etc.
- [x] Enum values complete (e.g., --tier shows key_facts, summary, detailed, complete)
- [x] Alias `hyper` also has completions

#### Pitfalls

- **Alias completions**: `clap_complete` generates for the binary name. For the `hyper` alias, users need to source completions twice or create a symlink.
- **Dynamic completions**: Some values (like available models, installed plugins) are dynamic. Static completions cannot enumerate these. Consider `clap_complete::dynamic` for supported shells.
