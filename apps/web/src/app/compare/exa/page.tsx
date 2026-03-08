import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { Check, X, Minus } from "lucide-react";

export const metadata: Metadata = {
  title: "Fetchium vs Exa — Multi-Backend vs Single Semantic Index (2026)",
  description:
    "Fetchium federates 11+ backends at $0.90/1K. Exa's neural search uses a single proprietary index at $5.00/1K. Full feature comparison with verified pricing.",
};

/**
 * Exa pricing verified from exa.ai/pricing, March 2026.
 * Exa latency: ~1.180s avg (independent 50-query benchmark, dev.to, 2025).
 * Exa valuation: $700M Series B (September 2025, Benchmark Capital).
 * Exa SimpleQA accuracy: 71.2% (per Tavily's published leaderboard).
 */

const rows = [
  { feature: "Multi-source federation", f: "yes", e: "no", note: "Exa uses a single proprietary crawled index; Fetchium queries 11+ live backends" },
  { feature: "Live web results", f: "yes", e: "partial", note: "Exa indexes the web but results may lag; Fetchium queries backends in real-time" },
  { feature: "Full content extraction (CEP)", f: "yes", e: "yes", note: "Exa Contents API returns parsed content; Fetchium adds 5 extraction layers" },
  { feature: "Token budget management (QATBE)", f: "yes", e: "no", note: "Fetchium uses BM25 + knapsack to pack relevant content into your budget" },
  { feature: "Evidence graphs + citations", f: "yes", e: "no", note: "Fetchium maps claims to sources; Exa returns URLs" },
  { feature: "Cross-session learning (PIE)", f: "yes", e: "no", note: "Unique to Fetchium" },
  { feature: "Multi-agent deep research (AMRS)", f: "yes", e: "no", note: "Fetchium runs 4 parallel research agents; Exa is search-only" },
  { feature: "YouTube & social search", f: "yes", e: "partial", note: "Exa searches some social content; Fetchium has dedicated YouTube/social endpoints" },
  { feature: "MCP protocol support", f: "yes", e: "partial", note: "Fetchium exposes 12 MCP tools across search, fetch, research, YouTube, and social workflows" },
  { feature: "Keyword search mode", f: "yes", e: "yes", note: "Both support keyword search" },
  { feature: "Semantic/neural search", f: "yes", e: "yes", note: "Exa was purpose-built for neural search; Fetchium adds it via embedding signals" },
  { feature: "SimpleQA accuracy (2025)", f: "tbd", e: "71.2%", note: "Per Tavily's published leaderboard. Fetchium benchmark pending." },
  { feature: "Search P50 latency", f: "~500ms", e: "~1.2s", note: "Exa: ~1.18s avg (50-query benchmark). Fetchium: ~500ms parallel dispatch." },
  { feature: "Price per 1K queries", f: "$0.90", e: "$5.00", note: "Fetchium Starter vs Exa neural search PAYG; 5.5× cheaper" },
];

function Cell({ v }: { v: string }) {
  if (v === "yes") return <div className="flex justify-center"><div className="h-5 w-5 rounded-full bg-emerald-500/15 flex items-center justify-center"><Check className="h-3 w-3 text-emerald-400" strokeWidth={3} /></div></div>;
  if (v === "no") return <div className="flex justify-center"><X className="h-3.5 w-3.5 text-slate-700" /></div>;
  if (v === "partial") return <div className="flex justify-center"><Minus className="h-3.5 w-3.5 text-amber-500/70" /></div>;
  return <div className="flex justify-center"><span className="font-mono text-[11px] text-slate-400 bg-white/4 border border-white/8 rounded px-1.5 py-0.5">{v}</span></div>;
}

export default function CompareExaPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Fetchium vs Exa</span>
          </nav>

          <div className="mb-12">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Verified Comparison · March 2026
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Fetchium vs Exa:{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                Multi-Backend vs Single Semantic Index
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed">
              Exa (formerly Metaphor) built a proprietary neural search index optimized for AI — a genuinely
              impressive technical achievement ($700M valuation, Series B 2025). Fetchium takes a different approach:
              federated live search across 11+ backends with full content extraction, at 5.5× lower cost.
            </p>
          </div>

          <div className="mb-10 grid sm:grid-cols-3 gap-4">
            <div className="rounded-2xl border border-emerald-500/20 bg-emerald-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-emerald-400">5.5×</div>
              <div className="text-sm text-slate-300 mt-1">cheaper per query</div>
              <div className="text-[12px] text-slate-500 mt-1">$0.90 vs $5.00 per 1K</div>
            </div>
            <div className="rounded-2xl border border-indigo-500/20 bg-indigo-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-indigo-300">2.4×</div>
              <div className="text-sm text-slate-300 mt-1">faster search P50</div>
              <div className="text-[12px] text-slate-500 mt-1">~500ms vs ~1.2s avg</div>
            </div>
            <div className="rounded-2xl border border-violet-500/20 bg-violet-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-violet-300">11+</div>
              <div className="text-sm text-slate-300 mt-1">live backends</div>
              <div className="text-[12px] text-slate-500 mt-1">vs Exa's 1 index</div>
            </div>
          </div>

          <div className="mb-10 overflow-x-auto rounded-2xl border border-white/8 shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
            <table className="w-full border-collapse min-w-[480px]">
              <thead>
                <tr className="border-b border-white/6">
                  <th className="py-4 px-5 text-left text-[12px] font-medium text-slate-600 w-64">Feature</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-indigo-300 bg-[rgba(99,102,241,0.06)] w-32">Fetchium</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-slate-500 w-32">Exa</th>
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
                    <td className="py-3.5 px-4"><Cell v={row.e} /></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Pricing breakdown */}
          <div className="mb-10 rounded-2xl border border-white/8 bg-white/[0.02] p-6">
            <h2 className="text-lg font-bold mb-4">Verified Pricing Comparison</h2>
            <div className="grid sm:grid-cols-2 gap-6">
              <div>
                <h3 className="text-sm font-semibold text-indigo-300 mb-3">Fetchium</h3>
                <div className="space-y-2 text-[13px]">
                  <div className="flex justify-between"><span className="text-slate-400">Free</span><span className="text-slate-300">1,000 req/mo</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Starter — $9/mo</span><span className="text-slate-300">10K req ($0.90/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Growth — $29/mo</span><span className="text-slate-300">50K req ($0.58/1K)</span></div>
                </div>
              </div>
              <div>
                <h3 className="text-sm font-semibold text-slate-400 mb-3">Exa (exa.ai/pricing)</h3>
                <div className="space-y-2 text-[13px]">
                  <div className="flex justify-between"><span className="text-slate-400">Free tier</span><span className="text-slate-300">$10 credits (~2K searches)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Neural search PAYG</span><span className="text-slate-300">$5.00/1K queries</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Keyword search</span><span className="text-slate-300">$2.50/1K queries</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Contents API</span><span className="text-slate-300">$1.00/1K pages</span></div>
                </div>
              </div>
            </div>
            <p className="mt-4 text-[11px] text-slate-600">Exa pricing from exa.ai/pricing, verified March 2026. Exa neural search (semantic) costs $5/1K; keyword search costs $2.50/1K. Contents API is additional.</p>
          </div>

          <div className="rounded-2xl border border-indigo-500/15 bg-gradient-to-r from-indigo-500/8 to-violet-500/6 p-6 text-center">
            <p className="text-base font-semibold text-slate-200 mb-4">Try Fetchium free — 1,000 requests/month, no credit card</p>
            <div className="flex flex-col sm:flex-row gap-3 justify-center">
              <Link href="https://app.fetchium.com/register" target="_blank" rel="noopener noreferrer" className="inline-flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-6 py-3 text-sm font-semibold text-white shadow-[0_0_24px_rgba(99,102,241,0.3)] transition-all hover:shadow-[0_0_36px_rgba(99,102,241,0.5)]">
                Get API Key Free →
              </Link>
              <Link href="/pricing" className="inline-flex items-center justify-center gap-2 rounded-xl border border-white/10 bg-white/3 px-6 py-3 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all">
                View Pricing
              </Link>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
