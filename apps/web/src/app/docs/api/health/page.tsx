import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Health Check API" };

const BADGE = ({ method }: { method: string }) => {
  const colors: Record<string, string> = {
    GET: "bg-sky-500/15 text-sky-300 border-sky-500/30",
  };
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border ${colors[method] ?? ""}`}>
      {method}
    </span>
  );
};

export default function HealthApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Health Check</h1>
      <p>
        The health endpoint provides real-time status of the Fetchium API server and
        all connected services. Useful for monitoring, load balancers, and uptime checks.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <BADGE method="GET" />
        <span className="text-slate-300">/health</span>
      </div>

      <div className="callout">
        <strong>No authentication required.</strong> The health endpoint is publicly accessible
        for use with load balancers and monitoring services.
      </div>

      <h2>Example request</h2>

      <CodeBlock language="bash" code={`curl https://api.fetchium.com/health`} />

      <h2>Response — healthy</h2>

      <CodeBlock language="json" filename="healthy.json" code={`{
  "status": "ok",
  "version": "1.0.0",
  "uptime_secs": 86412,
  "timestamp": "2025-11-20T14:30:00Z",
  "services": {
    "database": { "status": "ok", "latency_ms": 1 },
    "cache": { "status": "ok", "entries": 4821, "size_mb": 12.4 },
    "searxng": { "status": "ok", "url": "http://localhost:4040", "latency_ms": 23 },
    "intelligence": { "status": "ok", "sources_tracked": 1247 }
  },
  "backends": {
    "brave": { "status": "ok", "success_rate": 0.98 },
    "stackoverflow": { "status": "ok", "success_rate": 0.99 },
    "github": { "status": "ok", "success_rate": 0.97 },
    "reddit": { "status": "ok", "success_rate": 0.95 },
    "youtube": { "status": "ok", "success_rate": 0.99 }
  }
}`} />

      <h2>Response — degraded</h2>

      <CodeBlock language="json" filename="degraded.json" code={`{
  "status": "degraded",
  "version": "1.0.0",
  "uptime_secs": 3600,
  "timestamp": "2025-11-20T14:30:00Z",
  "services": {
    "database": { "status": "ok", "latency_ms": 1 },
    "cache": { "status": "ok", "entries": 100, "size_mb": 0.2 },
    "searxng": { "status": "unreachable", "url": "http://localhost:4040", "error": "connection refused" },
    "intelligence": { "status": "ok", "sources_tracked": 12 }
  }
}`} />

      <h2>HTTP status codes</h2>
      <table>
        <thead><tr><th>Code</th><th>status field</th><th>Meaning</th></tr></thead>
        <tbody>
          <tr><td><code>200</code></td><td><code>ok</code></td><td>All services healthy</td></tr>
          <tr><td><code>200</code></td><td><code>degraded</code></td><td>Partially operational (some backends down)</td></tr>
          <tr><td><code>503</code></td><td><code>unavailable</code></td><td>Critical service failure (database unreachable)</td></tr>
        </tbody>
      </table>

      <h2>Monitoring integration</h2>

      <CodeBlock language="bash" filename="uptime-check.sh" code={`#!/bin/bash
# Simple uptime check — exit 0 if healthy, 1 if not
STATUS=$(curl -sf https://api.fetchium.com/health | jq -r .status)
if [ "$STATUS" = "ok" ] || [ "$STATUS" = "degraded" ]; then
  echo "API is UP (status: $STATUS)"
  exit 0
else
  echo "API is DOWN"
  exit 1
fi`} />

      <CodeBlock language="yaml" filename="docker-healthcheck" code={`services:
  fetchium-api:
    image: ghcr.io/zuhabul/fetchium:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3050/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/usage", title: "Usage Stats", desc: "Token and request quotas" },
          { href: "/docs/api/search", title: "Search API", desc: "Core search endpoint" },
          { href: "/docs/self-hosting/docker", title: "Docker Setup", desc: "Self-hosted deployment" },
          { href: "/docs/errors", title: "Error Reference", desc: "All error codes" },
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
