"use client";

import { motion } from "framer-motion";
import Link from "next/link";
import { ArrowRight, Bot, FileSearch, Eye, Youtube, MessageSquare } from "lucide-react";

const cases = [
  {
    icon: Bot,
    title: "RAG Pipelines & AI Agents",
    description:
      "Drop Fetchium into any LangChain, LlamaIndex, or custom RAG stack. One call returns search results, extracted content, and citations — ready for your vector store or context window.",
    tags: ["LangChain", "LlamaIndex", "CrewAI", "AutoGen"],
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
      "The AMRS pipeline spawns 4 agent types in parallel — each searching a different angle, cross-validating findings, and assembling an evidence graph. Research that would take hours is done in minutes.",
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
      "Track any page, domain, or topic over time. Fetchium detects semantic changes — not just text diffs — and alerts you when the meaning of content shifts, not just when characters change.",
    tags: ["Real-time", "Semantic diff", "Alerts", "Webhooks"],
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
    href: "/docs/api/youtube",
    color: "from-red-500/15 to-orange-500/10",
    border: "border-red-500/20 hover:border-red-500/40",
    iconColor: "text-red-400",
    iconBg: "bg-red-500/10",
  },
  {
    icon: MessageSquare,
    title: "Social Intelligence",
    description:
      "Pull structured data from Reddit, HackerNews, and other social platforms. Understand sentiment, trending topics, and community signals across any niche — at API speed.",
    tags: ["Reddit", "HackerNews", "Sentiment", "Trends"],
    href: "/docs/api/social",
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
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-4 py-1.5 text-xs font-medium text-indigo-300">
            Built for real workloads
          </div>
          <h2 className="text-2xl sm:text-3xl md:text-4xl font-bold tracking-tight text-slate-100">
            What developers build with{" "}
            <span className="gradient-text">Fetchium</span>
          </h2>
          <p className="mt-4 sm:mt-5 mx-auto max-w-xl text-sm sm:text-lg text-slate-500">
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
                <div className={`mb-4 flex h-10 w-10 items-center justify-center rounded-xl ${item.iconBg}`}>
                  <Icon className={`h-5 w-5 ${item.iconColor}`} strokeWidth={1.75} />
                </div>

                <h3 className="mb-2 text-base sm:text-lg font-semibold text-slate-100">
                  {item.title}
                </h3>
                <p className="mb-4 flex-1 text-[13px] sm:text-sm leading-relaxed text-slate-500">
                  {item.description}
                </p>

                {/* Tags */}
                <div className="mb-4 flex flex-wrap gap-1.5">
                  {item.tags.map((tag) => (
                    <span
                      key={tag}
                      className="rounded-md border border-white/8 bg-white/4 px-2 py-0.5 text-[11px] font-medium text-slate-500"
                    >
                      {tag}
                    </span>
                  ))}
                </div>

                <Link
                  href={item.href}
                  className="group/link inline-flex items-center gap-1.5 text-[13px] font-medium text-slate-400 transition-colors hover:text-slate-200"
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
