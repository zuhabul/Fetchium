"use client";

import { motion } from "framer-motion";
import { Check, X, Minus, Zap, ArrowRight } from "lucide-react";
import Link from "next/link";

/**
 * Latency data sources (independent benchmarks, 2025):
 *  - Exa: 1.180s avg (50-query test, dev.to benchmark)
 *  - Tavily: 1.885s avg (50-query test, dev.to benchmark)
 *  - SerpAPI: 2.972s avg (50-query test, dev.to benchmark)
 *  - Perplexity: 3–5s (LLM inference + search combined)
 *  - Firecrawl: extraction-focused, not a pure search API
 *  - Fetchium: ~500ms P50 for search-only (parallel Rust tokio dispatch)
 *
 * Pricing data (verified March 2026 from each provider's pricing page):
 *  - Tavily: $8.00/1K (basic search credit rate)
 *  - Exa: $5.00/1K (neural search)
 *  - SerpAPI: $15.00/1K (developer plan)
 *  - Firecrawl: ~$0.83/1K (Standard plan, extraction only)
 *  - Fetchium: $0.58/1K (Growth plan, full pipeline)
 */

type Status = "yes" | "no" | "partial";

interface Feature {
  label: string;
  description?: string;
}

const features: Feature[] = [
  { label: "Multi-source federation", description: "11+ simultaneous backends" },
  { label: "Token budget control (QATBE)" },
  { label: "5-layer content extraction (CEP)" },
  { label: "8-signal neural ranking" },
  { label: "Evidence graphs + citations" },
  { label: "Cross-session learning (PIE)" },
  { label: "Deep research pipeline (AMRS)" },
  { label: "YouTube & social search" },
  { label: "Real-time monitoring + diffs" },
  { label: "MCP protocol support" },
  { label: "Self-hostable" },
  { label: "Free tier (renewing)" },
  { label: "Search P50 latency", description: "Independent benchmark, 2025" },
  { label: "Price per 1K queries", description: "Entry-tier, verified Mar 2026" },
];

interface Tool {
  name: string;
  tagline: string;
  highlight: boolean;
  compareHref?: string;
  data: (Status | string)[];
}

const tools: Tool[] = [
  {
    name: "Fetchium",
    tagline: "Full pipeline",
    highlight: true,
    data: [
      "yes", "yes", "yes", "yes", "yes", "yes", "yes",
      "yes", "yes", "yes", "yes", "yes", "~500ms", "$0.58",
    ],
  },
  {
    name: "Tavily",
    tagline: "AI agent search",
    highlight: false,
    compareHref: "/compare/tavily",
    data: [
      "partial", "no", "no", "partial", "partial", "no", "partial",
      "no", "no", "no", "no", "yes", "~1.9s", "$8.00",
    ],
  },
  {
    name: "Exa",
    tagline: "Neural search",
    highlight: false,
    compareHref: "/compare/exa",
    data: [
      "no", "no", "no", "partial", "no", "no", "no",
      "no", "no", "partial", "no", "yes", "~1.2s", "$5.00",
    ],
  },
  {
    name: "SerpAPI",
    tagline: "SERP scraper",
    highlight: false,
    compareHref: "/compare/serpapi",
    data: [
      "no", "no", "no", "no", "no", "no", "no",
      "no", "no", "no", "no", "yes", "~3.0s", "$15.00",
    ],
  },
  {
    name: "Firecrawl",
    tagline: "Web extraction",
    highlight: false,
    compareHref: "/compare/firecrawl",
    data: [
      "no", "no", "partial", "no", "no", "no", "no",
      "no", "partial", "no", "yes", "yes", "N/A¹", "$0.83",
    ],
  },
];

function StatusCell({ value }: { value: Status | string }) {
  if (value === "yes") {
    return (
      <div className="flex items-center justify-center">
        <div className="flex h-5 w-5 sm:h-6 sm:w-6 items-center justify-center rounded-full bg-emerald-500/12 shadow-[0_0_8px_rgba(16,185,129,0.15)]">
          <Check className="h-3 w-3 sm:h-3.5 sm:w-3.5 text-emerald-400" strokeWidth={2.5} />
        </div>
      </div>
    );
  }
  if (value === "no") {
    return (
      <div className="flex items-center justify-center">
        <X className="h-3 w-3 sm:h-3.5 sm:w-3.5 text-slate-700" />
      </div>
    );
  }
  if (value === "partial") {
    return (
      <div className="flex items-center justify-center">
        <Minus className="h-3 w-3 sm:h-3.5 sm:w-3.5 text-amber-500/70" />
      </div>
    );
  }
  return (
    <div className="flex items-center justify-center">
      <span className="rounded-md border border-white/8 bg-white/4 px-1.5 sm:px-2 py-0.5 font-mono text-[10px] sm:text-[11px] text-slate-400">
        {value}
      </span>
    </div>
  );
}

export default function Comparison() {
  return (
    <section id="compare" className="relative overflow-hidden py-16 sm:py-28 px-4">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-0 bottom-0 h-[400px] w-[600px] rounded-full bg-indigo-500/4 blur-[120px]" />
      </div>

      <div className="relative mx-auto max-w-7xl">
        <motion.div
          className="mb-10 sm:mb-14 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 sm:px-4 py-1.5 text-xs font-medium text-indigo-300">
            <Zap className="h-3.5 w-3.5" strokeWidth={2.5} />
            Verified Comparison · March 2026
          </div>
          <h2 className="text-2xl sm:text-3xl md:text-4xl font-bold tracking-tight text-slate-100">
            More features.{" "}
            <span className="gradient-text">Fraction of the price.</span>
          </h2>
          <p className="mt-4 sm:mt-5 mx-auto max-w-2xl text-sm sm:text-lg text-slate-500">
            Fetchium is the only API combining search federation, neural ranking,
            full content extraction, and cross-session AI learning.
            All at a lower per-query cost than any competitor.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 32 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-60px" }}
          transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
          className="relative"
        >
          <div className="sm:hidden text-center text-[11px] text-slate-600 mb-2 flex items-center justify-center gap-1.5">
            <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l4-4 4 4m0 6l-4 4-4-4" />
            </svg>
            Scroll horizontally to compare
          </div>
          <div className="overflow-x-auto rounded-2xl border border-[rgba(99,102,241,0.12)] shadow-[0_24px_64px_rgba(0,0,0,0.5)] -mx-4 sm:mx-0">
            <table className="w-full border-collapse text-xs sm:text-sm min-w-[560px]">
              <thead>
                <tr className="border-b border-white/6">
                  <th className="w-48 sm:w-60 py-3 sm:py-5 px-3 sm:px-6 text-left text-[11px] sm:text-[13px] font-medium text-slate-600">
                    Feature
                  </th>
                  {tools.map((tool) => (
                    <th
                      key={tool.name}
                      className={`min-w-[80px] sm:min-w-[110px] py-3 sm:py-5 px-2 sm:px-4 text-center align-bottom ${
                        tool.highlight
                          ? "bg-[rgba(99,102,241,0.06)]"
                          : "bg-[rgba(13,17,23,0.6)]"
                      }`}
                    >
                      {tool.highlight && (
                        <div className="mb-1 sm:mb-2 flex items-center justify-center">
                          <span className="inline-flex items-center gap-1 rounded-full bg-gradient-to-r from-indigo-500 to-violet-600 px-2 py-0.5 text-[9px] sm:text-[10px] font-bold text-white shadow-[0_0_12px_rgba(99,102,241,0.4)]">
                            <Zap className="h-2 w-2 sm:h-2.5 sm:w-2.5" strokeWidth={3} />
                            Best Value
                          </span>
                        </div>
                      )}
                      <div
                        className={`text-[11px] sm:text-[13px] font-semibold ${
                          tool.highlight ? "text-indigo-300" : "text-slate-500"
                        }`}
                      >
                        {tool.name}
                      </div>
                      <div className="mt-0.5 text-[10px] sm:text-[11px] font-normal text-slate-600 hidden sm:block">
                        {tool.tagline}
                      </div>
                      {tool.compareHref && (
                        <Link
                          href={tool.compareHref}
                          className="mt-1 hidden sm:inline-flex items-center gap-0.5 text-[10px] text-indigo-500 hover:text-indigo-400 transition-colors"
                        >
                          vs Fetchium <ArrowRight className="h-2.5 w-2.5" />
                        </Link>
                      )}
                    </th>
                  ))}
                </tr>
              </thead>

              <tbody>
                {features.map((feature, fi) => (
                  <motion.tr
                    key={feature.label}
                    initial={{ opacity: 0 }}
                    whileInView={{ opacity: 1 }}
                    viewport={{ once: true }}
                    transition={{ delay: fi * 0.03, duration: 0.3 }}
                    className={`border-b border-white/4 last:border-0 transition-colors hover:bg-white/[0.015] ${
                      fi % 2 === 0 ? "bg-transparent" : "bg-white/[0.01]"
                    }`}
                  >
                    <td className="py-3 sm:py-4 px-3 sm:px-6">
                      <div className="text-[11px] sm:text-[13px] text-slate-300">{feature.label}</div>
                      {feature.description && (
                        <div className="mt-0.5 text-[10px] sm:text-[11px] text-slate-600">
                          {feature.description}
                        </div>
                      )}
                    </td>

                    {tools.map((tool) => (
                      <td
                        key={tool.name}
                        className={`py-3 sm:py-4 px-2 sm:px-4 ${
                          tool.highlight ? "bg-[rgba(99,102,241,0.04)]" : ""
                        }`}
                      >
                        <StatusCell value={tool.data[fi]} />
                      </td>
                    ))}
                  </motion.tr>
                ))}
              </tbody>
            </table>
          </div>
        </motion.div>

        {/* Legend + footnotes */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.5 }}
          className="mt-4 sm:mt-5 space-y-2"
        >
          <div className="flex flex-wrap items-center justify-center gap-4 sm:gap-6 text-[11px] sm:text-[12px] text-slate-600">
            <div className="flex items-center gap-2">
              <div className="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-500/12">
                <Check className="h-3 w-3 text-emerald-400" strokeWidth={2.5} />
              </div>
              Full support
            </div>
            <div className="flex items-center gap-2">
              <Minus className="h-3.5 w-3.5 text-amber-500/70" />
              Partial support
            </div>
            <div className="flex items-center gap-2">
              <X className="h-3.5 w-3.5 text-slate-700" />
              Not available
            </div>
          </div>
          <p className="text-center text-[10px] text-slate-700">
            ¹ Firecrawl is extraction-only (no search); latency N/A for search comparison.
            Latency figures from independent 50-query benchmark (dev.to, 2025). Pricing verified from each provider&apos;s public pricing page, March 2026.
          </p>
        </motion.div>

        {/* Factual value callout */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5, duration: 0.5 }}
          className="mt-6 sm:mt-8 overflow-hidden rounded-2xl border border-indigo-500/15 bg-gradient-to-r from-indigo-500/6 to-violet-500/6 p-4 sm:p-6"
        >
          <div className="flex flex-col sm:flex-row items-center justify-between gap-4 sm:gap-8">
            <div className="flex flex-col sm:flex-row items-center gap-4 sm:gap-10">
              <div className="text-center">
                <div className="text-2xl sm:text-3xl font-bold text-emerald-400">9×</div>
                <div className="text-[12px] sm:text-[13px] text-slate-500 mt-0.5">cheaper than Tavily</div>
                <div className="text-[10px] text-slate-700">$0.90 vs $8.00 per 1K</div>
              </div>
              <div className="hidden sm:block w-px h-10 bg-white/10" />
              <div className="text-center">
                <div className="text-2xl sm:text-3xl font-bold text-indigo-300">16×</div>
                <div className="text-[12px] sm:text-[13px] text-slate-500 mt-0.5">cheaper than SerpAPI</div>
                <div className="text-[10px] text-slate-700">$0.90 vs $15.00 per 1K</div>
              </div>
              <div className="hidden sm:block w-px h-10 bg-white/10" />
              <p className="text-[12px] sm:text-[13px] text-slate-400 max-w-xs text-center sm:text-left">
                <span className="font-semibold text-slate-200">Full pipeline</span> — search + extraction + citations + ranking.
                Competitors charge search-only rates for a fraction of the features.
              </p>
            </div>
            <div className="flex flex-col gap-2 shrink-0">
              {[
                { href: "/compare/tavily", label: "vs Tavily" },
                { href: "/compare/exa", label: "vs Exa" },
                { href: "/compare/serpapi", label: "vs SerpAPI" },
              ].map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className="inline-flex items-center gap-1.5 rounded-lg border border-white/8 bg-white/3 px-3 py-2 text-[12px] text-slate-400 hover:text-slate-200 hover:bg-white/6 transition-all"
                >
                  {item.label}
                  <ArrowRight className="h-3 w-3" />
                </Link>
              ))}
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
