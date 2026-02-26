"use client";
import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";
import dynamic from "next/dynamic";

const NeuralCanvas = dynamic(() => import("./NeuralCanvas"), { ssr: false });

// ── Single short words only — no wrapping, no layout shift ──────────────────
const WORDS = ["thinks.", "learns.", "ranks.", "extracts.", "reasons."];

/**
 * Slot-machine word rotator.
 * Words scroll vertically inside a fixed-height overflow:hidden container.
 * Parent layout NEVER shifts — container dimensions are completely stable.
 */
function WordRotator() {
  const [idx, setIdx] = useState(0);
  useEffect(() => {
    const t = setInterval(() => setIdx(i => (i + 1) % WORDS.length), 2400);
    return () => clearInterval(t);
  }, []);
  return (
    <span
      className="relative inline-block overflow-hidden"
      style={{
        // Fix height to exactly one line so no reflow ever occurs
        height: "1.15em",
        verticalAlign: "bottom",
        // Width large enough for the longest word ("extracts.")
        minWidth: "4.2ch",
      }}
    >
      <AnimatePresence mode="wait" initial={false}>
        <motion.span
          key={WORDS[idx]}
          className="absolute inset-0 flex items-center gradient-text-purple whitespace-nowrap"
          initial={{ y: "110%" }}
          animate={{ y: "0%" }}
          exit={{ y: "-110%" }}
          transition={{ duration: 0.45, ease: [0.22, 1, 0.36, 1] }}
        >
          {WORDS[idx]}
        </motion.span>
      </AnimatePresence>
    </span>
  );
}

const STATS = [
  { value: "17",     label: "Novel algorithms" },
  { value: "11",     label: "Search backends"  },
  { value: "930+",   label: "Tests passing"    },
  { value: "<200ms", label: "Median latency"   },
];

const RESPONSE_LINES = [
  { k: '"query"',       v: '"best async rust patterns"'         },
  { k: '"latency_ms"',  v: "183"                                 },
  { k: '"results"',     v: "[ 11 sources × top 30 results ]"    },
  { k: '"ranked"',      v: "[ HyperFusion 8-signal score ]"     },
  { k: '"content"',     v: '"4 096 tokens extracted + cleaned"' },
  { k: '"citations"',   v: "8  (APA · IEEE · BibTeX)"           },
  { k: '"evidence"',    v: "{ nodes: 14, edges: 22 }"           },
];

function ApiDemo() {
  return (
    <div className="glass rounded-2xl border border-indigo-500/15 overflow-hidden w-full max-w-xl mx-auto text-left mb-14">
      {/* Title bar */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/[0.06] bg-white/[0.015]">
        <div className="flex gap-1.5">
          <div className="w-2.5 h-2.5 rounded-full bg-red-500/50" />
          <div className="w-2.5 h-2.5 rounded-full bg-yellow-500/50" />
          <div className="w-2.5 h-2.5 rounded-full bg-green-500/50" />
        </div>
        <span className="text-[11px] text-slate-500 font-mono">POST /v1/search → 200 OK</span>
        <span className="text-[11px] text-emerald-400 font-mono font-semibold">183ms</span>
      </div>

      {/* Response lines */}
      <div className="p-3 sm:p-4 space-y-1 font-mono text-[11px] sm:text-[12px] leading-relaxed">
        <div className="text-slate-600">{"{"}</div>
        {RESPONSE_LINES.map((line, i) => (
          <div key={i} className="flex items-baseline gap-1 pl-3 sm:pl-4 min-w-0">
            <span className="token-key shrink-0">{line.k}:</span>
            <span className="token-string truncate">{line.v}</span>
            {i < RESPONSE_LINES.length - 1 && <span className="token-punctuation shrink-0">,</span>}
          </div>
        ))}
        <div className="text-slate-600">{"}"}</div>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between px-4 py-2 border-t border-white/[0.04] bg-white/[0.01]">
        <span className="text-[10px] text-slate-600 font-mono">11 backends · 30 results · 4 096 tokens</span>
        <Link href="/docs/api" className="text-[10px] text-indigo-400 hover:text-indigo-300 font-mono transition-colors">
          API docs →
        </Link>
      </div>
    </div>
  );
}

export default function Hero() {
  return (
    <section className="relative min-h-screen flex flex-col items-center justify-center overflow-hidden">
      <div className="absolute inset-0"><NeuralCanvas /></div>

      {/* Atmosphere glows */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[600px] bg-indigo-600/10 blur-[130px] rounded-full" />
        <div className="absolute top-1/3 left-1/4 w-[350px] h-[350px] bg-violet-700/8 blur-[90px] rounded-full" />
        <div className="absolute top-1/3 right-1/4 w-[280px] h-[280px] bg-indigo-500/8 blur-[80px] rounded-full" />
      </div>

      {/* Grid */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          backgroundImage:
            "linear-gradient(rgba(99,102,241,0.035) 1px,transparent 1px),linear-gradient(90deg,rgba(99,102,241,0.035) 1px,transparent 1px)",
          backgroundSize: "72px 72px",
        }}
      />

      {/*
        w-full is CRITICAL: prevents flex-col + items-center from sizing this
        div to its intrinsic content width, which would cause overflow on mobile.
      */}
      <div className="relative z-10 w-full max-w-5xl mx-auto px-4 sm:px-6 text-center pt-24 sm:pt-32 pb-16">

        {/* Badge */}
        <motion.div
          initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.6 }}
          className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-indigo-500/25 bg-indigo-500/8 text-indigo-300 text-xs mb-6 sm:mb-8 cursor-default"
        >
          <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
          <span className="hidden sm:inline">Open Beta · 17 novel algorithms · 11 backends · Free to start</span>
          <span className="sm:hidden">17 algorithms · Free to start</span>
        </motion.div>

        {/* ── Headline ─────────────────────────────────────────────────────────
            The h1 has a FIXED layout — "The search API that" is static text.
            WordRotator uses overflow:hidden + fixed height so the h1 height
            never changes between animation frames. Zero layout shift.
        ──────────────────────────────────────────────────────────────────── */}
        <motion.h1
          initial={{ opacity: 0, y: 30 }} animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.1 }}
          className="text-[2.6rem] sm:text-5xl md:text-6xl lg:text-[72px] font-bold tracking-tight leading-[1.12] mb-5"
        >
          <span className="gradient-text">The search API</span>
          <br />
          <span className="text-slate-200">that </span>
          <WordRotator />
        </motion.h1>

        {/* Sub */}
        <motion.p
          initial={{ opacity: 0, y: 24 }} animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.2 }}
          className="text-base sm:text-xl text-slate-400 max-w-2xl mx-auto mb-8 leading-relaxed"
        >
          One API call. 11 federated search backends, 8-signal neural ranking,
          5-layer content extraction, and token-budgeted context with evidence graphs
          — in under 200ms.
        </motion.p>

        {/* CTAs */}
        <motion.div
          initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3 }}
          className="flex flex-col sm:flex-row gap-3 justify-center mb-12"
        >
          <Link
            href="https://app.hypersearchx.zuhabul.com"
            className="group inline-flex items-center justify-center gap-2 px-7 py-3.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-sm transition-all duration-200 shadow-[0_0_30px_rgba(99,102,241,0.3)] hover:shadow-[0_0_50px_rgba(99,102,241,0.5)] min-h-[48px]"
          >
            Get API Key — Free
            <svg className="w-4 h-4 transition-transform group-hover:translate-x-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </Link>
          <Link
            href="/docs"
            className="inline-flex items-center justify-center gap-2 px-7 py-3.5 rounded-xl glass text-slate-200 font-semibold text-sm transition-all duration-200 hover:border-indigo-500/35 hover:bg-indigo-500/5 min-h-[48px]"
          >
            <svg className="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            View Docs
          </Link>
        </motion.div>

        {/* API demo */}
        <motion.div
          initial={{ opacity: 0, y: 30 }} animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4 }}
        >
          <ApiDemo />
        </motion.div>

        {/* Stats */}
        <motion.div
          initial={{ opacity: 0 }} animate={{ opacity: 1 }}
          transition={{ delay: 0.6 }}
          className="grid grid-cols-2 sm:grid-cols-4 gap-3 sm:gap-4"
        >
          {STATS.map((s, i) => (
            <motion.div
              key={i}
              initial={{ opacity: 0, scale: 0.9 }} animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.55 + i * 0.07 }}
              className="glass-card rounded-xl px-4 py-4 sm:py-5 text-center"
            >
              <div className="text-xl sm:text-2xl font-bold gradient-text">{s.value}</div>
              <div className="text-[11px] sm:text-xs text-slate-500 mt-1">{s.label}</div>
            </motion.div>
          ))}
        </motion.div>
      </div>

      {/* Bottom fade */}
      <div className="absolute bottom-0 inset-x-0 h-48 bg-gradient-to-t from-[#06070d] to-transparent pointer-events-none" />

      {/* Scroll hint */}
      <motion.div
        initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 1.2 }}
        className="absolute bottom-6 sm:bottom-10 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 text-slate-600"
      >
        <motion.div
          animate={{ y: [0, 8, 0] }} transition={{ duration: 2, repeat: Infinity }}
          className="w-5 h-8 rounded-full border border-slate-700 flex items-start justify-center pt-1.5"
        >
          <div className="w-1 h-2 rounded-full bg-slate-500" />
        </motion.div>
      </motion.div>
    </section>
  );
}
