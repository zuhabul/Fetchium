import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium MCP Tools — Model Context Protocol Server for Claude & Cursor",
  description:
    "Fetchium MCP server with 12 tools covering search, fetch, estimate, research, YouTube, and social workflows. Works with Claude Desktop, Cursor, and other MCP-compatible clients.",
};

const tools = [
  {
    name: "fetchium_search",
    desc: "Search the web via 11+ backends. Returns ranked results with content snippets.",
    params: ["query: string", "max_results?: number", "backends?: string[]"],
  },
  {
    name: "fetchium_fetch",
    desc: "Fetch and extract clean content from a URL using the CEP pipeline.",
    params: ["url: string", "query?: string", "token_budget?: number"],
  },
  {
    name: "fetchium_estimate",
    desc: "Estimate token and processing costs before a request.",
    params: ["query?: string", "url?: string", "token_budget?: number"],
  },
  {
    name: "fetchium_research",
    desc: "Run a full multi-agent research task and return a structured report with citations.",
    params: ["query: string", "depth?: 'standard' | 'deep'", "max_sources?: number"],
  },
  {
    name: "fetchium_youtube_search",
    desc: "Search YouTube content and return structured video results.",
    params: ["query: string", "max_results?: number"],
  },
  {
    name: "fetchium_youtube_analyze",
    desc: "Analyze a YouTube video, transcript, or channel context.",
    params: ["url?: string", "video_id?: string"],
  },
  {
    name: "fetchium_social_research",
    desc: "Run cross-platform social research workflows.",
    params: ["query: string", "max_results?: number"],
  },
];

const clients = [
  { name: "Claude Desktop", status: "Supported", note: "Official Anthropic MCP client" },
  { name: "Cursor", status: "Supported", note: "Works as MCP extension" },
  { name: "Continue.dev", status: "Supported", note: "VS Code AI coding extension" },
  { name: "Any stdio MCP client", status: "Supported", note: "JSON-RPC 2.0 compatible" },
];

export default function ProductMCPPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <Link href="/product/search" className="hover:text-slate-400 transition-colors">Product</Link>
            <span>/</span>
            <span className="text-slate-400">MCP Tools</span>
          </nav>

          <div className="mb-14">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Product · MCP Tools
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Fetchium as an{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                MCP server
              </span>
              {" "}for AI clients
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed mb-7">
              The Model Context Protocol (MCP) lets AI clients like Claude Desktop and Cursor call external
              tools without writing any integration code. Fetchium ships a JSON-RPC 2.0 stdio MCP server
              with 12 tools spanning search, fetch, estimate, research, YouTube, and social workflows.
            </p>
            <div className="flex flex-wrap gap-3">
              <Link
                href="https://app.fetchium.com/register"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-6 py-3 text-sm font-semibold text-white shadow-[0_0_24px_rgba(99,102,241,0.3)] hover:shadow-[0_0_36px_rgba(99,102,241,0.5)] transition-all"
              >
                Get API Key Free →
              </Link>
              <Link
                href="https://docs.fetchium.com/sdk/mcp"
                className="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-white/3 px-6 py-3 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all"
              >
                MCP Setup Guide
              </Link>
            </div>
          </div>

          {/* tool overview */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-2">12 MCP Tools</h2>
            <p className="text-slate-500 mb-6 text-sm">
              Fetchium exposes 12 tool definitions in the current MCP server. A representative subset is shown below.
            </p>
            <div className="space-y-3">
              {tools.map((tool) => (
                <div key={tool.name} className="rounded-xl border border-white/6 bg-white/[0.02] p-5">
                  <div className="flex items-start gap-4">
                    <code className="shrink-0 rounded-lg bg-indigo-500/12 border border-indigo-500/20 px-3 py-1 text-[12px] font-mono text-indigo-300">
                      {tool.name}
                    </code>
                    <div className="flex-1 min-w-0">
                      <p className="text-[13px] text-slate-400 mb-3">{tool.desc}</p>
                      <div className="flex flex-wrap gap-2">
                        {tool.params.map((p) => (
                          <code key={p} className="text-[11px] font-mono text-slate-500 bg-white/3 border border-white/6 rounded px-2 py-0.5">
                            {p}
                          </code>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* Claude Desktop setup */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-4">Setup: Claude Desktop</h2>
            <div className="rounded-2xl border border-white/8 bg-[#0d0f1a] overflow-hidden">
              <div className="px-4 py-2.5 border-b border-white/6 bg-white/[0.015]">
                <span className="text-[12px] font-mono text-slate-500">claude_desktop_config.json</span>
              </div>
              <pre className="p-4 text-[13px] font-mono text-slate-300 overflow-x-auto leading-relaxed">
{`{
  "mcpServers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["mcp"],
      "env": {
        "FETCHIUM_API_KEY": "your_api_key_here"
      }
    }
  }
}`}
              </pre>
            </div>
            <p className="mt-3 text-[12px] text-slate-600">
              Install the CLI: <code className="bg-white/5 border border-white/8 rounded px-1.5 py-0.5 text-slate-400">npm install -g fetchium</code> or{" "}
              <code className="bg-white/5 border border-white/8 rounded px-1.5 py-0.5 text-slate-400">cargo install fetchium</code>
            </p>
          </section>

          {/* Compatible clients */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-6">Compatible Clients</h2>
            <div className="grid sm:grid-cols-2 gap-3">
              {clients.map((c) => (
                <div key={c.name} className="rounded-xl border border-white/6 bg-white/[0.02] p-4 flex items-center gap-4">
                  <div className="h-2 w-2 rounded-full bg-emerald-400 shrink-0" />
                  <div>
                    <div className="text-sm font-semibold text-slate-200">{c.name}</div>
                    <div className="text-[12px] text-slate-500 mt-0.5">{c.note}</div>
                  </div>
                  <span className="ml-auto text-[11px] text-emerald-400 font-medium shrink-0">{c.status}</span>
                </div>
              ))}
            </div>
          </section>

          <section className="rounded-2xl border border-indigo-500/15 bg-indigo-500/5 p-6">
            <h3 className="text-base font-semibold text-slate-200 mb-4">Related</h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {[
                { href: "https://docs.fetchium.com/sdk/mcp", label: "MCP Setup Guide", desc: "Full setup documentation" },
                { href: "/product/search", label: "Search API", desc: "Direct REST API access" },
                { href: "/pricing", label: "Pricing", desc: "MCP included in all plans" },
              ].map((l) => (
                <Link key={l.href} href={l.href} className="flex items-start gap-2 rounded-lg border border-white/6 bg-white/2 p-3 hover:bg-white/5 transition-all group">
                  <div>
                    <div className="text-[13px] font-medium text-slate-300 group-hover:text-white">{l.label}</div>
                    <div className="text-[11px] text-slate-600">{l.desc}</div>
                  </div>
                </Link>
              ))}
            </div>
          </section>
        </div>
      </main>

      <Footer />
    </div>
  );
}
