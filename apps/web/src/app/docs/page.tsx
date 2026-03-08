import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Introduction" };

const QUICKLINKS = [
  { title: "Quick Start", desc: "Make your first API call in under 2 minutes.", href: "https://docs.fetchium.com/quickstart", icon: "⚡" },
  { title: "Authentication", desc: "Bearer tokens, admin secrets, key rotation.", href: "https://docs.fetchium.com/authentication", icon: "🔑" },
  { title: "Search API", desc: "Federated search with ranking.", href: "https://docs.fetchium.com/api/search", icon: "🔍" },
  { title: "Research API", desc: "Deep multi-source synthesis.", href: "https://docs.fetchium.com/api/research", icon: "🔬" },
  { title: "MCP Protocol", desc: "12 MCP tools across core, YouTube, and social workflows.", href: "https://docs.fetchium.com/sdk/mcp", icon: "🧩" },
];

export default function DocsIndex() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-indigo-500/25 bg-indigo-500/8 text-indigo-300 text-xs mb-6">
        <span className="w-1.5 h-1.5 rounded-full bg-indigo-400 animate-pulse" />
        API v1
      </div>

      <h1>Fetchium Documentation</h1>

      <p>
        Fetchium provides search, extraction, research, YouTube intelligence, social intelligence,
        MCP tools, and API key management via a single Rust backend.
      </p>

      <div className="callout">
        <strong>Base URL:</strong> <code>***REMOVED***</code>
      </div>

      <h2>Quick links</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 mb-8 not-prose">
        {QUICKLINKS.map((q) => (
          <Link key={q.href} href={q.href} className="glass-card rounded-xl p-4 flex flex-col gap-2 no-underline group">
            <div className="text-2xl">{q.icon}</div>
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{q.title}</div>
            <div className="text-xs text-slate-500 leading-relaxed">{q.desc}</div>
          </Link>
        ))}
      </div>

      <h2>Your first request</h2>
      <CodeBlock
        language="bash"
        filename="search.sh"
        code={`curl -X POST ***REMOVED***/v1/search \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust async programming best practices",
    "tier": "detailed",
    "max_sources": 10
  }'`}
      />

      <h2>Endpoints overview</h2>
      <table>
        <thead><tr><th>Method</th><th>Endpoint</th><th>Auth</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>GET</code></td><td><code>/health</code>, <code>/v1/health</code></td><td>None</td><td>Health and dependency checks</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/search</code></td><td>Bearer</td><td>Federated web search</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/scrape</code>, <code>/v1/fetch</code></td><td>Bearer</td><td>Single URL extraction</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/research</code></td><td>Bearer</td><td>Deep research synthesis</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/research/jobs</code></td><td>Bearer</td><td>Async research submission</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/estimate</code></td><td>Bearer</td><td>Token estimate for a URL</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/search</code></td><td>Bearer</td><td>YouTube search pipeline</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/analyze</code></td><td>Bearer</td><td>Single video analysis</td></tr>
          <tr><td><code>GET</code></td><td><code>/v1/jobs/:id</code></td><td>Bearer</td><td>Poll async job status</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/research</code></td><td>Bearer</td><td>Unified social research</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/reddit</code></td><td>Bearer</td><td>Reddit-specific search</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/hackernews</code></td><td>Bearer</td><td>Hacker News-specific search</td></tr>
          <tr><td><code>GET</code></td><td><code>/v1/usage</code></td><td>Bearer</td><td>Per-key usage stats</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/keys</code></td><td><code>X-Admin-Secret</code></td><td>Create API key</td></tr>
          <tr><td><code>GET</code></td><td><code>/v1/keys</code></td><td><code>X-Admin-Secret</code></td><td>List API keys</td></tr>
          <tr><td><code>DELETE</code></td><td><code>/v1/keys/:id</code></td><td><code>X-Admin-Secret</code></td><td>Revoke API key</td></tr>
          <tr><td><code>GET/POST</code></td><td><code>/v1/proxy/*</code></td><td><code>X-Admin-Secret</code></td><td>Proxy pool administration</td></tr>
        </tbody>
      </table>

      <h2>TypeScript example</h2>
      <CodeBlock
        language="typescript"
        filename="fetchium.ts"
        code={`const FETCHIUM_BASE = "***REMOVED***";

async function search(query: string, tier = "summary") {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/search\`, {
    method: "POST",
    headers: {
      Authorization: \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query, tier, max_sources: 10 }),
  });
  if (!res.ok) throw new Error(\`Fetchium API error: \${res.status}\`);
  return res.json();
}`} />
    </article>
  );
}
