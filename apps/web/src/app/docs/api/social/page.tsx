import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Social API Reference" };

export default function SocialApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Social APIs</h1>
      <p>
        Fetchium provides one unified social pipeline endpoint and two platform-specific endpoints.
      </p>

      <h2>Endpoints</h2>
      <table>
        <thead><tr><th>Method</th><th>Path</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><code>POST</code></td><td><code>/v1/social/research</code></td><td>Cross-platform unified research</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/reddit</code></td><td>Reddit-only search pipeline</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/hackernews</code></td><td>Hacker News story search</td></tr>
        </tbody>
      </table>

      <h2>/v1/social/research request</h2>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Required</th></tr></thead>
        <tbody>
          <tr><td><code>query</code></td><td>string</td><td>Yes</td></tr>
          <tr><td><code>platforms</code></td><td>string[]</td><td>No</td></tr>
          <tr><td><code>max_per_platform</code></td><td>integer</td><td>No</td></tr>
          <tr><td><code>generate_ideas</code></td><td>boolean</td><td>No</td></tr>
        </tbody>
      </table>

      <CodeBlock language="bash" filename="social-research.sh" code={`curl -X POST https://api.fetchium.com/v1/social/research \\
  -H "Authorization: Bearer fetchium_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "best rust web framework",
    "platforms": ["reddit", "hackernews", "youtube"],
    "max_per_platform": 20,
    "generate_ideas": true
  }'`} />

      <h2>/v1/social/reddit request</h2>
      <CodeBlock language="json" code={`{
  "query": "rust performance",
  "subreddits": ["rust", "programming"],
  "max_posts": 25
}`} />

      <h2>/v1/social/hackernews request</h2>
      <CodeBlock language="json" code={`{
  "query": "rust async",
  "max_results": 20
}`} />

      <h2>Representative response shape</h2>
      <CodeBlock language="json" filename="social-response.json" code={`{
  "query": "best rust web framework",
  "platform_results": {
    "reddit": {
      "platform": "reddit",
      "posts": [],
      "trends": [],
      "stats": { "posts_analyzed": 20 }
    }
  },
  "cross_platform_trends": [],
  "top_posts": [],
  "content_ideas": [],
  "summary": "Analysed posts across selected platforms...",
  "duration_ms": 1432
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/youtube", title: "YouTube API", desc: "Video intelligence endpoints" },
          { href: "/docs/api/search", title: "Search API", desc: "General web search" },
          { href: "/docs/sdk/mcp", title: "MCP Protocol", desc: "Tool integration" },
          { href: "/docs/api/usage", title: "Usage API", desc: "Quota monitoring" },
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
