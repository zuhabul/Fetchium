import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { Check, X, Minus } from "lucide-react";

export const metadata: Metadata = {
  title: "Fetchium vs Firecrawl — Full Pipeline vs Extraction-Only (2026)",
  description:
    "Firecrawl Standard: $83/mo for 100K page extractions. Fetchium Growth: $29/mo for 50K full-pipeline requests including search + extraction + citations. Verified pricing.",
};

/**
 * Firecrawl pricing verified from firecrawl.dev/pricing, March 2026.
 * Firecrawl raised $16.2M Series A (August 2025). 350K+ users. Profitable.
 * Firecrawl Standard: $83/mo for 100K credits (extraction-focused).
 * Firecrawl is NOT a search API — it's an extraction/crawling API.
 * This comparison is most relevant for developers choosing between
 * "give it a URL" (Firecrawl) vs "give it a query" (Fetchium).
 */

const rows = [
  { feature: "Multi-backend web search", f: "yes", fc: "no", note: "Fetchium searches; Firecrawl scrapes URLs you provide" },
  { feature: "Full content extraction", f: "yes", fc: "yes", note: "Both extract clean content; Fetchium uses 5-layer CEP, Firecrawl uses Fire-Engine" },
  { feature: "Autonomous web crawling", f: "no", fc: "yes", note: "Firecrawl can crawl entire sites; Fetchium focuses on query-driven retrieval" },
  { feature: "JavaScript rendering", f: "yes", fc: "yes", note: "Both support headless JS rendering" },
  { feature: "Token budget management", f: "yes", fc: "no", note: "Fetchium QATBE packs relevant content into your token budget" },
  { feature: "Evidence graphs + citations", f: "yes", fc: "no", note: "Unique to Fetchium" },
  { feature: "Neural ranking (8-signal)", f: "yes", fc: "no", note: "Fetchium HyperFusion ranks results; Firecrawl extracts in order" },
  { feature: "Multi-agent deep research", f: "yes", fc: "no" },
  { feature: "Cross-session learning", f: "yes", fc: "no" },
  { feature: "YouTube & social search", f: "yes", fc: "no" },
  { feature: "MCP protocol support", f: "yes", fc: "no" },
  { feature: "Open source", f: "partial", fc: "yes", note: "Firecrawl core is Apache-2.0 open source; Fetchium CLI and core are source-available" },
  { feature: "Free tier (renewing)", f: "yes", fc: "partial", note: "Fetchium: 1K/mo forever. Firecrawl: 500 one-time credits (not renewing)." },
  { feature: "Price per 1K operations", f: "$0.58", fc: "$0.83", note: "Fetchium Growth ($29/mo, 50K) vs Firecrawl Standard ($83/mo, 100K pages)" },
];

function Cell({ v }: { v: string }) {
  if (v === "yes") return <div className="flex justify-center"><div className="h-5 w-5 rounded-full bg-emerald-500/15 flex items-center justify-center"><Check className="h-3 w-3 text-emerald-400" strokeWidth={3} /></div></div>;
  if (v === "no") return <div className="flex justify-center"><X className="h-3.5 w-3.5 text-slate-700" /></div>;
  if (v === "partial") return <div className="flex justify-center"><Minus className="h-3.5 w-3.5 text-amber-500/70" /></div>;
  return <div className="flex justify-center"><span className="font-mono text-[11px] text-slate-400 bg-white/4 border border-white/8 rounded px-1.5 py-0.5">{v}</span></div>;
}

export default function CompareFirecrawlPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Fetchium vs Firecrawl</span>
          </nav>

          <div className="mb-12">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Verified Comparison · March 2026
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Fetchium vs Firecrawl:{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                Full Pipeline vs Extraction-Only
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed">
              Firecrawl is an excellent web extraction tool — 350K+ users, profitable, open-source.
              But it&apos;s an extraction tool: you give it URLs, it gives you clean content.
              Fetchium starts one step earlier — you give it a query, it finds the URLs,
              extracts the content, ranks it, token-budgets it, and returns structured citations.
            </p>
          </div>

          {/* Key distinction */}
          <div className="mb-8 rounded-xl border border-blue-500/20 bg-blue-500/8 p-4">
            <p className="text-sm text-blue-300 font-semibold mb-1">Different product categories</p>
            <p className="text-[13px] text-slate-400 leading-relaxed">
              <strong className="text-slate-300">Firecrawl</strong>: You provide URLs → get clean markdown. Best for crawling known sites, building knowledge bases, structured data extraction.
              <br />
              <strong className="text-slate-300">Fetchium</strong>: You provide a query → get ranked, extracted, cited results from 11+ backends. Best for RAG pipelines, AI agents, research workflows.
            </p>
          </div>

          <div className="mb-10 grid sm:grid-cols-3 gap-4">
            <div className="rounded-2xl border border-emerald-500/20 bg-emerald-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-emerald-400">30%</div>
              <div className="text-sm text-slate-300 mt-1">cheaper per 1K ops</div>
              <div className="text-[12px] text-slate-500 mt-1">$0.58 vs $0.83 at scale</div>
            </div>
            <div className="rounded-2xl border border-indigo-500/20 bg-indigo-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-indigo-300">+8</div>
              <div className="text-sm text-slate-300 mt-1">exclusive features</div>
              <div className="text-[12px] text-slate-500 mt-1">Search, ranking, research, MCP...</div>
            </div>
            <div className="rounded-2xl border border-violet-500/20 bg-violet-500/8 p-5 text-center">
              <div className="text-3xl font-bold text-violet-300">1K</div>
              <div className="text-sm text-slate-300 mt-1">free/mo vs 500 one-time</div>
              <div className="text-[12px] text-slate-500 mt-1">Renewing vs one-time credits</div>
            </div>
          </div>

          <div className="mb-10 overflow-x-auto rounded-2xl border border-white/8 shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
            <table className="w-full border-collapse min-w-[480px]">
              <thead>
                <tr className="border-b border-white/6">
                  <th className="py-4 px-5 text-left text-[12px] font-medium text-slate-600 w-64">Feature</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-indigo-300 bg-[rgba(99,102,241,0.06)] w-32">Fetchium</th>
                  <th className="py-4 px-4 text-center text-[13px] font-semibold text-slate-500 w-32">Firecrawl</th>
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
                    <td className="py-3.5 px-4"><Cell v={row.fc} /></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Pricing */}
          <div className="mb-10 rounded-2xl border border-white/8 bg-white/[0.02] p-6">
            <h2 className="text-lg font-bold mb-4">Verified Pricing (March 2026)</h2>
            <div className="grid sm:grid-cols-2 gap-6">
              <div>
                <h3 className="text-sm font-semibold text-indigo-300 mb-3">Fetchium</h3>
                <div className="space-y-2 text-[13px]">
                  <div className="flex justify-between"><span className="text-slate-400">Free</span><span className="text-slate-300">1,000 req/mo (renewing)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Starter — $9/mo</span><span className="text-slate-300">10K req ($0.90/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Growth — $29/mo</span><span className="text-slate-300">50K req ($0.58/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Pro — $79/mo</span><span className="text-slate-300">200K req ($0.40/1K)</span></div>
                </div>
              </div>
              <div>
                <h3 className="text-sm font-semibold text-slate-400 mb-3">Firecrawl (firecrawl.dev/pricing)</h3>
                <div className="space-y-2 text-[13px]">
                  <div className="flex justify-between"><span className="text-slate-400">Free</span><span className="text-slate-300">500 credits (one-time)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Hobby — $16/mo</span><span className="text-slate-300">3K credits ($5.33/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Standard — $83/mo</span><span className="text-slate-300">100K credits ($0.83/1K)</span></div>
                  <div className="flex justify-between"><span className="text-slate-400">Growth — $333/mo</span><span className="text-slate-300">500K credits ($0.67/1K)</span></div>
                </div>
              </div>
            </div>
            <p className="mt-4 text-[11px] text-slate-600">Firecrawl pricing from firecrawl.dev/pricing. 1 credit = 1 page scrape (basic). JavaScript rendering, PDF parsing, and LLM extraction cost additional credits. Verified March 2026.</p>
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
