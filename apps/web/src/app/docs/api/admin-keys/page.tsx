import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Admin Keys API Reference" };

export default function AdminKeysApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Admin Keys API</h1>
      <p>
        Admin-only endpoints for API key lifecycle management. These endpoints require
        <code> X-Admin-Secret</code> header, not Bearer auth.
      </p>

      <h2>Endpoints</h2>
      <table>
        <thead><tr><th>Method</th><th>Path</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><code>POST</code></td><td><code>/v1/keys</code></td><td>Create API key</td></tr>
          <tr><td><code>GET</code></td><td><code>/v1/keys</code></td><td>List active API keys (masked)</td></tr>
          <tr><td><code>DELETE</code></td><td><code>/v1/keys/:id</code></td><td>Revoke API key</td></tr>
        </tbody>
      </table>

      <h2>Create key</h2>
      <CodeBlock language="bash" code={`curl -X POST ***REMOVED***/v1/keys \\
  -H "X-Admin-Secret: $***REMOVED***" \\
  -H "Content-Type: application/json" \\
  -d '{"name":"prod-service","plan":"pro"}'`} />

      <CodeBlock language="json" code={`{
  "meta": {
    "request_id": "09a2fd8c-bac4-46ab-bd84-c75d363f80f3",
    "status": "ok",
    "endpoint": "/v1/keys",
    "duration_ms": 4
  },
  "key": "fetchium_...",
  "id": "a8f31bdb-0456-44b6-a405-d4ad7dfd2cf7",
  "name": "prod-service",
  "plan": "pro",
  "created_at": "2026-03-02T19:20:40.424897+00:00",
  "warning": "This key will not be shown again. Store it securely."
}`} />

      <h2>List keys</h2>
      <CodeBlock language="bash" code={`curl ***REMOVED***/v1/keys \\
  -H "X-Admin-Secret: $***REMOVED***"`} />

      <CodeBlock language="json" code={`{
  "meta": {
    "request_id": "a5c1f65f-a4f4-4c22-b24e-143d0bfe0553",
    "status": "ok",
    "endpoint": "/v1/keys",
    "duration_ms": 1
  },
  "keys": [
    {
      "id": "a8f31bdb-0456-44b6-a405-d4ad7dfd2cf7",
      "name": "prod-service",
      "key_preview": "fetchium_4626...****",
      "plan": "pro",
      "created_at": "2026-03-02T19:20:40.424897+00:00",
      "last_used_at": "2026-03-08T17:00:00+00:00"
    }
  ],
  "count": 1
}`} />

      <h2>Revoke key</h2>
      <CodeBlock language="bash" code={`curl -X DELETE ***REMOVED***/v1/keys/a8f31bdb-0456-44b6-a405-d4ad7dfd2cf7 \\
  -H "X-Admin-Secret: $***REMOVED***"`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/authentication", title: "Authentication", desc: "Bearer and admin auth model" },
          { href: "https://docs.fetchium.com/api/usage", title: "Usage API", desc: "Per-key usage" },
          { href: "https://docs.fetchium.com/api/health", title: "Health API", desc: "Liveness and checks" },
          { href: "https://docs.fetchium.com/api/proxy-admin", title: "Proxy Admin", desc: "Operational controls for the proxy pool" },
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
