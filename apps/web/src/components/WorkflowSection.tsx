"use client";

import { motion } from "framer-motion";
import Link from "next/link";
import { ArrowRight } from "lucide-react";

/* ── SVG Illustrations ─────────────────────────────────────────── */

function RagIllustration() {
  return (
    <svg viewBox="0 0 280 180" className="w-full h-full" aria-hidden>
      <defs>
        <radialGradient id="rg-center" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="#22d3ee" stopOpacity="0.9" />
          <stop offset="60%" stopColor="#0891b2" stopOpacity="0.4" />
          <stop offset="100%" stopColor="#06b6d4" stopOpacity="0" />
        </radialGradient>
        <radialGradient id="rg-node" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="#67e8f9" stopOpacity="1" />
          <stop offset="100%" stopColor="#0e7490" stopOpacity="0.3" />
        </radialGradient>
      </defs>
      {/* Connection lines */}
      {[
        [140, 90, 60, 40], [140, 90, 220, 40], [140, 90, 40, 130],
        [140, 90, 240, 130], [140, 90, 100, 155], [140, 90, 180, 155],
        [60, 40, 30, 15], [60, 40, 95, 18], [220, 40, 255, 18],
        [220, 40, 190, 18], [40, 130, 15, 110], [40, 130, 20, 155],
        [240, 130, 265, 108], [240, 130, 260, 155],
      ].map(([x1, y1, x2, y2], i) => (
        <line key={i} x1={x1} y1={y1} x2={x2} y2={y2}
          stroke="#22d3ee" strokeWidth="0.8" strokeOpacity="0.3" />
      ))}
      {/* Outer nodes */}
      {[
        [60, 40], [220, 40], [40, 130], [240, 130], [100, 155], [180, 155],
        [30, 15], [95, 18], [255, 18], [190, 18],
        [15, 110], [20, 155], [265, 108], [260, 155],
      ].map(([cx, cy], i) => (
        <circle key={i} cx={cx} cy={cy} r={i < 6 ? 5 : 3}
          fill="#0e7490" stroke="#22d3ee" strokeWidth={i < 6 ? "1" : "0.6"}
          strokeOpacity="0.7" fillOpacity="0.8" />
      ))}
      {/* Center glow */}
      <circle cx="140" cy="90" r="45" fill="url(#rg-center)" opacity="0.4" />
      {/* Center node */}
      <circle cx="140" cy="90" r="14" fill="url(#rg-node)" />
      <circle cx="140" cy="90" r="8" fill="#67e8f9" fillOpacity="0.9" />
      <circle cx="140" cy="90" r="4" fill="#e0f2fe" />
    </svg>
  );
}

function ResearchIllustration() {
  return (
    <svg viewBox="0 0 280 180" className="w-full h-full" aria-hidden>
      <defs>
        <radialGradient id="rs-glow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="#818cf8" stopOpacity="0.6" />
          <stop offset="70%" stopColor="#4338ca" stopOpacity="0.15" />
          <stop offset="100%" stopColor="#312e81" stopOpacity="0" />
        </radialGradient>
      </defs>
      <circle cx="140" cy="90" r="70" fill="url(#rs-glow)" />
      {[55, 42, 30, 20].map((r, i) => (
        <circle key={i} cx="140" cy="90" r={r}
          fill="none" stroke="#6366f1"
          strokeWidth={i === 0 ? "1.5" : "0.8"}
          strokeOpacity={0.6 - i * 0.1}
          strokeDasharray={i % 2 === 1 ? "4 3" : "none"} />
      ))}
      {/* Orbit dots */}
      {[
        [140, 35], [183, 63], [183, 117], [140, 145], [97, 117], [97, 63],
      ].map(([cx, cy], i) => (
        <circle key={i} cx={cx} cy={cy} r="4"
          fill="#818cf8" fillOpacity="0.85"
          stroke="#c7d2fe" strokeWidth="0.5" />
      ))}
      {/* Outer ring markers */}
      {[
        [140, 20], [203, 55], [203, 125], [140, 160], [77, 125], [77, 55],
      ].map(([cx, cy], i) => (
        <circle key={i} cx={cx} cy={cy} r="2.5"
          fill="#4f46e5" fillOpacity="0.7" />
      ))}
      <circle cx="140" cy="90" r="13" fill="#312e81" stroke="#818cf8" strokeWidth="1.5" />
      <circle cx="140" cy="90" r="6" fill="#a5b4fc" />
    </svg>
  );
}

function SocialIllustration() {
  return (
    <svg viewBox="0 0 280 180" className="w-full h-full" aria-hidden>
      {/* Mock UI panel */}
      <rect x="30" y="18" width="220" height="144" rx="8"
        fill="#0f172a" stroke="#334155" strokeWidth="1" />
      {/* Title bar */}
      <rect x="30" y="18" width="220" height="28" rx="8"
        fill="#1e293b" />
      <rect x="30" y="34" width="220" height="12" fill="#1e293b" />
      <circle cx="48" cy="32" r="4" fill="#ef4444" fillOpacity="0.7" />
      <circle cx="62" cy="32" r="4" fill="#f59e0b" fillOpacity="0.7" />
      <circle cx="76" cy="32" r="4" fill="#22c55e" fillOpacity="0.7" />
      <text x="140" y="36" textAnchor="middle" fill="#94a3b8" fontSize="8" fontFamily="monospace">trending</text>
      {/* Data rows */}
      {[0, 1, 2, 3, 4].map((i) => (
        <g key={i}>
          <rect x="46" y={58 + i * 20} width={80 + (i % 3) * 20} height="6"
            rx="2" fill="#334155" />
          <rect x={46 + 90 + (i % 3) * 20 + 8} y={58 + i * 20} width={30 - (i % 2) * 8} height="6"
            rx="2" fill={["#22d3ee", "#818cf8", "#34d399", "#f472b6", "#fb923c"][i]}
            fillOpacity="0.7" />
        </g>
      ))}
      {/* Mini bar chart */}
      {[18, 28, 14, 35, 22, 30, 25].map((h, i) => (
        <rect key={i} x={180 + i * 9} y={130 - h} width="6" height={h}
          rx="1.5" fill="#6366f1" fillOpacity={0.5 + i * 0.07} />
      ))}
    </svg>
  );
}

function VideoIllustration() {
  return (
    <svg viewBox="0 0 280 180" className="w-full h-full" aria-hidden>
      <defs>
        <linearGradient id="vid-bg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#431407" />
          <stop offset="100%" stopColor="#7c2d12" />
        </linearGradient>
        <radialGradient id="vid-glow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="#fb923c" stopOpacity="0.3" />
          <stop offset="100%" stopColor="#c2410c" stopOpacity="0" />
        </radialGradient>
      </defs>
      <rect x="0" y="0" width="280" height="180" fill="url(#vid-bg)" />
      <circle cx="140" cy="84" r="80" fill="url(#vid-glow)" />
      {/* Screen frame */}
      <rect x="50" y="30" width="180" height="110" rx="10"
        fill="#1c0a00" stroke="#c2410c" strokeWidth="1.5" strokeOpacity="0.6" />
      {/* Play button circle */}
      <circle cx="140" cy="85" r="30"
        fill="#1c0a00" stroke="#fb923c" strokeWidth="1.5" strokeOpacity="0.8" />
      <polygon points="132,72 132,98 158,85" fill="#fb923c" fillOpacity="0.9" />
      {/* Progress bar */}
      <rect x="65" y="120" width="150" height="3" rx="1.5" fill="#374151" />
      <rect x="65" y="120" width="60" height="3" rx="1.5" fill="#fb923c" fillOpacity="0.8" />
      <circle cx="125" cy="121.5" r="4" fill="#fb923c" />
      {/* Transcript line */}
      <rect x="65" y="148" width="100" height="5" rx="2" fill="#374151" />
      <rect x="170" y="148" width="45" height="5" rx="2" fill="#374151" />
    </svg>
  );
}

/* ── Card data ─────────────────────────────────────────────────── */

const workflows = [
  {
    id: "rag",
    title: "RAG Pipelines",
    description: "Feed your vector DB with up-to-the-minute web data.",
    Illustration: RagIllustration,
    bg: "bg-[#040d14]",
    border: "border-cyan-500/20 hover:border-cyan-400/40",
    glow: "bg-cyan-500/5",
    href: "/product/search",
  },
  {
    id: "research",
    title: "Deep Research",
    description: "Multi-step autonomous browsing for complex queries.",
    Illustration: ResearchIllustration,
    bg: "bg-[#06070f]",
    border: "border-indigo-500/20 hover:border-indigo-400/40",
    glow: "bg-indigo-500/5",
    href: "/product/research",
  },
  {
    id: "social",
    title: "Social Monitoring",
    description: "Track trends and sentiment across X and LinkedIn.",
    Illustration: SocialIllustration,
    bg: "bg-[#070c14]",
    border: "border-slate-500/20 hover:border-slate-400/35",
    glow: "bg-slate-500/5",
    href: "https://docs.fetchium.com/api/social",
  },
  {
    id: "video",
    title: "Video Intel",
    description: "Search and transcribe within YouTube videos instantly.",
    Illustration: VideoIllustration,
    bg: "bg-[#0d0500]",
    border: "border-orange-500/25 hover:border-orange-400/45",
    glow: "bg-orange-500/5",
    href: "https://docs.fetchium.com/api/youtube",
  },
];

/* ── Component ─────────────────────────────────────────────────── */

export default function WorkflowSection() {
  return (
    <section className="relative py-16 sm:py-24 px-4 overflow-hidden">
      {/* Ambient glow */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[400px] bg-indigo-600/6 blur-[120px] rounded-full" />
      </div>

      <div className="relative mx-auto max-w-7xl">
        {/* Header row */}
        <motion.div
          className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 mb-10"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-60px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="max-w-xl">
            <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight text-slate-100">
              Built for every AI workflow
            </h2>
            <p className="mt-4 text-base sm:text-lg text-slate-300 leading-relaxed">
              From real-time search to complex research agents, Fetchium
              provides the foundation for reliable AI output.
            </p>
          </div>
          <Link
            href="/product/search"
            className="group inline-flex items-center gap-1.5 text-base font-semibold text-indigo-400 hover:text-indigo-300 transition-colors whitespace-nowrap shrink-0"
          >
            Explore use cases
            <ArrowRight className="w-5 h-5 transition-transform group-hover:translate-x-0.5" />
          </Link>
        </motion.div>

        {/* Cards */}
        <motion.div
          className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4"
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-40px" }}
          variants={{
            hidden: {},
            visible: { transition: { staggerChildren: 0.09 } },
          }}
        >
          {workflows.map((w) => {
            const Illus = w.Illustration;
            return (
              <motion.div
                key={w.id}
                variants={{
                  hidden: { opacity: 0, y: 28 },
                  visible: {
                    opacity: 1, y: 0,
                    transition: { duration: 0.5, ease: [0.22, 1, 0.36, 1] },
                  },
                }}
              >
                <Link
                  href={w.href}
                  className={`group flex flex-col rounded-2xl border ${w.border} ${w.bg} overflow-hidden transition-all duration-300 hover:shadow-[0_12px_40px_rgba(0,0,0,0.5)] hover:-translate-y-0.5`}
                >
                  {/* Illustration area */}
                  <div className={`relative h-44 w-full overflow-hidden ${w.glow}`}>
                    <Illus />
                    {/* Subtle inner vignette */}
                    <div className="absolute inset-0 bg-gradient-to-t from-black/40 via-transparent to-transparent pointer-events-none" />
                  </div>

                  {/* Text area */}
                  <div className="px-5 py-4 border-t border-slate-800">
                    <h3 className="text-[17px] font-bold text-slate-100 mb-2 group-hover:text-white transition-colors">
                      {w.title}
                    </h3>
                    <p className="text-[15px] text-slate-300 leading-relaxed">
                      {w.description}
                    </p>
                  </div>
                </Link>
              </motion.div>
            );
          })}
        </motion.div>
      </div>
    </section>
  );
}
