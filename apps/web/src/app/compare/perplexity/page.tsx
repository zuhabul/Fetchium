import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { Check, X, Minus } from "lucide-react";

export const metadata: Metadata = {
  title: "Fetchium vs Perplexity API — Inference-Free Evidence vs LLM-Bundled Search (2026)",
  description:
    "Perplexity bundles LLM inference you can't control at $5+/1K plus token costs. Fetchium returns raw evidence at $0.90/1K so you choose your model. Verified pricing comparison.",
};

/**
 * Perplexity API pricing: perplexity.ai/api, March 2026.
 * Perplexity valuation: $20B (September 2025).
 * Perplexity ARR: $200M (October 2025), 4.7× YoY growth.
 * No free API tier; requires paid account.
 * Latency: 3-5s (LLM inference + search combined).
 */

const rows = [
  { feature: "Raw search results (no LLM)", f: "yes", p: "no", note: "Perplexity always runs LLM inference; you can't get raw results without it" },
  { feature: "Choose your own LLM", f: "yes", p: "no", note: "Fetchium returns evidence; you use any LLM. Perplexity bundles their Sonar models." },
  { feature: "Predictable cost per query", f: "yes", p: "no", note: "Perplexity cost = request fee + tokens (unpredictable). Fetchium: flat $0.90/1K." },
  { feature: "Full content extraction", f: "yes", p: "partial", note: "Perplexity extracts content for its LLM; you get the answer, not the source content" },
  { feature: "Token budget control", f: "yes", p: "no", note: "Perplexity controls what goes into its context; you control Fetchium's token budget" },
  { feature: "Evidence graphs + source citations", f: "yes", p: "partial", note: "Perplexity includes citations; Fetchium adds structured evidence graphs" },
  { feature: "Multi-source federation", f: "yes", p: "partial", note: "Perplexity searches the web; exact backends and federation depth are opaque" },
  { feature: "MCP protocol support", f: "yes", p: "no" },
  { feature: "Free tier", f: "yes", p: "no", note: "Perplexity API has no free tier; requires paid account. Fetchium: 1K/mo free." },
  { feature: "Response includes reasoning", f: "no", p: "yes", note: "Perplexity Sonar Reasoning models include chain-of-thought; Fetchium is retrieval-only" },
  { feature: "Search P50 latency", f: "~500ms", p: "3–5s", note: "Perplexity latency includes LLM inference. Fetchium is retrieval-only (no LLM)." },
  { feature: "Price per 1K queries", f: "$0.90", p: "$6–22+", note: "Perplexity: $5–14/1K request fee + token costs. Total varies widely." },
];

function Cell({ v }: { v: string }) {
  if (v === "yes") return <div className="flex justify-center"><div className="h-5 w-5 rounded-full bg-emerald-500/15 flex items-center justify-center"><Check className="h-3 w-3 text-emerald-400" strokeWidth={3} /></div></div>;
  if (v === "no") return <div className="flex justify-center"><X className="h-3.5 w-3.5 text-slate-700" /></div>;
  if (v === "partial") return <div className="flex justify-center"><Minus className="h-3.5 w-3.5 text-amber-500/70" /></div>;
  return <div className="flex justify-center"><span className="font-mono text-[11px] text-slate-400 bg-white/4 border border-white/8 rounded px-1.5 py-0.5">{v}</span></div>;
}

export default function ComparePerplexityPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Fetchium vs Perplexity API</span>
          </nav>

          <div className="mb-12">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Verified Comparison · March 2026
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Fetchium vs Perplexity API:{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                Retrieval vs Inference-Bundled Search
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed">
              Perplexity is a $20B company (2025) with an excellent consumer AI search product.
              Their API bundles LLM inference into every call — great if you want AI answers,
              not suitable if you want raw evidence to feed your own LLM. Fetchium is pure retrieval:
              you get the evidence, you pick the model, you control the cost.
            </p>
          </div>

          {/* Key framing */}
          <div className="mb-8 rounded-xl border border-amber-500/20 bg-amber-500/8 p-4">
            <p className="text-sm text-amber-300 font-semibold mb-1">Different product categories</p>
            <p className="text-[13px] text-slate-400 leading-relaxed">
              Perplexity API is an <strong className="text-slate-300">AI answer engine</strong> — you ask a question, it searches + reasons + answers.
              Fetchium is a <strong className="text-slate-300">retrieval API</strong> — you ask for evidence, it searches + extracts + cites.
              They solve different problems. This comparison is most useful if you&apos;re deciding whether to
              use Perplexity as your search layer in an AI application, or to use a retrieval API + your own LLM.
            </p>
          </div>

          <div className="mb-10 grid sm:grid-cols-3 gap-4">
            <div className="rounded-2xl border border-emerald-500/20 bg-emerald-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-emerald-400">10×+</div>
              <div className="text-sm text-slate-300 mt-1">cheaper retrieval</div>
              <div className="text-[12px] text-slate-500 mt-1">$0.90 vs $6–22/1K total</div>
            </div>
            <div className="rounded-2xl border border-indigo-500/20 bg-indigo-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-indigo-300">Your LLM</div>
              <div className="text-sm text-slate-300 mt-1">full model control</div>
              <div className="text-[12px] text-slate-500 mt-1">Not locked to Sonar models</div>
            </div>
            <div className="rounded-2xl border border-violet-500/20 bg-violet-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-violet-300">Zero</div>
              <div className="text-sm text-slate-300 mt-1">surprise token costs</div>
              <div className="text-[12px] text-slate-500 mt-1">Flat $0.90/1K on Starter</div>
            </div>
          </div>

          <div className="mb-10 overflow-x-auto rounded-2xl border border-white/8 shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
            <table className="w-full border-collapse min-w-[480px]">
              <thead>
                <tr className="border-b border-white/6">
                  <th className="py-4 px-5 text-left text-[12px] font-medium text-slate-600 w-64">Feature</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-indigo-300 bg-[rgba(99,102,241,0.06)] w-32">Fetchium</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-slate-500 w-32">Perplexity API</th>
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
                    <td className="py-3.5 px-4"><Cell v={row.p} /></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="mb-10 rounded-2xl border border-white/8 bg-white/[0.02] p-6">
            <h2 className="text-lg font-bold mb-4">Verified Perplexity API Pricing</h2>
            <p className="text-[13px] text-slate-400 mb-4">
              Perplexity API pricing has two components: a per-request search fee + token costs. Both vary by model and search depth. There is no free tier.
            </p>
            <div className="grid sm:grid-cols-2 gap-4 text-[13px]">
              <div>
                <p className="text-slate-500 font-medium mb-2">Request fees (per 1K):</p>
                <div className="space-y-1">
                  <div className="flex justify-between"><span className="text-slate-400">Sonar (low depth)</span><span className="text-slate-300">$5/1K</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Sonar Pro (medium)</span><span className="text-slate-300">$10/1K</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Sonar Pro Search (high)</span><span className="text-slate-300">$22/1K</span></div>
                </div>
              </div>
              <div>
                <p className="text-slate-500 font-medium mb-2">Plus token costs:</p>
                <div className="space-y-1">
                  <div className="flex justify-between"><span className="text-slate-400">Sonar input tokens</span><span className="text-slate-300">$1/M tokens</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Sonar Pro input</span><span className="text-slate-300">$3/M tokens</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Sonar Pro output</span><span className="text-slate-300">$15/M tokens</span></div>
                </div>
              </div>
            </div>
            <p className="mt-4 text-[11px] text-slate-600">Source: perplexity.ai/api, March 2026. A typical Sonar query with 2K input + 500 output tokens costs: $5/1K request + $0.002 + $0.0075 = ~$5.01/1K total minimum.</p>
          </div>

          <div className="rounded-2xl border border-indigo-500/15 bg-gradient-to-r from-indigo-500/8 to-violet-500/6 p-6 text-center">
            <p className="text-base font-semibold text-slate-200 mb-4">Try Fetchium free — 1,000 requests/month, no credit card</p>
            <div className="flex flex-col sm:flex-row gap-3 justify-center">
              <Link href="https://app.fetchium.com/register" target="_blank" rel="noopener noreferrer" className="inline-flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-6 py-3 text-sm font-semibold text-white shadow-[0_0_24px_rgba(99,102,241,0.3)] transition-all">
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
