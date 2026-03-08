import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = { title: "SPRE — Speculative Pre-Ranking Engine" };

export default function SprePage() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Algorithms</div>
      <h1>SPRE — Speculative Pre-Ranking</h1>
      <p>
        <strong>Speculative Pre-Ranking Engine.</strong> SPRE dramatically reduces the cost
        of HyperFusion ranking by running a fast lightweight scorer before the full
        8-signal ranking pass. Only the top candidates proceed to full scoring.
      </p>

      <h2>The problem SPRE solves</h2>
      <p>
        A typical search returns 50–200 raw results across multiple backends. Running
        full HyperFusion on every result is expensive — especially when semantic
        embedding similarity is involved. SPRE filters this to the top 30 candidates
        before full scoring, reducing ranking time by ~70%.
      </p>

      <h2>SPRE scoring signals</h2>
      <table>
        <thead><tr><th>Signal</th><th>Weight</th><th>Cost</th></tr></thead>
        <tbody>
          <tr><td>BM25 title match</td><td>40%</td><td>~0.1ms</td></tr>
          <tr><td>URL keyword match</td><td>20%</td><td>~0.01ms</td></tr>
          <tr><td>Domain reputation (cached)</td><td>25%</td><td>~0.01ms</td></tr>
          <tr><td>Result position (backend rank)</td><td>15%</td><td>~0.01ms</td></tr>
        </tbody>
      </table>

      <h2>Pipeline integration</h2>
      <div className="not-prose my-4 p-4 rounded-xl bg-white/[0.02] border border-white/[0.06] font-mono text-sm">
        {[
          "1. Multi-backend parallel fetch → 50–200 raw results",
          "2. SPRE fast-score all results (~2ms total)",
          "3. Keep top-30 by SPRE score",
          "4. CEP extraction on top-30 only",
          "5. HyperFusion full 8-signal ranking on top-30",
          "6. Return top-10 ranked results",
        ].map((step, i) => (
          <div key={i} className="py-1.5 text-xs text-slate-400 border-b border-white/[0.04] last:border-0">
            {step}
          </div>
        ))}
      </div>

      <h2>Performance impact</h2>
      <ul>
        <li><strong>Without SPRE:</strong> CEP extracts all 100+ results → ~10s latency</li>
        <li><strong>With SPRE:</strong> CEP extracts only top-30 → ~2s latency</li>
        <li>SPRE adds ~2ms overhead to filter 10× fewer extractions</li>
        <li>Recall@10: 94%+ (SPRE rarely filters out relevant top results)</li>
      </ul>

      <h2>Configuring the candidate pool size</h2>
      <p>
        The SPRE candidate pool size (default: 30) can be tuned based on your
        quality vs. speed trade-off:
      </p>
      <ul>
        <li><strong>15 candidates</strong> — fastest, suitable for <code>key_facts</code> tier</li>
        <li><strong>30 candidates</strong> — default, good balance for most queries</li>
        <li><strong>50 candidates</strong> — highest quality, use for <code>detailed</code> tier</li>
      </ul>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/algorithms/hyperfusion", title: "HyperFusion", desc: "Full 8-signal ranking" },
          { href: "https://docs.fetchium.com/algorithms/cep", title: "CEP Extraction", desc: "Content extraction layers" },
          { href: "https://docs.fetchium.com/algorithms/qatbe", title: "QATBE", desc: "Token budget optimization" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "See all algorithms in action" },
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
