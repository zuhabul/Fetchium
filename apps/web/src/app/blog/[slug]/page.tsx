import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

interface Props {
  params: Promise<{ slug: string }>;
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  return {
    title: `${slug.replace(/-/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())} — Fetchium Blog`,
    description: "Developer resources for AI-native search: RAG retrieval, web extraction, and AI agent tooling.",
  };
}

const articles: Record<string, {
  title: string;
  category: string;
  date: string;
  readTime: string;
  body: { type: "p" | "h2" | "h3" | "code" | "ul"; content: string | string[] }[];
}> = {
  "how-to-build-rag-pipeline-fetchium": {
    title: "How to Build a RAG Pipeline with Fetchium and LangChain",
    category: "RAG Retrieval",
    date: "March 3, 2026",
    readTime: "8 min read",
    body: [
      { type: "p", content: "A Retrieval-Augmented Generation (RAG) pipeline needs three things: a way to search the web, a way to extract clean content, and a way to pass that content to your LLM. Most teams stitch together a SERP scraper, a web crawler, and a token counter. Fetchium replaces all three with one API call." },
      { type: "h2", content: "Prerequisites" },
      { type: "ul", content: ["Python 3.11+", "A Fetchium API key (free at app.fetchium.com)", "langchain and fetchium Python packages"] },
      { type: "h2", content: "Step 1: Install the packages" },
      { type: "code", content: "pip install langchain fetchium" },
      { type: "h2", content: "Step 2: Initialize the Fetchium retriever" },
      { type: "code", content: `from fetchium import FetchiumRetriever

retriever = FetchiumRetriever(
    api_key="your_api_key",
    k=5,                    # number of results
    token_budget=4096,      # max tokens per result
    extract_content=True    # full CEP extraction
)` },
      { type: "h2", content: "Step 3: Build the RAG chain" },
      { type: "code", content: `from langchain.chains import RetrievalQA
from langchain.llms import Anthropic

chain = RetrievalQA.from_chain_type(
    llm=Anthropic(model="claude-3-5-sonnet"),
    chain_type="stuff",
    retriever=retriever
)

result = chain.run("What are the best async patterns in Rust?")
print(result)` },
      { type: "p", content: "The retriever automatically handles multi-backend search, content extraction, token budgeting, and citation tracking. Your LLM receives clean, relevant content ready to use." },
      { type: "h2", content: "What Fetchium does behind the scenes" },
      { type: "ul", content: [
        "Dispatches your query to 11+ backends in parallel (DuckDuckGo, Brave, GitHub, StackOverflow, and more)",
        "Ranks results using HyperFusion — 8 signals including BM25, semantic similarity, and source authority",
        "Extracts clean content from each result URL using the 5-layer CEP pipeline",
        "Packs the most relevant content into your 4,096-token budget using QATBE",
        "Returns structured citations for every fact"
      ]},
      { type: "h2", content: "Further reading" },
      { type: "p", content: "See the Fetchium API reference at https://docs.fetchium.com/api/search and the Python SDK docs at https://docs.fetchium.com/sdk/python for the full parameter reference." },
    ],
  },
  "token-budgeted-extraction-llm-cost": {
    title: "Token-Budgeted Extraction: Why Context Size Matters for LLM Cost",
    category: "Web Extraction",
    date: "March 1, 2026",
    readTime: "6 min read",
    body: [
      { type: "p", content: "A typical news article is 1,500 words. The raw HTML of that page — including navigation, ads, scripts, and boilerplate — is 40,000–100,000 tokens. If you send the raw HTML to your LLM, you're paying 20–60x more than necessary and getting worse results (LLMs lose focus in long, noisy contexts)." },
      { type: "h2", content: "The problem with naive extraction" },
      { type: "p", content: "Most web scraping approaches either (a) send raw HTML to the LLM and let it figure it out, or (b) apply a generic boilerplate remover that strips everything and loses structure. Neither approach is query-aware. QATBE takes a different approach." },
      { type: "h2", content: "How QATBE works" },
      { type: "ul", content: [
        "Segment the extracted content into meaningful units (paragraphs, headings, lists, code blocks)",
        "Score each segment by BM25 relevance to your specific query",
        "Pack the highest-scoring segments into your token budget using a greedy knapsack algorithm",
        "Preserve the document order of selected segments so the LLM gets coherent context"
      ]},
      { type: "h2", content: "Real-world impact" },
      { type: "p", content: "In our internal benchmarks on 500 web pages, QATBE reduced context size by an average of 78% while preserving 94% of query-relevant information. At GPT-4o pricing ($5/M input tokens), that's a reduction from ~$250 to ~$55 for 10,000 pages." },
      { type: "h2", content: "Using QATBE via the API" },
      { type: "code", content: `curl -X POST ***REMOVED***/v1/scrape \\
  -H "Authorization: Bearer YOUR_KEY" \\
  -d '{
    "url": "https://example.com/article",
    "query": "async rust patterns",
    "token_budget": 4096
  }'` },
      { type: "p", content: "The token_budget parameter sets your target. QATBE guarantees the response fits within it while maximizing query-relevant content." },
    ],
  },
};

export default async function BlogArticlePage({ params }: Props) {
  const { slug } = await params;
  const article = articles[slug];

  if (!article) {
    return (
      <div className="min-h-screen bg-[#06070d] text-slate-100">
        <Navbar />
        <main className="pt-24 pb-16 px-4">
          <div className="mx-auto max-w-3xl text-center">
            <h1 className="text-3xl font-bold mb-4">Article Coming Soon</h1>
            <p className="text-slate-500 mb-6">This article is being written. Check back soon or browse other posts.</p>
            <Link href="/blog" className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-5 py-2.5 text-sm font-semibold text-white transition-all">
              ← Back to Blog
            </Link>
          </div>
        </main>
        <Footer />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-3xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <Link href="/blog" className="hover:text-slate-400">Blog</Link>
            <span>/</span>
            <span className="text-slate-400 truncate max-w-[200px]">{article.title}</span>
          </nav>

          {/* Article header */}
          <div className="mb-10">
            <div className="mb-4 flex flex-wrap items-center gap-3">
              <span className="rounded-full bg-indigo-500/12 border border-indigo-500/20 px-2.5 py-0.5 text-[11px] font-semibold text-indigo-300">
                {article.category}
              </span>
              <span className="text-[12px] text-slate-500">{article.date}</span>
              <span className="text-[12px] text-slate-600">·</span>
              <span className="text-[12px] text-slate-500">{article.readTime}</span>
            </div>
            <h1 className="text-2xl sm:text-3xl md:text-4xl font-bold leading-tight mb-4">
              {article.title}
            </h1>
          </div>

          {/* Article body */}
          <div className="prose prose-invert prose-sm sm:prose-base max-w-none">
            {article.body.map((block, i) => {
              if (block.type === "p") return <p key={i} className="text-slate-400 leading-relaxed mb-4">{block.content as string}</p>;
              if (block.type === "h2") return <h2 key={i} className="text-xl font-bold text-slate-100 mt-8 mb-3">{block.content as string}</h2>;
              if (block.type === "h3") return <h3 key={i} className="text-base font-semibold text-slate-200 mt-6 mb-2">{block.content as string}</h3>;
              if (block.type === "ul") return (
                <ul key={i} className="list-disc pl-5 space-y-1 mb-4 text-slate-400 text-sm">
                  {(block.content as string[]).map((item, j) => <li key={j}>{item}</li>)}
                </ul>
              );
              if (block.type === "code") return (
                <div key={i} className="mb-4 rounded-xl border border-white/8 bg-[#0d0f1a] overflow-hidden">
                  <pre className="p-4 text-[13px] font-mono text-slate-300 overflow-x-auto leading-relaxed">{block.content as string}</pre>
                </div>
              );
              return null;
            })}
          </div>

          {/* Related articles */}
          <div className="mt-12 pt-8 border-t border-white/6">
            <h3 className="text-base font-semibold mb-4">Related Articles</h3>
            <div className="grid sm:grid-cols-2 gap-3">
              {[
                { href: "/blog/token-budgeted-extraction-llm-cost", label: "Token-Budgeted Extraction: Why Context Size Matters" },
                { href: "/product/search", label: "Fetchium Search API — Technical Deep Dive" },
                { href: "https://docs.fetchium.com/algorithms/cep", label: "CEP Algorithm Documentation" },
                { href: "/compare/tavily", label: "Fetchium vs Tavily Comparison" },
              ].filter(l => l.href !== `/blog/${slug}`).slice(0, 3).map((l) => (
                <Link key={l.href} href={l.href} className="rounded-xl border border-white/6 bg-white/[0.02] p-3 text-[13px] text-slate-400 hover:text-slate-200 hover:bg-white/5 transition-all">
                  {l.label} →
                </Link>
              ))}
            </div>
          </div>

          {/* CTA */}
          <div className="mt-10 rounded-2xl border border-indigo-500/15 bg-indigo-500/5 p-6 text-center">
            <p className="text-sm font-semibold text-slate-200 mb-3">Try Fetchium free — 1,000 requests/month</p>
            <Link href="https://app.fetchium.com/register" target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-5 py-2.5 text-sm font-semibold text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] transition-all">
              Get API Key Free →
            </Link>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
