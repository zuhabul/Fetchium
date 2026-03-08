import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Self-Hosting with Docker" };

export default function SelfHostingDocker() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Self-Hosting</div>
      <h1>Self-Hosting with Docker</h1>
      <p>
        Run Fetchium on your own infrastructure. The entire stack runs in Docker
        Compose: the Rust API, SearXNG search backend, and SQLite database.
      </p>

      <div className="callout">
        <strong>No request limits:</strong> Self-hosted deployments have no API quotas. You only pay
        for your own server costs.
      </div>

      <h2>Prerequisites</h2>
      <ul>
        <li>Docker Engine 24+ and Docker Compose v2</li>
        <li>2 GB RAM minimum (4 GB recommended)</li>
        <li>Linux, macOS, or Windows with WSL2</li>
      </ul>

      <h2>Quick start</h2>

      <CodeBlock language="bash" code={`# Clone the repository
git clone https://github.com/zuhabul/Fetchium.git
cd fetchium

# Copy and configure environment
cp infra/fetchium.env.production infra/fetchium.env.production.local
# Edit infra/fetchium.env.production.local to set FETCHIUM_ADMIN_SECRET and other settings

# Start the stack
docker compose --env-file infra/fetchium.env.production.local -f infra/docker-compose.prod.yml up -d --build

# Verify it's running
curl http://localhost:3050/v1/health
curl -X POST http://localhost:3471/mcp \\
  -H "Content-Type: application/json" \\
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'`} />

      <h2>Docker Compose configuration</h2>

      <CodeBlock language="yaml" filename="infra/docker-compose.prod.yml" code={`services:
  searxng:
    image: searxng/searxng:latest
    ports:
      - "127.0.0.1:4040:8080"
    restart: unless-stopped

  api:
    build:
      context: ..
      dockerfile: infra/Dockerfile
    command: ["fetchium", "serve", "--mode", "rest", "--port", "3050"]
    ports:
      - "127.0.0.1:3050:3050"
    environment:
      - FETCHIUM_ADMIN_SECRET=\${FETCHIUM_ADMIN_SECRET}
      - SEARXNG_URL=http://searxng:8080
      - FETCHIUM_DATA_DIR=/data
    volumes:
      - fetchium-data:/data
    depends_on:
      - searxng
    restart: unless-stopped

  mcp:
    build:
      context: ..
      dockerfile: infra/Dockerfile
    command: ["fetchium", "serve", "--mode", "mcp", "--transport", "http", "--port", "3471"]
    ports:
      - "127.0.0.1:3471:3471"
    environment:
      - SEARXNG_URL=http://searxng:8080
      - FETCHIUM_DATA_DIR=/data
    volumes:
      - fetchium-data:/data
    depends_on:
      - searxng
    restart: unless-stopped

volumes:
  fetchium-data:`} />

      <h2>Environment variables</h2>
      <table>
        <thead><tr><th>Variable</th><th>Required</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>FETCHIUM_ADMIN_SECRET</code></td><td>Yes</td><td>Admin API secret (min 32 chars). Generate: <code>openssl rand -hex 32</code></td></tr>
          <tr><td><code>SEARXNG_URL</code></td><td>Yes</td><td>URL of your SearXNG instance</td></tr>
          <tr><td><code>FETCHIUM_DATA_DIR</code></td><td>No</td><td>SQLite/auth data directory. Default: <code>/data</code> in containers</td></tr>
          <tr><td><code>BRAVE_API_KEY</code></td><td>No</td><td>Optional Brave Search backend</td></tr>
          <tr><td><code>YOUTUBE_API_KEY</code></td><td>No</td><td>Optional YouTube integration</td></tr>
          <tr><td><code>RUST_LOG</code></td><td>No</td><td><code>error</code>, <code>warn</code>, <code>info</code>, <code>debug</code>, <code>trace</code></td></tr>
        </tbody>
      </table>

      <h2>Creating your first API key</h2>
      <p>
        After the stack is running, create an API key using the admin secret:
      </p>

      <CodeBlock language="bash" code={`curl -X POST http://localhost:3050/v1/keys \\
  -H "X-Admin-Secret: your_admin_secret_here" \\
  -H "Content-Type: application/json" \\
  -d '{"name": "My App", "plan": "pro"}'`} />

      <CodeBlock language="json" code={`{
  "meta": {
    "request_id": "req_01...",
    "status": "ok",
    "endpoint": "/v1/keys",
    "duration_ms": 3
  },
  "key": "fetchium_4626d3fc3fd6693aaaf2d8f5fd084a71...",
  "id": "key_abc123",
  "name": "My App",
  "plan": "pro",
  "created_at": "2025-06-20T14:30:00Z"
}`} />

      <h2>Building from source</h2>

      <CodeBlock language="bash" code={`# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/zuhabul/Fetchium.git
cd fetchium
cargo build -p fetchium-cli --release

# Run
FETCHIUM_ADMIN_SECRET=your_secret \\
SEARXNG_URL=http://localhost:8080 \\
  ./target/release/fetchium serve --port 3050`} />

      <h2>Reverse proxy with nginx</h2>

      <CodeBlock language="nginx" filename="nginx.conf" code={`server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    ssl_certificate /etc/ssl/certs/yourdomain.crt;
    ssl_certificate_key /etc/ssl/private/yourdomain.key;

    location / {
        proxy_pass http://localhost:3050;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 90s;
    }
}`} />

      <h2>Updating</h2>
      <CodeBlock language="bash" code={`docker compose --env-file infra/fetchium.env.production.local -f infra/docker-compose.prod.yml pull
docker compose --env-file infra/fetchium.env.production.local -f infra/docker-compose.prod.yml up -d --build`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/quickstart", title: "Quick Start", desc: "Make your first API call" },
          { href: "https://docs.fetchium.com/authentication", title: "Authentication", desc: "Key management" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Full API reference" },
          { href: "https://github.com/zuhabul/Fetchium", title: "GitHub", desc: "Source code and issues" },
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
