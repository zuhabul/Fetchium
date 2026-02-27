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
        The Research API runs a full multi-step investigation: it searches across backends, extracts
        content from top sources, synthesizes findings with citations, and returns a structured
        research report — all in a single request.
      </p>

      <div className="callout">
        The research pipeline typically takes 10–45 seconds depending on complexity.
        Results are substantially deeper than the Search API. Token consumption is higher.
      </div>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/research</span>
      </div>

      <h2>Request parameters</h2>
      <table>
        <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>query</code></td><td>string</td><td>Yes</td><td>Research question. Max 1,000 characters.</td></tr>
          <tr><td><code>max_sources</code></td><td>integer</td><td>No</td><td>Sources to investigate deeply. Range: 1–20. Default: 8.</td></tr>
          <tr><td><code>token_budget</code></td><td>integer</td><td>No</td><td>Total token budget. Range: 1,000–50,000. Default: 8,000.</td></tr>
          <tr><td><code>citation_style</code></td><td>string</td><td>No</td><td><code>apa</code>, <code>mla</code>, <code>chicago</code>, <code>inline</code>, <code>none</code>. Default: <code>inline</code>.</td></tr>
          <tr><td><code>depth</code></td><td>string</td><td>No</td><td><code>shallow</code> (fast), <code>standard</code> (default), <code>deep</code> (thorough, slower).</td></tr>
          <tr><td><code>include_evidence_graph</code></td><td>boolean</td><td>No</td><td>Include source agreement/disagreement analysis. Default: false.</td></tr>
          <tr><td><code>language</code></td><td>string</td><td>No</td><td>BCP-47 output language. Default: <code>en</code>.</td></tr>
          <tr><td><code>freshness</code></td><td>string</td><td>No</td><td>Filter sources by date: <code>week</code>, <code>month</code>, <code>year</code>.</td></tr>
        </tbody>
      </table>

      <h2>Example request</h2>

      <CodeBlock language="bash" filename="research.sh" code={`curl -X POST ***REMOVED***/v1/research \\
  -H "Authorization: Bearer fetchium_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "Compare vector databases for production use: pgvector vs Pinecone vs Weaviate vs Qdrant",
    "max_sources": 10,
    "citation_style": "inline",
    "depth": "deep",
    "freshness": "year"
  }'`} />

      <h2>Response</h2>

      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "query": "Compare vector databases for production use...",
    "depth": "deep",
    "tokens_used": 11482,
    "sources_searched": 47,
    "sources_used": 10,
    "duration_ms": 28340,
    "result_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
  },
  "report": "## Vector Database Comparison for Production\\n\\nVector databases have emerged as a critical infrastructure component for AI applications [1]. After analyzing 10 authoritative sources, here is a comprehensive comparison:\\n\\n### pgvector\\npgvector extends PostgreSQL with vector similarity search capabilities [2]...",
  "citations": [
    {
      "id": 1,
      "title": "Vector Databases: A Comprehensive Guide",
      "url": "https://www.pinecone.io/learn/vector-database/",
      "domain": "pinecone.io",
      "accessed_at": "2025-06-20T14:30:00Z"
    }
  ],
  "sources": [
    {
      "title": "Vector Databases: A Comprehensive Guide",
      "url": "https://www.pinecone.io/learn/vector-database/",
      "score": 0.923,
      "contribution": "primary"
    }
  ],
  "findings": [
    {
      "claim": "pgvector is best for teams already using PostgreSQL",
      "confidence": 0.91,
      "supporting_sources": [1, 2, 4],
      "contradicting_sources": []
    }
  ]
}`} />

      <h3>Response fields</h3>

      <h4>meta</h4>
      <table>
        <thead><tr><th>Field</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>sources_searched</code></td><td>Total URLs found across backends</td></tr>
          <tr><td><code>sources_used</code></td><td>Sources deeply extracted and included in report</td></tr>
          <tr><td><code>duration_ms</code></td><td>End-to-end processing time (can be 10–45s for deep)</td></tr>
        </tbody>
      </table>

      <h4>report</h4>
      <p>Markdown-formatted research synthesis with inline citations.</p>

      <h4>findings[]</h4>
      <table>
        <thead><tr><th>Field</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>claim</code></td><td>Synthesized finding or conclusion</td></tr>
          <tr><td><code>confidence</code></td><td>0–1 confidence based on source agreement</td></tr>
          <tr><td><code>supporting_sources</code></td><td>Citation IDs that support this claim</td></tr>
          <tr><td><code>contradicting_sources</code></td><td>Citation IDs that contradict this claim</td></tr>
        </tbody>
      </table>

      <h2>Evidence graph</h2>
      <p>
        Enable <code>include_evidence_graph: true</code> to get a detailed analysis of source
        agreement and contradictions:
      </p>
      <CodeBlock language="json" code={`{
  "query": "Is Rust faster than Go for web servers?",
  "include_evidence_graph": true,
  "depth": "deep"
}`} />

      <h2>Citation styles</h2>
      <table>
        <thead><tr><th>Style</th><th>Example</th></tr></thead>
        <tbody>
          <tr><td><code>inline</code></td><td>...is widely used [1]...</td></tr>
          <tr><td><code>apa</code></td><td>Smith, J. (2024). Title. Journal, 1(1).</td></tr>
          <tr><td><code>mla</code></td><td>Smith, John. "Title." Journal 1.1 (2024).</td></tr>
          <tr><td><code>chicago</code></td><td>Smith, John. "Title." Journal 1, no. 1 (2024).</td></tr>
          <tr><td><code>none</code></td><td>No citations appended</td></tr>
        </tbody>
      </table>

      <h2>TypeScript example with streaming</h2>
      <CodeBlock language="typescript" filename="research.ts" code={`async function research(query: string) {
  const res = await fetch("***REMOVED***/v1/research", {
    method: "POST",
    headers: {
      "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      query,
      max_sources: 8,
      citation_style: "inline",
      depth: "standard",
    }),
  });

  if (!res.ok) {
    const err = await res.json();
    throw new Error(\`Research API error: \${err.error.message}\`);
  }

  const data = await res.json();
  console.log(data.report);         // Markdown report
  console.log(data.citations);      // Cited sources
  console.log(data.findings);       // Synthesized claims

  return data;
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/search", title: "Search API", desc: "Faster, lighter search" },
          { href: "/docs/api/scrape", title: "Scrape API", desc: "Single-URL extraction" },
          { href: "/docs/api/youtube", title: "YouTube API", desc: "Video search and analysis" },
          { href: "/docs/algorithms/hyperfusion", title: "HyperFusion", desc: "Neural ranking algorithm" },
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
