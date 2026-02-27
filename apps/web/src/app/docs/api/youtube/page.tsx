import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "YouTube API Reference" };

export default function YoutubeApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>YouTube API</h1>
      <p>
        Search YouTube videos, extract transcripts, and analyze video content — all without a YouTube
        API key. The YouTube endpoint uses the same HyperFusion ranking and QATBE token-budgeted
        extraction as the main search API.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/youtube/search</span>
      </div>

      <h2>Request parameters</h2>
      <table>
        <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>query</code></td><td>string</td><td>Yes</td><td>Search query. Max 500 characters.</td></tr>
          <tr><td><code>max_results</code></td><td>integer</td><td>No</td><td>Videos to return. Range: 1–20. Default: 5.</td></tr>
          <tr><td><code>tier</code></td><td>string</td><td>No</td><td>Detail level for transcript extraction. Default: <code>summary</code>.</td></tr>
          <tr><td><code>include_transcript</code></td><td>boolean</td><td>No</td><td>Extract and include video transcripts. Default: true.</td></tr>
          <tr><td><code>language</code></td><td>string</td><td>No</td><td>Preferred transcript language (BCP-47). Default: <code>en</code>.</td></tr>
          <tr><td><code>min_duration</code></td><td>integer</td><td>No</td><td>Minimum video duration in seconds.</td></tr>
          <tr><td><code>max_duration</code></td><td>integer</td><td>No</td><td>Maximum video duration in seconds.</td></tr>
          <tr><td><code>freshness</code></td><td>string</td><td>No</td><td>Filter by upload date: <code>day</code>, <code>week</code>, <code>month</code>, <code>year</code>.</td></tr>
        </tbody>
      </table>

      <h2>Example request</h2>

      <CodeBlock language="bash" filename="youtube.sh" code={`curl -X POST https://api.fetchium.com/v1/youtube/search \\
  -H "Authorization: Bearer fetchium_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust async tokio tutorial",
    "max_results": 5,
    "tier": "summary",
    "include_transcript": true
  }'`} />

      <h2>Response</h2>

      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "query": "rust async tokio tutorial",
    "results_count": 5,
    "tokens_used": 2840,
    "duration_ms": 3120
  },
  "results": [
    {
      "video_id": "dQw4w9WgXcQ",
      "title": "Tokio Tutorial: Async Rust in Practice",
      "channel": "Let's Get Rusty",
      "channel_id": "UCpeX4D-ArTrsqvhLapAHprQ",
      "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
      "thumbnail": "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg",
      "duration_seconds": 2847,
      "view_count": 234521,
      "published_at": "2024-09-15T00:00:00Z",
      "score": 0.912,
      "transcript_snippet": "In this tutorial, we explore Tokio, the most popular async runtime for Rust. We cover the core concepts of async/await, the Tokio scheduler, and practical patterns for building concurrent applications..."
    }
  ]
}`} />

      <h3>Response fields</h3>
      <table>
        <thead><tr><th>Field</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>video_id</code></td><td>YouTube video ID</td></tr>
          <tr><td><code>duration_seconds</code></td><td>Video length in seconds</td></tr>
          <tr><td><code>view_count</code></td><td>Current view count</td></tr>
          <tr><td><code>score</code></td><td>HyperFusion relevance score (0–1)</td></tr>
          <tr><td><code>transcript_snippet</code></td><td>QATBE-extracted transcript excerpt (null if no transcript)</td></tr>
        </tbody>
      </table>

      <h2>Social research endpoint</h2>
      <p>Cross-platform social media research across Reddit, Hacker News, and dev communities:</p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/social/research</span>
      </div>

      <CodeBlock language="bash" code={`curl -X POST https://api.fetchium.com/v1/social/research \\
  -H "Authorization: Bearer fetchium_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "best rust web framework 2025",
    "platforms": ["reddit", "hackernews"],
    "max_sources": 10,
    "tier": "summary"
  }'`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/search", title: "Search API", desc: "Web search across all backends" },
          { href: "/docs/api/research", title: "Research API", desc: "Deep multi-source research" },
          { href: "/docs/api/scrape", title: "Scrape API", desc: "Single-URL content extraction" },
          { href: "/docs/api/usage", title: "Usage API", desc: "Monitor quota consumption" },
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
