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
        All Fetchium API endpoints can be called from the command line with <code>curl</code>.
        Set your API key once and copy-paste any example.
      </p>

      <h2>Setup</h2>
      <CodeBlock language="bash" code={`# Set your API key (add to ~/.bashrc or ~/.zshrc)
export FETCHIUM_API_KEY="fetchium_your_key_here"

# Optional: set base URL
export FETCHIUM_BASE="https://api.fetchium.com"`} />

      <h2>Search</h2>
      <CodeBlock language="bash" code={`# Basic search
curl -sX POST "$FETCHIUM_BASE/v1/search" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query": "rust tokio best practices"}' | jq .

# With options
curl -sX POST "$FETCHIUM_BASE/v1/search" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust tokio best practices",
    "tier": "detailed",
    "max_sources": 8,
    "freshness": "year",
    "backends": ["stackoverflow", "github", "brave"]
  }' | jq '.results[] | {title, url, score}'`} />

      <h2>Scrape a URL</h2>
      <CodeBlock language="bash" code={`# Extract content from any URL
curl -sX POST "$FETCHIUM_BASE/v1/scrape" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"url": "https://tokio.rs/tokio/tutorial", "tier": "detailed"}' \\
  | jq '.content'

# Extract just the text
curl -sX POST "$FETCHIUM_BASE/v1/scrape" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"url": "https://example.com/article", "tier": "summary"}' \\
  | jq -r '.content.text'`} />

      <h2>Deep research</h2>
      <CodeBlock language="bash" code={`# Run a deep research query (may take 30–60s)
curl -sX POST "$FETCHIUM_BASE/v1/research" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "compare Rust async runtimes for production use",
    "depth": "thorough"
  }' \\
  --max-time 120 | jq .`} />

      <h2>Social research</h2>
      <CodeBlock language="bash" code={`# Search Reddit and HackerNews
curl -sX POST "$FETCHIUM_BASE/v1/social" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust vs go 2025",
    "platforms": ["reddit", "hackernews"],
    "sort": "top",
    "time_range": "year"
  }' | jq '.results[] | {platform, title, score}'`} />

      <h2>YouTube search</h2>
      <CodeBlock language="bash" code={`# Search YouTube videos
curl -sX POST "$FETCHIUM_BASE/v1/youtube" \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query": "rust programming tutorial 2025", "max_results": 5}' \\
  | jq '.results[] | {title, channel, views, url}'`} />

      <h2>Usage stats</h2>
      <CodeBlock language="bash" code={`# Check your current usage
curl -sH "Authorization: Bearer $FETCHIUM_API_KEY" \\
  "$FETCHIUM_BASE/v1/usage" | jq .

# Health check
curl -s "$FETCHIUM_BASE/health" | jq .`} />

      <h2>Shell function wrappers</h2>
      <CodeBlock language="bash" filename="~/.bashrc" code={`# Quick search function
fetchium-search() {
  curl -sX POST "https://api.fetchium.com/v1/search" \\
    -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
    -H "Content-Type: application/json" \\
    -d "{\\"query\\": \\"$*\\", \\"tier\\": \\"summary\\"}" \\
    | jq -r '.results[] | "\\(.score | . * 100 | round / 100)  \\(.title)\\n   \\(.url)\\n"'
}

# Quick URL scraper
fetchium-scrape() {
  curl -sX POST "https://api.fetchium.com/v1/scrape" \\
    -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
    -H "Content-Type: application/json" \\
    -d "{\\"url\\": \\"$1\\", \\"tier\\": \\"summary\\"}" \\
    | jq -r '.content.text'
}

# Usage: fetchium-search "rust async runtime"
# Usage: fetchium-scrape https://tokio.rs/tokio/tutorial`} />

      <h2>Or use the CLI directly</h2>
      <CodeBlock language="bash" code={`# The fetchium CLI is even easier than curl
fetchium search "rust tokio best practices"
fetchium fetch https://tokio.rs/tokio/tutorial
fetchium agent-search "compare Rust async runtimes"
fetchium social reddit "rust performance"`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/sdk/typescript", title: "TypeScript SDK", desc: "JavaScript/TS examples" },
          { href: "/docs/sdk/python", title: "Python SDK", desc: "Python examples" },
          { href: "/docs/api/search", title: "Search API", desc: "Full parameter reference" },
          { href: "/docs/quickstart", title: "Quick Start", desc: "Get your API key" },
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
