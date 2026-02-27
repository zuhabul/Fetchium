import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Social Research API Reference" };

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

export default function SocialApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Social Research API</h1>
      <p>
        The Social Research API searches community platforms — Reddit, HackerNews, and Twitter/X —
        to surface real-world opinions, discussions, and reactions. Powered by the Social Search
        backend with HyperFusion ranking optimised for community signal quality.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <BADGE method="POST" />
        <span className="text-slate-300">/v1/social</span>
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
            <td><code>platforms</code></td><td>string[]</td><td>No</td>
            <td>
              Filter to specific platforms: <code>reddit</code>, <code>hackernews</code>, <code>twitter</code>.
              Default: all platforms.
            </td>
          </tr>
          <tr>
            <td><code>sort</code></td><td>string</td><td>No</td>
            <td>
              Sort order: <code>relevance</code> (default), <code>recent</code>, <code>top</code>.
            </td>
          </tr>
          <tr>
            <td><code>time_range</code></td><td>string</td><td>No</td>
            <td>
              Filter by date: <code>day</code>, <code>week</code>, <code>month</code>, <code>year</code>, <code>all</code> (default).
            </td>
          </tr>
          <tr>
            <td><code>max_results</code></td><td>integer</td><td>No</td>
            <td>Maximum results to return. Range: 1–50. Default: 20.</td>
          </tr>
          <tr>
            <td><code>include_comments</code></td><td>boolean</td><td>No</td>
            <td>Include top comments for each post. Default: <code>false</code>.</td>
          </tr>
          <tr>
            <td><code>min_score</code></td><td>integer</td><td>No</td>
            <td>Minimum upvote/point score to include. Default: 0.</td>
          </tr>
          <tr>
            <td><code>subreddits</code></td><td>string[]</td><td>No</td>
            <td>Restrict Reddit results to specific subreddits.</td>
          </tr>
        </tbody>
      </table>

      <h2>Example request</h2>

      <CodeBlock language="bash" filename="social.sh" code={`curl -X POST ***REMOVED***/v1/social \\
  -H "Authorization: Bearer fetchium_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust vs go performance 2025",
    "platforms": ["reddit", "hackernews"],
    "sort": "top",
    "time_range": "year",
    "max_results": 15,
    "include_comments": true
  }'`} />

      <CodeBlock language="typescript" filename="social.ts" code={`const res = await fetch("***REMOVED***/v1/social", {
  method: "POST",
  headers: {
    "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    query: "rust vs go performance 2025",
    platforms: ["reddit", "hackernews"],
    sort: "top",
    time_range: "year",
    max_results: 15,
    include_comments: true,
  }),
});
const data = await res.json();`} />

      <h2>Response</h2>

      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "query": "rust vs go performance 2025",
    "platforms_queried": ["reddit", "hackernews"],
    "total_results": 15,
    "duration_ms": 842
  },
  "results": [
    {
      "id": "reddit_abc123",
      "platform": "reddit",
      "title": "Rust vs Go in 2025 — My experience rewriting a service",
      "url": "https://reddit.com/r/rust/comments/abc123/",
      "author": "u/rustacean_dev",
      "subreddit": "rust",
      "score": 1847,
      "num_comments": 234,
      "created_at": "2025-03-15T14:22:00Z",
      "body": "After rewriting our API from Go to Rust, here's what we found...",
      "top_comments": [
        {
          "author": "u/go_gopher",
          "body": "Interesting. In my experience Go wins on developer velocity...",
          "score": 412
        }
      ],
      "relevance_score": 0.94
    },
    {
      "id": "hn_456789",
      "platform": "hackernews",
      "title": "Ask HN: Rust or Go for systems programming in 2025?",
      "url": "https://news.ycombinator.com/item?id=456789",
      "author": "thrower42",
      "score": 312,
      "num_comments": 187,
      "created_at": "2025-06-01T09:15:00Z",
      "relevance_score": 0.89
    }
  ]
}`} />

      <h3>Response fields</h3>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>platform</code></td><td>string</td><td><code>reddit</code>, <code>hackernews</code>, or <code>twitter</code></td></tr>
          <tr><td><code>score</code></td><td>integer</td><td>Platform-native upvotes/points</td></tr>
          <tr><td><code>num_comments</code></td><td>integer</td><td>Number of comments/replies</td></tr>
          <tr><td><code>subreddit</code></td><td>string?</td><td>Reddit-only: subreddit name</td></tr>
          <tr><td><code>top_comments</code></td><td>object[]?</td><td>Present when <code>include_comments: true</code></td></tr>
          <tr><td><code>relevance_score</code></td><td>float</td><td>HyperFusion-ranked relevance (0–1)</td></tr>
        </tbody>
      </table>

      <h2>CLI equivalent</h2>
      <CodeBlock language="bash" code={`# Search social platforms via CLI
fetchium social reddit "rust performance"
fetchium social hackernews "programming languages 2025"

# With options
fetchium social reddit "rust" --sort top --time week --min-score 100`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/search", title: "Search API", desc: "General web search" },
          { href: "/docs/api/research", title: "Research API", desc: "Deep multi-source research" },
          { href: "/docs/api/youtube", title: "YouTube API", desc: "Video search and analysis" },
          { href: "/docs/rate-limits", title: "Rate Limits", desc: "Quotas and headers" },
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
