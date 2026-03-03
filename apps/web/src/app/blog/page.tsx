import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Blog — RAG Retrieval, Web Extraction, AI Agent Tooling",
  description:
    "Developer resources for AI-native search: tutorials, benchmarks, comparisons, and deep dives on RAG pipelines, web extraction, and AI agent tooling.",
};

const categories = [
  { label: "RAG Retrieval", slug: "rag-retrieval", color: "text-indigo-400", bg: "bg-indigo-500/10 border-indigo-500/20" },
  { label: "Web Extraction", slug: "web-extraction", color: "text-blue-400", bg: "bg-blue-500/10 border-blue-500/20" },
  { label: "Agent Tooling", slug: "agent-tooling", color: "text-violet-400", bg: "bg-violet-500/10 border-violet-500/20" },
  { label: "Benchmarks", slug: "benchmarks", color: "text-emerald-400", bg: "bg-emerald-500/10 border-emerald-500/20" },
  { label: "Comparisons", slug: "comparisons", color: "text-amber-400", bg: "bg-amber-500/10 border-amber-500/20" },
];

const posts = [
  {
    slug: "how-to-build-rag-pipeline-fetchium",
    title: "How to Build a RAG Pipeline with Fetchium and LangChain",
    excerpt: "A step-by-step guide to replacing a naive web search + scrape setup with Fetchium's federated, token-budgeted retrieval in your LangChain pipeline.",
    category: "RAG Retrieval",
    date: "March 3, 2026",
    readTime: "8 min read",
    featured: true,
  },
  {
    slug: "token-budgeted-extraction-llm-cost",
    title: "Token-Budgeted Extraction: Why Context Size Matters for LLM Cost",
    excerpt: "A 100-page website has 60,000+ tokens. Your LLM needs 2,000. The QATBE algorithm bridges that gap — here's how it works and why it saves 60–90% on context costs.",
    category: "Web Extraction",
    date: "March 1, 2026",
    readTime: "6 min read",
    featured: true,
  },
  {
    slug: "web-scraping-for-ai-vs-humans",
    title: "Web Scraping for AI vs. Web Scraping for Humans: What's Different",
    excerpt: "Traditional web scraping delivers HTML. AI needs semantic content, structured citations, and token-budgeted output. This post explains the architectural differences.",
    category: "Web Extraction",
    date: "February 28, 2026",
    readTime: "5 min read",
    featured: false,
  },
  {
    slug: "mcp-tools-claude-cursor",
    title: "Using Fetchium as an MCP Tool in Claude Desktop and Cursor",
    excerpt: "The Model Context Protocol lets AI clients call external tools without code. This tutorial shows how to configure Fetchium's MCP server in Claude Desktop and Cursor in under 5 minutes.",
    category: "Agent Tooling",
    date: "February 25, 2026",
    readTime: "4 min read",
    featured: false,
  },
  {
    slug: "search-api-benchmark-2025",
    title: "Search API Benchmark 2025: Latency, Accuracy, and Cost Compared",
    excerpt: "We tested Fetchium, Tavily, Exa, SerpAPI, and Brave Search with 50 standardized queries. Here are the results: latency, success rate, content quality, and total cost per 1K queries.",
    category: "Benchmarks",
    date: "February 20, 2026",
    readTime: "10 min read",
    featured: false,
  },
];

export default function BlogPage() {
  const featured = posts.filter((p) => p.featured);
  const rest = posts.filter((p) => !p.featured);

  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-5xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Blog</span>
          </nav>

          <div className="mb-10">
            <h1 className="text-3xl sm:text-4xl font-bold mb-3">Developer Resources</h1>
            <p className="text-slate-500 max-w-xl">
              Tutorials, benchmarks, and deep dives on RAG retrieval, web extraction,
              and AI agent tooling.
            </p>
          </div>

          {/* Categories */}
          <div className="mb-10 flex flex-wrap gap-2">
            <Link href="/blog" className="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-[12px] font-medium text-slate-300 hover:bg-white/10 transition-all">
              All
            </Link>
            {categories.map((c) => (
              <Link
                key={c.slug}
                href={`/blog?category=${c.slug}`}
                className={`rounded-full border px-3 py-1.5 text-[12px] font-medium ${c.bg} ${c.color} hover:opacity-80 transition-all`}
              >
                {c.label}
              </Link>
            ))}
          </div>

          {/* Featured posts */}
          <div className="mb-10 grid sm:grid-cols-2 gap-5">
            {featured.map((post) => (
              <Link
                key={post.slug}
                href={`/blog/${post.slug}`}
                className="group rounded-2xl border border-white/6 bg-white/[0.02] p-6 hover:border-indigo-500/25 hover:bg-indigo-500/5 transition-all"
              >
                <div className="mb-3 flex items-center gap-2">
                  <span className="rounded-full bg-indigo-500/12 border border-indigo-500/20 px-2.5 py-0.5 text-[10px] font-semibold text-indigo-300 uppercase tracking-wide">
                    {post.category}
                  </span>
                  <span className="text-[11px] text-slate-600">{post.date}</span>
                  <span className="text-[11px] text-slate-600">· {post.readTime}</span>
                </div>
                <h2 className="text-base font-bold text-slate-100 mb-2 group-hover:text-white transition-colors leading-snug">
                  {post.title}
                </h2>
                <p className="text-[13px] text-slate-500 leading-relaxed">{post.excerpt}</p>
                <span className="mt-4 inline-flex items-center gap-1 text-[12px] text-indigo-400 group-hover:text-indigo-300 transition-colors">
                  Read more →
                </span>
              </Link>
            ))}
          </div>

          {/* All posts */}
          <div className="space-y-3">
            {rest.map((post) => (
              <Link
                key={post.slug}
                href={`/blog/${post.slug}`}
                className="group flex items-start gap-5 rounded-xl border border-white/5 bg-white/[0.015] p-5 hover:border-white/10 hover:bg-white/[0.03] transition-all"
              >
                <div className="flex-1 min-w-0">
                  <div className="mb-1.5 flex flex-wrap items-center gap-2">
                    <span className="text-[11px] font-medium text-slate-500">{post.category}</span>
                    <span className="text-[11px] text-slate-700">·</span>
                    <span className="text-[11px] text-slate-600">{post.date}</span>
                    <span className="text-[11px] text-slate-700">·</span>
                    <span className="text-[11px] text-slate-600">{post.readTime}</span>
                  </div>
                  <h2 className="text-sm font-semibold text-slate-200 group-hover:text-white transition-colors">
                    {post.title}
                  </h2>
                  <p className="mt-1 text-[12px] text-slate-500 line-clamp-2 leading-relaxed">{post.excerpt}</p>
                </div>
                <span className="shrink-0 text-slate-600 group-hover:text-slate-400 transition-colors mt-0.5 text-lg">→</span>
              </Link>
            ))}
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
