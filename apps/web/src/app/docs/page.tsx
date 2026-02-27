import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Introduction" };

const QUICKLINKS = [
  { title: "Quick Start", desc: "Make your first API call in under 2 minutes.", href: "/docs/quickstart", icon: "⚡" },
  { title: "Authentication", desc: "Bearer tokens, admin secrets, key rotation.", href: "/docs/authentication", icon: "🔑" },
  { title: "Search API", desc: "Multi-backend federated search with neural ranking.", href: "/docs/api/search", icon: "🔍" },
  { title: "Research API", desc: "Deep multi-source research with citations.", href: "/docs/api/research", icon: "🔬" },
  { title: "Rate Limits", desc: "Plan quotas, per-minute limits, headers.", href: "/docs/rate-limits", icon: "⏱" },
  { title: "Self-Hosting", desc: "Run on your own infrastructure for free.", href: "/docs/self-hosting/docker", icon: "🐳" },
];

export default function DocsIndex() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-indigo-500/25 bg-indigo-500/8 text-indigo-300 text-xs mb-6">
        <span className="w-1.5 h-1.5 rounded-full bg-indigo-400 animate-pulse" />
        API v1 · Open Beta
      </div>

      <h1>Fetchium Documentation</h1>

      <p>
        Fetchium is an intelligent search API that aggregates multiple search backends, extracts
        structured content from web pages, runs multi-step research pipelines, and delivers
        token-budgeted, AI-ready context through a clean REST API.
      </p>

      <p>
        Built in Rust for maximum performance, it implements 17 novel algorithms including
        HyperFusion (8-signal neural ranking), CEP (5-layer content extraction), and QATBE
        (query-aware token budget extraction).
      </p>

      <div className="callout">
        <strong>Open Beta:</strong> Fetchium is currently in open beta. The Free tier provides
        1,000 requests/month with no credit card required.{" "}
        <Link href="https://app.fetchium.com">Get your API key →</Link>
      </div>

      <h2>Quick links</h2>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 mb-8 not-prose">
        {QUICKLINKS.map(q => (
          <Link key={q.href} href={q.href}
            className="glass-card rounded-xl p-4 flex flex-col gap-2 no-underline group">
            <div className="text-2xl">{q.icon}</div>
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{q.title}</div>
            <div className="text-xs text-slate-500 leading-relaxed">{q.desc}</div>
          </Link>
        ))}
      </div>

      <h2>Base URL</h2>
      <p>All API requests are made to the following base URL:</p>
      <CodeBlock code="https://api.fetchium.com" language="text" />

      <h2>Your first request</h2>
      <p>
        Once you have an API key from the{" "}
        <Link href="https://app.fetchium.com">dashboard</Link>, you can make your first
        search request:
      </p>

      <CodeBlock
        language="bash"
        filename="search.sh"
        code={`curl -X POST https://api.fetchium.com/v1/search \\
  -H "Authorization: Bearer fetchium_your_api_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust async programming best practices",
    "tier": "detailed",
    "max_sources": 10
  }'`}
      />

      <CodeBlock
        language="json"
        filename="response.json"
        code={`{
  "meta": {
    "query": "rust async programming best practices",
    "tier": "detailed",
    "tokens_used": 2847,
    "sources_count": 10,
    "duration_ms": 1243,
    "result_id": "09742460-58db-44fd-9036-502ed26565e2"
  },
  "results": [
    {
      "title": "Async in Rust: A comprehensive guide",
      "url": "https://tokio.rs/tokio/tutorial",
      "snippet": "Tokio is an asynchronous runtime for the Rust programming...",
      "score": 0.923
    }
  ]
}`}
      />

      <h2>Key concepts</h2>

      <h3>Detail tiers</h3>
      <p>
        Every search and research request accepts a <code>tier</code> parameter that controls how much
        content is extracted and how many tokens are returned:
      </p>
      <table>
        <thead><tr><th>Tier</th><th>Tokens (approx)</th><th>Use case</th></tr></thead>
        <tbody>
          <tr><td><code>key_facts</code></td><td>~200</td><td>Quick answers, chatbot responses</td></tr>
          <tr><td><code>summary</code></td><td>~1,000</td><td>General AI context, RAG pipelines</td></tr>
          <tr><td><code>detailed</code></td><td>~5,000</td><td>Thorough analysis, long-form content</td></tr>
          <tr><td><code>complete</code></td><td>~20,000</td><td>Full document extraction</td></tr>
        </tbody>
      </table>

      <h3>Token budget</h3>
      <p>
        You can override the tier with an explicit <code>token_budget</code> (100–10,000 for search,
        1,000–50,000 for research). The API uses the QATBE algorithm to maximize relevant content
        within your budget.
      </p>

      <h3>Authentication</h3>
      <p>
        All API endpoints (except <code>/health</code>) require a Bearer token in the{" "}
        <code>Authorization</code> header. API keys are prefixed with <code>fetchium_</code> and contain
        64 hex characters (256-bit entropy).
      </p>

      <h2>Endpoints overview</h2>
      <table>
        <thead><tr><th>Method</th><th>Endpoint</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>POST</code></td><td><code>/v1/search</code></td><td>Multi-backend web search with neural ranking</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/scrape</code></td><td>URL content extraction with CEP pipeline</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/research</code></td><td>Full multi-source research with citations</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/search</code></td><td>YouTube video search &amp; analysis</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/research</code></td><td>Cross-platform social media research</td></tr>
          <tr><td><code>GET</code></td><td><code>/v1/usage</code></td><td>Your API usage statistics &amp; quota</td></tr>
          <tr><td><code>GET</code></td><td><code>/health</code></td><td>Service health &amp; dependency status</td></tr>
        </tbody>
      </table>

      <h2>Rate limits &amp; quotas</h2>
      <table>
        <thead><tr><th>Plan</th><th>Requests / month</th><th>Requests / minute</th><th>Price</th></tr></thead>
        <tbody>
          <tr><td>Free</td><td>1,000</td><td>60</td><td>$0</td></tr>
          <tr><td>Starter</td><td>10,000</td><td>200</td><td>$19 / mo</td></tr>
          <tr><td>Pro</td><td>100,000</td><td>500</td><td>$79 / mo</td></tr>
          <tr><td>Enterprise</td><td>Unlimited</td><td>2,000</td><td>Custom</td></tr>
        </tbody>
      </table>
      <p>
        When rate-limited, the API returns <code>429 Too Many Requests</code> with a{" "}
        <code>Retry-After: 60</code> header. See{" "}
        <Link href="/docs/rate-limits">Rate Limits</Link> for full details.
      </p>

      <h2>SDKs</h2>
      <p>Official SDKs are coming soon. In the meantime, use the REST API directly or with any HTTP client.</p>

      <CodeBlock
        language="typescript"
        filename="fetchium.ts"
        code={`// TypeScript — plain fetch
const FETCHIUM_BASE = "https://api.fetchium.com";

async function search(query: string, tier = "summary") {
  const res = await fetch(\`\${FETCHIUM_BASE}/v1/search\`, {
    method: "POST",
    headers: {
      "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query, tier, max_sources: 10 }),
  });
  if (!res.ok) throw new Error(\`HSX API error: \${res.status}\`);
  return res.json();
}

const result = await search("best vector databases 2025", "detailed");
console.log(result.results[0].title);`}
      />

      <CodeBlock
        language="python"
        filename="fetchium.py"
        code={`# Python — requests
import os, requests

FETCHIUM_BASE = "https://api.fetchium.com"
HEADERS = {"Authorization": f"Bearer {os.environ['FETCHIUM_API_KEY']}"}

def search(query: str, tier: str = "summary") -> dict:
    r = requests.post(f"{FETCHIUM_BASE}/v1/search",
        headers=HEADERS,
        json={"query": query, "tier": tier, "max_sources": 10})
    r.raise_for_status()
    return r.json()

result = search("best vector databases 2025", "detailed")
print(result["results"][0]["title"])`}
      />

      <div className="mt-10 p-5 rounded-2xl border border-white/[0.07] bg-gradient-to-br from-indigo-500/5 to-violet-500/5 not-prose">
        <div className="flex items-start gap-4">
          <div className="text-3xl">🚀</div>
          <div>
            <div className="font-semibold text-slate-200 mb-1">Ready to start building?</div>
            <p className="text-slate-400 text-sm mb-3">
              Get your free API key and make your first request in under 2 minutes.
            </p>
            <div className="flex gap-3">
              <Link href="https://app.fetchium.com"
                className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium transition-colors no-underline">
                Get API Key →
              </Link>
              <Link href="/docs/quickstart"
                className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-white/5 hover:bg-white/8 text-slate-300 text-sm font-medium transition-colors no-underline border border-white/10">
                Quick Start Guide
              </Link>
            </div>
          </div>
        </div>
      </div>
    </article>
  );
}
