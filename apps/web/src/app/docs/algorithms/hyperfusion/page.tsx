import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = { title: "HyperFusion Ranking Algorithm" };

export default function HyperFusionPage() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Algorithms</div>
      <h1>HyperFusion Ranking</h1>
      <p>
        HyperFusion is Fetchium&apos;s core ranking algorithm — an 8-signal neural fusion
        system that combines lexical, semantic, temporal, and structural signals into a
        single 0–1 relevance score. It eliminates the need to manually tune ranking
        parameters and adapts to query intent automatically.
      </p>

      <h2>The 8 signals</h2>

      <div className="not-prose grid grid-cols-1 gap-3 my-6">
        {[
          {
            name: "BM25", label: "Lexical relevance",
            color: "bg-indigo-500/10 border-indigo-500/20",
            desc: "Classic Okapi BM25 TF-IDF variant tuned for web content. Scores keyword overlap between query and document with length normalization. Fast and highly reliable for exact-match queries.",
          },
          {
            name: "Semantic", label: "Embedding similarity",
            color: "bg-violet-500/10 border-violet-500/20",
            desc: "Cosine similarity between query and document embeddings. Captures meaning beyond keywords — finds relevant content even when query and document use different vocabulary.",
          },
          {
            name: "Temporal", label: "Freshness decay",
            color: "bg-sky-500/10 border-sky-500/20",
            desc: "Exponential decay function that rewards recent content. Decay rate adapts to query intent: current-events queries decay fast (hours); reference queries decay slowly (years).",
          },
          {
            name: "Authority", label: "Domain trust",
            color: "bg-emerald-500/10 border-emerald-500/20",
            desc: "Source trust score maintained by the Persistent Intelligence Engine (PIE). Combines domain reputation, historical accuracy, and community citation frequency.",
          },
          {
            name: "Evidence", label: "Cross-source corroboration",
            color: "bg-amber-500/10 border-amber-500/20",
            desc: "Evidence Graph Builder (EGB) signal. Scores how many independent sources corroborate the same facts. Higher corroboration → higher confidence. Penalizes isolated claims.",
          },
          {
            name: "Diversity", label: "MMR penalty",
            color: "bg-rose-500/10 border-rose-500/20",
            desc: "Maximal Marginal Relevance penalty applied to results too similar to already-selected results. Ensures the final result set covers multiple perspectives and sources.",
          },
          {
            name: "Depth", label: "Content richness",
            color: "bg-orange-500/10 border-orange-500/20",
            desc: "Rewards comprehensive content. Signals include content length, heading structure, code block presence, table richness, citation density, and reading level.",
          },
          {
            name: "Consensus", label: "Community agreement",
            color: "bg-teal-500/10 border-teal-500/20",
            desc: "Social signal from Reddit upvotes, HackerNews points, GitHub stars, and StackOverflow accepted answer status. Measures whether practitioners endorse the content.",
          },
        ].map(s => (
          <div key={s.name} className={`rounded-xl p-4 border ${s.color}`}>
            <div className="flex items-center gap-2 mb-1">
              <span className="text-slate-100 font-semibold text-sm">{s.name}</span>
              <span className="text-xs text-slate-500">{s.label}</span>
            </div>
            <p className="text-slate-400 text-sm leading-relaxed m-0">{s.desc}</p>
          </div>
        ))}
      </div>

      <h2>Fusion formula</h2>
      <p>
        The 8 signals are combined using a learned weighted sum, where weights are
        adjusted per query-intent class:
      </p>

      <div className="not-prose my-4 p-4 rounded-xl bg-white/[0.02] border border-white/[0.06] font-mono text-sm">
        <div className="text-slate-300">
          score = w₁·BM25 + w₂·Semantic + w₃·Temporal + w₄·Authority
        </div>
        <div className="text-slate-300 mt-1 ml-8">
          + w₅·Evidence + w₆·Diversity + w₇·Depth + w₈·Consensus
        </div>
        <div className="text-slate-500 text-xs mt-3">where Σwᵢ = 1.0 and weights vary by QueryIntent</div>
      </div>

      <h2>Intent-based weight adaptation</h2>
      <table>
        <thead><tr><th>Query intent</th><th>Dominant signals</th><th>Example</th></tr></thead>
        <tbody>
          <tr><td>Factual</td><td>BM25, Evidence, Authority</td><td>"What is Rust?"</td></tr>
          <tr><td>CurrentEvents</td><td>Temporal, BM25, Consensus</td><td>"latest Rust release"</td></tr>
          <tr><td>Code / HowTo</td><td>BM25, Depth, Authority</td><td>"how to use tokio::select"</td></tr>
          <tr><td>Academic</td><td>Authority, Evidence, Depth</td><td>"transformer architecture paper"</td></tr>
          <tr><td>Opinion</td><td>Consensus, Diversity, Semantic</td><td>"is Rust worth learning"</td></tr>
          <tr><td>Comparison</td><td>Diversity, Depth, Evidence</td><td>"Rust vs Go performance"</td></tr>
        </tbody>
      </table>

      <h2>Performance characteristics</h2>
      <ul>
        <li>Ranking latency: &lt;2ms for 50 results (pure CPU, no GPU required)</li>
        <li>Signal computation is fully parallel via Rayon</li>
        <li>Semantic signal computed lazily (only when BM25 is insufficient)</li>
        <li>SPRE pre-ranking filters top-100 candidates before full HyperFusion scoring</li>
      </ul>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/algorithms/spre", title: "SPRE Pre-ranking", desc: "Speculative pre-ranking for speed" },
          { href: "/docs/algorithms/cep", title: "CEP Extraction", desc: "Content extraction pipeline" },
          { href: "/docs/algorithms/qatbe", title: "QATBE", desc: "Token-budgeted extraction" },
          { href: "/docs/api/search", title: "Search API", desc: "See HyperFusion in action" },
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
