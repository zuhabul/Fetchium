import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Estimate API Reference" };

export default function EstimateApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Estimate API</h1>
      <p>
        Estimates extraction token cost for a URL without running full fetch extraction.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/estimate</span>
      </div>

      <h2>Request body</h2>
      <CodeBlock language="json" code={`{
  "url": "https://tokio.rs/tokio/tutorial"
}`} />

      <h2>Response</h2>
      <CodeBlock language="json" code={`{
  "url": "https://tokio.rs/tokio/tutorial",
  "estimated_tokens": 1532,
  "estimated_relevant_tokens": 766,
  "extraction_layer": 1,
  "content_type": "text/html"
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/api/scrape", title: "Scrape API", desc: "Run full extraction" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Find pages first" },
          { href: "https://docs.fetchium.com/api/research", title: "Research API", desc: "End-to-end synthesis" },
          { href: "https://docs.fetchium.com/api/usage", title: "Usage API", desc: "Track quota" },
        ].map((l) => (
          <Link key={l.href} href={l.href} className="glass-card rounded-xl p-4 no-underline group">
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{l.title} →</div>
            <div className="text-xs text-slate-500 mt-1">{l.desc}</div>
          </Link>
        ))}
      </div>
    </article>
  );
}
