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
        Use Fetchium from any TypeScript or JavaScript project with the official
        REST API. No SDK package required — the API is simple enough to call directly
        with <code>fetch</code>.
      </p>

      <h2>Installation</h2>
      <p>No package to install. Just use the native <code>fetch</code> API.</p>

      <h2>Quickstart</h2>

      <CodeBlock language="typescript" filename="fetchium-client.ts" code={`const FETCHIUM_BASE = "https://api.fetchium.com";
const FETCHIUM_KEY = process.env.FETCHIUM_API_KEY!;

// ── Search ────────────────────────────────────────────────────────
export async function search(query: string, tier = "summary") {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/search\`, {
    method: "POST",
    headers: {
      "Authorization": \`Bearer \${FETCHIUM_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query, tier }),
  });
  if (!res.ok) throw new Error(\`Fetchium error: \${res.status} \${await res.text()}\`);
  return res.json();
}

// ── Scrape a URL ──────────────────────────────────────────────────
export async function scrape(url: string, tier = "summary") {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/scrape\`, {
    method: "POST",
    headers: {
      "Authorization": \`Bearer \${FETCHIUM_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ url, tier }),
  });
  if (!res.ok) throw new Error(\`Fetchium error: \${res.status} \${await res.text()}\`);
  return res.json();
}

// ── Deep research ─────────────────────────────────────────────────
export async function research(query: string) {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/research\`, {
    method: "POST",
    headers: {
      "Authorization": \`Bearer \${FETCHIUM_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query, depth: "thorough" }),
  });
  if (!res.ok) throw new Error(\`Fetchium error: \${res.status} \${await res.text()}\`);
  return res.json();
}`} />

      <h2>Next.js / App Router example</h2>

      <CodeBlock language="typescript" filename="app/api/search/route.ts" code={`import { NextResponse } from "next/server";

export async function POST(req: Request) {
  const { query } = await req.json();

  const res = await fetch("https://api.fetchium.com/v1/search", {
    method: "POST",
    headers: {
      "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      query,
      tier: "summary",
      max_sources: 5,
    }),
    // Cache for 5 minutes
    next: { revalidate: 300 },
  });

  const data = await res.json();
  return NextResponse.json(data);
}`} />

      <h2>Vercel AI SDK integration</h2>

      <CodeBlock language="typescript" filename="ai-search-tool.ts" code={`import { tool } from "ai";
import { z } from "zod";

export const searchTool = tool({
  description: "Search the web for current information on any topic",
  parameters: z.object({
    query: z.string().describe("The search query"),
    tier: z.enum(["key_facts", "summary", "detailed"]).optional(),
  }),
  execute: async ({ query, tier = "summary" }) => {
    const res = await fetch("https://api.fetchium.com/v1/search", {
      method: "POST",
      headers: {
        "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ query, tier }),
    });
    const data = await res.json();
    return data.results
      .map((r: any) => \`[\${r.title}](\${r.url}): \${r.snippet}\`)
      .join("\\n\\n");
  },
});`} />

      <h2>LangChain integration</h2>

      <CodeBlock language="typescript" filename="langchain-tool.ts" code={`import { DynamicStructuredTool } from "@langchain/core/tools";
import { z } from "zod";

export const hsxSearchTool = new DynamicStructuredTool({
  name: "web_search",
  description: "Search the web for up-to-date information",
  schema: z.object({
    query: z.string(),
    tier: z.enum(["key_facts", "summary", "detailed"]).optional(),
  }),
  func: async ({ query, tier = "summary" }) => {
    const res = await fetch("https://api.fetchium.com/v1/search", {
      method: "POST",
      headers: {
        "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ query, tier }),
    });
    const { results } = await res.json();
    return results.map((r: any) => r.snippet).join("\\n\\n");
  },
});`} />

      <h2>TypeScript types</h2>

      <CodeBlock language="typescript" filename="fetchium-types.ts" code={`export interface SearchRequest {
  query: string;
  tier?: "key_facts" | "summary" | "detailed" | "complete";
  token_budget?: number;
  max_sources?: number;
  backends?: string[];
  language?: string;
  freshness?: "day" | "week" | "month" | "year";
  include_domains?: string[];
  exclude_domains?: string[];
}

export interface SearchResult {
  title: string;
  url: string;
  domain: string;
  snippet: string;
  score: number;
  published_at?: string;
  source_type?: string;
}

export interface SearchResponse {
  meta: {
    query: string;
    tier: string;
    tokens_used: number;
    sources_count: number;
    duration_ms: number;
    result_id: string;
    backends_used: string[];
  };
  results: SearchResult[];
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/sdk/python", title: "Python SDK", desc: "Python examples" },
          { href: "/docs/sdk/curl", title: "curl Examples", desc: "CLI / shell usage" },
          { href: "/docs/api/search", title: "Search API", desc: "Full parameter reference" },
          { href: "/docs/authentication", title: "Authentication", desc: "API key setup" },
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
