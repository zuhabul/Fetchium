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
    a: "Fetchium is designed as a broader retrieval pipeline rather than a single-purpose search endpoint. In addition to search federation, it includes content extraction, token budgeting, citations, async research jobs, and MCP tooling. Because third-party pricing and benchmarks change frequently, the docs focus on capability differences rather than frozen competitor numbers.",
  },
  {
    q: "What is token-budgeted extraction (QATBE)?",
    a: "QATBE (Query-Aware Token-Budgeted Extraction) scores each content segment by BM25 relevance to your query, then packs the highest-scoring segments into your token budget using a greedy knapsack algorithm. Instead of sending 50,000 tokens of raw HTML to your LLM, you get 2,000–5,000 tokens of the most relevant content — cutting context costs by 60–90% vs. raw web content.",
  },
  {
    q: "Does Fetchium work with LangChain, LlamaIndex, and other RAG frameworks?",
    a: "Yes. The REST API works with any framework that can make HTTP requests, and MCP support lets Claude Desktop, Cursor, and other MCP-compatible clients call Fetchium without a custom integration layer. TypeScript and Python SDK examples are documented in the docs site.",
  },
  {
    q: "What is the Model Context Protocol (MCP) and how does Fetchium support it?",
    a: "The Model Context Protocol (MCP) is an open standard that lets AI clients call external tools without custom integrations. Fetchium runs a JSON-RPC 2.0 MCP server and currently exposes 12 tools covering search, fetch, estimate, research, YouTube, and social workflows.",
  },
  {
    q: "How fast is the Fetchium API?",
    a: "Latency depends on which backends respond, whether extraction is enabled, and whether you are using synchronous or async job endpoints. Fetchium is built in Rust with parallel backend dispatch, but exact timings vary by workload, so the docs avoid hard-coded benchmark promises.",
  },
  {
    q: "Can I self-host Fetchium?",
    a: "Yes. The repo includes the Rust services, CLI, MCP server, and self-hosting docs. The self-hosting guides on docs.fetchium.com cover Docker and configuration paths for running the stack yourself.",
  },
  {
    q: "Does Fetchium store my search queries?",
    a: "No. Fetchium operates with zero telemetry by default. Queries are processed in-memory and never logged or stored without your explicit opt-in. The PIE (cross-session learning) feature can optionally store patterns in your own local SQLite database — this never leaves your deployment. We never send queries to third-party analytics or advertising systems.",
  },
  {
    q: "What programming languages does Fetchium support?",
    a: "The REST API is language-agnostic. The docs include TypeScript, Python, and cURL examples, and the CLI can be installed through the project-supported distribution channels described in the repository and docs.",
  },
  {
    q: "How does Fetchium handle citations and evidence graphs?",
    a: "Every response includes structured citations: title, URL, author, and publication date. The evidence graph maps factual claims to their sources — showing which sources agree, which contradict, and consensus confidence. Citations export in APA, IEEE, BibTeX, or JSON. This is powered by the RAR (Retry-and-Refine) and AMRS algorithms that cross-validate claims across sources.",
  },
  {
    q: "What does the Free tier include?",
    a: "The Free tier gives you 1,000 API requests per month. In the current auth configuration, paid tiers differ primarily by monthly quota and per-minute rate limits: Free 60/min, Starter 200/min, Pro 500/min, and Enterprise 2,000/min.",
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
