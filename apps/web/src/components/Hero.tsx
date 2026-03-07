"use client";
import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";
import dynamic from "next/dynamic";

const NeuralCanvas = dynamic(() => import("./NeuralCanvas"), { ssr: false });

/* ─── Word Rotator ────────────────────────────────────────────────────────── */
const WORDS = ["thinks.", "learns.", "reasons.", "adapts.", "ranks."];

function WordRotator() {
  const [idx, setIdx] = useState(0);
  useEffect(() => {
    const t = setInterval(() => setIdx(i => (i + 1) % WORDS.length), 2400);
    return () => clearInterval(t);
  }, []);
  return (
    <span
      className="relative inline-block"
      style={{ height: "1.15em", verticalAlign: "baseline", clipPath: "inset(0)" }}
    >
      <AnimatePresence mode="popLayout" initial={false}>
        <motion.span
          key={WORDS[idx]}
          className="block gradient-text-purple whitespace-nowrap"
          initial={{ y: "110%" }}
          animate={{ y: "0%" }}
          exit={{ y: "-110%" }}
          transition={{ duration: 0.4, ease: [0.22, 1, 0.36, 1] }}
        >
          {WORDS[idx]}
        </motion.span>
      </AnimatePresence>
    </span>
  );
}

/* ─── SVG Flow Diagram — no box, just floating nodes ─────────────────────── */
const SOURCES = [
  { label: "DuckDuckGo", color: "#f06292" },
  { label: "Brave Search", color: "#fb8c00" },
  { label: "Bing",        color: "#4fc3f7" },
  { label: "Kagi",        color: "#a78bfa" },
  { label: "YouTube",     color: "#ff5252" },
  { label: "Arxiv",       color: "#66bb6a" },
  { label: "+5 more…",   color: "#546e7a" },
];

const OUTPUTS = [
  { label: "Ranked Results",     color: "#818cf8", icon: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" },
  { label: "Extracted Content",  color: "#34d399", icon: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" },
  { label: "Auto Citations",     color: "#f59e0b", icon: "M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" },
  { label: "Evidence Graph",     color: "#e879f9", icon: "M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" },
];

// SVG coordinate system: viewBox="0 0 560 360"
// Sources: left side (node circles at x≈55, y spread from 40 to 320)
// Center orb: (280, 180)
// Outputs: right side (x≈505, y spread from 80 to 280)
const SRC_X = 60;
const CTR_X = 280;
const CTR_Y = 180;
const OUT_X = 500;

const srcY = [40, 85, 130, 180, 230, 275, 320];
const outY = [85, 145, 215, 275];

function FlowDiagram() {
  return (
    <motion.div
      className="relative w-full select-none"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 1, delay: 0.4 }}
    >
      <svg
        viewBox="0 0 560 360"
        className="w-full h-auto"
        style={{ overflow: "visible" }}
        aria-hidden="true"
      >
        <defs>
          {/* Animated dash for source → center paths */}
          <style>{`
            @keyframes flowIn {
              from { stroke-dashoffset: 200; }
              to   { stroke-dashoffset: 0; }
            }
            @keyframes flowOut {
              from { stroke-dashoffset: 200; }
              to   { stroke-dashoffset: 0; }
            }
            @keyframes orbPulse {
              0%, 100% { opacity: 0.5; r: 38px; }
              50%       { opacity: 0.15; r: 52px; }
            }
            @keyframes dotFlow {
              0%   { offset-distance: 0%;   opacity: 0; }
              5%   { opacity: 1; }
              90%  { opacity: 1; }
              100% { offset-distance: 100%; opacity: 0; }
            }
          `}</style>

          {/* Radial glow filter for center orb */}
          <filter id="orbGlow" x="-60%" y="-60%" width="220%" height="220%">
            <feGaussianBlur stdDeviation="8" result="blur" />
            <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
          <filter id="nodeGlow" x="-40%" y="-40%" width="180%" height="180%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
        </defs>

        {/* ── Source → Center paths ── */}
        {SOURCES.map((src, i) => {
          const sy = srcY[i];
          const cpx = CTR_X - 60;
          const d = `M ${SRC_X} ${sy} C ${cpx} ${sy} ${cpx} ${CTR_Y} ${CTR_X} ${CTR_Y}`;
          const dur = 2.2 + i * 0.3;
          return (
            <g key={`src-path-${i}`}>
              {/* Base dim path */}
              <path d={d} fill="none" stroke={src.color} strokeWidth="1" strokeOpacity="0.15" />
              {/* Animated flowing path */}
              <path
                d={d}
                fill="none"
                stroke={src.color}
                strokeWidth="1.5"
                strokeOpacity="0.55"
                strokeDasharray="6 180"
                style={{
                  animation: `flowIn ${dur}s ease-in-out infinite`,
                  animationDelay: `${i * 0.4}s`,
                }}
              />
            </g>
          );
        })}

        {/* ── Center → Output paths ── */}
        {OUTPUTS.map((out, i) => {
          const oy = outY[i];
          const cpx = CTR_X + 60;
          const d = `M ${CTR_X} ${CTR_Y} C ${cpx} ${CTR_Y} ${cpx} ${oy} ${OUT_X} ${oy}`;
          const dur = 2.0 + i * 0.25;
          return (
            <g key={`out-path-${i}`}>
              <path d={d} fill="none" stroke={out.color} strokeWidth="1" strokeOpacity="0.15" />
              <path
                d={d}
                fill="none"
                stroke={out.color}
                strokeWidth="1.5"
                strokeOpacity="0.55"
                strokeDasharray="6 180"
                style={{
                  animation: `flowOut ${dur}s ease-in-out infinite`,
                  animationDelay: `${0.8 + i * 0.35}s`,
                }}
              />
            </g>
          );
        })}

        {/* ── Center orb ── */}
        {/* Outer pulse ring */}
        <circle
          cx={CTR_X} cy={CTR_Y} r="52"
          fill="none"
          stroke="rgba(99,102,241,0.18)"
          strokeWidth="1"
          style={{ animation: "orbPulse 3s ease-in-out infinite" }}
        />
        {/* Mid ring */}
        <circle
          cx={CTR_X} cy={CTR_Y} r="40"
          fill="rgba(99,102,241,0.08)"
          stroke="rgba(99,102,241,0.30)"
          strokeWidth="1"
          filter="url(#orbGlow)"
        />
        {/* Inner solid */}
        <circle
          cx={CTR_X} cy={CTR_Y} r="30"
          fill="rgba(99,102,241,0.18)"
          stroke="rgba(99,102,241,0.60)"
          strokeWidth="1.5"
        />
        {/* "F" label */}
        <text
          x={CTR_X} y={CTR_Y - 6}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="#a5b4fc"
          fontSize="16"
          fontWeight="800"
          fontFamily="system-ui, sans-serif"
        >
          F
        </text>
        <text
          x={CTR_X} y={CTR_Y + 10}
          textAnchor="middle"
          fill="#6366f1"
          fontSize="7"
          fontWeight="600"
          fontFamily="system-ui, sans-serif"
          letterSpacing="1"
        >
          FETCHIUM
        </text>

        {/* ── Source nodes ── */}
        {SOURCES.map((src, i) => (
          <g key={`src-node-${i}`} filter="url(#nodeGlow)">
            <circle cx={SRC_X} cy={srcY[i]} r="5" fill={src.color} fillOpacity="0.8" />
            <text
              x={SRC_X - 12}
              y={srcY[i]}
              textAnchor="end"
              dominantBaseline="middle"
              fill={src.color}
              fontSize="11"
              fontWeight="600"
              fontFamily="system-ui, sans-serif"
              fillOpacity="0.9"
            >
              {src.label}
            </text>
          </g>
        ))}

        {/* ── Output nodes ── */}
        {OUTPUTS.map((out, i) => (
          <g key={`out-node-${i}`}>
            <circle cx={OUT_X} cy={outY[i]} r="5" fill={out.color} fillOpacity="0.8" />
            <text
              x={OUT_X + 12}
              y={outY[i]}
              textAnchor="start"
              dominantBaseline="middle"
              fill={out.color}
              fontSize="11"
              fontWeight="600"
              fontFamily="system-ui, sans-serif"
              fillOpacity="0.9"
            >
              {out.label}
            </text>
          </g>
        ))}

        {/* ── Stats below orb ── */}
        <text x={CTR_X} y={CTR_Y + 54} textAnchor="middle" fill="#475569" fontSize="9" fontFamily="system-ui, sans-serif">
          17 algorithms · ~500ms P50
        </text>
      </svg>
    </motion.div>
  );
}

/* ─── Hero ────────────────────────────────────────────────────────────────── */
export default function Hero() {
  return (
    <section className="relative min-h-screen flex flex-col items-center justify-center overflow-hidden">
      <div className="absolute inset-0"><NeuralCanvas /></div>

      {/* Atmosphere glows */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[1100px] h-[650px] rounded-full"
          style={{ background: "radial-gradient(ellipse, rgba(99,102,241,0.12) 0%, transparent 70%)", filter: "blur(40px)" }}
        />
        <div className="absolute top-1/3 left-[15%] w-[400px] h-[400px] rounded-full"
          style={{ background: "radial-gradient(ellipse, rgba(139,92,246,0.07) 0%, transparent 70%)", filter: "blur(60px)" }}
        />
        <div className="absolute top-1/3 right-[10%] w-[350px] h-[350px] rounded-full"
          style={{ background: "radial-gradient(ellipse, rgba(99,102,241,0.08) 0%, transparent 70%)", filter: "blur(60px)" }}
        />
      </div>

      {/* Subtle grid */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          backgroundImage:
            "linear-gradient(rgba(99,102,241,0.04) 1px,transparent 1px),linear-gradient(90deg,rgba(99,102,241,0.04) 1px,transparent 1px)",
          backgroundSize: "72px 72px",
        }}
      />

      {/*
        Hero is full-viewport-width.
        Left column: slightly more padding than before (lg:pl-12 xl:pl-20).
        Other sections use max-w-7xl → content starts ~104px from edge.
        Hero at 1440px: ~1376px content. Clear visual difference.
      */}
      <div className="relative z-10 w-full pt-24 sm:pt-28 pb-16">
        <div className="flex flex-col lg:flex-row lg:items-center gap-10 lg:gap-0">

          {/* ── LEFT: copy + CTAs + stats ─────────────────────────────────────
              Extra left padding so text aligns with navbar logo.
              xl:pl-20 = slightly more breathing room than before (was pl-14).
          ─────────────────────────────────────────────────────────────────── */}
          <div className="w-full lg:w-[46%] xl:w-[44%] shrink-0 px-5 sm:px-8 lg:pl-12 xl:pl-20 2xl:pl-28 lg:pr-10 xl:pr-14 text-center lg:text-left">

            {/* Badge */}
            <motion.div
              initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.55 }}
              className="inline-flex items-center gap-2 px-3.5 py-2 rounded-full text-sm mb-6 sm:mb-7 cursor-default"
              style={{ border: "1px solid rgba(99,102,241,0.32)", background: "rgba(99,102,241,0.10)", color: "#c7d2fe" }}
            >
              <span className="w-1.5 h-1.5 rounded-full animate-pulse" style={{ background: "#34d399", boxShadow: "0 0 6px #34d399" }} />
              <span className="hidden sm:inline">Open Beta · 17 algorithms · 11+ backends · Free forever</span>
              <span className="sm:hidden">Open Beta · Free to start</span>
            </motion.div>

            {/* Headline */}
            <motion.h1
              initial={{ opacity: 0, y: 30 }} animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.7, delay: 0.1 }}
              className="text-[2.6rem] sm:text-5xl md:text-6xl lg:text-[58px] xl:text-[66px] font-bold tracking-tight leading-[1.1] mb-5"
            >
              <span className="gradient-text">The search API</span>
              <br />
              <span style={{ color: "#e2e8f0" }}>that </span>
              <WordRotator />
            </motion.h1>

            {/* Sub */}
            <motion.p
              initial={{ opacity: 0, y: 24 }} animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.7, delay: 0.2 }}
              className="text-lg sm:text-xl leading-relaxed mb-8"
              style={{ color: "#94a3b8" }}
            >
              One API call returns{" "}
              <span style={{ color: "#e2e8f0", fontWeight: 600 }}>search</span>
              {" "}+{" "}
              <span style={{ color: "#e2e8f0", fontWeight: 600 }}>extracted content</span>
              {" "}+{" "}
              <span style={{ color: "#e2e8f0", fontWeight: 600 }}>citations</span>
              {" "}— drop it into your RAG pipeline or AI agent. No scrapers, no plumbing.
            </motion.p>

            {/* CTAs */}
            <motion.div
              initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.3 }}
              className="flex flex-col sm:flex-row gap-3 justify-center lg:justify-start mb-10"
            >
              <Link
                href="https://app.fetchium.com/register"
                target="_blank"
                rel="noopener noreferrer"
                className="group inline-flex items-center justify-center gap-2 px-7 py-4 rounded-xl text-white font-bold text-base transition-all duration-200 min-h-[52px]"
                style={{ background: "linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)", boxShadow: "0 0 32px rgba(99,102,241,0.35)" }}
              >
                Get API Key — Free
                <svg className="w-4 h-4 transition-transform group-hover:translate-x-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                </svg>
              </Link>
              <Link
                href="/docs"
                className="inline-flex items-center justify-center gap-2 px-7 py-4 rounded-xl font-bold text-base transition-all duration-200 min-h-[52px]"
                style={{ border: "1px solid rgba(99,102,241,0.28)", background: "rgba(99,102,241,0.06)", color: "#c7d2fe" }}
              >
                <svg className="w-5 h-5" style={{ color: "#818cf8" }} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                View Docs
              </Link>
            </motion.div>

            {/* Stats */}
            <motion.div
              initial={{ opacity: 0 }} animate={{ opacity: 1 }}
              transition={{ delay: 0.5 }}
              className="grid grid-cols-2 gap-3"
            >
              {[
                { value: "17",     label: "Novel algorithms" },
                { value: "11+",    label: "Search backends"  },
                { value: "563+",   label: "Tests passing"    },
                { value: "~500ms", label: "P50 latency"      },
              ].map((s, i) => (
                <motion.div
                  key={i}
                  initial={{ opacity: 0, scale: 0.92 }} animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: 0.5 + i * 0.07 }}
                  className="rounded-xl px-3 py-3.5 text-center"
                  style={{ border: "1px solid rgba(99,102,241,0.18)", background: "rgba(99,102,241,0.05)" }}
                >
                  <div className="text-xl sm:text-2xl font-bold gradient-text">{s.value}</div>
                  <div className="text-[12px] sm:text-[13px] font-medium mt-1" style={{ color: "#64748b" }}>{s.label}</div>
                </motion.div>
              ))}
            </motion.div>
          </div>

          {/* ── RIGHT: SVG flow diagram — no box ──────────────────────────────
              Pure floating nodes + animated connection lines on the canvas.
              Communicates "11 sources → Fetchium core → structured output"
              without any panel or container.
          ─────────────────────────────────────────────────────────────────── */}
          <div className="w-full lg:flex-1 min-w-0 px-6 sm:px-8 lg:pl-4 lg:pr-10 xl:pr-16 2xl:pr-24">
            <FlowDiagram />
          </div>

        </div>
      </div>

      {/* Bottom fade */}
      <div
        className="absolute bottom-0 inset-x-0 h-48 pointer-events-none"
        style={{ background: "linear-gradient(to top, #06070d, transparent)" }}
      />

      {/* Scroll hint */}
      <motion.div
        initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 1.2 }}
        className="absolute bottom-6 sm:bottom-10 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2"
        style={{ color: "#334155" }}
      >
        <motion.div
          animate={{ y: [0, 8, 0] }} transition={{ duration: 2, repeat: Infinity }}
          className="w-5 h-8 rounded-full flex items-start justify-center pt-1.5"
          style={{ border: "1px solid #1e2a3a" }}
        >
          <div className="w-1 h-2 rounded-full" style={{ background: "#475569" }} />
        </motion.div>
      </motion.div>
    </section>
  );
}
