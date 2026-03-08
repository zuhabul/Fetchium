import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = { title: "QATBE — Query-Aware Token-Budgeted Extraction" };

export default function QatbePage() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Algorithms</div>
      <h1>QATBE</h1>
      <p>
        <strong>Query-Aware Token-Budgeted Extraction.</strong> QATBE solves the core
        problem of LLM context window management: given a token budget, how do you pack
        the most query-relevant content?
      </p>
      <p>
        QATBE uses BM25 scoring to rank content segments by relevance, then applies a
        greedy knapsack algorithm to select the optimal subset within the token budget.
        The result: maximum relevance density per token.
      </p>

      <h2>How it works</h2>
      <ol>
        <li>
          <strong>SCS segmentation</strong> — The extracted text is split into typed
          segments: headings, paragraphs, code blocks, lists, tables, quotes.
        </li>
        <li>
          <strong>BM25 scoring</strong> — Each segment is scored against the original query
          using BM25 TF-IDF. Query terms are expanded with synonyms first.
        </li>
        <li>
          <strong>Type-aware weighting</strong> — Scores are multiplied by a type efficiency
          factor. Code blocks and tables get a bonus (high information density per token);
          navigation and metadata get a penalty.
        </li>
        <li>
          <strong>Greedy knapsack</strong> — Segments are sorted by score/tokens ratio,
          then greedily selected until the budget is reached.
        </li>
        <li>
          <strong>Coherence restoration</strong> — Selected segments are reordered by
          document position to maintain reading flow.
        </li>
      </ol>

      <h2>Segment type weights</h2>
      <table>
        <thead><tr><th>Segment type</th><th>Efficiency weight</th><th>Rationale</th></tr></thead>
        <tbody>
          <tr><td>Code block</td><td>1.5×</td><td>High information density, directly actionable</td></tr>
          <tr><td>Table</td><td>1.4×</td><td>Structured data is very token-efficient</td></tr>
          <tr><td>Heading</td><td>1.3×</td><td>Provides context for surrounding content</td></tr>
          <tr><td>Paragraph</td><td>1.0×</td><td>Baseline</td></tr>
          <tr><td>List</td><td>1.1×</td><td>Slightly more efficient than prose</td></tr>
          <tr><td>Quote</td><td>0.9×</td><td>Often secondary content</td></tr>
          <tr><td>Metadata</td><td>0.5×</td><td>Rarely query-relevant</td></tr>
        </tbody>
      </table>

      <h2>Detail tiers</h2>
      <p>
        The <code>tier</code> parameter in the Search and Scrape APIs maps to a QATBE
        token budget:
      </p>
      <table>
        <thead><tr><th>Tier</th><th>Token budget</th><th>Best for</th></tr></thead>
        <tbody>
          <tr><td><code>key_facts</code></td><td>~200 tokens</td><td>Quick answers, chatbots, speed-critical</td></tr>
          <tr><td><code>summary</code></td><td>~1,000 tokens</td><td>General RAG, AI context injection</td></tr>
          <tr><td><code>detailed</code></td><td>~5,000 tokens</td><td>Thorough research, long-form generation</td></tr>
          <tr><td><code>complete</code></td><td>~20,000 tokens</td><td>Full extraction, document analysis</td></tr>
        </tbody>
      </table>

      <h2>Performance</h2>
      <ul>
        <li>BM25 scoring: ~0.5ms per 1,000 segments</li>
        <li>Knapsack selection: O(n log n) — always fast</li>
        <li>Works on CPU only, no GPU required</li>
        <li>Scales linearly with document length</li>
      </ul>

      <h2>Example: token budget in practice</h2>
      <p>
        For a 10,000-token web page with a 1,000-token budget, QATBE:
      </p>
      <ul>
        <li>Splits into ~80 segments across 8 types</li>
        <li>Scores each segment (takes ~1ms total)</li>
        <li>Selects top 15–20 segments by relevance/token ratio</li>
        <li>Returns ~1,000 tokens of maximally relevant content</li>
        <li>Typical relevance retention: 85–95% of key information</li>
      </ul>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/algorithms/cep", title: "CEP Extraction", desc: "How content is extracted first" },
          { href: "https://docs.fetchium.com/algorithms/hyperfusion", title: "HyperFusion", desc: "Ranking after extraction" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "tier parameter reference" },
          { href: "https://docs.fetchium.com/api/scrape", title: "Scrape API", desc: "Direct content extraction" },
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
