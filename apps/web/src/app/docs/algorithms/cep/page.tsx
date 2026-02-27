import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "CEP — Content Extraction Protocol" };

export default function CepPage() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Algorithms</div>
      <h1>CEP — Content Extraction Protocol</h1>
      <p>
        CEP is a 5-layer cascade extraction system that handles any web content — from clean
        articles to JavaScript-heavy SPAs to PDF documents. Each layer attempts extraction
        and falls back to the next on failure, ensuring maximum coverage with minimum cost.
      </p>

      <h2>The 5 layers</h2>

      <div className="not-prose my-6">
        {[
          {
            num: "1",
            name: "CSS Selectors",
            speed: "~1ms",
            color: "border-emerald-500/40 bg-emerald-500/5",
            numColor: "text-emerald-400",
            desc: "Tries known content selectors: article, main, .post-content, .article-body, [itemprop=\"articleBody\"], etc. Works for 70%+ of news and blog sites. Fastest layer — pure DOM traversal, no network calls.",
          },
          {
            num: "2",
            name: "Readability",
            speed: "~5ms",
            color: "border-sky-500/40 bg-sky-500/5",
            numColor: "text-sky-400",
            desc: "Mozilla Readability algorithm port. Scores DOM nodes by content density (text-to-tag ratio), cleans boilerplate (navs, footers, ads), and extracts the main content block. Works for most static HTML pages.",
          },
          {
            num: "3",
            name: "Headless JS",
            speed: "~2s",
            color: "border-indigo-500/40 bg-indigo-500/5",
            numColor: "text-indigo-400",
            desc: "Chromium-based full page render via chromiumoxide. Executes JavaScript, waits for DOM mutations, then applies Layer 1+2. Required for SPAs (React/Vue/Angular) that render content client-side. Controlled via fetchium setup --headless.",
          },
          {
            num: "4",
            name: "PDF Extraction",
            speed: "~50ms",
            color: "border-violet-500/40 bg-violet-500/5",
            numColor: "text-violet-400",
            desc: "Detects application/pdf content type and switches to PDF text extraction. Preserves heading hierarchy, extracts tables, and handles multi-column layouts. Handles academic papers, reports, and documentation PDFs.",
          },
          {
            num: "5",
            name: "Screenshot OCR",
            speed: "~3s",
            color: "border-rose-500/40 bg-rose-500/5",
            numColor: "text-rose-400",
            desc: "Last resort — captures a screenshot with headless Chrome and runs OCR. Handles image-based PDFs, paywalled previews, and sites that actively block extraction. Slowest but most universal.",
          },
        ].map(l => (
          <div key={l.num} className={`rounded-xl p-4 border ${l.color} mb-3`}>
            <div className="flex items-start gap-3">
              <span className={`text-2xl font-bold font-mono ${l.numColor} shrink-0 mt-0.5`}>{l.num}</span>
              <div>
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-slate-100 font-semibold text-sm">{l.name}</span>
                  <span className="text-xs text-slate-500 font-mono">{l.speed}</span>
                </div>
                <p className="text-slate-400 text-sm leading-relaxed m-0">{l.desc}</p>
              </div>
            </div>
          </div>
        ))}
      </div>

      <h2>Layer selection logic</h2>
      <p>
        CEP tries layers in order 1→2→3→4→5 until one succeeds. It also uses content-type
        hints to skip layers when possible:
      </p>

      <div className="not-prose my-4 p-4 rounded-xl bg-white/[0.02] border border-white/[0.06] font-mono text-sm">
        {[
          "Content-Type: application/pdf      → skip to Layer 4",
          "URL ends in .pdf                   → skip to Layer 4",
          "Layer 1 confidence < 0.4           → try Layer 2",
          "Layer 2 fails (empty body)         → try Layer 3 (headless)",
          "Layer 3 disabled (no Chrome)       → skip to Layer 4/5",
          "Layer 4 fails (not a PDF)          → try Layer 5 (OCR)",
        ].map((rule, i) => (
          <div key={i} className="py-1 text-xs text-slate-400 border-b border-white/[0.04] last:border-0">{rule}</div>
        ))}
      </div>

      <h2>Quality confidence scoring</h2>
      <p>
        Each layer reports an extraction confidence score (0–1) based on:
      </p>
      <ul>
        <li><strong>Content length</strong> — very short extractions score lower</li>
        <li><strong>Text-to-markup ratio</strong> — high markup density suggests boilerplate</li>
        <li><strong>Sentence completeness</strong> — truncated sentences score lower</li>
        <li><strong>Language detection</strong> — non-target-language content scores lower</li>
      </ul>

      <h2>QADD integration</h2>
      <p>
        After extraction, CEP passes content through QADD (Query-Aware DOM Distillation),
        which applies 5 pruning steps to reduce token count by 10–20x while preserving
        query-relevant content:
      </p>
      <ol>
        <li>Remove navigation, ads, footers, and cookie banners</li>
        <li>Collapse whitespace and normalize Unicode</li>
        <li>Score sentences by query relevance (BM25)</li>
        <li>Keep top-K sentences within token budget</li>
        <li>Reconstruct coherent paragraphs from retained sentences</li>
      </ol>

      <h2>CLI usage</h2>
      <CodeBlock language="bash" code={`# Extract content from a URL (auto-detects best layer)
fetchium fetch https://example.com/article

# Force specific layer
fetchium fetch https://spa-app.com --headless

# Extract as PDF
fetchium fetch https://arxiv.org/pdf/2301.00001.pdf

# With token budget
fetchium fetch https://example.com --tokens 2000`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/algorithms/qatbe", title: "QATBE", desc: "Token-budgeted extraction" },
          { href: "/docs/algorithms/hyperfusion", title: "HyperFusion", desc: "8-signal ranking" },
          { href: "/docs/api/scrape", title: "Scrape API", desc: "CEP via REST API" },
          { href: "/docs/self-hosting/config", title: "Configuration", desc: "Chrome and extraction settings" },
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
