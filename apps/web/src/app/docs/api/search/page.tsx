import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Search API Reference" };

const BADGE = ({ method }: { method: string }) => {
  const colors: Record<string, string> = {
    POST: "bg-indigo-500/15 text-indigo-300 border-indigo-500/30",
    GET: "bg-sky-500/15 text-sky-300 border-sky-500/30",
  };
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border ${colors[method] ?? ""}`}>
      {method}
    </span>
  );
};

export default function SearchApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Search API</h1>
      <p>
        The Search API performs federated multi-backend web search with HyperFusion neural ranking,
        QATBE token-budgeted extraction, and SCS semantic segmentation — all in a single request.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <BADGE method="POST" />
        <span className="text-slate-300">/v1/search</span>
      </div>

      <h2>Request</h2>

      <h3>Headers</h3>
      <table>
        <thead><tr><th>Header</th><th>Value</th><th>Required</th></tr></thead>
        <tbody>
          <tr><td><code>Authorization</code></td><td><code>Bearer fetchium_...</code></td><td>Yes</td></tr>
          <tr><td><code>Content-Type</code></td><td><code>application/json</code></td><td>Yes</td></tr>
        </tbody>
      </table>

      <h3>Body parameters</h3>
      <table>
        <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
        <tbody>
          <tr>
            <td><code>query</code></td><td>string</td><td>Yes</td>
            <td>Search query. Max 500 characters.</td>
          </tr>
          <tr>
            <td><code>tier</code></td><td>string</td><td>No</td>
            <td>
              Detail level: <code>key_facts</code> (~200 tok), <code>summary</code> (~1k tok, default),{" "}
              <code>detailed</code> (~5k tok), <code>complete</code> (~20k tok).
            </td>
          </tr>
          <tr>
            <td><code>token_budget</code></td><td>integer</td><td>No</td>
            <td>Override tier with exact token budget. Range: 100–10,000.</td>
          </tr>
          <tr>
            <td><code>max_sources</code></td><td>integer</td><td>No</td>
            <td>Maximum sources to search. Range: 1–20. Default: 10.</td>
          </tr>
          <tr>
            <td><code>backends</code></td><td>string[]</td><td>No</td>
            <td>
              Force specific backends:{" "}
              <code>brave</code>, <code>google</code>, <code>bing</code>,{" "}
              <code>stackoverflow</code>, <code>github</code>, <code>reddit</code>.
              Default: auto-selected by ABS algorithm.
            </td>
          </tr>
          <tr>
            <td><code>language</code></td><td>string</td><td>No</td>
            <td>BCP-47 language code for results (e.g. <code>en</code>, <code>de</code>). Default: <code>en</code>.</td>
          </tr>
          <tr>
            <td><code>freshness</code></td><td>string</td><td>No</td>
            <td>
              Filter by date: <code>day</code>, <code>week</code>, <code>month</code>, <code>year</code>.
              Default: no filter.
            </td>
          </tr>
          <tr>
            <td><code>include_domains</code></td><td>string[]</td><td>No</td>
            <td>Restrict results to these domains.</td>
          </tr>
          <tr>
            <td><code>exclude_domains</code></td><td>string[]</td><td>No</td>
            <td>Exclude results from these domains.</td>
          </tr>
        </tbody>
      </table>

      <h2>Example request</h2>

      <CodeBlock language="bash" filename="search.sh" code={`curl -X POST https://api.fetchium.com/v1/search \\
  -H "Authorization: Bearer fetchium_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust async programming best practices",
    "tier": "detailed",
    "max_sources": 8,
    "freshness": "year"
  }'`} />

      <CodeBlock language="typescript" filename="search.ts" code={`const res = await fetch("https://api.fetchium.com/v1/search", {
  method: "POST",
  headers: {
    "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    query: "rust async programming best practices",
    tier: "detailed",
    max_sources: 8,
    freshness: "year",
  }),
});
const data = await res.json();`} />

      <CodeBlock language="python" filename="search.py" code={`import os, requests

r = requests.post(
    "https://api.fetchium.com/v1/search",
    headers={"Authorization": f"Bearer {os.environ['FETCHIUM_API_KEY']}"},
    json={
        "query": "rust async programming best practices",
        "tier": "detailed",
        "max_sources": 8,
        "freshness": "year",
    }
)
r.raise_for_status()
data = r.json()`} />

      <h2>Response</h2>

      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "query": "rust async programming best practices",
    "tier": "detailed",
    "tokens_used": 4821,
    "sources_count": 8,
    "duration_ms": 1247,
    "result_id": "09742460-58db-44fd-9036-502ed26565e2",
    "backends_used": ["brave", "stackoverflow", "github"]
  },
  "results": [
    {
      "title": "Async in Rust: A comprehensive guide",
      "url": "https://tokio.rs/tokio/tutorial",
      "domain": "tokio.rs",
      "snippet": "Tokio is an asynchronous runtime for the Rust programming language...",
      "score": 0.941,
      "published_at": "2024-11-15T00:00:00Z",
      "source_type": "documentation"
    }
  ]
}`} />

      <h3>Response fields</h3>

      <h4>meta</h4>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>query</code></td><td>string</td><td>The query as processed (post-expansion)</td></tr>
          <tr><td><code>tier</code></td><td>string</td><td>Detail tier used</td></tr>
          <tr><td><code>tokens_used</code></td><td>integer</td><td>Tokens consumed from your budget</td></tr>
          <tr><td><code>sources_count</code></td><td>integer</td><td>Total sources searched across all backends</td></tr>
          <tr><td><code>duration_ms</code></td><td>integer</td><td>End-to-end latency in milliseconds</td></tr>
          <tr><td><code>result_id</code></td><td>string</td><td>UUID for this result set</td></tr>
          <tr><td><code>backends_used</code></td><td>string[]</td><td>Which search backends were queried</td></tr>
        </tbody>
      </table>

      <h4>results[]</h4>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>title</code></td><td>string</td><td>Page title</td></tr>
          <tr><td><code>url</code></td><td>string</td><td>Full URL of the result</td></tr>
          <tr><td><code>domain</code></td><td>string</td><td>Hostname of the result</td></tr>
          <tr><td><code>snippet</code></td><td>string</td><td>Token-budgeted QATBE-extracted content</td></tr>
          <tr><td><code>score</code></td><td>float</td><td>HyperFusion neural ranking score (0–1)</td></tr>
          <tr><td><code>published_at</code></td><td>string?</td><td>ISO 8601 publication date if available</td></tr>
          <tr><td><code>source_type</code></td><td>string?</td><td>Content classification: <code>documentation</code>, <code>blog</code>, <code>forum</code>, <code>paper</code>, etc.</td></tr>
        </tbody>
      </table>

      <h2>Detail tiers</h2>
      <p>
        The <code>tier</code> parameter controls the QATBE (Query-Aware Token-Budgeted Extraction)
        algorithm. Higher tiers extract more content per source but consume more of your token budget:
      </p>
      <table>
        <thead><tr><th>Tier</th><th>Approx tokens</th><th>Best for</th></tr></thead>
        <tbody>
          <tr><td><code>key_facts</code></td><td>~200</td><td>Quick answers, chatbot responses, speed-critical apps</td></tr>
          <tr><td><code>summary</code></td><td>~1,000</td><td>General AI context, RAG pipelines, default choice</td></tr>
          <tr><td><code>detailed</code></td><td>~5,000</td><td>Thorough research, long-form content generation</td></tr>
          <tr><td><code>complete</code></td><td>~20,000</td><td>Full document extraction, comprehensive analysis</td></tr>
        </tbody>
      </table>

      <h2>Backend selection</h2>
      <p>
        By default, the Adaptive Backend Selector (ABS) algorithm analyzes your query and selects the
        optimal combination of backends. You can override this:
      </p>
      <CodeBlock language="json" code={`{
  "query": "tokio select macro explanation",
  "tier": "summary",
  "backends": ["stackoverflow", "github", "brave"]
}`} />

      <p>Available backends: <code>brave</code>, <code>google</code>, <code>bing</code>, <code>stackoverflow</code>, <code>github</code>, <code>reddit</code>, <code>searxng</code></p>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/research", title: "Research API", desc: "Deep multi-source research" },
          { href: "/docs/api/scrape", title: "Scrape API", desc: "URL content extraction" },
          { href: "/docs/rate-limits", title: "Rate Limits", desc: "Quotas and headers" },
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
