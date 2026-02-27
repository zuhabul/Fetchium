"use client";

import { motion } from "framer-motion";
import {
  Globe,
  Layers,
  Brain,
  Coins,
  FlaskConical,
  ShieldCheck,
  Youtube,
  Cpu,
  GitBranch,
} from "lucide-react";

const features = [
  {
    icon: Globe,
    title: "11-Backend Search Federation",
    description:
      "Fan out a single query across SearXNG, Brave, GitHub, Reddit, HackerNews, StackOverflow, YouTube, Wikipedia, ArXiv, Bing, and DuckDuckGo — simultaneously. Adaptive Backend Selector picks the right mix per query intent. No other search API comes close.",
    badge: "Unique",
    accent: "from-indigo-500/20 to-indigo-600/5",
    iconBg: "bg-indigo-500/10",
    iconColor: "text-indigo-400",
  },
  {
    icon: Brain,
    title: "HyperFusion Neural Ranking",
    description:
      "8-signal ranking engine: BM25 lexical match + semantic similarity + temporal decay + domain authority + evidence density + source diversity + content depth + cross-source consensus. Results are measurably better than single-signal ranking — every time.",
    badge: "Novel",
    accent: "from-violet-500/20 to-violet-600/5",
    iconBg: "bg-violet-500/10",
    iconColor: "text-violet-400",
  },
  {
    icon: Layers,
    title: "5-Layer CEP Content Extraction",
    description:
      "Content Extraction Protocol: CSS selectors → Readability algorithm → Headless JS rendering → PDF parsing → Screenshot OCR. Every page — SPAs, paywalled content, PDFs — returns clean, structured text. Zero pages escape extraction.",
    badge: "Novel",
    accent: "from-blue-500/20 to-blue-600/5",
    iconBg: "bg-blue-500/10",
    iconColor: "text-blue-400",
  },
  {
    icon: Coins,
    title: "QATBE Token Budget Control",
    description:
      "Query-Aware Token-Budgeted Extraction scores every content segment with BM25 then solves a greedy knapsack to pack maximum relevance into your exact LLM context window. You always get the most useful content — not just the first N characters.",
    badge: "Novel",
    accent: "from-amber-500/20 to-amber-600/5",
    iconBg: "bg-amber-500/10",
    iconColor: "text-amber-400",
  },
  {
    icon: FlaskConical,
    title: "Deep Research Pipeline",
    description:
      "AMRS multi-agent research swarm: 4 specialist agent types communicate over async channels, synthesise findings, and generate full evidence graphs with every claim traced to a source. Auto-generate citations in APA, IEEE, BibTeX, or Chicago.",
    badge: "Novel",
    accent: "from-emerald-500/20 to-emerald-600/5",
    iconBg: "bg-emerald-500/10",
    iconColor: "text-emerald-400",
  },
  {
    icon: ShieldCheck,
    title: "Production Resilience",
    description:
      "Circuit breakers, bulkhead isolation, adaptive rate limiting, and latency prediction across all backends. Automatic failover, retry-and-refine with 5-checkpoint self-correction, and 99.9% uptime SLA on Pro and Enterprise plans.",
    badge: "Production",
    accent: "from-cyan-500/20 to-cyan-600/5",
    iconBg: "bg-cyan-500/10",
    iconColor: "text-cyan-400",
  },
  {
    icon: Youtube,
    title: "YouTube & Social Intelligence",
    description:
      "VideoFusion ranking for YouTube: transcript extraction, comment sentiment, teaching quality scoring. Native Reddit, HackerNews, and StackOverflow backends with community signal weighting. Social search no other API provides.",
    badge: "Novel",
    accent: "from-red-500/20 to-red-600/5",
    iconBg: "bg-red-500/10",
    iconColor: "text-red-400",
  },
  {
    icon: GitBranch,
    title: "PIE Cross-Session Learning",
    description:
      "Persistent Intelligence Engine tracks source trust, failure patterns, and query predictions across sessions via SQLite. Fetchium gets smarter with every query your application makes — improving result quality automatically over time.",
    badge: "Novel",
    accent: "from-pink-500/20 to-pink-600/5",
    iconBg: "bg-pink-500/10",
    iconColor: "text-pink-400",
  },
  {
    icon: Cpu,
    title: "MCP Protocol Native",
    description:
      "First-class Model Context Protocol support. Expose all 17 algorithms as MCP tools to Claude, GPT-4, and any LLM with tool use. Structured, schema-validated outputs are AI-consumption-ready out of the box — no post-processing needed.",
    badge: "Unique",
    accent: "from-teal-500/20 to-teal-600/5",
    iconBg: "bg-teal-500/10",
    iconColor: "text-teal-400",
  },
];

const badgeStyles: Record<string, string> = {
  Unique:
    "bg-violet-500/10 text-violet-300 border-violet-500/25 shadow-[0_0_8px_rgba(139,92,246,0.1)]",
  Novel:
    "bg-indigo-500/10 text-indigo-300 border-indigo-500/25 shadow-[0_0_8px_rgba(99,102,241,0.1)]",
  Production:
    "bg-emerald-500/10 text-emerald-300 border-emerald-500/25 shadow-[0_0_8px_rgba(16,185,129,0.1)]",
};

const containerVariants = {
  hidden: {},
  visible: {
    transition: {
      staggerChildren: 0.07,
    },
  },
};

const cardVariants = {
  hidden: { opacity: 0, y: 28, scale: 0.97 },
  visible: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: {
      duration: 0.5,
      ease: [0.22, 1, 0.36, 1] as [number, number, number, number],
    },
  },
};

export default function Features() {
  return (
    <section id="features" className="relative py-16 sm:py-28 px-4 overflow-hidden">
      {/* Background accent */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/4 top-0 h-[500px] w-[800px] -translate-x-1/2 rounded-full bg-indigo-500/5 blur-[120px]" />
        <div className="absolute right-0 bottom-0 h-[400px] w-[600px] rounded-full bg-violet-500/4 blur-[100px]" />
      </div>

      <div className="relative mx-auto max-w-7xl">
        {/* Section header */}
        <motion.div
          className="mb-12 sm:mb-20 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 sm:px-4 py-1.5 text-xs font-medium text-indigo-300">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-indigo-400" />
            17 Novel Algorithms
          </div>
          <h2 className="mx-auto max-w-3xl text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold tracking-tight text-slate-100">
            Capabilities no other{" "}
            <span className="gradient-text">search API has</span>
          </h2>
          <p className="mt-4 sm:mt-5 mx-auto max-w-2xl text-sm sm:text-lg text-slate-500">
            Every algorithm was designed from scratch to solve a real gap in the
            AI search stack — not a wrapper around an existing tool.
          </p>
        </motion.div>

        {/* Feature grid */}
        <motion.div
          className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3"
          variants={containerVariants}
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-60px" }}
        >
          {features.map((feature) => {
            const Icon = feature.icon;
            return (
              <motion.div
                key={feature.title}
                variants={cardVariants}
                className="group relative overflow-hidden rounded-2xl"
              >
                {/* Card glass layer */}
                <div className="glass-card relative h-full rounded-2xl p-6 transition-all duration-300">
                  {/* Inner glow on hover */}
                  <div
                    className={`absolute inset-0 rounded-2xl bg-gradient-to-br ${feature.accent} opacity-0 transition-opacity duration-300 group-hover:opacity-100`}
                  />

                  {/* Top row: icon + badge */}
                  <div className="relative mb-5 flex items-start justify-between">
                    <div
                      className={`flex h-11 w-11 items-center justify-center rounded-xl border border-white/5 ${feature.iconBg} transition-all duration-300 group-hover:scale-110 group-hover:shadow-lg`}
                    >
                      <Icon className={`h-5 w-5 ${feature.iconColor}`} strokeWidth={1.75} />
                    </div>
                    <span
                      className={`relative rounded-full border px-2.5 py-0.5 text-[11px] font-semibold tracking-wide uppercase ${badgeStyles[feature.badge]}`}
                    >
                      {feature.badge}
                    </span>
                  </div>

                  {/* Text */}
                  <h3 className="relative mb-2.5 text-[15px] font-semibold text-slate-100">
                    {feature.title}
                  </h3>
                  <p className="relative text-[13.5px] leading-relaxed text-slate-500 group-hover:text-slate-400 transition-colors duration-300">
                    {feature.description}
                  </p>

                  {/* Bottom shimmer line */}
                  <div className="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-indigo-500/30 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100" />
                </div>
              </motion.div>
            );
          })}
        </motion.div>

        {/* Bottom stats row */}
        <motion.div
          className="mt-10 sm:mt-16 grid grid-cols-2 gap-3 sm:gap-4"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.3 }}
        >
          {[
            { value: "11+", label: "Search backends" },
            { value: "17", label: "Novel algorithms" },
            { value: "930+", label: "Unit tests" },
            { value: "< 200ms", label: "Median latency" },
          ].map((stat) => (
            <div
              key={stat.label}
              className="glass-card rounded-xl p-4 sm:p-5 text-center"
            >
              <div className="text-xl sm:text-3xl font-bold tracking-tight text-slate-100">
                {stat.value}
              </div>
              <div className="mt-1 text-xs font-medium text-slate-500">
                {stat.label}
              </div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
