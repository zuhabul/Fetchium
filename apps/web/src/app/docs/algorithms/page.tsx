import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = { title: "Algorithms Overview" };

const ALGORITHMS = [
  {
    id: "hyperfusion",
    name: "HyperFusion",
    tagline: "8-signal neural ranking",
    desc: "Combines BM25 keyword relevance, semantic similarity, temporal freshness, domain authority, evidence strength, diversity penalty, content depth, and source consensus into a single 0–1 score.",
    badge: "Ranking",
    badgeColor: "bg-indigo-500/15 text-indigo-300 border-indigo-500/30",
  },
  {
    id: "cep",
    name: "CEP",
    tagline: "Content Extraction Protocol — 5-layer cascade",
    desc: "CSS selectors → Readability → Headless JS → PDF extraction → Screenshot OCR. Each layer is tried in order, falling back gracefully to handle any web content.",
    badge: "Extraction",
    badgeColor: "bg-violet-500/15 text-violet-300 border-violet-500/30",
  },
  {
    id: "qatbe",
    name: "QATBE",
    tagline: "Query-Aware Token-Budgeted Extraction",
    desc: "BM25-scored segment ranking combined with a greedy knapsack algorithm to pack maximum relevance within a token budget. Powers all detail tiers.",
    badge: "Tokens",
    badgeColor: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
  },
  {
    id: "hyperfusion",
    name: "SCS",
    tagline: "Semantic Content Segmentation",
    desc: "Classifies content into 8 typed segments: heading, paragraph, code, list, table, quote, metadata, other. Each type has a different token efficiency weight.",
    badge: "Extraction",
    badgeColor: "bg-violet-500/15 text-violet-300 border-violet-500/30",
  },
  {
    id: "hyperfusion",
    name: "ABS",
    tagline: "Adaptive Backend Selector",
    desc: "Analyzes query intent, complexity, and freshness requirements to automatically select the optimal combination of search backends for each request.",
    badge: "Search",
    badgeColor: "bg-sky-500/15 text-sky-300 border-sky-500/30",
  },
  {
    id: "hyperfusion",
    name: "AMRS",
    tagline: "Adaptive Multi-Agent Research Swarm",
    desc: "Spawns parallel Tokio agents (planner, searcher, extractor, synthesizer) to process complex research queries with inter-agent coordination via async channels.",
    badge: "Research",
    badgeColor: "bg-amber-500/15 text-amber-300 border-amber-500/30",
  },
  {
    id: "hyperfusion",
    name: "PIE",
    tagline: "Persistent Intelligence Engine",
    desc: "Cross-session learning via SQLite. Tracks source trust scores, failure patterns, and query prediction to improve results over time.",
    badge: "Intelligence",
    badgeColor: "bg-rose-500/15 text-rose-300 border-rose-500/30",
  },
  {
    id: "hyperfusion",
    name: "QFD",
    tagline: "Query Fingerprinting & Deduplication",
    desc: "SimHash-based fingerprinting identifies semantically equivalent queries and routes to cached results, reducing redundant backend calls.",
    badge: "Cache",
    badgeColor: "bg-slate-500/15 text-slate-300 border-slate-500/30",
  },
];

export default function AlgorithmsOverview() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Algorithms</div>
      <h1>Algorithms Overview</h1>
      <p>
        Fetchium implements 17 novel algorithms that don&apos;t exist in any other search tool.
        Each algorithm is purpose-built to solve a specific aspect of the intelligent search pipeline.
      </p>

      <div className="not-prose grid grid-cols-1 gap-4 my-8">
        {ALGORITHMS.map((alg, i) => (
          <div key={i} className="glass-card rounded-2xl p-5">
            <div className="flex items-start justify-between gap-4 mb-2">
              <div>
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="text-slate-100 font-semibold text-base">{alg.name}</h3>
                  <span className={`inline-flex items-center px-2 py-0.5 rounded text-[11px] font-semibold border ${alg.badgeColor}`}>
                    {alg.badge}
                  </span>
                </div>
                <div className="text-xs text-indigo-400 font-mono mb-2">{alg.tagline}</div>
              </div>
            </div>
            <p className="text-slate-400 text-sm leading-relaxed">{alg.desc}</p>
          </div>
        ))}
      </div>

      <h2>Algorithm pipeline</h2>
      <p>Algorithms execute in a defined pipeline order for each request type:</p>

      <div className="not-prose my-6 p-5 rounded-2xl bg-white/[0.02] border border-white/[0.06] font-mono text-sm">
        <div className="text-slate-400 mb-4 text-xs uppercase tracking-wider">Search pipeline</div>
        {[
          "QFD → Cache lookup (skip if miss)",
          "QCE → Query complexity analysis",
          "QXE → Query expansion",
          "CLQB → Cross-lingual query building",
          "ABS → Backend selection",
          "LP → Latency prediction",
          "SPRE → Speculative pre-ranking",
          "Multi-backend parallel fetch",
          "CEP → Content extraction per source",
          "SCS → Semantic segmentation",
          "QATBE → Token-budgeted extraction",
          "HyperFusion → 8-signal neural ranking",
          "RCE → Result clustering + dedup",
          "RDO → MMR diversity optimization",
          "SSE → Smart snippet generation",
          "EGB → Evidence graph building",
          "PIE → Intelligence update",
        ].map((step, i) => (
          <div key={i} className="flex items-center gap-3 py-1.5 border-b border-white/[0.04] last:border-0">
            <span className="text-slate-600 w-6 text-right text-xs">{i + 1}</span>
            <span className="text-slate-300">{step}</span>
          </div>
        ))}
      </div>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "See algorithms in action" },
          { href: "https://docs.fetchium.com/api/research", title: "Research API", desc: "AMRS research swarm" },
          { href: "https://docs.fetchium.com/quickstart", title: "Quick Start", desc: "Make your first API call" },
          { href: "https://docs.fetchium.com/self-hosting/docker", title: "Self-Hosting", desc: "Run your own instance" },
        ].map(l => (
          <Link key={l.href} href={l.href} className="glass-card rounded-xl p-4 no-underline group">
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{l.title} →</div>
            <div className="text-xs text-slate-500 mt-1">{l.desc}</div>
          </Link>
        ))}
      </div>
    </article>
  );
}
