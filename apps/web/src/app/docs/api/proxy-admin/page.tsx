import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Proxy Admin API Reference" };

export default function ProxyAdminApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Proxy Admin</h1>
      <p>
        Admin-only proxy management endpoints for inspecting and maintaining the Fetchium proxy
        pool. These routes use <code>X-Admin-Secret</code>, not Bearer auth.
      </p>

      <h2>Endpoints</h2>
      <table>
        <thead><tr><th>Method</th><th>Path</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><code>GET</code></td><td><code>/v1/proxy/stats</code></td><td>Inspect pool summary and per-proxy stats</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/proxy/reset</code></td><td>Reset all proxies to active</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/proxy/purge</code></td><td>Remove dead proxies from the pool</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/proxy/test</code></td><td>Run a live test through one proxy</td></tr>
        </tbody>
      </table>

      <h2>Inspect pool state</h2>
      <CodeBlock language="bash" code={`curl ***REMOVED***/v1/proxy/stats \\
  -H "X-Admin-Secret: $***REMOVED***"`} />

      <CodeBlock language="json" code={`{
  "enabled": true,
  "summary": {
    "total": 12,
    "active": 10,
    "dead": 2
  },
  "proxies": [
    {
      "host": "203.0.113.10",
      "port": 8080,
      "successes": 184,
      "failures": 6,
      "last_latency_ms": 921
    }
  ]
}`} />

      <h2>Reset and purge</h2>
      <CodeBlock language="bash" code={`curl -X POST ***REMOVED***/v1/proxy/reset \\
  -H "X-Admin-Secret: $***REMOVED***"

curl -X POST ***REMOVED***/v1/proxy/purge \\
  -H "X-Admin-Secret: $***REMOVED***"`} />

      <h2>Test one proxy</h2>
      <CodeBlock language="bash" code={`curl -X POST ***REMOVED***/v1/proxy/test \\
  -H "X-Admin-Secret: $***REMOVED***"`} />

      <div className="callout">
        These endpoints are operational controls for self-hosted or internal admin deployments.
        If proxy rotation is disabled, the API returns a JSON response with <code>enabled: false</code>
        or a status message explaining that proxy rotation is not configured.
      </div>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/admin-keys", title: "Admin Keys", desc: "Manage API keys with the same admin secret" },
          { href: "https://docs.fetchium.com/self-hosting/config", title: "Configuration", desc: "Enable and tune proxy rotation" },
          { href: "https://docs.fetchium.com/self-hosting/docker", title: "Docker Setup", desc: "Run the API in production" },
          { href: "https://docs.fetchium.com/errors", title: "Error Reference", desc: "Current admin error formats" },
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
