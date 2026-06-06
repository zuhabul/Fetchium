import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Search API Reference" };

export default function SearchApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Search API</h1>
      <p>
        Federated web search with ranking and token-budgeted response shaping.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/search</span>
      </div>

      <h2>Request body</h2>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Required</th><th>Notes</th></tr></thead>
        <tbody>
          <tr><td><code>query</code></td><td>string</td><td>Yes</td><td>1-500 chars</td></tr>
          <tr><td><code>token_budget</code></td><td>integer</td><td>No</td><td>100-10,000</td></tr>
          <tr><td><code>tier</code></td><td>string</td><td>No</td><td><code>key_facts</code> | <code>summary</code> | <code>detailed</code> | <code>complete</code></td></tr>
          <tr><td><code>max_sources</code></td><td>integer</td><td>No</td><td>1-20</td></tr>
          <tr><td><code>validate</code></td><td>boolean</td><td>No</td><td>Optional additional validation toggle</td></tr>
          <tr><td><code>include_content</code></td><td>boolean</td><td>No</td><td>Inline extracted content for each result</td></tr>
        </tbody>
      </table>

      <h2>Example request</h2>
      <CodeBlock language="bash" filename="search.sh" code={`curl -X POST https://api.fetchium.com/v1/search \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust async programming best practices",
    "tier": "detailed",
    "max_sources": 8,
    "token_budget": 5000,
    "include_content": true
  }'`} />

      <h2>Response</h2>
      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "query": "rust async programming best practices",
    "tier": "detailed",
    "tokens_used": 4821,
    "sources_count": 8,
    "duration_ms": 1247,
    "result_id": "09742460-58db-44fd-9036-502ed26565e2"
  },
  "results": [
    {
      "title": "Async in Rust: A comprehensive guide",
      "url": "https://tokio.rs/tokio/tutorial",
      "snippet": "Tokio is an asynchronous runtime for Rust...",
      "score": 0.941,
      "content": "Async Rust lets you interleave waiting work while preserving structured code."
    }
  ]
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/scrape", title: "Scrape API", desc: "Single URL extraction" },
          { href: "https://docs.fetchium.com/api/research", title: "Research API", desc: "Multi-source synthesis" },
          { href: "https://docs.fetchium.com/api/async-jobs", title: "Async Jobs", desc: "Queue longer-running pipelines" },
          { href: "https://docs.fetchium.com/api/estimate", title: "Estimate API", desc: "Token estimation" },
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
