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
        YouTube endpoints return pipeline output for search and deep single-video analysis.
      </p>

      <h2>Endpoints</h2>
      <table>
        <thead><tr><th>Method</th><th>Path</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/search</code></td><td>Search/rank videos</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/analyze</code></td><td>Analyze one video URL</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/search/jobs</code></td><td>Async YouTube search</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/analyze/jobs</code></td><td>Async video analysis</td></tr>
        </tbody>
      </table>

      <h2>/v1/youtube/search request</h2>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Required</th></tr></thead>
        <tbody>
          <tr><td><code>query</code></td><td>string</td><td>Yes</td></tr>
          <tr><td><code>max_results</code></td><td>integer</td><td>No</td></tr>
          <tr><td><code>fact_check</code></td><td>boolean</td><td>No</td></tr>
        </tbody>
      </table>

      <CodeBlock language="bash" filename="youtube-search.sh" code={`curl -X POST https://api.fetchium.com/v1/youtube/search \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "Java learning",
    "max_results": 10,
    "fact_check": false
  }'`} />

      <h2>/v1/youtube/analyze request</h2>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Required</th></tr></thead>
        <tbody>
          <tr><td><code>url</code></td><td>string</td><td>Yes</td></tr>
          <tr><td><code>transcript</code></td><td>boolean</td><td>No</td></tr>
          <tr><td><code>comments</code></td><td>boolean</td><td>No</td></tr>
          <tr><td><code>teaching</code></td><td>boolean</td><td>No</td></tr>
        </tbody>
      </table>

      <CodeBlock language="bash" filename="youtube-analyze.sh" code={`curl -X POST https://api.fetchium.com/v1/youtube/analyze \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    "transcript": true,
    "comments": true,
    "teaching": false
  }'`} />

      <h2>Representative response shape</h2>
      <CodeBlock language="json" filename="youtube-response.json" code={`{
  "meta": {
    "request_id": "3ea58c50-5091-4f57-9224-7727ddfb6698",
    "status": "ok",
    "endpoint": "/v1/youtube/search",
    "duration_ms": 912,
    "query": "Java learning",
    "tokens_used": 0
  },
  "data": {
    "query": "Java learning",
    "videos": [
      {
        "metadata": {
          "video_id": "abc123",
          "title": "Java Tutorial for Beginners",
          "channel": {
            "name": "Example Channel",
            "id": "UCxxxx",
            "subscriber_count": 120000,
            "verified": true
          },
          "duration_secs": 3600,
          "view_count": 250000,
          "like_count": 8400,
          "published": "2025-01-08"
        },
        "credibility": {
          "score": 0.78,
          "tier": "established"
        }
      }
    ],
    "rankings": [
      {
        "video_id": "abc123",
        "final_score": 0.87,
        "educational_score": 0.74,
        "clickbait_score": 0.18
      }
    ]
  }
}`} />

      <p>
        Response may include additional fields such as transcript, comments, timeline, learning
        path, and fact checks depending on request and pipeline state.
      </p>

      <p>
        Synchronous endpoints wrap the pipeline output in <code>meta</code> plus <code>data</code>.
        The async <code>/jobs</code> variants return a queued job first, then expose the final
        pipeline payload under <code>result</code> when you poll <code>/v1/jobs/:id</code>.
      </p>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/social", title: "Social API", desc: "Cross-platform social research" },
          { href: "https://docs.fetchium.com/api/async-jobs", title: "Async Jobs", desc: "Queue YouTube searches and analysis" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Web search endpoint" },
          { href: "https://docs.fetchium.com/sdk/mcp", title: "MCP Protocol", desc: "Tool integration" },
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
