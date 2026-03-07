import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "MCP Protocol Integration" };

export default function McpSDK() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">SDKs & Integrations</div>

      <h1>MCP Protocol</h1>
      <p>
        Fetchium exposes an MCP server with tool schemas backed by the same core pipelines as
        the REST API.
      </p>

      <p>
        Use stdio for local agent integration, or run the HTTP MCP endpoint on <code>/mcp</code>
        for remote deployments.
      </p>

      <h2>Available MCP tools (12)</h2>
      <table>
        <thead><tr><th>Tool name</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>fetchium_search</code></td><td>Web search with ranking + extraction</td></tr>
          <tr><td><code>fetchium_fetch</code></td><td>Query-aware URL extraction</td></tr>
          <tr><td><code>fetchium_research</code></td><td>Deep multi-source research</td></tr>
          <tr><td><code>fetchium_estimate</code></td><td>Token cost estimate before fetch</td></tr>
          <tr><td><code>fetchium_expand</code></td><td>Progressive detail expansion</td></tr>
        <tr><td><code>youtube_search</code></td><td>YouTube search with VideoFusion ranking</td></tr>
          <tr><td><code>youtube_analyze</code></td><td>Single-video deep analysis</td></tr>
          <tr><td><code>youtube_watch</code></td><td>Unified watch report (metadata + transcript + moments)</td></tr>
          <tr><td><code>youtube_transcript</code></td><td>Transcript extraction with quality and key moments</td></tr>
          <tr><td><code>social_research</code></td><td>Unified multi-platform social research</td></tr>
          <tr><td><code>reddit_search</code></td><td>Reddit-focused search pipeline</td></tr>
          <tr><td><code>hackernews_search</code></td><td>Hacker News search pipeline</td></tr>
        </tbody>
      </table>

      <h2>Claude Desktop setup</h2>
      <CodeBlock language="json" filename="~/Library/Application Support/Claude/claude_desktop_config.json" code={`{
  "mcpServers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["serve", "--mode", "mcp"]
    }
  }
}`} />

      <h2>Cursor setup</h2>
      <CodeBlock language="json" filename=".cursor/mcp.json" code={`{
  "mcpServers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["serve", "--mode", "mcp"]
    }
  }
}`} />

      <h2>Smoke test</h2>
      <CodeBlock language="bash" code={`fetchium serve --mode mcp --transport http --port 3471
curl -X POST http://127.0.0.1:3471/mcp \\
  -H "Content-Type: application/json" \\
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/search", title: "Search API", desc: "REST endpoint reference" },
          { href: "/docs/api/youtube", title: "YouTube API", desc: "Video search and analysis" },
          { href: "/docs/api/social", title: "Social API", desc: "Cross-platform social research" },
          { href: "/docs/quickstart", title: "Quick Start", desc: "Get started fast" },
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
