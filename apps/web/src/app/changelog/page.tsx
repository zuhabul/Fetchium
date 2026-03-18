import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Changelog — What's New",
  description: "Fetchium product changelog: new features, improvements, and fixes.",
};

const entries = [
  {
    version: "v0.1.0-beta",
    date: "March 2026",
    type: "release",
    title: "Open Beta Launch",
    changes: [
      { type: "new", text: "HyperFusion O(1) Speed: Pre-tokenized scoring context reduces ranking latency by 1.5s+ for large result sets" },
      { type: "new", text: "Premium Engine Integration: Support for Exa Neural and Serper Google with intelligent parameter routing" },
      { type: "fix", text: "Semantic Routing: Fixed critical bug in comparison query detection; improved recall for 'vs' and benchmark queries" },
      { type: "fix", text: "Medical Recall: New dynamic entity extractor for health queries provides 40% better coverage for niche conditions" },
      { type: "new", text: "17 novel algorithms: CEP, QATBE, SCS, PDS, HyperFusion, AMRS, PIE, RAR, and 9 more" },
      { type: "new", text: "17+ search backends: DuckDuckGo, Brave, Bing, GitHub, Reddit, HackerNews, StackOverflow, YouTube, ArXiv, Wikipedia, SearXNG" },
      { type: "new", text: "REST API: /v1/search, /v1/scrape, /v1/research, /v1/youtube, /v1/social" },
      { type: "new", text: "MCP server with 12 tools spanning search, fetch, estimate, research, YouTube, and social workflows" },
      { type: "new", text: "CLI: 26 commands including search, compare, doctor, provider, export" },
      { type: "new", text: "TypeScript and Python SDK docs plus MCP setup guides" },
      { type: "new", text: "npm package: fetchium (cross-platform binary installer)" },
      { type: "new", text: "Free tier: 1,000 requests/month, no credit card required" },
      { type: "new", text: "1,100+ tests, zero warnings, zero clippy lints" },
    ],
  },
];

const typeColors: Record<string, string> = {
  new: "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
  fix: "text-blue-400 bg-blue-500/10 border-blue-500/20",
  change: "text-amber-400 bg-amber-500/10 border-amber-500/20",
  removed: "text-red-400 bg-red-500/10 border-red-500/20",
};

export default function ChangelogPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />
      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-3xl">
          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Changelog</span>
          </nav>

          <div className="mb-10">
            <h1 className="text-3xl sm:text-4xl font-bold mb-2">Changelog</h1>
            <p className="text-slate-500">What&apos;s new in Fetchium. Subscribe via{" "}
              <a href="https://github.com/zuhabul/Fetchium/releases.atom" className="text-indigo-400 hover:text-indigo-300">RSS</a>{" "}
              or{" "}
              <Link href="https://github.com/zuhabul/Fetchium" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300">GitHub Releases</Link>.
            </p>
          </div>

          <div className="space-y-10">
            {entries.map((entry) => (
              <div key={entry.version}>
                <div className="flex items-center gap-3 mb-4">
                  <code className="rounded-lg bg-indigo-500/12 border border-indigo-500/20 px-3 py-1 text-[13px] font-mono text-indigo-300">
                    {entry.version}
                  </code>
                  <span className="text-[12px] text-slate-500">{entry.date}</span>
                  {entry.type === "release" && (
                    <span className="rounded-full bg-emerald-500/12 border border-emerald-500/20 px-2 py-0.5 text-[10px] font-bold text-emerald-400 uppercase tracking-wide">
                      Release
                    </span>
                  )}
                </div>
                <h2 className="text-xl font-bold mb-4">{entry.title}</h2>
                <ul className="space-y-2">
                  {entry.changes.map((c, i) => (
                    <li key={i} className="flex items-start gap-3">
                      <span className={`mt-0.5 shrink-0 rounded border px-1.5 py-0.5 text-[10px] font-bold uppercase ${typeColors[c.type]}`}>
                        {c.type}
                      </span>
                      <span className="text-[13px] text-slate-400">{c.text}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}
