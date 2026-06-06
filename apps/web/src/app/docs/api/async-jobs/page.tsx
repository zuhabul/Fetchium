import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Async Jobs API Reference" };

export default function AsyncJobsApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Async Jobs</h1>
      <p>
        Long-running pipelines expose async submission endpoints that return a job ID immediately.
        Poll <code>/v1/jobs/:id</code> until the job reaches <code>completed</code> or{" "}
        <code>failed</code>.
      </p>

      <h2>Supported async endpoints</h2>
      <table>
        <thead><tr><th>Method</th><th>Path</th><th>Returns</th></tr></thead>
        <tbody>
          <tr><td><code>POST</code></td><td><code>/v1/research/jobs</code></td><td>Accepted research job</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/search/jobs</code></td><td>Accepted YouTube search job</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/youtube/analyze/jobs</code></td><td>Accepted YouTube analysis job</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/research/jobs</code></td><td>Accepted social research job</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/reddit/jobs</code></td><td>Accepted Reddit job</td></tr>
          <tr><td><code>POST</code></td><td><code>/v1/social/hackernews/jobs</code></td><td>Accepted Hacker News job</td></tr>
          <tr><td><code>GET</code></td><td><code>/v1/jobs/:id</code></td><td>Current job status or result</td></tr>
        </tbody>
      </table>

      <h2>Submit a job</h2>
      <CodeBlock language="bash" filename="submit-research-job.sh" code={`curl -X POST https://api.fetchium.com/v1/research/jobs \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "Compare Rust web frameworks for production APIs",
    "max_sources": 8,
    "strict_evidence": true,
    "citation_style": "apa"
  }'`} />

      <CodeBlock language="json" filename="accepted.json" code={`{
  "meta": {
    "request_id": "8b51f4e0-2d63-4800-a4f0-9a7e0d6500de",
    "status": "ok",
    "endpoint": "/v1/research/jobs",
    "duration_ms": 0,
    "result_id": "f6ca2d8d-4b7c-4b26-8187-676121c1a8c1"
  },
  "job_id": "f6ca2d8d-4b7c-4b26-8187-676121c1a8c1",
  "status": "queued",
  "status_url": "/v1/jobs/f6ca2d8d-4b7c-4b26-8187-676121c1a8c1"
}`} />

      <h2>Poll job status</h2>
      <CodeBlock language="bash" filename="poll-job.sh" code={`curl https://api.fetchium.com/v1/jobs/f6ca2d8d-4b7c-4b26-8187-676121c1a8c1 \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY"`} />

      <CodeBlock language="json" filename="completed.json" code={`{
  "meta": {
    "request_id": "2ff9b6e3-03d8-46c4-bfcf-2a36d3e92f3a",
    "status": "completed",
    "endpoint": "/v1/jobs/:id",
    "duration_ms": 0
  },
  "job_id": "f6ca2d8d-4b7c-4b26-8187-676121c1a8c1",
  "job_type": "research",
  "status": "completed",
  "created_at": "2026-03-08T17:00:00Z",
  "started_at": "2026-03-08T17:00:01Z",
  "completed_at": "2026-03-08T17:00:12Z",
  "result": {
    "meta": {
      "endpoint": "/v1/research/jobs",
      "status": "completed"
    },
    "report": "## Comparison\\n...",
    "reference_section": "[1] ...",
    "sources": [],
    "confidence": 0.86
  }
}`} />

      <h2>Status codes</h2>
      <table>
        <thead><tr><th>Response</th><th>Meaning</th></tr></thead>
        <tbody>
          <tr><td><code>202 Accepted</code></td><td>Job is still queued or running</td></tr>
          <tr><td><code>200 OK</code></td><td>Job completed or failed and final payload is available</td></tr>
          <tr><td><code>404 Not Found</code></td><td>Job ID does not exist for the authenticated key</td></tr>
        </tbody>
      </table>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/research", title: "Research API", desc: "Synchronous research endpoint" },
          { href: "https://docs.fetchium.com/api/youtube", title: "YouTube API", desc: "Search and analysis routes" },
          { href: "https://docs.fetchium.com/api/social", title: "Social API", desc: "Cross-platform and platform-specific jobs" },
          { href: "https://docs.fetchium.com/errors", title: "Error Reference", desc: "Job and auth error formats" },
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
