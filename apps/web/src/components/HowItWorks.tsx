"use client";

import { motion, useInView } from "framer-motion";
import { useRef } from "react";
import {
  Search,
  Globe,
  Brain,
  Layers,
  Coins,
  Sparkles,
  ArrowRight,
} from "lucide-react";

interface Step {
  number: number;
  icon: React.ElementType;
  title: string;
  subtitle: string;
  description: string;
  detail: string;
  color: string;
  glow: string;
  iconBg: string;
  borderColor: string;
  numberColor: string;
}

const steps: Step[] = [
  {
    number: 1,
    icon: Search,
    title: "Query Analysis",
    subtitle: "QFD + QCE + QXE",
    description:
      "Your query is fingerprinted, classified by intent, scored for complexity, and expanded with semantic variants. The system chooses the optimal backend mix before a single network call is made.",
    detail: "Stage 1",
    color: "from-indigo-500 to-indigo-600",
    glow: "rgba(99,102,241,0.35)",
    iconBg: "bg-indigo-500/15 border-indigo-500/25",
    borderColor: "border-indigo-500/30",
    numberColor: "text-indigo-400",
  },
  {
    number: 2,
    icon: Globe,
    title: "Multi-Backend Federation",
    subtitle: "ABS + Resilience Layer",
    description:
      "The Adaptive Backend Selector fans your query across up to 11 sources in parallel — SearXNG, Brave, GitHub, Reddit, StackOverflow, YouTube, and more. Circuit breakers handle backend failures invisibly.",
    detail: "Stage 2",
    color: "from-blue-500 to-blue-600",
    glow: "rgba(59,130,246,0.3)",
    iconBg: "bg-blue-500/15 border-blue-500/25",
    borderColor: "border-blue-500/30",
    numberColor: "text-blue-400",
  },
  {
    number: 3,
    icon: Brain,
    title: "HyperFusion Ranking",
    subtitle: "8-Signal Neural Ranking",
    description:
      "Results are scored on 8 signals: BM25 lexical match, semantic similarity, temporal freshness, domain authority, evidence density, source diversity, content depth, and cross-source consensus.",
    detail: "Stage 3",
    color: "from-violet-500 to-violet-600",
    glow: "rgba(139,92,246,0.3)",
    iconBg: "bg-violet-500/15 border-violet-500/25",
    borderColor: "border-violet-500/30",
    numberColor: "text-violet-400",
  },
  {
    number: 4,
    icon: Layers,
    title: "CEP Content Extraction",
    subtitle: "5-Layer Cascade",
    description:
      "Top-ranked URLs are deep-extracted via the Content Extraction Protocol: CSS selectors, Readability, headless JS rendering, PDF parsing, and screenshot OCR.",
    detail: "Stage 4",
    color: "from-cyan-500 to-cyan-600",
    glow: "rgba(6,182,212,0.3)",
    iconBg: "bg-cyan-500/15 border-cyan-500/25",
    borderColor: "border-cyan-500/30",
    numberColor: "text-cyan-400",
  },
  {
    number: 5,
    icon: Coins,
    title: "Token Budget Control",
    subtitle: "QATBE Algorithm",
    description:
      "Extracted content is segmented, BM25-scored for query relevance, then packed into your token budget via greedy knapsack. You always get the most relevant content that fits your LLM context window.",
    detail: "Stage 5",
    color: "from-amber-500 to-amber-600",
    glow: "rgba(245,158,11,0.3)",
    iconBg: "bg-amber-500/15 border-amber-500/25",
    borderColor: "border-amber-500/30",
    numberColor: "text-amber-400",
  },
  {
    number: 6,
    icon: Sparkles,
    title: "AI-Ready Response",
    subtitle: "Evidence Graph + Citations",
    description:
      "The final response includes ranked results, extracted content within your budget, an evidence graph tracing every claim to a source, and auto-generated citations in APA, IEEE, BibTeX, or Chicago format.",
    detail: "Stage 6",
    color: "from-emerald-500 to-emerald-600",
    glow: "rgba(16,185,129,0.3)",
    iconBg: "bg-emerald-500/15 border-emerald-500/25",
    borderColor: "border-emerald-500/30",
    numberColor: "text-emerald-400",
  },
];

function StepCard({ step, index }: { step: Step; index: number }) {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { once: true, margin: "-60px" });
  const Icon = step.icon;

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 32, scale: 0.96 }}
      animate={inView ? { opacity: 1, y: 0, scale: 1 } : {}}
      transition={{
        delay: index * 0.1,
        duration: 0.55,
        ease: [0.22, 1, 0.36, 1],
      }}
      className="group relative"
    >
      {/* Connector line (hidden on last item) */}
      {index < steps.length - 1 && (
        <div className="absolute left-6 top-[52px] hidden h-[calc(100%+1.25rem)] w-px bg-gradient-to-b from-white/10 to-transparent lg:block" />
      )}

      <div
        className={`glass-card relative overflow-hidden rounded-2xl p-6 transition-all duration-300 hover:border-[${step.borderColor}]`}
        style={
          {
            "--glow-color": step.glow,
          } as React.CSSProperties
        }
      >
        {/* Inner glow on hover */}
        <div
          className="absolute inset-0 rounded-2xl opacity-0 transition-opacity duration-300 group-hover:opacity-100"
          style={{
            background: `radial-gradient(ellipse at 30% 30%, ${step.glow.replace("0.3", "0.08")}, transparent 70%)`,
          }}
        />

        <div className="relative flex items-start gap-3 sm:gap-5">
          {/* Step number + icon column */}
          <div className="flex shrink-0 flex-col items-center gap-1 sm:gap-2">
            {/* Large step number */}
            <span
              className={`text-[10px] sm:text-[11px] font-bold tracking-widest uppercase ${step.numberColor} opacity-60`}
            >
              {String(step.number).padStart(2, "0")}
            </span>
            {/* Icon */}
            <div
              className={`flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center rounded-xl border ${step.iconBg} transition-all duration-300 group-hover:scale-110`}
              style={{
                boxShadow: `0 0 20px ${step.glow.replace("0.3", "0")} group-hover:0 0 20px ${step.glow}`,
              }}
            >
              <Icon
                className={`h-5 w-5 sm:h-6 sm:w-6`}
                style={{ color: step.glow.replace("rgba(", "rgb(").replace(/,[\d.]+\)$/, ")") }}
                strokeWidth={1.75}
              />
            </div>
          </div>

          {/* Content */}
          <div className="min-w-0 flex-1">
            <div className="mb-0.5 flex items-center justify-between gap-2">
              <h3 className="text-base sm:text-lg font-bold text-slate-100">
                {step.title}
              </h3>
              <span className="shrink-0 rounded-full border border-indigo-500/35 bg-indigo-500/15 px-2.5 py-0.5 font-mono text-[11px] sm:text-[12px] font-bold text-indigo-200">
                {step.detail}
              </span>
            </div>
            <div className="mb-2 text-[11px] sm:text-[12px] font-semibold text-slate-400 tracking-wide uppercase">
              {step.subtitle}
            </div>
            <p className="text-sm sm:text-base leading-relaxed text-slate-300 group-hover:text-slate-200 transition-colors duration-300">
              {step.description}
            </p>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function PipelineDiagram() {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { once: true });

  const labels = [
    { label: "Query", color: "bg-indigo-500" },
    { label: "Federation", color: "bg-blue-500" },
    { label: "Ranking", color: "bg-violet-500" },
    { label: "Extraction", color: "bg-cyan-500" },
    { label: "Token Budget", color: "bg-amber-500" },
    { label: "Response", color: "bg-emerald-500" },
  ];

  return (
    <div className="mb-16">
      <div className="sm:hidden text-center text-[11px] text-slate-400 mb-2 flex items-center justify-center gap-1.5">
        <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
        </svg>
        Swipe to see pipeline
      </div>
    <div
      ref={ref}
      className="flex items-center justify-center gap-0 overflow-x-auto pb-2 px-2"
    >
      {labels.map((l, i) => (
        <div key={l.label} className="flex items-center">
          <motion.div
            initial={{ opacity: 0, scale: 0.7 }}
            animate={inView ? { opacity: 1, scale: 1 } : {}}
            transition={{ delay: 0.2 + i * 0.08, duration: 0.4 }}
            className="flex flex-col items-center gap-2"
          >
            <div
              className={`h-2.5 w-2.5 rounded-full ${l.color} shadow-lg`}
              style={{ boxShadow: `0 0 8px currentColor` }}
            />
            <span className="whitespace-nowrap text-[11px] font-medium text-slate-300">
              {l.label}
            </span>
          </motion.div>
          {i < labels.length - 1 && (
            <motion.div
              initial={{ opacity: 0, scaleX: 0 }}
              animate={inView ? { opacity: 1, scaleX: 1 } : {}}
              style={{ transformOrigin: "left" }}
              transition={{
                delay: 0.25 + i * 0.08,
                duration: 0.3,
              }}
              className="mx-2 flex items-center"
            >
              <div className="h-px w-8 bg-gradient-to-r from-white/15 to-white/5" />
              <ArrowRight className="h-3 w-3 text-slate-700" />
            </motion.div>
          )}
        </div>
      ))}
    </div>
    </div>
  );
}

export default function HowItWorks() {
  return (
    <section className="relative overflow-hidden py-16 sm:py-28 px-4">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute right-0 top-0 h-[500px] w-[600px] rounded-full bg-violet-500/4 blur-[120px]" />
        <div className="absolute bottom-0 left-0 h-[400px] w-[500px] rounded-full bg-indigo-500/4 blur-[100px]" />
      </div>

      <div className="relative mx-auto max-w-4xl">
        {/* Header */}
        <motion.div
          className="mb-10 sm:mb-14 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-5 inline-flex items-center gap-2 rounded-full border border-violet-500/30 bg-violet-500/10 px-4 py-2 text-sm font-semibold text-violet-200">
            <Sparkles className="h-4 w-4" />
            Pipeline Architecture
          </div>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight text-slate-100">
            How it{" "}
            <span className="gradient-text">works</span>
          </h2>
          <p className="mt-5 sm:mt-6 mx-auto max-w-xl text-base sm:text-xl text-slate-300 leading-relaxed">
            Six stages. Search, extraction, ranking, and evidence tracing in one pipeline with
            an evidence graph.
          </p>
        </motion.div>

        {/* Horizontal pipeline diagram */}
        <PipelineDiagram />

        {/* Steps */}
        <div className="space-y-4">
          {steps.map((step, i) => (
            <StepCard key={step.number} step={step} index={i} />
          ))}
        </div>

        {/* Bottom callout */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.4 }}
          className="mt-8 sm:mt-10 overflow-hidden rounded-2xl border border-indigo-500/20 bg-gradient-to-r from-indigo-500/8 to-violet-500/8 p-4 sm:p-6"
        >
          <div className="flex flex-col items-start gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <div className="mb-1 text-base sm:text-lg font-bold text-slate-100">
                Start building in 2 minutes
              </div>
              <div className="text-sm sm:text-base text-slate-300 leading-relaxed">
                Get an API key, make your first search call, get back ranked results with
                extracted content and evidence graphs. Free plan — no credit card required.
              </div>
            </div>
            <a
              href="https://app.fetchium.com"
              className="group flex shrink-0 items-center gap-2 rounded-xl border border-indigo-500/30 bg-indigo-500/10 px-5 py-3 text-sm sm:text-base font-bold text-indigo-200 transition-all hover:bg-indigo-500/20 hover:text-indigo-100 w-full sm:w-auto justify-center min-h-[48px]"
            >
              Get API Key Free
              <ArrowRight className="h-5 w-5 transition-transform group-hover:translate-x-0.5" />
            </a>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
