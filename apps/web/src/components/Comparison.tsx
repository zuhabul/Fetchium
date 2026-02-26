"use client";

import { motion } from "framer-motion";
import { Check, X, Minus, Zap } from "lucide-react";

type Status = "yes" | "no" | "partial";

interface Feature {
  label: string;
  description?: string;
  category?: string;
}

const features: Feature[] = [
  { label: "Multi-source federation", description: "11+ simultaneous backends", category: "Search" },
  { label: "Self-hostable (free, unlimited)", category: "Deployment" },
  { label: "Token budget control (QATBE)", category: "Intelligence" },
  { label: "5-layer content extraction (CEP)", category: "Extraction" },
  { label: "8-signal neural ranking", category: "Intelligence" },
  { label: "Evidence graphs + citations", category: "Intelligence" },
  { label: "Cross-session learning (PIE)", category: "Intelligence" },
  { label: "Deep research pipeline (AMRS)", category: "Intelligence" },
  { label: "YouTube & social search", category: "Search" },
  { label: "Real-time monitoring + diffs", category: "Features" },
  { label: "MCP protocol support", category: "Integration" },
  { label: "CLI tool included", category: "Tooling" },
  { label: "Free tier included", category: "Pricing" },
  { label: "Open source (MIT/Apache)", category: "Licensing" },
  { label: "Median API latency", description: "Lower is better", category: "Performance" },
];

interface Tool {
  name: string;
  tagline: string;
  highlight: boolean;
  data: (Status | string)[];
}

const tools: Tool[] = [
  {
    name: "HyperSearchX",
    tagline: "The search API that thinks",
    highlight: true,
    data: [
      "yes", "yes", "yes", "yes", "yes", "yes", "yes", "yes",
      "yes", "yes", "yes", "yes", "yes", "yes", "< 200ms",
    ],
  },
  {
    name: "Firecrawl",
    tagline: "Web scraping API",
    highlight: false,
    data: [
      "no", "no", "no", "partial", "no", "no", "no", "no",
      "no", "partial", "no", "no", "yes", "no", "~800ms",
    ],
  },
  {
    name: "SerpAPI",
    tagline: "Google SERP data",
    highlight: false,
    data: [
      "partial", "no", "no", "no", "no", "no", "no", "no",
      "partial", "no", "no", "no", "yes", "no", "~400ms",
    ],
  },
  {
    name: "Perplexity",
    tagline: "AI answer engine",
    highlight: false,
    data: [
      "partial", "no", "no", "partial", "partial", "partial", "no", "partial",
      "no", "no", "no", "no", "partial", "no", "~2000ms",
    ],
  },
  {
    name: "Exa",
    tagline: "Neural search",
    highlight: false,
    data: [
      "partial", "no", "no", "no", "partial", "no", "no", "no",
      "no", "no", "no", "no", "no", "no", "~300ms",
    ],
  },
];

function StatusCell({ value }: { value: Status | string }) {
  if (value === "yes") {
    return (
      <div className="flex items-center justify-center">
        <div className="flex h-6 w-6 items-center justify-center rounded-full bg-emerald-500/12 shadow-[0_0_8px_rgba(16,185,129,0.15)]">
          <Check className="h-3.5 w-3.5 text-emerald-400" strokeWidth={2.5} />
        </div>
      </div>
    );
  }
  if (value === "no") {
    return (
      <div className="flex items-center justify-center">
        <X className="h-3.5 w-3.5 text-slate-700" />
      </div>
    );
  }
  if (value === "partial") {
    return (
      <div className="flex items-center justify-center">
        <Minus className="h-3.5 w-3.5 text-amber-500/70" />
      </div>
    );
  }
  // String (like latency)
  return (
    <div className="flex items-center justify-center">
      <span className="rounded-md border border-white/8 bg-white/4 px-2 py-0.5 font-mono text-[11px] text-slate-400">
        {value}
      </span>
    </div>
  );
}

export default function Comparison() {
  return (
    <section id="compare" className="relative overflow-hidden py-28 px-4">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-0 bottom-0 h-[400px] w-[600px] rounded-full bg-indigo-500/4 blur-[120px]" />
      </div>

      <div className="relative mx-auto max-w-7xl">
        {/* Header */}
        <motion.div
          className="mb-14 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-4 py-1.5 text-xs font-medium text-indigo-300">
            <Zap className="h-3.5 w-3.5" strokeWidth={2.5} />
            Honest Comparison
          </div>
          <h2 className="text-4xl font-bold tracking-tight text-slate-100 sm:text-5xl">
            We win{" "}
            <span className="gradient-text">every row</span>
          </h2>
          <p className="mt-5 mx-auto max-w-2xl text-lg text-slate-500">
            HyperSearchX is the only tool that combines search federation, deep
            extraction, intelligent ranking, and cross-session learning in a
            single API.
          </p>
        </motion.div>

        {/* Table wrapper */}
        <motion.div
          initial={{ opacity: 0, y: 32 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-60px" }}
          transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
          className="overflow-x-auto rounded-2xl border border-[rgba(99,102,241,0.12)] shadow-[0_24px_64px_rgba(0,0,0,0.5)]"
        >
          <table className="w-full border-collapse text-sm">
            {/* Column headers */}
            <thead>
              <tr className="border-b border-white/6">
                <th className="w-64 py-5 px-6 text-left text-[13px] font-medium text-slate-600">
                  Feature
                </th>
                {tools.map((tool) => (
                  <th
                    key={tool.name}
                    className={`min-w-[120px] py-5 px-4 text-center align-bottom ${
                      tool.highlight
                        ? "bg-[rgba(99,102,241,0.06)]"
                        : "bg-[rgba(13,17,23,0.6)]"
                    }`}
                  >
                    {tool.highlight && (
                      <div className="mb-2 flex items-center justify-center">
                        <span className="inline-flex items-center gap-1 rounded-full bg-gradient-to-r from-indigo-500 to-violet-600 px-2.5 py-0.5 text-[10px] font-bold text-white shadow-[0_0_12px_rgba(99,102,241,0.4)]">
                          <Zap className="h-2.5 w-2.5" strokeWidth={3} />
                          Best
                        </span>
                      </div>
                    )}
                    <div
                      className={`text-[13px] font-semibold ${
                        tool.highlight ? "text-indigo-300" : "text-slate-500"
                      }`}
                    >
                      {tool.name}
                    </div>
                    <div className="mt-0.5 text-[10px] font-normal text-slate-600">
                      {tool.tagline}
                    </div>
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
                  {/* Feature label */}
                  <td className="py-4 px-6">
                    <div className="text-[13px] text-slate-300">
                      {feature.label}
                    </div>
                    {feature.description && (
                      <div className="mt-0.5 text-[11px] text-slate-600">
                        {feature.description}
                      </div>
                    )}
                  </td>

                  {/* Tool columns */}
                  {tools.map((tool) => (
                    <td
                      key={tool.name}
                      className={`py-4 px-4 ${
                        tool.highlight
                          ? "bg-[rgba(99,102,241,0.04)]"
                          : ""
                      }`}
                    >
                      <StatusCell value={tool.data[fi]} />
                    </td>
                  ))}
                </motion.tr>
              ))}
            </tbody>
          </table>
        </motion.div>

        {/* Legend */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.5 }}
          className="mt-5 flex flex-wrap items-center justify-center gap-6 text-[12px] text-slate-600"
        >
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
        </motion.div>

        {/* Value callout */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5, duration: 0.5 }}
          className="mt-8 overflow-hidden rounded-2xl border border-indigo-500/15 bg-gradient-to-r from-indigo-500/6 to-violet-500/6 p-6 text-center"
        >
          <p className="text-[14px] text-slate-400">
            <span className="font-semibold text-slate-100">
              vs Firecrawl Pro ($599/mo):
            </span>{" "}
            Our Pro plan delivers a superset of features at{" "}
            <span className="font-bold text-emerald-400">87% less cost</span>.
            Federation, neural ranking, and cross-session learning that
            Firecrawl doesn&apos;t offer at any price.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
