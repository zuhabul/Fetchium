"use client";

import { useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Copy, Check, Terminal, Code2 } from "lucide-react";

/* ── Language tab definitions ───────────────────────────────────────────── */

type Lang = "typescript" | "python" | "curl";

interface LangConfig {
  label: string;
  ext: string;
  code: string;
}

const examples: Record<Lang, LangConfig> = {
  typescript: {
    label: "TypeScript",
    ext: ".ts",
    code: `import { HyperSearchX } from "@hypersearchx/sdk";

const hsx = new HyperSearchX({
  apiKey: process.env.HSX_API_KEY!,
  baseUrl: "https://api.hypersearchx.zuhabul.com",
});

// Multi-source federated search
const results = await hsx.search("rust async programming", {
  backends: ["searxng", "brave", "github", "stackoverflow"],
  maxResults: 10,
  tier: "summary",       // key_facts | summary | detailed | complete
  tokenBudget: 2000,     // QATBE greedy-knapsack packing
  ranking: "hyperfusion" // 8-signal neural ranking
});

console.log(results.items[0].title);
console.log(results.meta.tokensUsed);    // always within budget
console.log(results.evidenceGraph);      // citations + trust scores

// Deep-extract any URL
const page = await hsx.extract("https://docs.rs/tokio", {
  format: "markdown",
  tokenBudget: 4096,
  layer: "readability",  // css | readability | headless | pdf | ocr
});`,
  },
  python: {
    label: "Python",
    ext: ".py",
    code: `from hypersearchx import HyperSearchX
import os

hsx = HyperSearchX(
    api_key=os.environ["HSX_API_KEY"],
    base_url="https://api.hypersearchx.zuhabul.com",
)

# Multi-source federated search
results = hsx.search(
    "rust async programming",
    backends=["searxng", "brave", "github", "stackoverflow"],
    max_results=10,
    tier="summary",        # key_facts | summary | detailed | complete
    token_budget=2000,     # QATBE greedy-knapsack packing
    ranking="hyperfusion", # 8-signal neural ranking
)

print(results.items[0].title)
print(results.meta.tokens_used)    # always within budget
print(results.evidence_graph)      # citations + trust scores

# Deep-extract any URL
page = hsx.extract(
    "https://docs.rs/tokio",
    format="markdown",
    token_budget=4096,
    layer="readability",   # css | readability | headless | pdf | ocr
)`,
  },
  curl: {
    label: "cURL",
    ext: ".sh",
    code: `# Federated search across 11 backends
curl -X POST https://api.hypersearchx.zuhabul.com/v1/search \\
  -H "Authorization: Bearer $HSX_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "rust async programming",
    "backends": ["searxng", "brave", "github", "stackoverflow"],
    "max_results": 10,
    "tier": "summary",
    "token_budget": 2000,
    "ranking": "hyperfusion"
  }'

# Deep-extract a URL (5-layer CEP)
curl -X POST https://api.hypersearchx.zuhabul.com/v1/extract \\
  -H "Authorization: Bearer $HSX_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://docs.rs/tokio",
    "format": "markdown",
    "token_budget": 4096,
    "layer": "readability"
  }'`,
  },
};

/* ── Mock API response ──────────────────────────────────────────────────── */

const mockResponse = {
  query: "rust async programming",
  tier: "summary",
  meta: {
    tokensUsed: 1984,
    tokenBudget: 2000,
    backends: ["searxng", "brave", "github", "stackoverflow"],
    latencyMs: 187,
    totalFetched: 43,
    afterRank: 10,
  },
  items: [
    {
      rank: 1,
      title: "Asynchronous Programming in Rust",
      url: "https://rust-lang.github.io/async-book",
      source: "brave",
      score: 0.97,
      snippet: "The async book covers futures, async/await syntax, Tokio runtime, and concurrent task management...",
      trustScore: 0.94,
    },
    {
      rank: 2,
      title: "tokio-rs/tokio — GitHub",
      url: "https://github.com/tokio-rs/tokio",
      source: "github",
      score: 0.93,
      snippet: "A runtime for writing reliable, asynchronous, and slim applications. Stars: 28k...",
      trustScore: 0.98,
    },
    {
      rank: 3,
      title: "How does async/await work in Rust?",
      url: "https://stackoverflow.com/questions/52835077",
      source: "stackoverflow",
      score: 0.88,
      snippet: "Accepted answer (1.2k votes): Rust's async/await desugars into state machines at compile time...",
      trustScore: 0.91,
    },
  ],
  evidenceGraph: {
    nodes: 10,
    edges: 23,
    consensusScore: 0.89,
  },
};

/* ── Syntax rendering ───────────────────────────────────────────────────── */

function renderCode(code: string, lang: Lang) {
  // Simple hand-tuned tokenizer for visual output
  const lines = code.split("\n");
  return lines.map((line, i) => {
    // Produce spans per segment – minimal but effective for demo
    const segments = tokenizeLine(line, lang);
    return (
      <div key={i} className="table-row">
        <span className="table-cell select-none pr-5 text-right text-slate-600 text-[12px]">
          {i + 1}
        </span>
        <span className="table-cell">
          {segments.map((seg, j) => (
            <span key={j} className={seg.cls}>
              {seg.text}
            </span>
          ))}
        </span>
      </div>
    );
  });
}

function tokenizeLine(
  line: string,
  _lang: Lang
): { text: string; cls: string }[] {
  // Segment the line with regex priority chain
  const result: { text: string; cls: string }[] = [];
  let remaining = line;

  const patterns: [RegExp, string][] = [
    [/^(\/\/.*|#.*)/, "token-comment"],
    [/^(import|from|export|const|let|var|async|await|return|print|process)\b/, "token-keyword"],
    [/^(new\s+\w+|\w+(?=\())\b/, "token-function"],
    [/^"[^"]*"|'[^']*'|`[^`]*`/, "token-string"],
    [/^\b\d+(\.\d+)?\b/, "token-number"],
    [/^[{}[\]().,;:?!\\]/, "token-punctuation"],
    [/^[-+*/%=<>!&|^~]+/, "token-operator"],
    [/^\b(HyperSearchX|hsx|results|page|meta|items)\b/, "token-type"],
    [/^\b\w+\b/, "text-slate-300"],
    [/^\s+/, "text-transparent"],
  ];

  while (remaining.length > 0) {
    let matched = false;
    for (const [re, cls] of patterns) {
      const m = remaining.match(re);
      if (m) {
        result.push({ text: m[0], cls });
        remaining = remaining.slice(m[0].length);
        matched = true;
        break;
      }
    }
    if (!matched) {
      result.push({ text: remaining[0], cls: "text-slate-400" });
      remaining = remaining.slice(1);
    }
  }

  return result;
}

/* ── Component ──────────────────────────────────────────────────────────── */

export default function CodeDemo() {
  const [lang, setLang] = useState<Lang>("typescript");
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(examples[lang].code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // clipboard not available in insecure context
    }
  }, [lang]);

  return (
    <section className="relative py-16 sm:py-28 px-4 overflow-hidden">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/2 top-1/2 h-[600px] w-[1000px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-indigo-500/4 blur-[140px]" />
      </div>

      <div className="relative mx-auto max-w-7xl">
        {/* Header */}
        <motion.div
          className="mb-10 sm:mb-16 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-3 sm:px-4 py-1.5 text-xs font-medium text-indigo-300">
            <Code2 className="h-3.5 w-3.5" />
            Simple API
          </div>
          <h2 className="text-2xl sm:text-3xl md:text-4xl font-bold tracking-tight text-slate-100">
            First result in{" "}
            <span className="gradient-text">60 seconds</span>
          </h2>
          <p className="mt-3 sm:mt-4 text-sm sm:text-lg text-slate-500">
            Install the SDK, paste your key, ship. Real multi-source search with
            zero boilerplate.
          </p>
        </motion.div>

        {/* Split layout */}
        <motion.div
          className="grid gap-4 lg:grid-cols-2"
          initial={{ opacity: 0, y: 32 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-60px" }}
          transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
        >
          {/* Code panel */}
          <div className="flex flex-col overflow-hidden rounded-2xl border border-[rgba(99,102,241,0.15)] bg-[rgba(13,17,23,0.9)] shadow-[0_24px_64px_rgba(0,0,0,0.5)]">
            {/* Tab bar */}
            <div className="flex items-center justify-between border-b border-white/6 bg-[rgba(6,7,13,0.6)] px-3 sm:px-4">
              <div className="flex">
                {(Object.keys(examples) as Lang[]).map((l) => (
                  <button
                    key={l}
                    onClick={() => setLang(l)}
                    className={`relative px-3 sm:px-4 py-3 text-xs sm:text-[13px] font-medium transition-colors duration-150 ${
                      lang === l
                        ? "text-indigo-300"
                        : "text-slate-500 hover:text-slate-300"
                    }`}
                  >
                    {lang === l && (
                      <motion.div
                        layoutId="tab-indicator"
                        className="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-indigo-500 to-violet-500"
                        transition={{ duration: 0.2 }}
                      />
                    )}
                    {examples[l].label}
                  </button>
                ))}
              </div>

              {/* Filename + copy */}
              <div className="flex items-center gap-2 sm:gap-3">
                <span className="text-[10px] sm:text-[11px] text-slate-600 hidden xs:block">
                  search{examples[lang].ext}
                </span>
                <button
                  onClick={handleCopy}
                  className="flex items-center gap-1.5 rounded-lg border border-white/8 px-2.5 py-1.5 text-[11px] sm:text-[12px] text-slate-500 transition-all hover:border-indigo-500/30 hover:text-slate-200 min-h-[32px]"
                >
                  <AnimatePresence mode="wait" initial={false}>
                    {copied ? (
                      <motion.span
                        key="check"
                        initial={{ scale: 0.7, opacity: 0 }}
                        animate={{ scale: 1, opacity: 1 }}
                        exit={{ scale: 0.7, opacity: 0 }}
                        className="flex items-center gap-1 text-emerald-400"
                      >
                        <Check className="h-3 w-3" />
                        <span className="hidden sm:inline">Copied</span>
                      </motion.span>
                    ) : (
                      <motion.span
                        key="copy"
                        initial={{ scale: 0.7, opacity: 0 }}
                        animate={{ scale: 1, opacity: 1 }}
                        exit={{ scale: 0.7, opacity: 0 }}
                        className="flex items-center gap-1"
                      >
                        <Copy className="h-3 w-3" />
                        <span className="hidden sm:inline">Copy</span>
                      </motion.span>
                    )}
                  </AnimatePresence>
                </button>
              </div>
            </div>

            {/* Code body */}
            <div className="flex-1 overflow-x-auto p-3 sm:p-5">
              <AnimatePresence mode="wait">
                <motion.pre
                  key={lang}
                  initial={{ opacity: 0, x: 8 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -8 }}
                  transition={{ duration: 0.2 }}
                  className="table w-full font-mono text-[11px] sm:text-[13px] leading-[1.7]"
                >
                  {renderCode(examples[lang].code, lang)}
                </motion.pre>
              </AnimatePresence>
            </div>

            {/* Install strip */}
            <div className="border-t border-white/6 bg-[rgba(6,7,13,0.4)] px-3 sm:px-5 py-2 sm:py-3">
              <div className="flex items-center gap-2 font-mono text-[11px] sm:text-[12px]">
                <Terminal className="h-3.5 w-3.5 shrink-0 text-indigo-400" />
                <span className="text-slate-600">$</span>
                <AnimatePresence mode="wait">
                  <motion.span
                    key={lang}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -4 }}
                    transition={{ duration: 0.2 }}
                    className="text-slate-400 truncate"
                  >
                    {lang === "typescript"
                      ? "npm install @hypersearchx/sdk"
                      : lang === "python"
                      ? "pip install hypersearchx"
                      : "curl -fsSL ..."}
                  </motion.span>
                </AnimatePresence>
              </div>
            </div>
          </div>

          {/* Response panel */}
          <div className="overflow-hidden rounded-2xl border border-[rgba(99,102,241,0.15)] bg-[rgba(13,17,23,0.9)] shadow-[0_24px_64px_rgba(0,0,0,0.5)]">
            {/* Panel header */}
            <div className="flex items-center justify-between border-b border-white/6 bg-[rgba(6,7,13,0.6)] px-3 sm:px-5 py-2.5 sm:py-3.5">
              <div className="flex items-center gap-2">
                <span className="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.6)]" />
                <span className="text-xs sm:text-[13px] font-medium text-slate-300">
                  API Response
                </span>
              </div>
              <div className="flex items-center gap-2 text-[10px] sm:text-[11px] text-slate-600">
                <span className="rounded-md border border-white/6 bg-white/3 px-1.5 sm:px-2 py-0.5">
                  200 OK
                </span>
                <span className="text-emerald-500">
                  {mockResponse.meta.latencyMs}ms
                </span>
              </div>
            </div>

            {/* Response body */}
            <div className="overflow-y-auto p-3 sm:p-5">
              {/* Meta strip */}
              <div className="mb-4 grid grid-cols-3 gap-2">
                {[
                  {
                    label: "Tokens",
                    value: `${mockResponse.meta.tokensUsed}/${mockResponse.meta.tokenBudget}`,
                  },
                  { label: "Backends", value: mockResponse.meta.backends.length },
                  { label: "Ranked", value: mockResponse.meta.afterRank },
                ].map((m) => (
                  <div
                    key={m.label}
                    className="rounded-xl border border-white/6 bg-white/2 p-2 sm:p-3 text-center"
                  >
                    <div className="text-[10px] sm:text-[11px] text-slate-600">{m.label}</div>
                    <div className="mt-0.5 text-[11px] sm:text-[13px] font-semibold text-slate-200">
                      {m.value}
                    </div>
                  </div>
                ))}
              </div>

              {/* Token bar */}
              <div className="mb-5">
                <div className="mb-1.5 flex items-center justify-between text-[11px]">
                  <span className="text-slate-600">Token budget usage</span>
                  <span className="text-indigo-400">
                    {Math.round(
                      (mockResponse.meta.tokensUsed /
                        mockResponse.meta.tokenBudget) *
                        100
                    )}
                    %
                  </span>
                </div>
                <div className="h-1.5 w-full overflow-hidden rounded-full bg-white/5">
                  <motion.div
                    className="h-full rounded-full bg-gradient-to-r from-indigo-500 to-violet-500"
                    initial={{ width: 0 }}
                    whileInView={{
                      width: `${(mockResponse.meta.tokensUsed / mockResponse.meta.tokenBudget) * 100}%`,
                    }}
                    viewport={{ once: true }}
                    transition={{ duration: 1, delay: 0.4, ease: "easeOut" }}
                  />
                </div>
              </div>

              {/* Result items */}
              <div className="space-y-2 sm:space-y-3">
                {mockResponse.items.map((item, i) => (
                  <motion.div
                    key={item.rank}
                    initial={{ opacity: 0, x: 12 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    viewport={{ once: true }}
                    transition={{ delay: 0.5 + i * 0.1, duration: 0.4 }}
                    className="rounded-xl border border-white/6 bg-white/2 p-3 sm:p-4 transition-colors hover:border-indigo-500/20 hover:bg-white/4"
                  >
                    <div className="mb-1 flex items-start justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-md bg-indigo-500/15 text-[10px] font-bold text-indigo-400">
                          {item.rank}
                        </span>
                        <span className="text-[11px] sm:text-[12px] font-medium text-slate-200 leading-tight truncate">
                          {item.title}
                        </span>
                      </div>
                      <span className="shrink-0 rounded-md border border-white/6 bg-white/3 px-1.5 py-0.5 text-[9px] sm:text-[10px] font-medium text-indigo-400">
                        {item.source}
                      </span>
                    </div>
                    <p className="mb-2 text-[10px] sm:text-[11px] leading-relaxed text-slate-600 line-clamp-2">
                      {item.snippet}
                    </p>
                    <div className="flex items-center gap-2 sm:gap-3">
                      <div className="flex items-center gap-1">
                        <span className="text-[10px] sm:text-[11px] text-slate-600">score</span>
                        <div className="h-1 w-10 sm:w-12 overflow-hidden rounded-full bg-white/5">
                          <div
                            className="h-full rounded-full bg-gradient-to-r from-indigo-500 to-violet-500"
                            style={{ width: `${item.score * 100}%` }}
                          />
                        </div>
                        <span className="text-[10px] sm:text-[11px] text-indigo-400">
                          {item.score}
                        </span>
                      </div>
                      <div className="flex items-center gap-1">
                        <span className="text-[10px] sm:text-[11px] text-slate-600">trust</span>
                        <span className="text-[10px] sm:text-[11px] text-emerald-400">
                          {item.trustScore}
                        </span>
                      </div>
                    </div>
                  </motion.div>
                ))}
              </div>

              {/* Evidence graph summary */}
              <div className="mt-3 sm:mt-4 rounded-xl border border-indigo-500/15 bg-indigo-500/5 p-3 sm:p-4">
                <div className="mb-2 text-[10px] sm:text-[11px] font-semibold text-indigo-300">
                  Evidence Graph
                </div>
                <div className="flex gap-3 sm:gap-4 text-[10px] sm:text-[11px] text-slate-500">
                  <span>
                    <span className="text-slate-300">
                      {mockResponse.evidenceGraph.nodes}
                    </span>{" "}
                    nodes
                  </span>
                  <span>
                    <span className="text-slate-300">
                      {mockResponse.evidenceGraph.edges}
                    </span>{" "}
                    edges
                  </span>
                  <span>
                    consensus{" "}
                    <span className="text-emerald-400">
                      {mockResponse.evidenceGraph.consensusScore}
                    </span>
                  </span>
                </div>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
