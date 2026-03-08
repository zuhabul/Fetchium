import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "SearXNG Integration" };

export default function SearXNGIntegration() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Self-Hosting</div>
      <h1>SearXNG Integration</h1>
      <p>
        SearXNG is the federated search backbone powering Fetchium. It aggregates results
        from Google, Bing, DuckDuckGo, Brave, Wikipedia, StackOverflow, GitHub, arXiv, Reddit,
        and HackerNews — simultaneously, without API keys.
      </p>

      <div className="callout">
        <strong>One-command setup:</strong> Run <code>fetchium setup --searxng</code> to automatically
        pull the Docker image, generate a config, and start SearXNG on port 4040.
      </div>

      <h2>Automated setup</h2>

      <CodeBlock language="bash" code={`# Requires Docker to be installed and running
fetchium setup --searxng

# Output:
# ── SearXNG (federated search backbone) ─────────────
#   Config: ~/.fetchium/searxng/settings.yml (created)
#   Port  : 4040 (container port 8080 → host 4040)
#   Image: pulling docker.io/searxng/searxng:latest
#   Container: creating on port 4040...
#   Health: waiting..... ready! (6s)
#   ✓ SearXNG running at http://localhost:4040`} />

      <h2>What the setup does</h2>
      <ol>
        <li>Creates <code>~/.fetchium/searxng/settings.yml</code> with an optimised config</li>
        <li>Pulls <code>docker.io/searxng/searxng:latest</code></li>
        <li>Starts container: <code>docker run -d --name fetchium-searxng --restart unless-stopped -p 4040:8080</code></li>
        <li>Polls the JSON health endpoint until ready (up to 30 seconds)</li>
        <li>Idempotent — safe to run multiple times (skips steps already done)</li>
      </ol>

      <h2>Container management</h2>

      <CodeBlock language="bash" code={`# View logs
docker logs fetchium-searxng -f

# Restart
docker restart fetchium-searxng

# Stop
docker stop fetchium-searxng

# Start (if stopped)
docker start fetchium-searxng

# Remove and recreate
docker rm -f fetchium-searxng && fetchium setup --searxng`} />

      <h2>Enabled search engines</h2>
      <table>
        <thead><tr><th>Engine</th><th>Category</th><th>Shortcut</th><th>Timeout</th></tr></thead>
        <tbody>
          <tr><td>Google</td><td>General</td><td><code>!g</code></td><td>4s</td></tr>
          <tr><td>Bing</td><td>General</td><td><code>!b</code></td><td>4s</td></tr>
          <tr><td>DuckDuckGo</td><td>General</td><td><code>!d</code></td><td>4s</td></tr>
          <tr><td>Brave</td><td>General</td><td><code>!br</code></td><td>4s</td></tr>
          <tr><td>Wikipedia</td><td>General</td><td><code>!w</code></td><td>4s</td></tr>
          <tr><td>StackOverflow</td><td>IT</td><td><code>!so</code></td><td>4s</td></tr>
          <tr><td>GitHub</td><td>IT</td><td><code>!gh</code></td><td>4s</td></tr>
          <tr><td>arXiv</td><td>Science</td><td><code>!arx</code></td><td>5s</td></tr>
          <tr><td>Reddit</td><td>Social</td><td><code>!re</code></td><td>4s</td></tr>
          <tr><td>HackerNews</td><td>Social</td><td><code>!hn</code></td><td>4s</td></tr>
        </tbody>
      </table>

      <h2>Generated settings.yml</h2>
      <p>
        The auto-generated config is saved at <code>~/.fetchium/searxng/settings.yml</code>.
        You can edit it freely — the container mounts it as a volume.
      </p>

      <CodeBlock language="yaml" filename="settings.yml (key sections)" code={`use_default_settings:
  engines:
    keep_only: [google, bing, duckduckgo, brave, wikipedia, stackoverflow, github, arxiv, reddit, hackernews]

server:
  secret_key: "auto-generated-64-hex-chars"
  limiter: false          # No rate limiting (local use)
  image_proxy: false

search:
  safe_search: 0
  default_lang: en
  formats:
    - html
    - json              # Required for Fetchium JSON API calls

outgoing:
  request_timeout: 4.0
  max_request_timeout: 7.0
  pool_connections: 200
  pool_maxsize: 64
  enable_http2: true`} />

      <h2>Manual Docker setup</h2>
      <p>If you prefer to set up SearXNG manually:</p>

      <CodeBlock language="bash" code={`# Create config directory
mkdir -p ~/.fetchium/searxng

# Generate a secret key
SECRET=$(openssl rand -hex 32)

# Create minimal settings.yml
cat > ~/.fetchium/searxng/settings.yml << EOF
use_default_settings: true
server:
  secret_key: "$SECRET"
  limiter: false
search:
  formats:
    - html
    - json
EOF

# Start the container
docker run -d \\
  --name fetchium-searxng \\
  --restart unless-stopped \\
  -p 4040:8080 \\
  -v ~/.fetchium/searxng:/etc/searxng:rw \\
  docker.io/searxng/searxng:latest

# Configure Fetchium to use it
fetchium config set search.searxng_url http://localhost:4040`} />

      <h2>Using a remote SearXNG instance</h2>
      <CodeBlock language="bash" code={`# Point to any public or private SearXNG instance
fetchium config set search.searxng_url https://searx.example.com

# Or via environment variable
export SEARXNG_URL=https://searx.example.com`} />

      <div className="callout">
        <strong>Note:</strong> Remote instances must have JSON format enabled in their
        settings (<code>formats: [html, json]</code>). Many public instances disable
        the JSON API to prevent abuse.
      </div>

      <h2>Troubleshooting</h2>
      <table>
        <thead><tr><th>Issue</th><th>Fix</th></tr></thead>
        <tbody>
          <tr><td>Container won&apos;t start</td><td><code>docker logs fetchium-searxng</code> — check for port conflicts</td></tr>
          <tr><td>Port 4040 in use</td><td>Change with <code>fetchium config set search.searxng_url http://localhost:4041</code> and re-run setup</td></tr>
          <tr><td>JSON API disabled</td><td>Add <code>- json</code> under <code>search.formats</code> in settings.yml</td></tr>
          <tr><td>No results from engine</td><td>Check Docker logs — engine may be rate-limited or blocked</td></tr>
        </tbody>
      </table>

      <h2>Check status</h2>
      <CodeBlock language="bash" code={`# Quick health check
fetchium setup --check

# Or curl directly
curl "http://localhost:4040/search?q=test&format=json" | jq .number_of_results`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/self-hosting/config", title: "Configuration", desc: "Full config.toml reference" },
          { href: "https://docs.fetchium.com/self-hosting/docker", title: "Docker Setup", desc: "Full stack deployment" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Use SearXNG-powered search" },
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
