"use client";

import { motion } from "framer-motion";
import { Check, X, Minus, Zap, ArrowRight } from "lucide-react";
import Link from "next/link";

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
  { label: "Plan-based rate limits", description: "From current API auth configuration" },
  { label: "Free tier available", description: "1,000 requests/month in current API auth configuration" },
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
      "yes", "yes", "yes", "yes", "yes", "60-2000/min", "yes",
    ],
  },
  {
    name: "Tavily",
    tagline: "AI agent search",
    highlight: false,
    compareHref: "/compare/tavily",
    data: [
      "partial", "no", "no", "partial", "partial", "no", "partial",
      "no", "no", "no", "no", "yes", "varies", "yes",
    ],
  },
  {
    name: "Exa",
    tagline: "Neural search",
    highlight: false,
    compareHref: "/compare/exa",
    data: [
      "no", "no", "no", "partial", "no", "no", "no",
      "no", "no", "partial", "no", "yes", "varies", "yes",
    ],
  },
  {
    name: "SerpAPI",
    tagline: "SERP scraper",
    highlight: false,
    compareHref: "/compare/serpapi",
    data: [
      "no", "no", "no", "no", "no", "no", "no",
      "no", "no", "no", "no", "yes", "varies", "yes",
    ],
  },
  {
    name: "Firecrawl",
    tagline: "Web extraction",
    highlight: false,
    compareHref: "/compare/firecrawl",
    data: [
      "no", "no", "partial", "no", "no", "no", "no",
      "no", "partial", "no", "yes", "yes", "varies", "yes",
    ],
  },
];

function StatusCell({ value }: { value: Status | string }) {
  if (value === "yes") {
    return (
      <div className="flex items-center justify-center">
        <div className="flex h-6 w-6 sm:h-7 sm:w-7 items-center justify-center rounded-full bg-emerald-500/15 shadow-[0_0_10px_rgba(16,185,129,0.2)]">
          <Check className="h-4 w-4 sm:h-4.5 sm:w-4.5 text-emerald-400" strokeWidth={2.5} />
        </div>
      </div>
    );
  }
  if (value === "no") {
    return (
      <div className="flex items-center justify-center">
        <X className="h-4 w-4 sm:h-5 sm:w-5 text-slate-500" />
      </div>
    );
  }
  if (value === "partial") {
    return (
      <div className="flex items-center justify-center">
        <Minus className="h-4 w-4 sm:h-5 sm:w-5 text-amber-500/70" />
      </div>
    );
  }
  return (
    <div className="flex items-center justify-center">
      <span className="rounded-md border border-slate-700 bg-slate-900/60 px-2 sm:px-2.5 py-0.5 font-mono text-[12px] sm:text-[13px] text-slate-300">
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
          <div className="mb-5 inline-flex items-center gap-2 rounded-full border border-indigo-500/30 bg-indigo-500/10 px-4 py-2 text-sm font-semibold text-indigo-200">
            <Zap className="h-4 w-4" strokeWidth={2.5} />
            Capability Comparison
          </div>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight text-slate-100">
            Fetchium capability{" "}
            <span className="gradient-text">shape at a glance.</span>
          </h2>
          <p className="mt-5 sm:mt-6 mx-auto max-w-2xl text-base sm:text-xl text-slate-300 leading-relaxed">
            This view focuses on first-party Fetchium capabilities and broad product-shape
            differences. It intentionally avoids hard benchmark and pricing claims for
            third-party services that can change independently of this repo.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 32 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-60px" }}
          transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
          className="relative"
        >
          <div className="sm:hidden text-center text-[13px] text-slate-300 mb-3 flex items-center justify-center gap-1.5">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l4-4 4 4m0 6l-4 4-4-4" />
            </svg>
            Scroll horizontally to compare
          </div>
          <div className="overflow-x-auto rounded-2xl border border-slate-800 shadow-[0_24px_80px_rgba(0,0,0,0.6)] -mx-4 sm:mx-0">
            <table className="w-full border-collapse text-sm sm:text-base min-w-[560px]">
              <thead>
                <tr className="border-b border-slate-800">
                  <th className="w-48 sm:w-60 py-3 sm:py-5 px-3 sm:px-6 text-left text-[13px] sm:text-[15px] font-semibold text-slate-300">
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
                        <div className="mb-2 flex items-center justify-center">
                          <span className="inline-flex items-center gap-1 rounded-full bg-gradient-to-r from-indigo-500 to-violet-600 px-2.5 py-1 text-[10px] sm:text-[11px] font-bold text-white shadow-[0_0_12px_rgba(99,102,241,0.4)]">
                            <Zap className="h-2.5 w-2.5 sm:h-3 sm:w-3" strokeWidth={3} />
                            Best Value
                          </span>
                        </div>
                      )}
                      <div
                        className={`text-[13px] sm:text-[15px] font-bold ${
                          tool.highlight ? "text-indigo-300" : "text-slate-200"
                        }`}
                      >
                        {tool.name}
                      </div>
                      <div className="mt-1 text-[11px] sm:text-[13px] font-medium text-slate-400 hidden sm:block">
                        {tool.tagline}
                      </div>
                      {tool.compareHref && (
                        <Link
                          href={tool.compareHref}
                          className="mt-1.5 hidden sm:inline-flex items-center gap-1 text-[11px] sm:text-[12px] font-semibold text-indigo-400 hover:text-indigo-300 transition-colors"
                        >
                          vs Fetchium <ArrowRight className="h-3 w-3" />
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
                    className={`border-b border-slate-800 last:border-0 transition-colors hover:bg-slate-900/40 ${
                      fi % 2 === 0 ? "bg-transparent" : "bg-slate-900/20"
                    }`}
                  >
                    <td className="py-3 sm:py-4 px-3 sm:px-6">
                      <div className="text-[13px] sm:text-[15px] font-semibold text-slate-200">{feature.label}</div>
                      {feature.description && (
                        <div className="mt-1 text-[12px] sm:text-[13px] text-slate-400">
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
          <div className="flex flex-wrap items-center justify-center gap-4 sm:gap-6 text-[13px] sm:text-[14px] text-slate-300">
            <div className="flex items-center gap-2">
              <div className="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-500/12">
                <Check className="h-3.5 w-3.5 text-emerald-400" strokeWidth={2.5} />
              </div>
              Full support
            </div>
            <div className="flex items-center gap-2">
              <Minus className="h-4 w-4 text-amber-500/70" />
              Partial support
            </div>
            <div className="flex items-center gap-2">
              <X className="h-4 w-4 text-slate-500" />
              Not available
            </div>
          </div>
          <p className="text-center text-[12px] sm:text-[13px] text-slate-400 leading-relaxed">
            Fetchium values in this table are tied to the current codebase and auth
            configuration. Non-Fetchium entries are shown as broad capability comparisons only.
          </p>
        </motion.div>

        {/* Fetchium callout */}
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
                <div className="text-3xl sm:text-4xl font-bold text-emerald-400">11+</div>
                <div className="text-[14px] sm:text-[15px] text-slate-300 mt-1 font-medium">federated backends</div>
                <div className="text-[12px] sm:text-[13px] text-slate-400">single-query fanout</div>
              </div>
              <div className="hidden sm:block w-px h-10 bg-slate-800" />
              <div className="text-center">
                <div className="text-3xl sm:text-4xl font-bold text-indigo-300">17</div>
                <div className="text-[14px] sm:text-[15px] text-slate-300 mt-1 font-medium">algorithms</div>
                <div className="text-[12px] sm:text-[13px] text-slate-400">ranking, extraction, validation</div>
              </div>
              <div className="hidden sm:block w-px h-10 bg-slate-800" />
              <p className="text-[14px] sm:text-[15px] text-slate-300 max-w-xs text-center sm:text-left leading-relaxed">
                <span className="font-bold text-slate-100">Full pipeline</span> means search,
                extraction, ranking, citations, and research workflows in one product surface.
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
                  className="inline-flex items-center gap-1.5 rounded-lg border border-slate-700/60 bg-slate-900/40 px-4 py-2.5 text-[14px] font-semibold text-slate-300 hover:text-slate-100 hover:bg-slate-800/60 hover:border-slate-600/60 transition-all"
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
