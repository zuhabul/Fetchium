"use client";

import { motion } from "framer-motion";
import Link from "next/link";
import { ArrowRight, Bot, FileSearch, Eye, Youtube, MessageSquare } from "lucide-react";

const cases = [
  {
    icon: Bot,
    title: "RAG Pipelines & AI Agents",
    description:
      "Drop Fetchium into custom RAG stacks and AI agents over HTTP or MCP. One call returns search results, extracted content, and citations ready for downstream retrieval or prompting.",
    tags: ["HTTP API", "MCP", "RAG", "Agents"],
    href: "/product/search",
    color: "from-indigo-500/15 to-violet-500/10",
    border: "border-indigo-500/20 hover:border-indigo-500/40",
    iconColor: "text-indigo-400",
    iconBg: "bg-indigo-500/10",
  },
  {
    icon: FileSearch,
    title: "Deep Research Reports",
    description:
      "The AMRS pipeline spawns 4 agent types in parallel, searches different angles, cross-validates findings, and assembles an evidence graph.",
    tags: ["AMRS", "Evidence graphs", "Multi-agent", "Citations"],
    href: "/product/research",
    color: "from-violet-500/15 to-purple-500/10",
    border: "border-violet-500/20 hover:border-violet-500/40",
    iconColor: "text-violet-400",
    iconBg: "bg-violet-500/10",
  },
  {
    icon: Eye,
    title: "Content Monitoring & Diffs",
    description:
      "Track pages, domains, or topics over time and compare fetched content across runs for change detection workflows.",
    tags: ["Diffs", "Monitoring", "Extraction", "Analysis"],
    href: "/product/extract",
    color: "from-blue-500/15 to-cyan-500/10",
    border: "border-blue-500/20 hover:border-blue-500/40",
    iconColor: "text-blue-400",
    iconBg: "bg-blue-500/10",
  },
  {
    icon: Youtube,
    title: "YouTube Intelligence",
    description:
      "Search, extract, and analyze YouTube content at scale. Get transcripts, metadata, engagement signals, and semantic summaries — all through the same unified API.",
    tags: ["Transcripts", "Metadata", "Channels", "Sentiment"],
    href: "https://docs.fetchium.com/api/youtube",
    color: "from-red-500/15 to-orange-500/10",
    border: "border-red-500/20 hover:border-red-500/40",
    iconColor: "text-red-400",
    iconBg: "bg-red-500/10",
  },
  {
    icon: MessageSquare,
    title: "Social Intelligence",
    description:
      "Pull structured data from Reddit and Hacker News alongside broader web retrieval to understand community signals around a topic.",
    tags: ["Reddit", "HackerNews", "Community", "Trends"],
    href: "https://docs.fetchium.com/api/social",
    color: "from-emerald-500/15 to-teal-500/10",
    border: "border-emerald-500/20 hover:border-emerald-500/40",
    iconColor: "text-emerald-400",
    iconBg: "bg-emerald-500/10",
  },
];

export default function UseCases() {
  return (
    <section id="use-cases" className="relative overflow-hidden py-16 sm:py-28 px-4">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute right-0 top-1/3 h-[400px] w-[500px] rounded-full bg-violet-600/5 blur-[120px]" />
      </div>

      <div className="relative mx-auto max-w-7xl">
        <motion.div
          className="mb-10 sm:mb-14 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-5 inline-flex items-center gap-2 rounded-full border border-indigo-500/30 bg-indigo-500/10 px-4 py-2 text-sm font-semibold text-indigo-200">
            Built for real workloads
          </div>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight text-slate-100">
            What developers build with{" "}
            <span className="gradient-text">Fetchium</span>
          </h2>
          <p className="mt-5 sm:mt-6 mx-auto max-w-xl text-base sm:text-xl text-slate-300 leading-relaxed">
            From quick RAG prototypes to production research pipelines — one API handles every retrieval workload.
          </p>
        </motion.div>

        <motion.div
          className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3"
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-60px" }}
          variants={{ hidden: {}, visible: { transition: { staggerChildren: 0.08 } } }}
        >
          {cases.map((item, i) => {
            const Icon = item.icon;
            return (
              <motion.div
                key={item.title}
                variants={{
                  hidden: { opacity: 0, y: 24 },
                  visible: { opacity: 1, y: 0, transition: { duration: 0.5, ease: [0.22, 1, 0.36, 1] } },
                }}
                className={`group relative flex flex-col rounded-2xl border bg-gradient-to-br ${item.color} ${item.border} p-6 transition-all duration-300 hover:shadow-[0_16px_40px_rgba(0,0,0,0.3)] ${i === 0 ? "lg:col-span-2" : ""}`}
              >
                <div className={`mb-4 flex h-11 w-11 items-center justify-center rounded-xl ${item.iconBg}`}>
                  <Icon className={`h-6 w-6 ${item.iconColor}`} strokeWidth={1.75} />
                </div>

                <h3 className="mb-3 text-lg sm:text-xl font-bold text-slate-100">
                  {item.title}
                </h3>
                <p className="mb-4 flex-1 text-[15px] sm:text-base leading-relaxed text-slate-300">
                  {item.description}
                </p>

                {/* Tags */}
                <div className="mb-4 flex flex-wrap gap-2">
                  {item.tags.map((tag) => (
                    <span
                      key={tag}
                      className="rounded-md border border-slate-700 bg-slate-900/50 px-2.5 py-1 text-[13px] font-semibold text-slate-300"
                    >
                      {tag}
                    </span>
                  ))}
                </div>

                <Link
                  href={item.href}
                  className="group/link inline-flex items-center gap-1.5 text-[15px] font-semibold text-indigo-400 transition-colors hover:text-indigo-300"
                >
                  Learn more
                  <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover/link:translate-x-0.5" />
                </Link>
              </motion.div>
            );
          })}
        </motion.div>
      </div>
    </section>
  );
}
