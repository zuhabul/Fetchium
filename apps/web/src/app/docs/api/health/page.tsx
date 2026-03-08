import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Health Check API" };

export default function HealthApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Health Check</h1>
      <p>
        Public liveness and readiness endpoint for Fetchium API. This endpoint requires no
        authentication and is intended for uptime checks and load balancers.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-sky-500/15 text-sky-300 border-sky-500/30">GET</span>
        <span className="text-slate-300">/health</span>
      </div>

      <p>
        Equivalent path: <code>/v1/health</code>.
      </p>

      <h2>Example request</h2>
      <CodeBlock language="bash" code={`curl https://api.fetchium.com/health`} />

      <h2>Response (healthy)</h2>
      <CodeBlock language="json" filename="healthy.json" code={`{
  "status": "ok",
  "version": "1.0.0",
  "checks": {
    "api": "ok",
    "search_backbone": "ok",
    "auth_store": "ok"
  },
  "timestamp": "2026-03-02T19:08:06.057021096+00:00"
}`} />

      <h2>Response (degraded)</h2>
      <CodeBlock language="json" filename="degraded.json" code={`{
  "status": "degraded",
  "version": "1.0.0",
  "checks": {
    "api": "ok",
    "search_backbone": "degraded",
    "auth_store": "ok"
  },
  "timestamp": "2026-03-02T19:08:06.057021096+00:00"
}`} />

      <h2>HTTP status codes</h2>
      <table>
        <thead><tr><th>Code</th><th><code>status</code></th><th>Meaning</th></tr></thead>
        <tbody>
          <tr><td><code>200</code></td><td><code>ok</code></td><td>API and critical dependencies are healthy</td></tr>
          <tr><td><code>200</code></td><td><code>degraded</code></td><td>API is up but search backbone check failed</td></tr>
          <tr><td><code>503</code></td><td><code>unavailable</code></td><td>Critical auth store is unavailable</td></tr>
        </tbody>
      </table>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/usage", title: "Usage API", desc: "Quota and monthly usage" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Core search endpoint" },
          { href: "https://docs.fetchium.com/api/estimate", title: "Estimate API", desc: "Token estimate before fetch" },
          { href: "https://docs.fetchium.com/errors", title: "Error Reference", desc: "Common error formats" },
        ].map((l) => (
          <Link key={l.href} href={l.href} className="glass-card rounded-xl p-4 no-underline group">
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{l.title} →</div>
            <div className="text-xs text-slate-500 mt-1">{l.desc}</div>
          </Link>
        ))}
      </div>
    </article>
  );
}
