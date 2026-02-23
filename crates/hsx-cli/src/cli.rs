//! CLI definition using clap derive (PRD §10).

use clap::{Parser, Subcommand, ValueEnum};

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

    /// Config file path
    #[arg(long, global = true)]
    pub config: Option<String>,
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
    #[arg(short, long)]
    pub query: Option<String>,
}

// ─── Research ────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct ResearchArgs {
    /// Research query
    pub query: String,

    /// Citation style
    #[arg(long, default_value = "apa")]
    pub citations: CitationStyle,

    /// Output file
    #[arg(short, long)]
    pub output: Option<String>,
}

// ─── AI ──────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct AiArgs {
    /// Query for AI analysis
    pub query: String,

    /// Model to use
    #[arg(short, long)]
    pub model: Option<String>,
}

// ─── Deep ────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct DeepArgs {
    /// Deep research query
    pub query: String,

    /// Maximum research depth
    #[arg(long, default_value = "3")]
    pub max_depth: u32,

    /// Maximum number of agents
    #[arg(long, default_value = "5")]
    pub max_agents: u32,
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

    /// Token budget
    #[arg(short, long, default_value = "8000")]
    pub budget: u32,

    /// PDS tier
    #[arg(short, long, default_value = "detailed")]
    pub tier: Tier,
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
}

// ─── Value Enums ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, ValueEnum)]
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
