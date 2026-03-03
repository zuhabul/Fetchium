import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Research API — Autonomous Multi-Agent Research (AMRS)",
  description:
    "Spawn 4 agent types in parallel for deep research reports. Evidence graphs, cross-source validation, and RAR self-correction. Built on tokio async Rust.",
};

const agents = [
  {
    name: "Search Agent",
    role: "Broad query coverage",
    desc: "Dispatches the primary query and multiple semantic variants across all backends. Collects the initial evidence pool.",
    color: "text-indigo-400",
    bg: "bg-indigo-500/10 border-indigo-500/20",
  },
  {
    name: "Verification Agent",
    role: "Cross-source fact checking",
    desc: "Takes each claim from the search agent and independently searches for corroborating or contradicting evidence. Flags contradictions.",
    color: "text-emerald-400",
    bg: "bg-emerald-500/10 border-emerald-500/20",
  },
  {
    name: "Deep Dive Agent",
    role: "Source exploration",
    desc: "For the highest-scoring sources, follows links, extracts referenced papers, and builds a second-degree evidence graph.",
    color: "text-violet-400",
    bg: "bg-violet-500/10 border-violet-500/20",
  },
  {
    name: "Synthesis Agent",
    role: "Report assembly",
    desc: "Combines findings, resolves contradictions using the RAR protocol, ranks evidence by consensus confidence, and structures the final report.",
    color: "text-blue-400",
    bg: "bg-blue-500/10 border-blue-500/20",
  },
];

const rarSteps = [
  { n: "R1", name: "Initial response", desc: "First synthesis attempt from evidence pool" },
  { n: "R2", name: "Coverage check", desc: "Identifies gaps — claims without supporting sources" },
  { n: "R3", name: "Contradiction scan", desc: "Flags conflicting claims across sources" },
  { n: "R4", name: "Confidence calibration", desc: "Scores each claim by evidence weight" },
  { n: "R5", name: "Final refinement", desc: "Rewrites low-confidence sections with stronger evidence" },
];

export default function ProductResearchPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <Link href="/product/search" className="hover:text-slate-400 transition-colors">Product</Link>
            <span>/</span>
            <span className="text-slate-400">Research API</span>
          </nav>

          <div className="mb-14">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 py-1.5 text-xs font-medium text-indigo-300">
              Product · Research API
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Deep research mode —{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                multi-agent, evidence-backed
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed mb-7">
              The AMRS (Adaptive Multi-Agent Research Swarm) pipeline runs 4 specialized agents in parallel —
              each searching a different angle, cross-validating findings, and assembling a structured evidence
              graph. Research that takes hours of manual work completes in minutes.
            </p>
            <div className="flex flex-wrap gap-3">
              <Link
                href="https://app.fetchium.com/register"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-6 py-3 text-sm font-semibold text-white shadow-[0_0_24px_rgba(99,102,241,0.3)] hover:shadow-[0_0_36px_rgba(99,102,241,0.5)] transition-all"
              >
                Get API Key Free →
              </Link>
              <Link
                href="/docs/api/research"
                className="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-white/3 px-6 py-3 text-sm font-semibold text-slate-300 hover:bg-white/6 transition-all"
              >
                API Reference
              </Link>
            </div>
          </div>

          {/* 4 agent types */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-2">4 Agent Types Running in Parallel</h2>
            <p className="text-slate-500 mb-6 text-sm">
              Agents communicate via tokio channels. Each runs independently; the synthesis agent
              assembles the final report once all others complete.
            </p>
            <div className="grid sm:grid-cols-2 gap-4">
              {agents.map((agent) => (
                <div key={agent.name} className={`rounded-xl border ${agent.bg} p-5`}>
                  <div className={`text-xs font-semibold uppercase tracking-widest ${agent.color} mb-1`}>
                    {agent.role}
                  </div>
                  <h3 className="text-base font-bold text-slate-100 mb-2">{agent.name}</h3>
                  <p className="text-[13px] text-slate-400 leading-relaxed">{agent.desc}</p>
                </div>
              ))}
            </div>
          </section>

          {/* RAR checkpoints */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-2">
              RAR: Retry-and-Refine Self-Correction
            </h2>
            <p className="text-slate-500 mb-6 text-sm">
              The synthesis agent runs 5 reflection checkpoints before returning the final report.
              Low-confidence sections are rewritten; contradictions are resolved or flagged.
            </p>
            <div className="space-y-2">
              {rarSteps.map((step) => (
                <div key={step.n} className="flex items-start gap-4 rounded-xl border border-white/6 bg-white/[0.02] p-4">
                  <div className="h-8 w-10 rounded-lg bg-indigo-500/15 flex items-center justify-center shrink-0 text-xs font-bold text-indigo-400 font-mono">
                    {step.n}
                  </div>
                  <div>
                    <div className="text-sm font-semibold text-slate-200">{step.name}</div>
                    <div className="text-[12px] text-slate-500 mt-0.5">{step.desc}</div>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* API example */}
          <section className="mb-14">
            <h2 className="text-xl sm:text-2xl font-bold mb-4">Start a Research Task</h2>
            <div className="rounded-2xl border border-white/8 bg-[#0d0f1a] overflow-hidden">
              <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/6 bg-white/[0.015]">
                <span className="text-[12px] font-mono text-slate-500">cURL</span>
                <span className="text-[11px] font-mono text-emerald-400">POST /v1/research</span>
              </div>
              <pre className="p-4 text-[13px] font-mono text-slate-300 overflow-x-auto leading-relaxed">
{`curl -X POST ***REMOVED***/v1/research \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "State of AI inference optimization in 2025",
    "depth": "standard",
    "max_sources": 20,
    "citation_format": "apa",
    "token_budget": 16384
  }'`}
              </pre>
            </div>
          </section>

          <section className="rounded-2xl border border-indigo-500/15 bg-indigo-500/5 p-6">
            <h3 className="text-base font-semibold text-slate-200 mb-4">Related</h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {[
                { href: "/product/search", label: "Search API", desc: "Quick single-query search" },
                { href: "/docs/algorithms/cep", label: "Algorithm Docs", desc: "CEP, AMRS, RAR explained" },
                { href: "/pricing", label: "Pricing", desc: "Research API included in Growth+" },
              ].map((l) => (
                <Link key={l.href} href={l.href} className="flex items-start gap-2 rounded-lg border border-white/6 bg-white/2 p-3 hover:bg-white/5 transition-all group">
                  <div>
                    <div className="text-[13px] font-medium text-slate-300 group-hover:text-white">{l.label}</div>
                    <div className="text-[11px] text-slate-600">{l.desc}</div>
                  </div>
                </Link>
              ))}
            </div>
          </section>
        </div>
      </main>

      <Footer />
    </div>
  );
}
