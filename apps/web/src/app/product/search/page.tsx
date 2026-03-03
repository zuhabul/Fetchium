import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Search API — 11-Backend Federated Web Search for AI Apps",
  description:
    "Parallel search across DuckDuckGo, Brave, Bing, GitHub, Reddit, StackOverflow, YouTube, and 4+ more backends. BM25 + semantic HyperFusion reranking. JSON with citations in ~500ms.",
};

const backends = [
  { name: "DuckDuckGo", type: "General", note: "Privacy-first, no tracking" },
  { name: "Brave Search", type: "General", note: "Independent index, no Google dependency" },
  { name: "Bing", type: "General", note: "Microsoft index, broad coverage" },
  { name: "SearXNG", type: "Meta-search", note: "Self-hosted, privacy-preserving" },
  { name: "GitHub", type: "Code", note: "Code search + repositories" },
  { name: "Reddit", type: "Social", note: "Community discussions + opinions" },
  { name: "HackerNews", type: "Tech", note: "Developer community, tech news" },
  { name: "StackOverflow", type: "Q&A", note: "Technical Q&A, code solutions" },
  { name: "YouTube", type: "Video", note: "Video metadata + transcripts" },
  { name: "ArXiv", type: "Research", note: "Academic papers, preprints" },
  { name: "Wikipedia", type: "Reference", note: "Structured knowledge base" },
];

const signals = [
  { name: "BM25 Relevance", desc: "Term-frequency scoring against the query" },
  { name: "Semantic Similarity", desc: "Embedding-based meaning matching" },
  { name: "Temporal Freshness", desc: "Exponential decay favoring recent content" },
  { name: "Source Authority", desc: "Domain trust tiers + SSL/redirect penalties" },
  { name: "Evidence Consensus", desc: "Cross-source factual agreement score" },
  { name: "Content Diversity", desc: "Penalizes duplicate angles in results" },
  { name: "Content Depth", desc: "Rewards thorough over shallow content" },
  { name: "Query-Intent Match", desc: "Intent classification alignment" },
];

export default function ProductSearchPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          {/* Breadcrumb */}
          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Search API</span>
          </nav>

          {/* Hero */}
          <div className="mb-14">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Product · Search API
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Federated web search for{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                AI applications
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed mb-7">
              One query dispatched to 11+ backends in parallel. Results ranked by 8 independent
              signals — BM25, semantic similarity, temporal freshness, source authority, and more.
              Returns structured JSON with citations in approximately 500ms.
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
                href="/docs/api/search"
                className="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-white/3 px-6 py-3 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all"
              >
                API Reference
              </Link>
            </div>
          </div>

          {/* Key stats */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-14">
            {[
              { v: "11+", l: "Search backends" },
              { v: "~500ms", l: "P50 latency" },
              { v: "8", l: "Ranking signals" },
              { v: "$0.58", l: "per 1K queries" },
            ].map((s) => (
              <div key={s.l} className="rounded-xl border border-white/6 bg-white/[0.02] p-4 text-center">
                <div className="text-2xl font-bold bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">{s.v}</div>
                <div className="text-xs text-slate-500 mt-1">{s.l}</div>
              </div>
            ))}
          </div>

          {/* Backend grid */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-2">11+ Search Backends</h2>
            <p className="text-slate-500 mb-6 text-sm">
              All backends are queried in parallel. Circuit breakers handle failures transparently.
              Results arrive from the fastest responding backends first.
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {backends.map((b) => (
                <div key={b.name} className="rounded-xl border border-white/6 bg-white/[0.02] p-4 flex items-start gap-3">
                  <div className="h-2 w-2 rounded-full bg-emerald-400 mt-1.5 shrink-0" />
                  <div>
                    <div className="text-sm font-semibold text-slate-200">{b.name}</div>
                    <div className="text-[11px] text-indigo-400 mt-0.5">{b.type}</div>
                    <div className="text-[11px] text-slate-600 mt-0.5">{b.note}</div>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* HyperFusion 8-signal ranking */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-2">HyperFusion: 8-Signal Ranking</h2>
            <p className="text-slate-500 mb-6 text-sm">
              Results from all backends are merged and re-ranked using HyperFusion — 8 independent signals
              combined into a single relevance score. No single signal dominates; each is weighted by
              query intent and content type.
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {signals.map((s, i) => (
                <div key={s.name} className="rounded-xl border border-white/6 bg-white/[0.02] p-4 flex items-start gap-3">
                  <div className="h-6 w-6 rounded-lg bg-indigo-500/15 flex items-center justify-center shrink-0 text-[11px] font-bold text-indigo-400">
                    {i + 1}
                  </div>
                  <div>
                    <div className="text-sm font-semibold text-slate-200">{s.name}</div>
                    <div className="text-[12px] text-slate-500 mt-0.5">{s.desc}</div>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* Code example */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-2">Quick Start</h2>
            <p className="text-slate-500 mb-4 text-sm">Get ranked search results in one API call:</p>
            <div className="rounded-2xl border border-white/8 bg-[#0d0f1a] overflow-hidden">
              <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/6 bg-white/[0.015]">
                <span className="text-[12px] font-mono text-slate-500">cURL</span>
                <span className="text-[11px] font-mono text-emerald-400">POST /v1/search</span>
              </div>
              <pre className="p-4 text-[13px] font-mono text-slate-300 overflow-x-auto leading-relaxed">
{`curl -X POST https://api.fetchium.com/v1/search \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "best async rust patterns 2025",
    "backends": ["duckduckgo", "brave", "github"],
    "max_results": 10,
    "extract_content": true,
    "token_budget": 4096
  }'`}
              </pre>
            </div>
          </section>

          {/* Internal links */}
          <section className="rounded-2xl border border-indigo-500/15 bg-indigo-500/5 p-6">
            <h3 className="text-base font-semibold text-slate-200 mb-4">Related</h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {[
                { href: "/product/extract", label: "Extract API", desc: "Full CEP content extraction" },
                { href: "/compare/tavily", label: "vs Tavily", desc: "Feature + price comparison" },
                { href: "/docs/api/search", label: "API Reference", desc: "Full parameter documentation" },
              ].map((l) => (
                <Link
                  key={l.href}
                  href={l.href}
                  className="flex items-start gap-2 rounded-lg border border-white/6 bg-white/2 p-3 hover:bg-white/5 transition-all group"
                >
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
