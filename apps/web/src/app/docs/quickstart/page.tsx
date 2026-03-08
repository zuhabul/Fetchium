import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Quick Start" };

export default function Quickstart() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Getting Started</div>
      <h1>Quick Start</h1>
      <p>
        This guide will get you from zero to your first API response in under 2 minutes.
      </p>

      <h2>Step 1 — Get an API key</h2>
      <p>
        Sign up at the <Link href="https://app.fetchium.com">dashboard</Link>.
        The Free tier gives you 1,000 requests/month with no credit card required.
        Your API key will look like: <code>fetchium_4626d3fc3fd669...</code>
      </p>

      <div className="callout">
        Store your API key securely. It is shown <strong>only once</strong> at creation time.
        If lost, create a new key from the dashboard.
      </div>

      <h2>Step 2 — Make a search request</h2>
      <p>Export your key first, then reuse it in examples:</p>

      <CodeBlock language="bash" filename="env.sh" code={`export FETCHIUM_API_KEY="fetchium_..."`} />

      <CodeBlock language="bash" filename="search.sh" code={`curl -X POST https://api.fetchium.com/v1/search \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query": "what is tokio in rust", "tier": "summary"}'`} />

      <h2>Step 3 — Understand the response</h2>
      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "query": "what is tokio in rust",
    "tier": "summary",
    "tokens_used": 1024,
    "sources_count": 8,
    "duration_ms": 987,
    "result_id": "a1b2c3d4-..."
  },
  "results": [
    {
      "title": "Tokio — An asynchronous Rust runtime",
      "url": "https://tokio.rs",
      "snippet": "Tokio is an event-driven, non-blocking I/O platform for writing asynchronous applications with Rust...",
      "score": 0.941
    }
  ]
}`} />

      <p>Key fields:</p>
      <ul>
        <li><code>meta.tokens_used</code> — tokens consumed from your budget</li>
        <li><code>meta.sources_count</code> — number of sources searched across all backends</li>
        <li><code>results[].score</code> — HyperFusion neural ranking score (0–1)</li>
        <li><code>results[].snippet</code> — token-budgeted extracted content</li>
      </ul>

      <h2>Step 4 — Try different tiers</h2>
      <p>The <code>tier</code> parameter controls how much content is extracted:</p>

      <CodeBlock language="bash" code={`# Just key facts (~200 tokens) — fast, cheap
curl -X POST .../v1/search -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -d '{"query": "python vs rust performance", "tier": "key_facts"}'

# Detailed analysis (~5,000 tokens) — thorough
curl -X POST .../v1/search -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -d '{"query": "best approaches to async rust", "tier": "detailed"}'`} />

      <h2>Step 5 — Try the research pipeline</h2>
      <p>The research endpoint runs a full multi-step investigation with citations:</p>

      <CodeBlock language="bash" code={`curl -X POST https://api.fetchium.com/v1/research \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "Compare vector databases for production use in 2025",
    "max_sources": 8,
    "citation_style": "apa"
  }'`} />

      <h2>Step 6 — Switch to async jobs for long-running work</h2>
      <p>
        For longer research, YouTube, or social runs, submit an async job and poll its status:
      </p>
      <CodeBlock language="bash" code={`curl -X POST https://api.fetchium.com/v1/research/jobs \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query":"compare Rust web frameworks","strict_evidence":true}'

curl https://api.fetchium.com/v1/jobs/YOUR_JOB_ID \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY"`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/search", title: "Search API Reference", desc: "Full parameter docs" },
          { href: "https://docs.fetchium.com/api/research", title: "Research API Reference", desc: "Deep research pipeline" },
          { href: "https://docs.fetchium.com/authentication", title: "Authentication", desc: "Key management, rotation" },
          { href: "https://docs.fetchium.com/rate-limits", title: "Rate Limits", desc: "Quota behavior and 429 handling" },
          { href: "https://docs.fetchium.com/api/async-jobs", title: "Async Jobs", desc: "Queued execution for long-running pipelines" },
        ].slice(0, 4).map(l => (
          <Link key={l.href} href={l.href} className="glass-card rounded-xl p-4 no-underline group">
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{l.title} →</div>
            <div className="text-xs text-slate-500 mt-1">{l.desc}</div>
          </Link>
        ))}
      </div>
    </article>
  );
}
