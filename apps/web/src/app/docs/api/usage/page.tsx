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
        Monitor your API quota consumption, view per-endpoint breakdowns, and check your plan limits
        programmatically.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-sky-500/15 text-sky-300 border-sky-500/30">GET</span>
        <span className="text-slate-300">/v1/usage</span>
      </div>

      <h2>Example request</h2>
      <CodeBlock language="bash" code={`curl https://api.hypersearchx.zuhabul.com/v1/usage \\
  -H "Authorization: Bearer hsx_your_key"`} />

      <h2>Response</h2>
      <CodeBlock language="json" filename="response.json" code={`{
  "plan": "pro",
  "quota": 100000,
  "used": 12847,
  "remaining": 87153,
  "reset_at": "2025-07-01T00:00:00Z",
  "per_minute_limit": 500,
  "endpoints": {
    "search": 10234,
    "research": 1892,
    "scrape": 721
  },
  "daily": [
    { "date": "2025-06-19", "requests": 542 },
    { "date": "2025-06-20", "requests": 891 }
  ]
}`} />

      <h2>Health endpoint</h2>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-sky-500/15 text-sky-300 border-sky-500/30">GET</span>
        <span className="text-slate-300">/health</span>
      </div>

      <p>
        The health endpoint does not require authentication. Returns <code>200 OK</code> when all
        dependencies are healthy, <code>503</code> if any critical dependency is down.
      </p>

      <CodeBlock language="bash" code={`curl https://api.hypersearchx.zuhabul.com/health`} />

      <CodeBlock language="json" code={`{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "dependencies": {
    "search_backend": "healthy",
    "database": "healthy"
  }
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/rate-limits", title: "Rate Limits", desc: "Quotas and retry strategies" },
          { href: "/docs/api/search", title: "Search API", desc: "Full search reference" },
          { href: "/docs/authentication", title: "Authentication", desc: "Key management" },
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
