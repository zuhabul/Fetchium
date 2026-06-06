import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { Check, X, Minus } from "lucide-react";

export const metadata: Metadata = {
  title: "Fetchium vs SerpAPI — AI-Native Search vs Google SERP Scraping (2026)",
  description:
    "SerpAPI charges $15/1K for raw Google SERP JSON and faces a Google DMCA lawsuit (Dec 2025). Fetchium delivers full-pipeline AI search at $0.90/1K using independent backends.",
};

/**
 * SerpAPI pricing: serpapi.com/pricing, March 2026.
 * Avg latency: 2.972s (independent 50-query benchmark, dev.to, 2025).
 * Google DMCA lawsuit filed December 2025 (publicly reported).
 * SerpAPI motion to dismiss filed February 2026.
 */

const rows = [
  { feature: "Multi-source federation", f: "yes", s: "partial", note: "SerpAPI covers 20+ engines but each is a separate endpoint, not parallel" },
  { feature: "Full content extraction", f: "yes", s: "no", note: "SerpAPI returns SERP metadata (titles, snippets, URLs); no full page content" },
  { feature: "Token budget management", f: "yes", s: "no", note: "Fetchium delivers LLM-ready token-budgeted content; SerpAPI delivers raw JSON" },
  { feature: "Evidence graphs + citations", f: "yes", s: "no", note: "Fetchium's evidence graphs are unique; SerpAPI has no citation support" },
  { feature: "AI/LLM-ready output", f: "yes", s: "no", note: "SerpAPI is optimized for SEO tools, not AI ingestion" },
  { feature: "Cross-session learning", f: "yes", s: "no" },
  { feature: "Multi-agent deep research", f: "yes", s: "no" },
  { feature: "YouTube & social search", f: "yes", s: "yes", note: "SerpAPI has YouTube and Google Shopping; Fetchium has dedicated endpoints" },
  { feature: "MCP protocol support", f: "yes", s: "no" },
  { feature: "Google-independent backends", f: "yes", s: "no", note: "SerpAPI depends entirely on Google; Fetchium uses DDG, Brave, SearXNG, etc." },
  { feature: "Legal risk (Google lawsuit)", f: "none", s: "active", note: "Google filed DMCA lawsuit vs SerpAPI, December 2025. Case ongoing." },
  { feature: "Free tier (renewing)", f: "yes", s: "partial", note: "SerpAPI: 100 free searches/month. Fetchium: 1,000/month." },
  { feature: "Search P50 latency", f: "~500ms", s: "~3.0s", note: "SerpAPI: 2.972s avg (50-query benchmark, 2025)" },
  { feature: "Price per 1K queries", f: "$0.90", s: "$15.00", note: "SerpAPI Developer plan ($75/mo, 5K searches). Fetchium Starter ($9/mo, 10K)." },
];

function Cell({ v }: { v: string }) {
  if (v === "yes" || v === "none") return <div className="flex justify-center"><div className="h-5 w-5 rounded-full bg-emerald-500/15 flex items-center justify-center"><Check className="h-3 w-3 text-emerald-400" strokeWidth={3} /></div></div>;
  if (v === "no") return <div className="flex justify-center"><X className="h-3.5 w-3.5 text-slate-700" /></div>;
  if (v === "partial") return <div className="flex justify-center"><Minus className="h-3.5 w-3.5 text-amber-500/70" /></div>;
  if (v === "active") return <div className="flex justify-center"><span className="text-[11px] text-red-400 font-medium">⚠ Active</span></div>;
  return <div className="flex justify-center"><span className="font-mono text-[11px] text-slate-400 bg-white/4 border border-white/8 rounded px-1.5 py-0.5">{v}</span></div>;
}

export default function CompareSerpAPIPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Fetchium vs SerpAPI</span>
          </nav>

          <div className="mb-12">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Verified Comparison · March 2026
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Fetchium vs SerpAPI:{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                AI-Native Search vs Google SERP Scraping
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed">
              SerpAPI has been a reliable SERP scraper since 2017 — but it&apos;s built for SEO monitoring,
              not AI ingestion. It returns raw SERP metadata (titles, URLs, snippets), costs $15/1K queries,
              and faces an active Google DMCA lawsuit filed December 2025. Fetchium delivers a full AI-ready
              pipeline at $0.90/1K with no Google dependency.
            </p>
          </div>

          {/* Legal risk callout */}
          <div className="mb-8 rounded-xl border border-red-500/20 bg-red-500/8 p-4">
            <div className="flex items-start gap-3">
              <span className="text-red-400 text-xl shrink-0">⚠</span>
              <div>
                <p className="text-sm font-semibold text-red-300">SerpAPI Legal Risk — December 2025</p>
                <p className="text-[13px] text-slate-400 mt-1 leading-relaxed">
                  Google filed a DMCA lawsuit against SerpAPI in December 2025, alleging the use of fake bot
                  names and cloaking to bypass Google&apos;s SearchGuard system. SerpAPI filed a motion to dismiss in
                  February 2026. The case is ongoing. Fetchium uses independent backends (DuckDuckGo, Brave,
                  SearXNG) and does not scrape Google — no equivalent legal risk.
                </p>
              </div>
            </div>
          </div>

          <div className="mb-10 grid sm:grid-cols-3 gap-4">
            <div className="rounded-2xl border border-emerald-500/20 bg-emerald-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-emerald-400">16×</div>
              <div className="text-sm text-slate-300 mt-1">cheaper per query</div>
              <div className="text-[12px] text-slate-500 mt-1">$0.90 vs $15.00 per 1K</div>
            </div>
            <div className="rounded-2xl border border-indigo-500/20 bg-indigo-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-indigo-300">6×</div>
              <div className="text-sm text-slate-300 mt-1">faster search P50</div>
              <div className="text-[12px] text-slate-500 mt-1">~500ms vs ~3.0s avg</div>
            </div>
            <div className="rounded-2xl border border-violet-500/20 bg-violet-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-violet-300">0</div>
              <div className="text-sm text-slate-300 mt-1">Google dependency</div>
              <div className="text-[12px] text-slate-500 mt-1">Independent backends only</div>
            </div>
          </div>

          <div className="mb-10 overflow-x-auto rounded-2xl border border-white/8 shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
            <table className="w-full border-collapse min-w-[480px]">
              <thead>
                <tr className="border-b border-white/6">
                  <th className="py-4 px-5 text-left text-[12px] font-medium text-slate-600 w-64">Feature</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-indigo-300 bg-[rgba(99,102,241,0.06)] w-32">Fetchium</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-slate-500 w-32">SerpAPI</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((row, i) => (
                  <tr key={row.feature} className={`border-b border-white/4 last:border-0 ${i % 2 === 0 ? "" : "bg-white/[0.01]"}`}>
                    <td className="py-3.5 px-5">
                      <div className="text-[13px] text-slate-300">{row.feature}</div>
                      {row.note && <div className="text-[11px] text-slate-600 mt-0.5">{row.note}</div>}
                    </td>
                    <td className="py-3.5 px-4 bg-[rgba(99,102,241,0.03)]"><Cell v={row.f} /></td>
                    <td className="py-3.5 px-4"><Cell v={row.s} /></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="rounded-2xl border border-indigo-500/15 bg-gradient-to-r from-indigo-500/8 to-violet-500/6 p-6 text-center">
            <p className="text-base font-semibold text-slate-200 mb-4">Switch from SerpAPI to Fetchium — 1,000 free requests/month</p>
            <div className="flex flex-col sm:flex-row gap-3 justify-center">
              <Link href="https://app.fetchium.com/register" target="_blank" rel="noopener noreferrer" className="inline-flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-6 py-3 text-sm font-semibold text-white shadow-[0_0_24px_rgba(99,102,241,0.3)] transition-all">
                Get API Key Free →
              </Link>
              <Link href="https://docs.fetchium.com/quickstart" className="inline-flex items-center justify-center gap-2 rounded-xl border border-white/10 bg-white/3 px-6 py-3 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all">
                Migration Guide
              </Link>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
