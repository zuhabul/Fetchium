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
      "Fan out a single query across SearXNG, Brave, GitHub, Reddit, HackerNews, StackOverflow, YouTube, Wikipedia, ArXiv, Bing, and DuckDuckGo in parallel. Adaptive Backend Selector picks the right mix per query intent.",
    badge: "Unique",
    accent: "from-indigo-500/20 to-indigo-600/5",
    iconBg: "bg-indigo-500/10",
    iconColor: "text-indigo-400",
  },
  {
    icon: Brain,
    title: "HyperFusion Neural Ranking",
    description:
      "8-signal ranking engine: BM25 lexical match, semantic similarity, temporal decay, domain authority, evidence density, source diversity, content depth, and cross-source consensus.",
    badge: "Novel",
    accent: "from-violet-500/20 to-violet-600/5",
    iconBg: "bg-violet-500/10",
    iconColor: "text-violet-400",
  },
  {
    icon: Layers,
    title: "5-Layer CEP Content Extraction",
    description:
      "Content Extraction Protocol: CSS selectors, Readability, headless JS rendering, PDF parsing, and screenshot OCR. Designed to recover structured text from a wide range of pages, including SPAs and PDFs.",
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
      "AMRS multi-agent research swarm: 4 specialist agent types communicate over async channels, synthesize findings, and generate evidence graphs with claims traced to sources. Citation export supports APA, IEEE, BibTeX, and Chicago.",
    badge: "Novel",
    accent: "from-emerald-500/20 to-emerald-600/5",
    iconBg: "bg-emerald-500/10",
    iconColor: "text-emerald-400",
  },
  {
    icon: ShieldCheck,
    title: "Production Resilience",
    description:
      "Circuit breakers, bulkhead isolation, adaptive rate limiting, and backend-aware timeouts. Automatic failover and retry-and-refine validation help stabilize retrieval across heterogeneous sources.",
    badge: "Production",
    accent: "from-cyan-500/20 to-cyan-600/5",
    iconBg: "bg-cyan-500/10",
    iconColor: "text-cyan-400",
  },
  {
    icon: Youtube,
    title: "YouTube & Social Intelligence",
    description:
      "VideoFusion ranking for YouTube with transcript extraction and channel/video analysis. Native Reddit, HackerNews, and StackOverflow backends add community-signal retrieval alongside web search.",
    badge: "Novel",
    accent: "from-red-500/20 to-red-600/5",
    iconBg: "bg-red-500/10",
    iconColor: "text-red-400",
  },
  {
    icon: GitBranch,
    title: "PIE Cross-Session Learning",
    description:
      "Persistent Intelligence Engine tracks source trust, failure patterns, and query predictions across sessions via SQLite for deployments that enable persistence.",
    badge: "Novel",
    accent: "from-pink-500/20 to-pink-600/5",
    iconBg: "bg-pink-500/10",
    iconColor: "text-pink-400",
  },
  {
    icon: Cpu,
    title: "MCP Protocol Native",
    description:
      "First-class Model Context Protocol support. Fetchium exposes 12 MCP tools spanning search, fetch, estimate, research, YouTube, and social workflows.",
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
          <div className="mb-5 inline-flex items-center gap-2 rounded-full border border-indigo-500/30 bg-indigo-500/10 px-4 py-2 text-sm font-semibold text-indigo-200">
            <span className="h-2 w-2 animate-pulse rounded-full bg-indigo-400" />
            17 Novel Algorithms
          </div>
          <h2 className="mx-auto max-w-3xl text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-bold tracking-tight text-slate-100">
            Capabilities no other{" "}
            <span className="gradient-text">search API has</span>
          </h2>
          <p className="mt-5 sm:mt-6 mx-auto max-w-2xl text-base sm:text-xl text-slate-300 leading-relaxed">
            Fetchium combines federation, ranking, extraction, and research primitives
            into one API surface for AI search workloads.
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
                      className={`flex h-12 w-12 items-center justify-center rounded-xl border border-slate-700/50 ${feature.iconBg} transition-all duration-300 group-hover:scale-110 group-hover:shadow-lg`}
                    >
                      <Icon className={`h-6 w-6 ${feature.iconColor}`} strokeWidth={1.75} />
                    </div>
                    <span
                      className={`relative rounded-full border px-3 py-1 text-[12px] font-bold tracking-wide uppercase ${badgeStyles[feature.badge]}`}
                    >
                      {feature.badge}
                    </span>
                  </div>

                  {/* Text */}
                  <h3 className="relative mb-3 text-[17px] font-bold text-slate-100">
                    {feature.title}
                  </h3>
                  <p className="relative text-[15px] leading-relaxed text-slate-300 group-hover:text-slate-200 transition-colors duration-300">
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
            { value: "563+", label: "Unit tests" },
            { value: "12", label: "MCP tools" },
          ].map((stat) => (
            <div
              key={stat.label}
              className="glass-card rounded-xl p-4 sm:p-5 text-center"
            >
              <div className="text-2xl sm:text-4xl font-bold tracking-tight text-slate-100">
                {stat.value}
              </div>
              <div className="mt-2 text-sm sm:text-base font-semibold text-slate-300">
                {stat.label}
              </div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
