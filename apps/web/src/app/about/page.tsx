import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "About Fetchium — The Search API That Thinks",
  description:
    "Fetchium is building the information layer for the AI age. 17 novel algorithms, 11+ backends, built in Rust. Learn about our mission, team, and roadmap.",
};

const values = [
  {
    title: "Technically honest",
    desc: "We aim to keep public product claims tied to current code, docs, and verifiable configuration.",
  },
  {
    title: "Privacy by default",
    desc: "Zero telemetry unless you opt in. Full self-hosting with complete feature parity. Your queries are yours.",
  },
  {
    title: "Open infrastructure",
    desc: "Self-hostable. Standards-based (MCP, REST, CLI). No lock-in. If you want to run Fetchium on your own servers tomorrow, you can.",
  },
  {
    title: "Built to last",
    desc: "Written in Rust. 563+ tests. Zero warnings policy. Production-grade resilience with circuit breakers, retries, and graceful degradation.",
  },
];

const techStack = [
  { name: "fetchium-core", desc: "All 17 algorithms: search orchestration, CEP extraction, HyperFusion ranking, QATBE, AMRS, PIE, intelligence", lang: "Rust" },
  { name: "fetchium-cli", desc: "26-command CLI: search, research, compare, doctor, provider, export, and more", lang: "Rust" },
  { name: "fetchium-mcp", desc: "JSON-RPC 2.0 MCP server with 12 tools for Claude Desktop, Cursor, and other MCP clients", lang: "Rust" },
  { name: "fetchium-api", desc: "axum 0.7 REST API server with PostgreSQL, authentication, and rate limiting", lang: "Rust" },
  { name: "apps/web", desc: "Marketing site and documentation hub", lang: "Next.js 15" },
  { name: "apps/dashboard", desc: "API key management, usage analytics, playground", lang: "Next.js 15" },
];

export default function AboutPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-4xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">About</span>
          </nav>

          {/* Mission */}
          <div className="mb-14">
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-6">
              We&apos;re building the{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                information layer for AI
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 leading-relaxed max-w-2xl mb-6">
              AI applications need reliable, structured, citeable information from the open web.
              Fetching that information reliably — at scale, with privacy, across diverse sources —
              is a hard engineering problem that most teams shouldn&apos;t have to solve themselves.
            </p>
            <p className="text-base sm:text-lg text-slate-400 leading-relaxed max-w-2xl">
              Fetchium is the API that solves it. 17 novel algorithms, 11+ backends, built in Rust.
              From individual developers building their first RAG prototype to teams running production
              AI pipelines — one API handles every retrieval workload.
            </p>
          </div>

          {/* Values */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-6">What we stand for</h2>
            <div className="grid sm:grid-cols-2 gap-4">
              {values.map((v) => (
                <div key={v.title} className="rounded-xl border border-white/6 bg-white/[0.02] p-5">
                  <h3 className="text-sm font-semibold text-slate-200 mb-2">{v.title}</h3>
                  <p className="text-[13px] text-slate-500 leading-relaxed">{v.desc}</p>
                </div>
              ))}
            </div>
          </section>

          {/* By the numbers */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-6">By the numbers</h2>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
              {[
                { v: "17", l: "Novel algorithms", sub: "CEP, QATBE, AMRS, PIE, RAR..." },
                { v: "563+", l: "Tests passing", sub: "Zero failures, zero warnings" },
                { v: "11+", l: "Search backends", sub: "Federated in one call" },
                { v: "26", l: "CLI commands", sub: "Complete developer toolset" },
              ].map((s) => (
                <div key={s.l} className="rounded-xl border border-white/6 bg-white/[0.02] p-4 text-center">
                  <div className="text-2xl font-bold bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">{s.v}</div>
                  <div className="text-[12px] font-medium text-slate-300 mt-1">{s.l}</div>
                  <div className="text-[10px] text-slate-600 mt-0.5">{s.sub}</div>
                </div>
              ))}
            </div>
          </section>

          {/* Tech stack */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-6">Architecture</h2>
            <div className="space-y-3">
              {techStack.map((t) => (
                <div key={t.name} className="flex items-start gap-4 rounded-xl border border-white/6 bg-white/[0.02] p-4">
                  <code className="shrink-0 rounded-lg bg-indigo-500/12 border border-indigo-500/20 px-2.5 py-1 text-[12px] font-mono text-indigo-300">
                    {t.name}
                  </code>
                  <div className="flex-1 min-w-0">
                    <p className="text-[13px] text-slate-400">{t.desc}</p>
                  </div>
                  <span className="shrink-0 text-[11px] text-slate-600 font-mono">{t.lang}</span>
                </div>
              ))}
            </div>
          </section>

          {/* Contact */}
          <div className="rounded-2xl border border-indigo-500/15 bg-indigo-500/5 p-6 text-center">
            <h2 className="text-lg font-bold mb-2">Get in touch</h2>
            <p className="text-[13px] text-slate-500 mb-4">
              Questions, partnership enquiries, or just want to say hi?
            </p>
            <div className="flex flex-wrap justify-center gap-3">
              <Link href="/contact" className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-5 py-2.5 text-sm font-semibold text-white transition-all hover:shadow-[0_0_20px_rgba(99,102,241,0.4)]">
                Contact Us
              </Link>
              <Link href="https://github.com/zuhabul/Fetchium" target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-white/3 px-5 py-2.5 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all">
                GitHub
              </Link>
              <Link href="https://discord.gg/fetchium" target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-white/3 px-5 py-2.5 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all">
                Discord
              </Link>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
