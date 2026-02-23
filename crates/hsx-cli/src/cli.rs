//! CLI definition using clap derive (PRD §10).

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// HyperSearchX — AI-native search engine for humans and agents.
#[derive(Debug, Parser)]
#[command(name = "hsx", version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(short, long, global = true, default_value = "markdown")]
    pub format: Format,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode (suppress non-essential output)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Bypass cache for this request
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Config file path
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Privacy mode (standard, private, tor, air-gap)
    #[arg(long, global = true)]
    pub privacy: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Search the web (Mode A — human-friendly)
    Search(SearchArgs),

    /// Fetch and extract content from a URL (Mode D)
    Fetch(FetchArgs),

    /// View a URL with content extraction (alias for fetch)
    View(FetchArgs),

    /// Multi-source research with citations (Mode B)
    Research(ResearchArgs),

    /// AI-powered analysis of search results (Mode C)
    Ai(AiArgs),

    /// Deep multi-agent research (Mode E)
    Deep(DeepArgs),

    /// Agent-optimized search (JSON segments output)
    AgentSearch(AgentSearchArgs),

    /// Agent-optimized fetch (JSON segments output)
    AgentFetch(AgentFetchArgs),

    /// Agent-optimized research
    AgentResearch(AgentResearchArgs),

    /// System health check
    Doctor,

    /// Manage configuration
    Config(ConfigArgs),

    /// Manage cache
    Cache(CacheArgs),

    /// Start API/MCP server
    Serve(ServeArgs),

    /// Compare two or more items side-by-side (e.g. "Rust vs Go")
    Compare(CompareArgs),

    /// Monitor a URL for content changes
    Monitor(MonitorArgs),

    /// Manage the local document index
    Index(IndexArgs),

    /// Manage the Persistent Intelligence Engine (PIE) and run intelligence algorithms
    Intelligence {
        #[command(subcommand)]
        sub: crate::commands::intelligence::IntelligenceSubcmd,
    },

    /// Manage plugins (install, remove, list, create)
    Plugin {
        #[command(subcommand)]
        sub: crate::commands::plugin::PluginCommand,
    },

    /// Collaborative research workspaces (create, fork, merge, sync)
    Workspace {
        #[command(subcommand)]
        sub: crate::commands::workspace::WorkspaceCommand,
    },

    /// Manage topic subscriptions
    Subscribe {
        #[command(subcommand)]
        sub: crate::commands::subscribe::SubscribeCommand,
    },

    /// Show personalized research radar based on history
    Radar(RadarArgs),

    /// Generate a research digest for topics
    Digest(DigestArgs),

    /// Launch interactive TUI
    Tui,

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

// ─── Search ──────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Maximum number of results
    #[arg(short = 'n', long, default_value = "10")]
    pub max_results: u32,

    /// Search backends to use
    #[arg(short, long)]
    pub backends: Vec<String>,

    /// Write output to a file
    #[arg(short, long)]
    pub output: Option<String>,

    /// Analyse results with Adversarial Content Shield (ACS) and report trust scores
    #[arg(long)]
    pub trust_verify: bool,
}

// ─── Fetch ───────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct FetchArgs {
    /// URL to fetch
    pub url: String,

    /// Token budget
    #[arg(short, long, default_value = "4000")]
    pub budget: u32,

    /// PDS tier
    #[arg(short, long, default_value = "summary")]
    pub tier: Tier,

    /// Extract with query context (QATBE)
    #[arg(long)]
    pub query: Option<String>,

    /// Report AI-generation probability and adversarial signals for the fetched URL
    #[arg(long)]
    pub check_ai: bool,

    /// Write output to a file
    #[arg(short, long)]
    pub output: Option<String>,
}

// ─── Research ────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct ResearchArgs {
    /// Research query
    pub query: String,

    /// Citation style
    #[arg(long, default_value = "apa")]
    pub citations: CitationStyle,

    /// Validation mode (off, fast, standard, strict)
    #[arg(long, default_value = "standard")]
    pub validate: String,

    /// Require every claim to cite a source
    #[arg(long)]
    pub strict_evidence: bool,

    /// Generate evidence graph JSON alongside report
    #[arg(long)]
    pub evidence_graph: bool,

    /// Trace source genealogy using SGT to find the original source of claims
    #[arg(long)]
    pub trace_sources: bool,

    /// Check sources with Adversarial Content Shield (ACS) and Confidence Calibration Engine (CCE)
    #[arg(long)]
    pub trust_verify: bool,

    /// Maximum number of sources to fetch
    #[arg(long, default_value = "10")]
    pub max_sources: usize,

    /// Output file
    #[arg(short, long)]
    pub output: Option<String>,
}

// ─── AI ──────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct AiArgs {
    /// Query for AI analysis
    pub query: String,

    /// Override model selection (e.g., "deepseek-r1:7b", "gemma3:9b")
    #[arg(long)]
    pub model: Option<String>,

    /// Token budget for context assembly
    #[arg(long, default_value = "4096")]
    pub budget: usize,

    /// Disable streaming output (wait for full response before printing)
    #[arg(long)]
    pub no_stream: bool,

    /// Maximum number of sources to include in context
    #[arg(long, default_value = "8")]
    pub max_sources: usize,
}

// ─── Deep ────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct DeepArgs {
    /// Deep research query
    pub query: String,

    /// Maximum multi-hop depth
    #[arg(long, default_value = "3")]
    pub max_depth: u32,

    /// Maximum number of agents
    #[arg(long, default_value = "5")]
    pub max_agents: u32,

    /// Export evidence graph as JSON file
    #[arg(long)]
    pub evidence_graph: bool,

    /// Use Tree-of-Thoughts Research (ToTR) for multi-faceted reasoning
    #[arg(long)]
    pub tree_of_thoughts: bool,

    /// Run the Advocate-Critic-Judge self-debate protocol with ToTR
    #[arg(long)]
    pub self_debate: bool,

    /// Output file for the research report
    #[arg(short, long)]
    pub output: Option<String>,

    /// Show full audit trail
    #[arg(long)]
    pub audit: bool,

    /// Token budget for the research session
    #[arg(long, default_value = "20000")]
    pub budget: usize,
}

// ─── Agent Commands ──────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct AgentSearchArgs {
    /// Search query
    pub query: String,

    /// Token budget
    #[arg(short, long, default_value = "4000")]
    pub budget: u32,

    /// PDS tier
    #[arg(short, long, default_value = "summary")]
    pub tier: Tier,

    /// Number of results
    #[arg(short = 'n', long, default_value = "5")]
    pub max_results: u32,
}

#[derive(Debug, Parser)]
pub struct AgentFetchArgs {
    /// URL to fetch
    pub url: String,

    /// Token budget
    #[arg(short, long, default_value = "4000")]
    pub budget: u32,

    /// PDS tier
    #[arg(short, long, default_value = "summary")]
    pub tier: Tier,

    /// Query context for QATBE
    #[arg(short, long)]
    pub query: Option<String>,
}

#[derive(Debug, Parser)]
pub struct AgentResearchArgs {
    /// Research query
    pub query: String,

    /// Token budget for the entire output
    #[arg(short, long, default_value = "8000")]
    pub budget: u32,

    /// PDS tier
    #[arg(short, long, default_value = "detailed")]
    pub tier: Tier,

    /// Maximum number of sources
    #[arg(long, default_value = "10")]
    pub max_sources: usize,

    /// Require every claim to cite a source
    #[arg(long)]
    pub strict_evidence: bool,

    /// Generate evidence graph JSON alongside report
    #[arg(long)]
    pub evidence_graph: bool,

    /// JSON schema file for structured output validation
    #[arg(long)]
    pub schema: Option<String>,

    /// Framework adapter (langchain, crewai, mcp)
    #[arg(long)]
    pub framework: Option<String>,
}

// ─── Config ──────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set { key: String, value: String },
    /// Get a configuration value
    Get { key: String },
    /// Reset to defaults
    Reset,
    /// Open config file in editor
    Edit,
}

// ─── Cache ───────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub action: CacheAction,
}

#[derive(Debug, Subcommand)]
pub enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear all cache
    Clear,
    /// Clear expired entries only
    Prune,
}

// ─── Serve ───────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct ServeArgs {
    /// Server mode
    #[arg(long, default_value = "rest")]
    pub mode: ServerMode,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    /// Transport for MCP server (stdio or sse)
    #[arg(long, default_value = "stdio")]
    pub transport: McpTransport,
}

/// MCP transport mode.
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum McpTransport {
    Stdio,
    Sse,
}

// ─── Value Enums ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum Format {
    Markdown,
    Json,
    Csv,
    Yaml,
    Html,
    Segments,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Tier {
    KeyFacts,
    Summary,
    Detailed,
    Complete,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CitationStyle {
    Apa,
    Mla,
    Chicago,
    Harvard,
    Ieee,
    Bibtex,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ServerMode {
    Rest,
    Mcp,
    Both,
}

// ─── Compare ─────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct CompareArgs {
    /// Comparison query (e.g. "Rust vs Go" or "compare Python and Ruby")
    pub query: String,

    /// Maximum sources to fetch per item
    #[arg(long, default_value = "5")]
    pub max_sources: usize,

    /// Token budget per item
    #[arg(long, default_value = "2000")]
    pub budget: usize,

    /// Output file
    #[arg(short, long)]
    pub output: Option<String>,
}

// ─── Monitor ─────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct MonitorArgs {
    #[command(subcommand)]
    pub action: MonitorAction,
}

#[derive(Debug, Subcommand)]
pub enum MonitorAction {
    /// Add a URL to the monitor list
    Add {
        url: String,
        /// Check interval (e.g. 30s, 5m, 1h, 7d)
        #[arg(short, long, default_value = "1h")]
        interval: String,
        /// Notification method (log, email)
        #[arg(long)]
        notify: Option<String>,
    },
    /// Remove a URL from the monitor list
    Remove { url: String },
    /// Check a monitored URL for changes now
    Check { url: String },
    /// List all monitored URLs
    List,
    /// Show diff between last two snapshots
    Diff { url: String },
}

// ─── Radar ───────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct RadarArgs {
    /// Maximum number of suggestions to show.
    #[arg(short = 'n', long, default_value = "10")]
    pub limit: usize,
}

// ─── Digest ──────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct DigestArgs {
    /// Period (daily, weekly, monthly).
    #[arg(long, default_value = "weekly")]
    pub period: String,

    /// Comma-separated list of topics.
    #[arg(long, value_delimiter = ',')]
    pub topics: Vec<String>,

    /// Write output to a file.
    #[arg(short, long)]
    pub output: Option<String>,
}

// ─── Index ───────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct IndexArgs {
    #[command(subcommand)]
    pub action: IndexAction,
}

#[derive(Debug, Subcommand)]
pub enum IndexAction {
    /// Add a URL to the local index
    Add {
        url: String,
        /// Query context for QATBE extraction
        #[arg(short, long)]
        query: Option<String>,
    },
    /// Search the local index
    Search {
        query: String,
        /// Maximum number of results
        #[arg(short = 'n', long, default_value = "10")]
        max_results: usize,
    },
    /// Show index statistics
    Stats,
    /// Clear the index
    Clear,
}
