import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { CheckCircle, Circle, Clock } from "lucide-react";

export const metadata: Metadata = {
  title: "Fetchium Roadmap — What We're Building",
  description: "Fetchium product roadmap: what's shipped, what's in progress, and what's coming next.",
};

const phases = [
  {
    phase: "Phase 0–3 · Shipped",
    status: "done",
    items: [
      "17 novel algorithms (CEP, QATBE, SCS, PDS, HyperFusion, AMRS, PIE, RAR + 9 more)",
      "17+ search backends with parallel dispatch and circuit breakers",
      "5-layer content extraction pipeline (CEP)",
      "HyperFusion 8-signal ranking",
      "QATBE token-budgeted extraction",
      "Evidence graphs and structured citations",
      "YouTube & social intelligence endpoints",
      "AMRS multi-agent research pipeline",
      "MCP server with 12 tools",
      "CLI with 26 commands",
      "TypeScript and Python SDK docs",
      "1,100+ tests, zero warnings",
    ],
  },
  {
    phase: "Phase 4 · In Progress",
    status: "active",
    items: [
      "Production REST API with authentication and rate limiting",
      "PostgreSQL-backed usage tracking",
      "Dashboard: API key management, usage analytics, playground",
      "TypeScript SDK (@fetchium/sdk on npm)",
      "Python SDK (fetchium on PyPI)",
    ],
  },
  {
    phase: "Phase 5 · Next",
    status: "planned",
    items: [
      "Semantic search with fastembed-rs embeddings",
      "Vector similarity search with usearch",
      "Streaming search results (SSE)",
      "Webhook notifications for monitoring alerts",
      "Batch API for high-volume use cases",
    ],
  },
  {
    phase: "Phase 6–8 · Future",
    status: "planned",
    items: [
      "SOC 2 Type II certification",
      "EU data residency option",
      "Team accounts and role-based access",
      "Enterprise support and deployment assistance",
      "Mobile SDKs (React Native, Flutter)",
      "Zapier and Make.com integration",
    ],
  },
];

const statusIcon = {
  done: <CheckCircle className="h-4 w-4 text-emerald-400" strokeWidth={2} />,
  active: <Clock className="h-4 w-4 text-indigo-400 animate-pulse" strokeWidth={2} />,
  planned: <Circle className="h-4 w-4 text-slate-600" strokeWidth={2} />,
};

export default function RoadmapPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />
      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-3xl">
          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Roadmap</span>
          </nav>

          <div className="mb-10">
            <h1 className="text-3xl sm:text-4xl font-bold mb-3">Roadmap</h1>
            <p className="text-slate-500 leading-relaxed">
              What we&apos;ve built, what we&apos;re building, and what&apos;s coming.
              Want to influence the roadmap?{" "}
              <Link href="/contact" className="text-indigo-400 hover:text-indigo-300">Let us know</Link>{" "}
              or open a{" "}
              <Link href="https://github.com/zuhabul/Fetchium/issues" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300">GitHub issue</Link>.
            </p>
          </div>

          <div className="space-y-8">
            {phases.map((phase) => (
              <div key={phase.phase}>
                <div className="flex items-center gap-3 mb-4">
                  {statusIcon[phase.status as keyof typeof statusIcon]}
                  <h2 className="text-lg font-bold">{phase.phase}</h2>
                </div>
                <ul className="space-y-2 pl-7">
                  {phase.items.map((item) => (
                    <li key={item} className={`flex items-start gap-2 text-[13px] ${
                      phase.status === "done" ? "text-slate-400" :
                      phase.status === "active" ? "text-slate-300" : "text-slate-500"
                    }`}>
                      <span className={`mt-1.5 h-1.5 w-1.5 rounded-full shrink-0 ${
                        phase.status === "done" ? "bg-emerald-400" :
                        phase.status === "active" ? "bg-indigo-400" : "bg-slate-600"
                      }`} />
                      {item}
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
