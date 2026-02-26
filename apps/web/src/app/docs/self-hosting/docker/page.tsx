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
        Run HyperSearchX on your own infrastructure at no cost. The entire stack runs in Docker
        Compose: the Rust API, SearXNG search backend, and SQLite database.
      </p>

      <div className="callout">
        <strong>Free forever:</strong> Self-hosting has no request limits or pricing. You only pay
        for your own server costs. The source code is MIT-licensed.
      </div>

      <h2>Prerequisites</h2>
      <ul>
        <li>Docker Engine 24+ and Docker Compose v2</li>
        <li>2 GB RAM minimum (4 GB recommended)</li>
        <li>Linux, macOS, or Windows with WSL2</li>
      </ul>

      <h2>Quick start</h2>

      <CodeBlock language="bash" code={`# Clone the repository
git clone https://github.com/zuhabul/hypersearchx.git
cd hypersearchx

# Copy and configure environment
cp .env.example .env
# Edit .env to set HSX_ADMIN_SECRET and other settings

# Start the stack
docker compose up -d

# Verify it's running
curl http://localhost:3050/health`} />

      <h2>Docker Compose configuration</h2>

      <CodeBlock language="yaml" filename="docker-compose.yml" code={`version: "3.9"

services:
  hsx-api:
    image: ghcr.io/zuhabul/hypersearchx:latest
    ports:
      - "3050:3050"
    environment:
      - HSX_ADMIN_SECRET=\${HSX_ADMIN_SECRET}
      - SEARXNG_URL=http://searxng:8080
      - DATABASE_PATH=/data/hsx.db
    volumes:
      - hsx-data:/data
    depends_on:
      - searxng
    restart: unless-stopped

  searxng:
    image: searxng/searxng:latest
    volumes:
      - ./searxng:/etc/searxng
    restart: unless-stopped

volumes:
  hsx-data:`} />

      <h2>Environment variables</h2>
      <table>
        <thead><tr><th>Variable</th><th>Required</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>HSX_ADMIN_SECRET</code></td><td>Yes</td><td>Admin API secret (min 32 chars). Generate: <code>openssl rand -hex 32</code></td></tr>
          <tr><td><code>SEARXNG_URL</code></td><td>Yes</td><td>URL of your SearXNG instance</td></tr>
          <tr><td><code>DATABASE_PATH</code></td><td>No</td><td>SQLite database path. Default: <code>~/.hypersearchx/hsx.db</code></td></tr>
          <tr><td><code>HSX_PORT</code></td><td>No</td><td>API port. Default: <code>3050</code></td></tr>
          <tr><td><code>HSX_LOG_LEVEL</code></td><td>No</td><td><code>debug</code>, <code>info</code>, <code>warn</code>, <code>error</code>. Default: <code>info</code></td></tr>
          <tr><td><code>ALLOWED_ORIGINS</code></td><td>No</td><td>Comma-separated CORS origins. Default: none (API only)</td></tr>
        </tbody>
      </table>

      <h2>Creating your first API key</h2>
      <p>
        After the stack is running, create an API key using the admin secret:
      </p>

      <CodeBlock language="bash" code={`curl -X POST http://localhost:3050/v1/admin/keys \\
  -H "X-Admin-Secret: your_admin_secret_here" \\
  -H "Content-Type: application/json" \\
  -d '{"name": "My App", "tier": "unlimited"}'`} />

      <CodeBlock language="json" code={`{
  "key": "hsx_4626d3fc3fd6693aaaf2d8f5fd084a71...",
  "id": "key_abc123",
  "name": "My App",
  "tier": "unlimited",
  "created_at": "2025-06-20T14:30:00Z"
}`} />

      <h2>Building from source</h2>

      <CodeBlock language="bash" code={`# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/zuhabul/hypersearchx.git
cd hypersearchx
cargo build -p hsx-cli --release

# Run
HSX_ADMIN_SECRET=your_secret \\
SEARXNG_URL=http://localhost:8080 \\
  ./target/release/hsx serve --port 3050`} />

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
      <CodeBlock language="bash" code={`# Pull latest image and restart
docker compose pull
docker compose up -d`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/quickstart", title: "Quick Start", desc: "Make your first API call" },
          { href: "/docs/authentication", title: "Authentication", desc: "Key management" },
          { href: "/docs/api/search", title: "Search API", desc: "Full API reference" },
          { href: "https://github.com/zuhabul/hypersearchx", title: "GitHub", desc: "Source code and issues" },
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
