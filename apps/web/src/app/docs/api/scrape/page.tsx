import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Scrape API Reference" };

export default function ScrapeApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Scrape (Fetch) API</h1>
      <p>
        Fetch and extract content from a single URL.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/scrape</span>
      </div>

      <p>
        Alias endpoint: <code>/v1/fetch</code> (same behavior).
      </p>

      <h2>Request body</h2>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Required</th><th>Notes</th></tr></thead>
        <tbody>
          <tr><td><code>url</code></td><td>string</td><td>Yes</td><td>HTTP(S) URL, max 2048 chars</td></tr>
          <tr><td><code>query</code></td><td>string</td><td>No</td><td>Optional relevance hint for extraction</td></tr>
          <tr><td><code>token_budget</code></td><td>integer</td><td>No</td><td>100-10,000</td></tr>
          <tr><td><code>format</code></td><td>string</td><td>No</td><td><code>markdown</code> | <code>text</code> | <code>html</code></td></tr>
          <tr><td><code>schema</code></td><td>object</td><td>No</td><td>Optional JSON schema for structured extraction</td></tr>
        </tbody>
      </table>

      <h2>Example request</h2>
      <CodeBlock language="bash" filename="scrape.sh" code={`curl -X POST ***REMOVED***/v1/scrape \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://tokio.rs/tokio/tutorial/hello-tokio",
    "format": "markdown",
    "token_budget": 3000
  }'`} />

      <h2>Response</h2>
      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "request_id": "19b3d621-e7a6-44dc-875b-3104badb29a4",
    "status": "ok",
    "endpoint": "/v1/scrape",
    "duration_ms": 428,
    "tier": "markdown",
    "tokens_used": 2871,
    "result_id": "f1e2d3c4-0c80-4637-aa62-bf4ec6a714af"
  },
  "url": "https://tokio.rs/tokio/tutorial/hello-tokio",
  "title": "Hello Tokio - Tokio",
  "content": "Tokio is an asynchronous runtime for Rust...",
  "tokens": 2871,
  "format": "markdown",
  "result_id": "f1e2d3c4-0c80-4637-aa62-bf4ec6a714af"
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Find candidate URLs" },
          { href: "https://docs.fetchium.com/api/research", title: "Research API", desc: "Synthesize across sources" },
          { href: "https://docs.fetchium.com/api/estimate", title: "Estimate API", desc: "Estimate token cost" },
          { href: "https://docs.fetchium.com/errors", title: "Error Reference", desc: "Error payloads" },
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
