import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "TypeScript / Node.js SDK" };

export default function TypeScriptSDK() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">SDKs & Integrations</div>
      <h1>TypeScript / Node.js</h1>
      <p>
        Call the Fetchium REST API directly with <code>fetch</code>.
      </p>

      <h2>Client helper</h2>
      <CodeBlock language="typescript" filename="fetchium-client.ts" code={`const FETCHIUM_BASE = "***REMOVED***";
const FETCHIUM_KEY = process.env.FETCHIUM_API_KEY!;

const headers = {
  Authorization: \`Bearer \${FETCHIUM_KEY}\`,
  "Content-Type": "application/json",
};

export async function search(query: string) {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/search\`, {
    method: "POST",
    headers,
    body: JSON.stringify({ query, tier: "summary", max_sources: 8 }),
  });
  if (!res.ok) throw new Error(\`Fetchium error: \${res.status}\`);
  return res.json();
}

export async function scrape(url: string) {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/scrape\`, {
    method: "POST",
    headers,
    body: JSON.stringify({ url, format: "markdown", token_budget: 3000 }),
  });
  if (!res.ok) throw new Error(\`Fetchium error: \${res.status}\`);
  return res.json();
}

export async function research(query: string) {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/research\`, {
    method: "POST",
    headers,
    body: JSON.stringify({ query, depth: "standard", citation_style: "inline" }),
  });
  if (!res.ok) throw new Error(\`Fetchium error: \${res.status}\`);
  return res.json();
}`} />

      <h2>Type definitions</h2>
      <CodeBlock language="typescript" filename="fetchium-types.ts" code={`export interface SearchRequest {
  query: string;
  token_budget?: number;
  tier?: "key_facts" | "summary" | "detailed" | "complete";
  max_sources?: number;
  validate?: boolean;
}

export interface SearchResultItem {
  title: string;
  url: string;
  snippet?: string;
  score?: number;
}

export interface SearchResponse {
  meta: {
    query: string;
    tier: string;
    tokens_used: number;
    sources_count: number;
    duration_ms: number;
    result_id: string;
  };
  results: SearchResultItem[];
}`} />

      <h2>YouTube + social examples</h2>
      <CodeBlock language="typescript" filename="extra-endpoints.ts" code={`// YouTube search
await fetch(\`\${FETCHIUM_BASE}/v1/youtube/search\`, {
  method: "POST",
  headers,
  body: JSON.stringify({ query: "Java learning", max_results: 10 }),
});

// Unified social research
await fetch(\`\${FETCHIUM_BASE}/v1/social/research\`, {
  method: "POST",
  headers,
  body: JSON.stringify({
    query: "rust vs go",
    platforms: ["reddit", "hackernews"],
    max_per_platform: 20,
  }),
});`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/sdk/python", title: "Python SDK", desc: "Python examples" },
          { href: "/docs/sdk/curl", title: "curl Examples", desc: "CLI usage" },
          { href: "/docs/api/search", title: "Search API", desc: "Full reference" },
          { href: "/docs/api/youtube", title: "YouTube API", desc: "Video endpoints" },
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
