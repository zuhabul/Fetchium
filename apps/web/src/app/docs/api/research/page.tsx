import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Research API Reference" };

export default function ResearchApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Research API</h1>
      <p>
        Executes a full multi-source research pipeline and returns a synthesized report with
        references and confidence.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/research</span>
      </div>

      <h2>Request body</h2>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Required</th><th>Notes</th></tr></thead>
        <tbody>
          <tr><td><code>query</code></td><td>string</td><td>Yes</td><td>1-500 chars</td></tr>
          <tr><td><code>token_budget</code></td><td>integer</td><td>No</td><td>1,000-50,000</td></tr>
          <tr><td><code>max_sources</code></td><td>integer</td><td>No</td><td>1-20</td></tr>
          <tr><td><code>depth</code></td><td>string</td><td>No</td><td><code>quick</code> | <code>standard</code> | <code>deep</code></td></tr>
          <tr><td><code>strict_evidence</code></td><td>boolean</td><td>No</td><td>Require stronger evidence checks</td></tr>
          <tr><td><code>citation_style</code></td><td>string</td><td>No</td><td><code>inline</code> | <code>apa</code> | <code>ieee</code> | <code>mla</code> | <code>chicago</code> | <code>bibtex</code></td></tr>
        </tbody>
      </table>

      <h2>Example request</h2>
      <CodeBlock language="bash" filename="research.sh" code={`curl -X POST ***REMOVED***/v1/research \\
  -H "Authorization: Bearer fetchium_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "Compare vector databases for production use",
    "max_sources": 10,
    "depth": "deep",
    "citation_style": "inline",
    "strict_evidence": true
  }'`} />

      <h2>Response</h2>
      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "query": "Compare vector databases for production use",
    "tier": "detailed",
    "tokens_used": 11482,
    "sources_count": 10,
    "duration_ms": 28340,
    "result_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
  },
  "report": "## Comparison\\n...",
  "reference_section": "[1] ...",
  "sources": [
    {
      "index": 1,
      "title": "Vector Databases: A Comprehensive Guide",
      "url": "https://www.pinecone.io/learn/vector-database/"
    }
  ],
  "confidence": 0.87
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/search", title: "Search API", desc: "Fast retrieval" },
          { href: "/docs/api/scrape", title: "Scrape API", desc: "Single URL extraction" },
          { href: "/docs/api/youtube", title: "YouTube API", desc: "Video research workflows" },
          { href: "/docs/errors", title: "Error Reference", desc: "Error payloads" },
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
