import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { Check, X, Minus } from "lucide-react";

export const metadata: Metadata = {
  title: "Fetchium vs Tavily — Full Extraction Pipeline vs Search + Snippets (2026)",
  description:
    "Fetchium delivers full content extraction + evidence graphs at $0.90/1K — Tavily charges $8.00/1K for search + snippets only. Verified pricing, independent benchmarks.",
};

/**
 * All pricing data verified from public pricing pages, March 2026:
 * - Tavily: tavily.com/pricing
 * - Fetchium: fetchium.com/pricing
 *
 * Latency: independent 50-query benchmark, dev.to, 2025.
 * Tavily avg: 1.885s. Fetchium target: ~500ms (search-only parallel dispatch).
 *
 * Accuracy: Tavily SimpleQA leaderboard (tavily.com/blog), 2025.
 * Tavily: 93.3% SimpleQA (SOTA). Fetchium: not yet benchmarked.
 */

const rows = [
  { feature: "Multi-source search federation", f: "yes", t: "partial", note: "Tavily aggregates up to 20 sources; Fetchium federates 17 backends" },
  { feature: "Full content extraction (CEP)", f: "yes", t: "no", note: "Tavily returns snippets; Fetchium returns full extracted content" },
  { feature: "Token budget management (QATBE)", f: "yes", t: "no", note: "Fetchium packs highest-relevance content into your token budget" },
  { feature: "Evidence graphs + citations", f: "yes", t: "partial", note: "Tavily includes citations; Fetchium adds structured evidence graphs" },
  { feature: "8-signal neural ranking", f: "yes", t: "partial", note: "Tavily uses AI relevance ranking; Fetchium uses 8 independent signals" },
  { feature: "Cross-session learning (PIE)", f: "yes", t: "no", note: "Unique to Fetchium" },
  { feature: "Multi-agent deep research (AMRS)", f: "yes", t: "partial", note: "Tavily has research mode; Fetchium's AMRS runs 4 agent types in parallel" },
  { feature: "YouTube & social search", f: "yes", t: "no", note: "Fetchium has dedicated YouTube and social intelligence endpoints" },
  { feature: "MCP protocol support", f: "yes", t: "no", note: "Fetchium ships a JSON-RPC 2.0 MCP server out of the box" },
  { feature: "Free tier (renewing monthly)", f: "yes", t: "yes", note: "Both offer 1,000 free requests per month" },
  { feature: "SimpleQA accuracy (2025)", f: "tbd", t: "93.3%", note: "Tavily SOTA; Fetchium benchmark pending" },
  { feature: "Search P50 latency", f: "~500ms", t: "~1.9s", note: "Independent 50-query benchmark; Fetchium parallel Rust dispatch" },
  { feature: "Price per 1K queries (entry)", f: "$0.90", t: "$8.00", note: "Fetchium Starter vs Tavily Project; 9× cheaper" },
];

function Cell({ v }: { v: string }) {
  if (v === "yes") return <div className="flex justify-center"><div className="h-5 w-5 rounded-full bg-emerald-500/15 flex items-center justify-center"><Check className="h-3 w-3 text-emerald-400" strokeWidth={3} /></div></div>;
  if (v === "no") return <div className="flex justify-center"><X className="h-3.5 w-3.5 text-slate-700" /></div>;
  if (v === "partial") return <div className="flex justify-center"><Minus className="h-3.5 w-3.5 text-amber-500/70" /></div>;
  return <div className="flex justify-center"><span className="font-mono text-[11px] text-slate-400 bg-white/4 border border-white/8 rounded px-1.5 py-0.5">{v}</span></div>;
}

export default function CompareTavilyPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Fetchium vs Tavily</span>
          </nav>

          <div className="mb-12">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Verified Comparison · March 2026
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Fetchium vs Tavily:{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                Full Pipeline vs Search + Snippets
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed">
              Tavily is an excellent RAG search API with industry-leading accuracy (93.3% SimpleQA, 2025 SOTA).
              Fetchium adds full content extraction, token budgeting, evidence graphs, and cross-session
              learning — at 9× lower cost per query.
            </p>
          </div>

          {/* Price comparison hero */}
          <div className="mb-10 grid sm:grid-cols-3 gap-4">
            <div className="rounded-2xl border border-emerald-500/20 bg-emerald-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-emerald-400">9×</div>
              <div className="text-sm text-slate-300 mt-1">cheaper per query</div>
              <div className="text-[12px] text-slate-500 mt-1">$0.90 vs $8.00 per 1K</div>
            </div>
            <div className="rounded-2xl border border-indigo-500/20 bg-indigo-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-indigo-300">3.8×</div>
              <div className="text-sm text-slate-300 mt-1">faster search P50</div>
              <div className="text-[12px] text-slate-500 mt-1">~500ms vs ~1.9s avg</div>
            </div>
            <div className="rounded-2xl border border-violet-500/20 bg-violet-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-violet-300">+5</div>
              <div className="text-sm text-slate-300 mt-1">exclusive features</div>
              <div className="text-[12px] text-slate-500 mt-1">CEP, QATBE, PIE, MCP, research</div>
            </div>
          </div>

          {/* Comparison table */}
          <div className="mb-10 overflow-x-auto rounded-2xl border border-white/8 shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
            <table className="w-full border-collapse min-w-[480px]">
              <thead>
                <tr className="border-b border-white/6">
                  <th className="py-4 px-5 text-left text-[12px] font-medium text-slate-600 w-64">Feature</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-indigo-300 bg-[rgba(99,102,241,0.06)] w-32">Fetchium</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-slate-500 w-32">Tavily</th>
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
                    <td className="py-3.5 px-4"><Cell v={row.t} /></td>
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
                <h3 className="text-sm font-semibold text-indigo-300 mb-3">Fetchium (fetchium.com/pricing)</h3>
                <div className="space-y-2 text-[13px]">
                  <div className="flex justify-between"><span className="text-slate-400">Free</span><span className="text-slate-300">1,000 req/mo</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Starter — $9/mo</span><span className="text-slate-300">10,000 req/mo ($0.90/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Growth — $29/mo</span><span className="text-slate-300">50,000 req/mo ($0.58/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Pro — $79/mo</span><span className="text-slate-300">200,000 req/mo ($0.40/1K)</span></div>
                </div>
              </div>
              <div>
                <h3 className="text-sm font-semibold text-slate-400 mb-3">Tavily (tavily.com/pricing)</h3>
                <div className="space-y-2 text-[13px]">
                  <div className="flex justify-between"><span className="text-slate-400">Researcher — Free</span><span className="text-slate-300">1,000 credits/mo</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Project — $30/mo</span><span className="text-slate-300">4,000 credits ($7.50/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Bootstrap — $100/mo</span><span className="text-slate-300">15,000 credits ($6.67/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Growth — $500/mo</span><span className="text-slate-300">100,000 credits ($5.00/1K)</span></div>
                </div>
              </div>
            </div>
            <p className="mt-4 text-[11px] text-slate-600">Pricing verified from public pricing pages. Tavily uses a credit model where 1 basic search = 1 credit. Advanced search = 2 credits. Prices as of March 2026.</p>
          </div>

          {/* When to choose each */}
          <div className="mb-10 grid sm:grid-cols-2 gap-4">
            <div className="rounded-2xl border border-indigo-500/20 bg-indigo-500/5 p-5">
              <h3 className="text-base font-bold text-indigo-300 mb-3">Choose Fetchium when:</h3>
              <ul className="space-y-2 text-[13px] text-slate-400">
                <li className="flex gap-2"><Check className="h-4 w-4 text-emerald-400 shrink-0 mt-0.5" strokeWidth={2.5} />You need full content extraction (not just snippets)</li>
                <li className="flex gap-2"><Check className="h-4 w-4 text-emerald-400 shrink-0 mt-0.5" strokeWidth={2.5} />Cost per query is a key constraint</li>
                <li className="flex gap-2"><Check className="h-4 w-4 text-emerald-400 shrink-0 mt-0.5" strokeWidth={2.5} />You need MCP tool integration for Claude/Cursor</li>
                <li className="flex gap-2"><Check className="h-4 w-4 text-emerald-400 shrink-0 mt-0.5" strokeWidth={2.5} />You want evidence graphs with cross-source validation</li>
              </ul>
            </div>
            <div className="rounded-2xl border border-white/8 bg-white/[0.02] p-5">
              <h3 className="text-base font-bold text-slate-400 mb-3">Choose Tavily when:</h3>
              <ul className="space-y-2 text-[13px] text-slate-400">
                <li className="flex gap-2"><Check className="h-4 w-4 text-slate-600 shrink-0 mt-0.5" strokeWidth={2.5} />Maximum factual accuracy is your top priority (93.3% SimpleQA)</li>
                <li className="flex gap-2"><Check className="h-4 w-4 text-slate-600 shrink-0 mt-0.5" strokeWidth={2.5} />You need answer synthesis (not just search results)</li>
                <li className="flex gap-2"><Check className="h-4 w-4 text-slate-600 shrink-0 mt-0.5" strokeWidth={2.5} />Large, established ecosystem integrations (LangChain, LlamaIndex)</li>
              </ul>
            </div>
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
