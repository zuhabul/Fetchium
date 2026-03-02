import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Usage API Reference" };

export default function UsageApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Usage API</h1>
      <p>
        Returns usage statistics for the currently authenticated API key.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-sky-500/15 text-sky-300 border-sky-500/30">GET</span>
        <span className="text-slate-300">/v1/usage</span>
      </div>

      <h2>Example request</h2>
      <CodeBlock language="bash" code={`curl https://api.fetchium.com/v1/usage \\
  -H "Authorization: Bearer fetchium_your_key"`} />

      <h2>Response</h2>
      <CodeBlock language="json" filename="response.json" code={`{
  "key_id": "9e57a4f8-2f8a-4f7b-b527-2f5cfca2ad2f",
  "plan": "pro",
  "requests_this_month": 12847,
  "requests_today": 212,
  "tokens_this_month": 4912032,
  "monthly_limit": 250000,
  "quota_remaining": 237153
}`} />

      <h2>Field notes</h2>
      <table>
        <thead><tr><th>Field</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>monthly_limit</code></td><td><code>null</code> for unlimited plans</td></tr>
          <tr><td><code>quota_remaining</code></td><td><code>null</code> when no monthly cap exists</td></tr>
          <tr><td><code>tokens_this_month</code></td><td>Recorded token usage for current month</td></tr>
        </tbody>
      </table>

      <h2>Related health endpoint</h2>
      <CodeBlock language="bash" code={`curl https://api.fetchium.com/health`} />

      <CodeBlock language="json" code={`{
  "status": "ok",
  "version": "1.0.0",
  "checks": {
    "api": "ok",
    "search_backbone": "ok",
    "auth_store": "ok"
  },
  "timestamp": "2026-03-02T19:08:06.057021096+00:00"
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/rate-limits", title: "Rate Limits", desc: "Quota policies and limits" },
          { href: "/docs/api/search", title: "Search API", desc: "Core search endpoint" },
          { href: "/docs/authentication", title: "Authentication", desc: "Key and admin secret setup" },
          { href: "/docs/errors", title: "Error Reference", desc: "Error formats and codes" },
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
