import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "curl Examples" };

export default function CurlSDK() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">SDKs & Integrations</div>
      <h1>curl Examples</h1>
      <p>
        Use these copy-paste examples with your Fetchium API key.
      </p>

      <h2>Setup</h2>
      <CodeBlock language="bash" code={`export FETCHIUM_API_KEY="fetchium_..."
export FETCHIUM_BASE="https://api.fetchium.com"`} />

      <h2>Search</h2>
      <CodeBlock language="bash" code={`curl -sX POST "$FETCHIUM_BASE/v1/search" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust tokio best practices",
    "tier": "detailed",
    "max_sources": 8
  }' | jq .`} />

      <h2>Scrape</h2>
      <CodeBlock language="bash" code={`curl -sX POST "$FETCHIUM_BASE/v1/scrape" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://tokio.rs/tokio/tutorial",
    "format": "markdown",
    "token_budget": 3000
  }' | jq .`} />

      <h2>Research</h2>
      <CodeBlock language="bash" code={`curl -sX POST "$FETCHIUM_BASE/v1/research" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "compare Rust async runtimes for production",
    "depth": "deep",
    "citation_style": "inline"
  }' \\
  --max-time 120 | jq .`} />

      <h2>YouTube</h2>
      <CodeBlock language="bash" code={`curl -sX POST "$FETCHIUM_BASE/v1/youtube/search" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query": "Java learning", "max_results": 10}' | jq .

curl -sX POST "$FETCHIUM_BASE/v1/youtube/analyze" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"url":"https://www.youtube.com/watch?v=dQw4w9WgXcQ","transcript":true,"comments":true}' | jq .`} />

      <h2>Social</h2>
      <CodeBlock language="bash" code={`curl -sX POST "$FETCHIUM_BASE/v1/social/research" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query":"rust vs go","platforms":["reddit","hackernews"],"max_per_platform":20}' | jq .

curl -sX POST "$FETCHIUM_BASE/v1/social/reddit" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query":"rust async","max_posts":25}' | jq .

curl -sX POST "$FETCHIUM_BASE/v1/social/hackernews" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query":"rust async","max_results":20}' | jq .`} />

      <h2>Estimate, usage, health</h2>
      <CodeBlock language="bash" code={`curl -sX POST "$FETCHIUM_BASE/v1/estimate" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"url":"https://tokio.rs/tokio/tutorial"}' | jq .

curl -sH "Authorization: Bearer $FETCHIUM_API_KEY" \\
  "$FETCHIUM_BASE/v1/usage" | jq .

curl -s "$FETCHIUM_BASE/health" | jq .`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/sdk/typescript", title: "TypeScript", desc: "TS/Node integration" },
          { href: "https://docs.fetchium.com/sdk/python", title: "Python", desc: "Python integration" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Reference docs" },
          { href: "https://docs.fetchium.com/sdk/mcp", title: "MCP Protocol", desc: "Model Context Protocol" },
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
