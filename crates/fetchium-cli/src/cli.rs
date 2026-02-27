//! CLI definition using clap derive (PRD §10).

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Fetchium — AI-native search engine for humans and agents.
#[derive(Debug, Parser)]
#[command(name = "fetchium", version, about, long_about = None)]
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

    /// Show total execution time after command completes
    #[arg(long, global = true)]
    pub time: bool,
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

    /// YouTube Intelligence System — search, analyze, transcribe, research YouTube videos
    #[command(name = "youtube")]
    YouTube(YouTubeArgs),

    /// Social Media Intelligence — Twitter/X, Reddit, TikTok, HackerNews, unified research
    Social(SocialArgs),

    /// Manage AI provider authentication and fallback chain
    Provider {
        #[command(subcommand)]
        action: ProviderAction,
    },

    /// Set up Fetchium environment (download Chromium, check dependencies)
    ///
    /// Examples:
    ///   fetchium setup --headless     # download Chrome for Testing (~200 MB)
    ///   fetchium setup --check        # verify all dependencies are present
    ///   fetchium setup                # check + download anything missing
    Setup(SetupArgs),

    /// X (Twitter) Intelligence — search, trends, sentiment, monitor, fetch
    #[command(name = "twitter", alias = "x", alias = "tw")]
    Twitter(TwitterArgs),

    /// Reddit Intelligence — search, hot, top, research, fetch
    #[command(name = "reddit")]
    Reddit(RedditArgs),

    /// Hacker News Intelligence — search, top, new, fetch
    #[command(name = "hackernews", alias = "hn")]
    Hackernews(HackernewsArgs),

    /// Facebook Intelligence — search, fetch
    #[command(name = "facebook", alias = "fb")]
    Facebook(FacebookArgs),

    /// TikTok Intelligence — search, trends, fetch
    #[command(name = "tiktok")]
    Tiktok(TiktokArgs),

    /// Transcribe audio/video from any URL
    Transcribe(TranscribeArgs),

    /// AI-powered URL/text summarization
    Summarize(SummarizeArgs),
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
    #[arg(short, long, default_value = "8000")]
    pub budget: u32,

    /// PDS tier
    #[arg(short, long, default_value = "detailed")]
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

    /// Disable AI synthesis (use heuristic listing instead)
    #[arg(long)]
    pub no_ai: bool,

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
    #[arg(long, default_value = "5")]
    pub max_sources: usize,

    /// Fast mode: skip full-page fetches, use search snippets as context.
    /// ~3x faster with minimal quality loss for simple queries.
    #[arg(long)]
    pub fast: bool,
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

    /// Global timeout in seconds (default: auto based on resource tier)
    #[arg(long)]
    pub timeout: Option<u64>,
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

    /// Query context for QATBE (use --query to avoid conflict with global -q)
    #[arg(long)]
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

    /// Use AI for dimension extraction (more accurate, requires AI provider)
    #[arg(long)]
    pub ai: bool,

    /// Override model for AI comparison
    #[arg(long)]
    pub model: Option<String>,
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

// ─── Provider ────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
pub enum ProviderAction {
    /// Show all providers with connectivity status and configured model
    List,

    /// Interactive setup wizard for one or all providers
    ///
    /// Examples:
    ///   fetchium provider setup           # wizard for all providers
    ///   fetchium provider setup gemini    # configure Gemini only
    Setup {
        /// Provider slug: ollama, openai, anthropic, gemini, gemini_cli, openrouter
        provider: Option<String>,
    },

    /// Set API key and model for a specific provider
    ///
    /// Examples:
    ///   fetchium provider set gemini --key AIza... --model gemini-2.0-flash
    ///   fetchium provider set openai --key sk-...
    ///   fetchium provider set openrouter --key sk-or-... --model google/gemini-2.0-flash-001
    Set(ProviderSetArgs),

    /// Configure the provider fallback chain (tried in order)
    ///
    /// Examples:
    ///   fetchium provider chain gemini openai ollama
    ///   fetchium provider chain anthropic openrouter
    Chain {
        /// Ordered list of provider slugs
        #[arg(required = true)]
        providers: Vec<String>,
    },

    /// Test provider connectivity and API key validity
    ///
    /// Examples:
    ///   fetchium provider test             # test all chain providers
    ///   fetchium provider test gemini      # test Gemini only
    Test {
        /// Provider slug to test (omit to test all in chain)
        provider: Option<String>,
    },

    /// Show all API key sources, config paths, and how to set keys permanently
    ///
    /// Examples:
    ///   fetchium provider keys             # show key storage guide
    Keys,

    /// Interactive authentication wizard for one or all providers
    ///
    /// Examples:
    ///   fetchium provider auth              # wizard for all providers
    ///   fetchium provider auth gemini       # Gemini only (API key or OAuth)
    ///   fetchium provider auth anthropic    # Anthropic API key
    Auth {
        /// Provider slug (omit for full wizard)
        provider: Option<String>,
    },

    /// List available models for a provider (with tiers and short aliases)
    ///
    /// Examples:
    ///   fetchium provider models              # list all providers' models
    ///   fetchium provider models gemini_cli   # GeminiCli models only
    ///   fetchium provider models anthropic    # Claude models with aliases
    Models {
        /// Provider slug (omit to show all providers)
        provider: Option<String>,
    },
}

/// Arguments for `fetchium provider set`.
#[derive(Debug, Parser)]
pub struct ProviderSetArgs {
    /// Provider slug: ollama, openai, anthropic, gemini, gemini_cli, openrouter
    pub provider: String,

    /// Set the primary API key (replaces any previously stored single key).
    /// Stored securely in ~/.fetchium/auth.json (0600 permissions).
    /// To add keys to a rotation pool, use --add-key instead.
    #[arg(long)]
    pub key: Option<String>,

    /// Add an API key to the rotation pool without replacing existing keys.
    /// The pool is tried in random order; rate-limited keys (429) are skipped.
    ///
    /// Examples:
    ///   fetchium provider set gemini --add-key AIzaSyAbc...
    ///   fetchium provider set gemini --add-key KEY1 --add-key KEY2
    #[arg(long, value_name = "KEY")]
    pub add_key: Vec<String>,

    /// Model name override (e.g. gemini-2.5-flash, claude-sonnet-4-6, gpt-4o)
    #[arg(long)]
    pub model: Option<String>,

    /// Custom base URL (for proxies or self-hosted compatible APIs)
    #[arg(long)]
    pub base_url: Option<String>,

    /// Enable or disable this provider in the fallback chain
    #[arg(long)]
    pub enable: Option<bool>,
}

// ─── Setup ───────────────────────────────────────────────────

/// Arguments for `fetchium setup`.
#[derive(Debug, Parser)]
pub struct SetupArgs {
    /// Download and install headless Chromium (Chrome for Testing, ~200 MB).
    ///
    /// Installs to ~/.fetchium/chromium/ — no root required.
    /// The binary is auto-detected by all headless commands after install.
    #[arg(long)]
    pub headless: bool,

    /// Set up SearXNG via Docker (requires Docker to be installed).
    ///
    /// Pulls searxng/searxng:latest, writes an optimised settings.yml, and
    /// starts the container on port 4040. Idempotent — safe to re-run.
    #[arg(long)]
    pub searxng: bool,

    /// Check environment only — do not install anything.
    #[arg(long)]
    pub check: bool,
}

// ─── YouTube ──────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct YouTubeArgs {
    #[command(subcommand)]
    pub action: YouTubeAction,
}

#[derive(Debug, Subcommand)]
pub enum YouTubeAction {
    /// Search YouTube for videos
    Search(YtSearchArgs),
    /// Analyze a specific YouTube video
    Analyze(YtAnalyzeArgs),
    /// Extract transcript from a YouTube video
    Transcript(YtTranscriptArgs),
    /// Full research pipeline with fact-checking
    Research(YtResearchArgs),
    /// Compare two YouTube videos
    Compare(YtCompareArgs),
}

#[derive(Debug, Parser)]
pub struct YtSearchArgs {
    /// Search query
    pub query: String,
    /// Maximum number of results
    #[arg(short = 'n', long, default_value = "10")]
    pub max_results: usize,
}

#[derive(Debug, Parser)]
pub struct YtAnalyzeArgs {
    /// YouTube video URL
    pub url: String,
    /// Fetch and analyze transcript
    #[arg(long)]
    pub transcript: bool,
    /// Fetch and analyze comments
    #[arg(long)]
    pub comments: bool,
    /// Generate teaching content (flashcards, quiz)
    #[arg(long)]
    pub teaching: bool,
}

#[derive(Debug, Parser)]
pub struct YtTranscriptArgs {
    /// YouTube video URL
    pub url: String,
    /// Align transcript to video chapters
    #[arg(long)]
    pub chapters: bool,
}

#[derive(Debug, Parser)]
pub struct YtResearchArgs {
    /// Research query
    pub query: String,
    /// Maximum videos to analyze
    #[arg(long, default_value = "5")]
    pub max_videos: usize,
    /// Build topic timeline
    #[arg(long)]
    pub timeline: bool,
    /// Generate learning path
    #[arg(long)]
    pub learning_path: bool,
    /// Enable cross-video fact checking
    #[arg(long)]
    pub fact_check: bool,
    /// Output file for the report
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Debug, Parser)]
pub struct YtCompareArgs {
    /// Two YouTube video URLs to compare
    #[arg(required = true, num_args = 2)]
    pub urls: Vec<String>,
}

// ─── Social ──────────────────────────────────────────────────────

/// Arguments for `fetchium social`.
///
/// Quick usage:
///   fetchium social "AI tools"                            # unified (all platforms)
///   fetchium social twitter "GPT-5 release"               # X/Twitter shorthand
///   fetchium social reddit "mechanical keyboards"          # Reddit shorthand
///   fetchium social hn "Show HN: my project"              # Hacker News shorthand
///   fetchium social "AI tools" --twitter                  # Twitter flag style
///   fetchium social "AI tools" --reddit --tiktok          # Reddit + TikTok
///   fetchium social "AI tools" --unified --ideas          # Unified + content ideas
///   fetchium social "AI tools" --reddit --subreddits r/ML,r/AI
#[derive(Debug, Parser)]
pub struct SocialArgs {
    /// Platform (x/twitter/reddit/hn/hackernews/facebook/tiktok/youtube) or search query
    #[arg(value_name = "PLATFORM_OR_QUERY")]
    pub query: String,

    /// Search query — provide this when first arg is a platform name
    #[arg(value_name = "QUERY")]
    pub extra_query: Option<String>,

    // ── Platform selection ──────────────────────────────────────
    /// Search all platforms simultaneously (default when no platform flag given)
    #[arg(long)]
    pub unified: bool,

    /// Search X/Twitter (via Nitter instances + DDG site:x.com)
    #[arg(long, alias = "x")]
    pub twitter: bool,

    /// Search Reddit posts and communities
    #[arg(long)]
    pub reddit: bool,

    /// Search TikTok videos and hashtag trends
    #[arg(long)]
    pub tiktok: bool,

    /// Search Hacker News stories (Algolia API)
    #[arg(long, alias = "hn")]
    pub hackernews: bool,

    /// Search Facebook via DDG site:search + Open Graph metadata
    #[arg(long)]
    pub facebook: bool,

    /// Search YouTube videos
    #[arg(long)]
    pub youtube: bool,

    // ── Common options ──────────────────────────────────────────
    /// Maximum posts to fetch (per platform in unified mode, total otherwise)
    #[arg(short = 'n', long, default_value = "50")]
    pub max: usize,

    // ── Platform-specific options ───────────────────────────────
    /// Subreddits to search — Reddit only (comma-separated, e.g. r/MachineLearning,r/Python)
    #[arg(long, value_delimiter = ',')]
    pub subreddits: Vec<String>,

    /// Also fetch Twitter trending topics
    #[arg(long)]
    pub trends: bool,

    /// Generate 20 content ideas from viral patterns (unified mode)
    #[arg(long)]
    pub ideas: bool,

    /// Deep analysis mode — more thorough, slower
    #[arg(long)]
    pub deep: bool,

    /// Facebook Graph API token for richer data (APP_ID|APP_SECRET or bearer token)
    #[arg(long)]
    pub token: Option<String>,

    // ── Output ──────────────────────────────────────────────────
    /// Save report to file
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
        #[arg(short = 'Q', long)]
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

// ─── Twitter ─────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct TwitterArgs {
    #[command(subcommand)]
    pub action: TwitterAction,
}

#[derive(Debug, Subcommand)]
pub enum TwitterAction {
    /// Search Twitter/X for tweets matching a query
    Search {
        /// Search query
        query: String,
        /// Maximum number of tweets
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,
    },
    /// Fetch trending topics on Twitter/X
    Trends {
        /// Country code (e.g. us, uk, worldwide)
        #[arg(default_value = "us")]
        country: String,
    },
    /// Analyze sentiment of tweets matching a query
    Sentiment {
        /// Search query
        query: String,
        /// Maximum tweets to analyze
        #[arg(short = 'n', long, default_value = "50")]
        max: usize,
    },
    /// Fetch a single tweet by URL (via oEmbed)
    Fetch {
        /// Tweet URL (https://x.com/user/status/...)
        url: String,
    },
    /// Monitor Twitter/X for new tweets matching a query (realtime)
    Monitor {
        /// Search query
        query: String,
        /// Poll interval in seconds
        #[arg(long, default_value = "120")]
        interval: u64,
    },
    /// Deep Twitter research (delegates to social pipeline)
    Research {
        /// Research query
        query: String,
        /// Maximum tweets
        #[arg(short = 'n', long, default_value = "50")]
        max: usize,
    },
    /// Fetch profile info for a Twitter user
    Profile {
        /// Twitter username (without @)
        username: String,
    },
}

// ─── Reddit ──────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct RedditArgs {
    #[command(subcommand)]
    pub action: RedditAction,
}

#[derive(Debug, Subcommand)]
pub enum RedditAction {
    /// Search Reddit posts
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,
        /// Subreddits to search (comma-separated)
        #[arg(long, value_delimiter = ',')]
        subreddits: Vec<String>,
    },
    /// Deep Reddit research (delegates to social pipeline)
    Research {
        /// Research query
        query: String,
        /// Maximum posts
        #[arg(short = 'n', long, default_value = "50")]
        max: usize,
    },
    /// Fetch hot posts from a subreddit
    Hot {
        /// Subreddit name (without r/)
        subreddit: String,
        /// Maximum posts
        #[arg(short = 'n', long, default_value = "25")]
        max: usize,
    },
    /// Fetch top posts from a subreddit
    Top {
        /// Subreddit name (without r/)
        subreddit: String,
        /// Time period (day, week, month, year, all)
        #[arg(long, default_value = "week")]
        period: String,
        /// Maximum posts
        #[arg(short = 'n', long, default_value = "25")]
        max: usize,
    },
    /// Fetch a Reddit post by URL
    Fetch {
        /// Reddit post URL
        url: String,
    },
}

// ─── HackerNews ──────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct HackernewsArgs {
    #[command(subcommand)]
    pub action: HackernewsAction,
}

#[derive(Debug, Subcommand)]
pub enum HackernewsAction {
    /// Search Hacker News stories (via Algolia)
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,
    },
    /// Deep HN research (delegates to social pipeline)
    Research {
        /// Research query
        query: String,
        /// Maximum stories
        #[arg(short = 'n', long, default_value = "50")]
        max: usize,
    },
    /// Fetch HN top stories
    Top {
        /// Maximum stories
        #[arg(short = 'n', long, default_value = "30")]
        max: usize,
    },
    /// Fetch HN newest stories
    New {
        /// Maximum stories
        #[arg(short = 'n', long, default_value = "30")]
        max: usize,
    },
    /// Fetch a HN story by URL or ID
    Fetch {
        /// HN story URL or numeric ID
        url: String,
    },
}

// ─── Facebook ────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct FacebookArgs {
    #[command(subcommand)]
    pub action: FacebookAction,
}

#[derive(Debug, Subcommand)]
pub enum FacebookAction {
    /// Search Facebook (via social pipeline)
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,
    },
    /// Fetch a Facebook page/post by URL
    Fetch {
        /// Facebook URL
        url: String,
    },
}

// ─── TikTok ──────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct TiktokArgs {
    #[command(subcommand)]
    pub action: TiktokAction,
}

#[derive(Debug, Subcommand)]
pub enum TiktokAction {
    /// Search TikTok videos
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,
    },
    /// Fetch TikTok trending content
    Trends {
        /// Maximum results
        #[arg(short = 'n', long, default_value = "25")]
        max: usize,
    },
    /// Fetch a TikTok page by URL
    Fetch {
        /// TikTok URL
        url: String,
    },
}

// ─── Transcribe ──────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct TranscribeArgs {
    /// URL of the audio/video to transcribe
    pub url: String,

    /// Align transcript to video chapters (YouTube only)
    #[arg(long)]
    pub chapters: bool,
}

// ─── Summarize ───────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct SummarizeArgs {
    /// URL or raw text to summarize
    pub input: String,

    /// Summary length: short, medium, long
    #[arg(short, long)]
    pub length: Option<String>,
}
