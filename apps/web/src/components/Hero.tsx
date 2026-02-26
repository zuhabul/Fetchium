"use client";
import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import Link from "next/link";
import dynamic from "next/dynamic";

const NeuralCanvas = dynamic(() => import("./NeuralCanvas"), { ssr: false });

const STATS = [
  { value: "17", label: "Novel algorithms" },
  { value: "930+", label: "Tests passing" },
  { value: "5", label: "Search backends" },
  { value: "<200ms", label: "Median latency" },
];

const TYPED = ["intelligent search.", "deep research.", "content extraction.", "AI-ready context.", "neural ranking."];

function TypedText() {
  const [idx, setIdx] = useState(0);
  const [chars, setChars] = useState(0);
  const [del, setDel] = useState(false);
  useEffect(() => {
    const word = TYPED[idx];
    const t = setTimeout(() => {
      if (!del) {
        if (chars < word.length) setChars(c => c + 1);
        else setTimeout(() => setDel(true), 1800);
      } else {
        if (chars > 0) setChars(c => c - 1);
        else { setDel(false); setIdx(i => (i + 1) % TYPED.length); }
      }
    }, del ? 35 : 68);
    return () => clearTimeout(t);
  }, [chars, del, idx]);
  return (
    <span className="gradient-text-purple">
      {TYPED[idx].slice(0, chars)}
      <span className="text-indigo-400 animate-pulse">|</span>
    </span>
  );
}

export default function Hero() {
  return (
    <section className="relative min-h-screen flex flex-col items-center justify-center overflow-hidden">
      {/* Three.js neural network */}
      <div className="absolute inset-0"><NeuralCanvas /></div>

      {/* Atmosphere glows */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[600px] bg-indigo-600/10 blur-[130px] rounded-full" />
        <div className="absolute top-1/3 left-1/4 w-[350px] h-[350px] bg-violet-700/8 blur-[90px] rounded-full" />
        <div className="absolute top-1/3 right-1/4 w-[280px] h-[280px] bg-indigo-500/8 blur-[80px] rounded-full" />
      </div>

      {/* Grid */}
      <div className="absolute inset-0 pointer-events-none"
        style={{
          backgroundImage: "linear-gradient(rgba(99,102,241,0.035) 1px, transparent 1px), linear-gradient(90deg, rgba(99,102,241,0.035) 1px, transparent 1px)",
          backgroundSize: "72px 72px",
        }}
      />

      <div className="relative z-10 max-w-5xl mx-auto px-6 text-center pt-32 pb-20">
        {/* Badge */}
        <motion.div initial={{ opacity:0, y:20 }} animate={{ opacity:1, y:0 }} transition={{ duration:0.6 }}
          className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full border border-indigo-500/25 bg-indigo-500/8 text-indigo-300 text-sm mb-8 cursor-default">
          <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
          Open Beta · 17 novel search algorithms · Free to start
        </motion.div>

        {/* Headline */}
        <motion.h1 initial={{ opacity:0, y:30 }} animate={{ opacity:1, y:0 }} transition={{ duration:0.7, delay:0.1 }}
          className="text-5xl sm:text-6xl lg:text-[72px] font-bold tracking-tight leading-[1.07] mb-6">
          <span className="gradient-text">The search API</span><br />
          <span className="text-slate-200">built for </span><TypedText />
        </motion.h1>

        {/* Sub */}
        <motion.p initial={{ opacity:0, y:24 }} animate={{ opacity:1, y:0 }} transition={{ duration:0.7, delay:0.2 }}
          className="text-lg sm:text-xl text-slate-400 max-w-2xl mx-auto mb-10 leading-relaxed">
          Federate 5 search backends, extract structured content, run multi-step research,
          and get token-budgeted AI-ready context — through a single authenticated API call.
        </motion.p>

        {/* CTAs */}
        <motion.div initial={{ opacity:0, y:20 }} animate={{ opacity:1, y:0 }} transition={{ duration:0.6, delay:0.3 }}
          className="flex flex-wrap gap-4 justify-center mb-14">
          <Link href="https://app.hypersearchx.zuhabul.com"
            className="group inline-flex items-center gap-2 px-7 py-3.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-sm transition-all duration-200 shadow-glow hover:shadow-[0_0_50px_rgba(99,102,241,0.5)]">
            Get API Key — Free
            <svg className="w-4 h-4 transition-transform group-hover:translate-x-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </Link>
          <Link href="/docs"
            className="inline-flex items-center gap-2 px-7 py-3.5 rounded-xl glass text-slate-200 font-semibold text-sm transition-all duration-200 hover:border-indigo-500/35 hover:bg-indigo-500/5">
            <svg className="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            View Docs
          </Link>
          <a href="https://github.com/zuhabul/hypersearchx" target="_blank" rel="noopener noreferrer"
            className="inline-flex items-center gap-2 px-7 py-3.5 rounded-xl glass text-slate-300 font-semibold text-sm transition-all duration-200 hover:border-white/20 hover:text-white">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
            </svg>
            GitHub
          </a>
        </motion.div>

        {/* Terminal snippet */}
        <motion.div initial={{ opacity:0, y:30 }} animate={{ opacity:1, y:0 }} transition={{ duration:0.8, delay:0.4 }}
          className="glass rounded-2xl border border-indigo-500/15 overflow-hidden max-w-2xl mx-auto text-left mb-16">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/[0.06] bg-white/[0.015]">
            <div className="flex gap-1.5">
              <div className="w-3 h-3 rounded-full bg-red-500/50" />
              <div className="w-3 h-3 rounded-full bg-yellow-500/50" />
              <div className="w-3 h-3 rounded-full bg-green-500/50" />
            </div>
            <span className="text-xs text-slate-500 font-mono">POST /v1/search</span>
            <div className="text-xs text-emerald-400 font-mono">200 OK</div>
          </div>
          <div className="p-4 text-[13px] font-mono overflow-x-auto">
            <div className="flex">
              <span className="text-slate-600 select-none mr-4 text-right" style={{minWidth:"1.5rem"}}>1</span>
              <span><span className="token-keyword">curl</span> <span className="token-operator">-X</span> <span className="token-string">POST</span> https://api.hypersearchx.zuhabul.com/v1/search <span className="token-operator">\</span></span>
            </div>
            <div className="flex">
              <span className="text-slate-600 select-none mr-4 text-right" style={{minWidth:"1.5rem"}}>2</span>
              <span>  <span className="token-operator">-H</span> <span className="token-string">&quot;Authorization: Bearer hsx_your_key&quot;</span> <span className="token-operator">\</span></span>
            </div>
            <div className="flex">
              <span className="text-slate-600 select-none mr-4 text-right" style={{minWidth:"1.5rem"}}>3</span>
              <span>  <span className="token-operator">-d</span> <span className="token-punctuation">{"{"}</span><span className="token-string">&quot;query&quot;</span><span className="token-punctuation">:</span><span className="token-string">&quot;best async rust frameworks 2025&quot;</span><span className="token-punctuation">,</span><span className="token-string">&quot;tier&quot;</span><span className="token-punctuation">:</span><span className="token-string">&quot;detailed&quot;</span><span className="token-punctuation">{"}"}</span></span>
            </div>
          </div>
        </motion.div>

        {/* Stats */}
        <motion.div initial={{ opacity:0 }} animate={{ opacity:1 }} transition={{ delay:0.6 }}
          className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          {STATS.map((s, i) => (
            <motion.div key={i} initial={{ opacity:0, scale:0.9 }} animate={{ opacity:1, scale:1 }} transition={{ delay:0.55 + i*0.07 }}
              className="glass-card rounded-xl px-4 py-5 text-center">
              <div className="text-2xl font-bold gradient-text">{s.value}</div>
              <div className="text-xs text-slate-500 mt-1">{s.label}</div>
            </motion.div>
          ))}
        </motion.div>
      </div>

      {/* Bottom fade */}
      <div className="absolute bottom-0 inset-x-0 h-48 bg-gradient-to-t from-[#06070d] to-transparent pointer-events-none" />

      {/* Scroll hint */}
      <motion.div initial={{ opacity:0 }} animate={{ opacity:1 }} transition={{ delay:1.2 }}
        className="absolute bottom-10 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 text-slate-600">
        <motion.div animate={{ y:[0,8,0] }} transition={{ duration:2, repeat:Infinity }}
          className="w-5 h-8 rounded-full border border-slate-700 flex items-start justify-center pt-1.5">
          <div className="w-1 h-2 rounded-full bg-slate-500" />
        </motion.div>
      </motion.div>
    </section>
  );
}
