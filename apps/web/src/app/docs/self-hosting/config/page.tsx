import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Configuration Reference" };

export default function ConfigReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Self-Hosting</div>
      <h1>Configuration Reference</h1>
      <p>
        Fetchium is configured via a TOML file at <code>~/.fetchium/config.toml</code>.
        Every setting has a sensible default — the file is optional and created automatically
        on first run. You can also override any setting with environment variables.
      </p>

      <div className="callout">
        <strong>Auto-generated:</strong> Run <code>fetchium setup</code> to create a pre-configured
        instance with Chrome and SearXNG already set up.
      </div>

      <h2>Full config.toml reference</h2>

      <CodeBlock language="toml" filename="~/.fetchium/config.toml" code={`# Fetchium Configuration
# Location: ~/.fetchium/config.toml
# All values shown are defaults unless noted.

[search]
# SearXNG instance URL for federated search backbone.
# Set by: fetchium setup --searxng
# Default: none (uses direct HTTP scrapers)
searxng_url = "http://localhost:4040"

# Maximum parallel search backends to query simultaneously.
max_concurrent_backends = 5

# Per-backend timeout in seconds.
backend_timeout_secs = 10

# Default number of results per backend.
results_per_backend = 10

[extract]
# Maximum page size to download (bytes). Default: 5 MB.
max_page_bytes = 5_242_880

# Default detail tier: key_facts | summary | detailed | complete
default_tier = "summary"

# Default token budget (overrides tier if set).
# default_token_budget = 1000

[headless]
# Path to Chrome/Chromium binary.
# Set by: fetchium setup --headless
# Auto-detected from: $FETCHIUM_CHROME_PATH env, this field, fetchium-managed, system paths.
# chrome_path = "/home/user/.fetchium/chromium/chrome-linux64/chrome"

[cache]
# SQLite cache database path.
db_path = "~/.fetchium/cache.db"

# Cache TTL for search results (seconds). Default: 1 hour.
search_ttl_secs = 3600

# Cache TTL for extracted content (seconds). Default: 24 hours.
extract_ttl_secs = 86400

# Maximum cache database size (MB). Default: 500 MB.
max_size_mb = 500

[intelligence]
# Enable Persistent Intelligence Engine (PIE) — cross-session learning.
enabled = true

# SQLite database for intelligence data.
db_path = "~/.fetchium/intelligence.db"

[token]
# Default token budget for extractions.
default_budget = 1000

# Maximum allowed token budget.
max_budget = 20000

[http]
# HTTP connection pool size per host.
pool_connections = 10

# Request timeout (seconds).
timeout_secs = 30

# Maximum redirects to follow.
max_redirects = 10

# User-agent string.
user_agent = "Fetchium/1.0"

[logging]
# Log level: trace | debug | info | warn | error
level = "info"

# Log format: json | pretty
format = "pretty"
`} />

      <h2>Environment variable overrides</h2>
      <p>
        Every config value can be overridden with an environment variable using the
        <code>FETCHIUM_</code> prefix and underscores replacing dots:
      </p>

      <table>
        <thead><tr><th>Environment Variable</th><th>Config key</th><th>Example</th></tr></thead>
        <tbody>
          <tr><td><code>FETCHIUM_CHROME_PATH</code></td><td><code>headless.chrome_path</code></td><td><code>/usr/bin/chromium</code></td></tr>
          <tr><td><code>SEARXNG_URL</code></td><td><code>search.searxng_url</code></td><td><code>http://localhost:4040</code></td></tr>
          <tr><td><code>FETCHIUM_LOG_LEVEL</code></td><td><code>logging.level</code></td><td><code>debug</code></td></tr>
          <tr><td><code>FETCHIUM_PORT</code></td><td>API server port</td><td><code>3050</code></td></tr>
          <tr><td><code>FETCHIUM_ADMIN_SECRET</code></td><td>Admin authentication key</td><td>(32+ char secret)</td></tr>
        </tbody>
      </table>

      <h2>Configuration precedence</h2>
      <ol>
        <li><strong>Environment variables</strong> — highest priority, always win</li>
        <li><strong>config.toml</strong> — user-specific settings</li>
        <li><strong>Built-in defaults</strong> — lowest priority, always present</li>
      </ol>

      <h2>Using fetchium config commands</h2>

      <CodeBlock language="bash" code={`# View all current settings
fetchium config

# Get a specific value
fetchium config get search.searxng_url

# Set a value
fetchium config set search.searxng_url http://localhost:4040

# Open config file in $EDITOR
fetchium config edit`} />

      <h2>Multiple environments</h2>
      <p>
        Use environment variables to switch between configurations without editing the file:
      </p>

      <CodeBlock language="bash" code={`# Development (local SearXNG)
SEARXNG_URL=http://localhost:4040 fetchium search "rust async"

# Production (remote instance)
SEARXNG_URL=https://search.yourdomain.com fetchium search "rust async"`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/self-hosting/docker", title: "Docker Setup", desc: "Full Docker Compose stack" },
          { href: "https://docs.fetchium.com/self-hosting/searxng", title: "SearXNG Integration", desc: "Configure search backbone" },
          { href: "https://docs.fetchium.com/authentication", title: "Authentication", desc: "API key management" },
          { href: "https://docs.fetchium.com/quickstart", title: "Quick Start", desc: "Make your first request" },
        ].map(l => (
          <Link key={l.href} href={l.href} className="glass-card rounded-xl p-4 no-underline group">
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{l.title} →</div>
            <div className="text-xs text-slate-500 mt-1">{l.desc}</div>
          </Link>
        ))}
      </div>
    </article>
  );
}
