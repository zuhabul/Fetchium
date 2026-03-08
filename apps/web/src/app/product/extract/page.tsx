import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Extract API — 5-Layer Web Content Extraction (CEP)",
  description:
    "CSS selectors → readability → headless JS → PDF → screenshot OCR. The Content Extraction Protocol delivers clean, LLM-ready content from any URL. Token-budgeted output (QATBE).",
};

const layers = [
  {
    n: 1,
    name: "CSS Selector Extraction",
    desc: "Fast-path: if the page has semantic HTML (article, main, .content), CSS selectors extract clean content in milliseconds without rendering.",
    speed: "~5ms",
    color: "text-emerald-400",
    bg: "bg-emerald-500/10",
  },
  {
    n: 2,
    name: "Readability Parsing",
    desc: "Mozilla Readability algorithm removes boilerplate (navigation, ads, footers) and extracts the main article body with high accuracy on most news and blog content.",
    speed: "~15ms",
    color: "text-blue-400",
    bg: "bg-blue-500/10",
  },
  {
    n: 3,
    name: "Headless JS Rendering",
    desc: "For JavaScript-heavy SPAs, Chromium renders the page fully before extraction. Captures dynamically loaded content that pure HTML parsing misses.",
    speed: "~800ms",
    color: "text-indigo-400",
    bg: "bg-indigo-500/10",
  },
  {
    n: 4,
    name: "PDF Extraction",
    desc: "PDFs, academic papers, and document URLs are handled natively — text extracted, tables preserved, images skipped unless OCR is requested.",
    speed: "~200ms",
    color: "text-violet-400",
    bg: "bg-violet-500/10",
  },
  {
    n: 5,
    name: "Screenshot OCR",
    desc: "Last resort: if the page is an image-only document or has text encoded as graphics, OCR extracts the visible text. This is the slowest layer, used rarely.",
    speed: "~2,000ms",
    color: "text-orange-400",
    bg: "bg-orange-500/10",
  },
];

export default function ProductExtractPage() {
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
            <span className="text-slate-400">Extract API</span>
          </nav>

          <div className="mb-14">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Product · Extract API
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Clean content extraction for{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                LLMs — 5 layers deep
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed mb-7">
              The Content Extraction Protocol (CEP) cascades through 5 techniques — from fast CSS selectors
              to headless rendering to OCR — selecting the right method for each URL automatically.
              The result: clean, token-budgeted content ready for any LLM context window.
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
                href="https://docs.fetchium.com/api/scrape"
                className="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-white/3 px-6 py-3 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all"
              >
                API Reference
              </Link>
            </div>
          </div>

          {/* Stats */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-14">
            {[
              { v: "5", l: "Extraction layers" },
              { v: "~15ms", l: "Typical (CSS)" },
              { v: "~800ms", l: "JS render path" },
              { v: "90%", l: "Token reduction" },
            ].map((s) => (
              <div key={s.l} className="rounded-xl border border-white/6 bg-white/[0.02] p-4 text-center">
                <div className="text-2xl font-bold bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">{s.v}</div>
                <div className="text-xs text-slate-500 mt-1">{s.l}</div>
              </div>
            ))}
          </div>

          {/* 5-layer cascade */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-2">The 5-Layer Extraction Cascade</h2>
            <p className="text-slate-500 mb-6 text-sm">
              CEP automatically selects the right layer for each URL. Fast layers run first;
              slower layers only activate when needed. Average extraction time is 15–200ms for most web pages.
            </p>
            <div className="space-y-3">
              {layers.map((layer) => (
                <div key={layer.n} className="rounded-xl border border-white/6 bg-white/[0.02] p-5 flex items-start gap-4">
                  <div className={`h-8 w-8 rounded-lg ${layer.bg} flex items-center justify-center shrink-0 text-sm font-bold ${layer.color}`}>
                    {layer.n}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-1">
                      <span className="text-sm font-semibold text-slate-200">{layer.name}</span>
                      <span className={`text-[11px] font-mono ${layer.color}`}>{layer.speed}</span>
                    </div>
                    <p className="text-[13px] text-slate-500 leading-relaxed">{layer.desc}</p>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* QATBE */}
          <section className="mb-14 rounded-2xl border border-violet-500/15 bg-violet-500/5 p-6 sm:p-8">
            <div className="mb-3 text-xs font-semibold uppercase tracking-widest text-violet-400">
              QATBE Algorithm
            </div>
            <h2 className="text-xl sm:text-2xl font-bold mb-3">
              Token-Budgeted Extraction
            </h2>
            <p className="text-slate-400 text-sm leading-relaxed mb-6">
              After extraction, QATBE (Query-Aware Token-Budgeted Extraction) scores each content segment
              by BM25 relevance to your query, then packs the highest-scoring segments into your token budget
              using a greedy knapsack algorithm.
            </p>
            <div className="grid sm:grid-cols-3 gap-4">
              {[
                { stat: "60–90%", desc: "Token reduction vs. raw HTML", color: "text-emerald-400" },
                { stat: "BM25", desc: "Query-relevance scoring per segment", color: "text-indigo-400" },
                { stat: "4096", desc: "Default token budget (configurable)", color: "text-violet-400" },
              ].map((item) => (
                <div key={item.stat} className="rounded-xl border border-white/6 bg-white/3 p-4 text-center">
                  <div className={`text-2xl font-bold ${item.color}`}>{item.stat}</div>
                  <div className="text-[12px] text-slate-500 mt-1">{item.desc}</div>
                </div>
              ))}
            </div>
          </section>

          {/* Code example */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-4">Extract a URL</h2>
            <div className="rounded-2xl border border-white/8 bg-[#0d0f1a] overflow-hidden">
              <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/6 bg-white/[0.015]">
                <span className="text-[12px] font-mono text-slate-500">cURL</span>
                <span className="text-[11px] font-mono text-emerald-400">POST /v1/scrape</span>
              </div>
              <pre className="p-4 text-[13px] font-mono text-slate-300 overflow-x-auto leading-relaxed">
{`curl -X POST https://api.fetchium.com/v1/scrape \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://example.com/article",
    "query": "async rust patterns",
    "token_budget": 4096,
    "format": "markdown",
    "extract_citations": true
  }'`}
              </pre>
            </div>
          </section>

          {/* Internal links */}
          <section className="rounded-2xl border border-indigo-500/15 bg-indigo-500/5 p-6">
            <h3 className="text-base font-semibold text-slate-200 mb-4">Related</h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {[
                { href: "/product/search", label: "Search API", desc: "Search + extract in one call" },
                { href: "/compare/firecrawl", label: "vs Firecrawl", desc: "Compare extraction APIs" },
                { href: "https://docs.fetchium.com/algorithms/cep", label: "CEP Algorithm Docs", desc: "Deep technical reference" },
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
