"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown, Zap } from "lucide-react";
import Link from "next/link";

/**
 * FAQ section targeting high-intent keywords:
 * "search API", "RAG retrieval", "web scraping for LLMs",
 * "MCP tools", "token budget", "citations", "self-hosting".
 *
 * FAQPage schema is rendered server-side in the parent layout
 * or page component to avoid CSP issues.
 */

const faqs = [
  {
    q: "What is Fetchium?",
    a: "Fetchium is a search API built for AI applications. Unlike traditional SERP scrapers, Fetchium delivers a complete pipeline in one API call: multi-backend search (11+ sources), 5-layer content extraction (CEP), neural ranking (HyperFusion), token-budgeted output (QATBE), and structured citations with evidence graphs. It's designed to feed RAG pipelines, AI agents, and research workflows.",
  },
  {
    q: "How does Fetchium differ from Firecrawl?",
    a: "Firecrawl is an extraction tool — you give it a URL and it returns cleaned markdown. Fetchium is a search + extraction + ranking + citation pipeline. You give it a query and it returns ranked, extracted, token-budgeted results from 11+ backends with evidence graphs. Fetchium includes Firecrawl's core extraction use case plus search federation, neural ranking, and cross-source validation.",
  },
  {
    q: "How does Fetchium compare to Tavily and Exa?",
    a: "Tavily ($8/1K) and Exa ($5/1K) focus on search + snippets. Fetchium ($0.58/1K on Growth) adds full content extraction, token budgeting, evidence graphs, and cross-session learning. Fetchium's Starter plan at $0.90/1K is 9× cheaper than Tavily and 5× cheaper than Exa for a feature superset. Latency sources: Exa averages ~1.2s and Tavily ~1.9s per an independent 50-query benchmark (2025). Fetchium targets ~500ms P50 for search-only mode.",
  },
  {
    q: "What is token-budgeted extraction (QATBE)?",
    a: "QATBE (Query-Aware Token-Budgeted Extraction) scores each content segment by BM25 relevance to your query, then packs the highest-scoring segments into your token budget using a greedy knapsack algorithm. Instead of sending 50,000 tokens of raw HTML to your LLM, you get 2,000–5,000 tokens of the most relevant content — cutting context costs by 60–90% vs. raw web content.",
  },
  {
    q: "Does Fetchium work with LangChain, LlamaIndex, and other RAG frameworks?",
    a: "Yes. Fetchium has official adapters for LangChain (Python retriever) and CrewAI (Python tool). The REST API works with any framework that can make HTTP requests. MCP support lets Claude Desktop, Cursor, and any MCP-compatible client call Fetchium without any code. TypeScript and Python SDKs are available on npm and PyPI.",
  },
  {
    q: "What is the Model Context Protocol (MCP) and how does Fetchium support it?",
    a: "The Model Context Protocol (MCP) is an open standard that lets AI clients (Claude Desktop, Cursor, etc.) call external tools without custom integrations. Fetchium runs a JSON-RPC 2.0 stdio MCP server with 5 tools: search, extract, research, youtube, and social. Any MCP-compatible client can call Fetchium just by pointing at the server — no code required.",
  },
  {
    q: "How fast is the Fetchium API?",
    a: "Fetchium targets ~500ms P50 for search-only mode (results without full content extraction). The full pipeline — search + CEP extraction + QATBE token budgeting + citations — typically completes in 1–4 seconds. For comparison: independent 2025 benchmarks show Exa at ~1.2s avg and Tavily at ~1.9s avg for search-only. Fetchium is built in Rust with tokio for minimal overhead; latency is dominated by external backend response times, not the server.",
  },
  {
    q: "Can I self-host Fetchium?",
    a: "Yes — Fetchium is fully self-hostable with complete feature parity. The Rust binary runs anywhere Docker runs. All 17 algorithms, all 11+ backends, the MCP server, and the complete CLI are available in self-hosted mode. No feature is cloud-only. This is unique in the space: all major competitors restrict advanced features to their hosted API.",
  },
  {
    q: "Does Fetchium store my search queries?",
    a: "No. Fetchium operates with zero telemetry by default. Queries are processed in-memory and never logged or stored without your explicit opt-in. The PIE (cross-session learning) feature can optionally store patterns in your own local SQLite database — this never leaves your deployment. We never send queries to third-party analytics or advertising systems.",
  },
  {
    q: "What programming languages does Fetchium support?",
    a: "The REST API is language-agnostic. Official SDKs: TypeScript/JavaScript (npm: fetchium) and Python (pypi: fetchium). The CLI is available via cargo, npm, or Homebrew. LangChain and CrewAI adapters are Python-native. MCP server integration works with any MCP client. cURL examples are in the docs for direct API access.",
  },
  {
    q: "How does Fetchium handle citations and evidence graphs?",
    a: "Every response includes structured citations: title, URL, author, and publication date. The evidence graph maps factual claims to their sources — showing which sources agree, which contradict, and consensus confidence. Citations export in APA, IEEE, BibTeX, or JSON. This is powered by the RAR (Retry-and-Refine) and AMRS algorithms that cross-validate claims across sources.",
  },
  {
    q: "What does the Free tier include?",
    a: "The Free tier gives you 1,000 API requests per month, forever — no credit card required, no time limit. It includes all features: all 11+ search backends, 5-layer CEP extraction, HyperFusion 8-signal ranking, QATBE token budgeting, evidence graphs, citations, and MCP tools. Paid plans only differ by volume (10K–200K/mo) and rate limit (60–600 req/min).",
  },
];

export default function FAQ() {
  const [open, setOpen] = useState<number | null>(null);

  return (
    <section id="faq" className="relative overflow-hidden py-16 sm:py-28 px-4">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/2 top-0 h-[400px] w-[800px] -translate-x-1/2 rounded-full bg-indigo-500/4 blur-[130px]" />
      </div>

      <div className="relative mx-auto max-w-3xl">
        <motion.div
          className="mb-10 sm:mb-14 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-5 inline-flex items-center gap-2 rounded-full border border-indigo-500/30 bg-indigo-500/10 px-4 py-2 text-sm font-semibold text-indigo-200">
            <Zap className="h-4 w-4" strokeWidth={2.5} />
            Common Questions
          </div>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight text-slate-100">
            Frequently asked{" "}
            <span className="gradient-text">questions</span>
          </h2>
          <p className="mt-5 text-base sm:text-lg text-slate-300">
            Everything you need to decide if Fetchium is right for your project.
          </p>
        </motion.div>

        <motion.div
          className="space-y-2"
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-40px" }}
          variants={{ hidden: {}, visible: { transition: { staggerChildren: 0.05 } } }}
        >
          {faqs.map((faq, i) => (
            <motion.div
              key={i}
              variants={{
                hidden: { opacity: 0, y: 12 },
                visible: { opacity: 1, y: 0, transition: { duration: 0.4, ease: [0.22, 1, 0.36, 1] } },
              }}
              className={`rounded-xl border transition-all duration-200 ${
                open === i
                  ? "border-indigo-500/25 bg-indigo-500/5"
                  : "border-slate-800 bg-slate-900/30 hover:border-slate-700 hover:bg-slate-900/50"
              }`}
            >
              <button
                onClick={() => setOpen(open === i ? null : i)}
                className="flex w-full items-start justify-between gap-4 px-5 py-4 text-left"
                aria-expanded={open === i}
              >
                <span className={`text-base sm:text-lg font-semibold transition-colors ${open === i ? "text-slate-100" : "text-slate-200"}`}>
                  {faq.q}
                </span>
                <ChevronDown
                  className={`h-4 w-4 shrink-0 mt-0.5 transition-all duration-200 ${
                    open === i ? "rotate-180 text-indigo-400" : "text-slate-400"
                  }`}
                />
              </button>

              <AnimatePresence>
                {open === i && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.25, ease: [0.22, 1, 0.36, 1] }}
                    className="overflow-hidden"
                  >
                    <p className="px-5 pb-5 text-base leading-relaxed text-slate-300">
                      {faq.a}
                    </p>
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
          ))}
        </motion.div>

        <motion.div
          className="mt-8 text-center"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.3, duration: 0.5 }}
        >
          <p className="text-base text-slate-300">
            Still have questions?{" "}
            <Link href="/contact" className="text-indigo-400 hover:text-indigo-300 underline underline-offset-2 transition-colors font-semibold">
              Contact us
            </Link>
            {" "}or join our{" "}
            <Link href="https://discord.gg/fetchium" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300 underline underline-offset-2 transition-colors font-semibold">
              Discord community
            </Link>
            .
          </p>
        </motion.div>
      </div>
    </section>
  );
}
