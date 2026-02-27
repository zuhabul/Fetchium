import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "MCP Protocol Integration" };

export default function McpSDK() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">SDKs & Integrations</div>

      <div className="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-md bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 text-[11px] font-bold mb-4">
        NEW
      </div>

      <h1>MCP Protocol</h1>
      <p>
        Fetchium implements the <strong>Model Context Protocol (MCP)</strong> — the open
        standard for connecting AI models to external tools and data sources. Give any
        MCP-compatible AI assistant (Claude, Cursor, Zed, etc.) direct access to
        Fetchium search, scraping, and research capabilities.
      </p>

      <h2>What MCP gives you</h2>
      <ul>
        <li>Claude Desktop, Cursor, and Zed can call Fetchium as a native tool</li>
        <li>No API calls in your prompt — the AI calls the tool directly</li>
        <li>Fully typed tool schemas with automatic parameter validation</li>
        <li>Real-time web search grounding for any AI workflow</li>
      </ul>

      <h2>Available MCP tools</h2>
      <table>
        <thead><tr><th>Tool name</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>fetchium_search</code></td><td>Web search with HyperFusion ranking</td></tr>
          <tr><td><code>fetchium_scrape</code></td><td>Extract content from any URL</td></tr>
          <tr><td><code>fetchium_research</code></td><td>Deep multi-source research</td></tr>
          <tr><td><code>fetchium_social</code></td><td>Reddit and HackerNews search</td></tr>
          <tr><td><code>fetchium_youtube</code></td><td>YouTube video search</td></tr>
        </tbody>
      </table>

      <h2>Claude Desktop setup</h2>
      <p>Add Fetchium to your Claude Desktop config:</p>

      <CodeBlock language="json" filename="~/Library/Application Support/Claude/claude_desktop_config.json" code={`{
  "mcpServers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["mcp"],
      "env": {
        "FETCHIUM_API_KEY": "fetchium_your_key_here"
      }
    }
  }
}`} />

      <h2>Cursor setup</h2>
      <CodeBlock language="json" filename=".cursor/mcp.json" code={`{
  "mcpServers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["mcp"],
      "env": {
        "FETCHIUM_API_KEY": "fetchium_your_key_here"
      }
    }
  }
}`} />

      <h2>Remote MCP server</h2>
      <p>
        If you prefer not to install the CLI, use the hosted MCP endpoint:
      </p>
      <CodeBlock language="json" code={`{
  "mcpServers": {
    "fetchium": {
      "url": "https://mcp.fetchium.com/sse",
      "headers": {
        "Authorization": "Bearer fetchium_your_key_here"
      }
    }
  }
}`} />

      <h2>Tool schema reference</h2>

      <CodeBlock language="json" filename="fetchium_search tool" code={`{
  "name": "fetchium_search",
  "description": "Search the web with HyperFusion multi-signal ranking. Returns ranked results with extracted content.",
  "inputSchema": {
    "type": "object",
    "required": ["query"],
    "properties": {
      "query": {
        "type": "string",
        "description": "The search query"
      },
      "tier": {
        "type": "string",
        "enum": ["key_facts", "summary", "detailed"],
        "description": "Content detail level",
        "default": "summary"
      },
      "max_sources": {
        "type": "integer",
        "description": "Maximum sources to search (1-20)",
        "default": 5
      }
    }
  }
}`} />

      <h2>Testing the MCP server</h2>
      <CodeBlock language="bash" code={`# Start the MCP server manually to test
fetchium mcp

# In another terminal, use the MCP inspector
npx @modelcontextprotocol/inspector fetchium mcp`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/sdk/typescript", title: "TypeScript SDK", desc: "REST API integration" },
          { href: "/docs/sdk/python", title: "Python SDK", desc: "Python integration" },
          { href: "/docs/api/search", title: "Search API", desc: "Full REST reference" },
          { href: "/docs/quickstart", title: "Quick Start", desc: "Get started in 60 seconds" },
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
