"use client";

import { useState } from "react";
import { Send, Copy, Check } from "lucide-react";

const endpoints = ["/v1/search", "/v1/scrape", "/v1/research", "/v1/youtube/search"];

const defaultBodies: Record<string, string> = {
  "/v1/search": JSON.stringify({ query: "rust async programming", max_sources: 5, tier: "summary" }, null, 2),
  "/v1/scrape": JSON.stringify({ url: "https://doc.rust-lang.org/book/", token_budget: 3000 }, null, 2),
  "/v1/research": JSON.stringify({ query: "best practices for LLM agents in 2025", token_budget: 4000 }, null, 2),
  "/v1/youtube/search": JSON.stringify({ query: "rust programming tutorial", max_results: 5 }, null, 2),
};

export default function PlaygroundPage() {
  const [endpoint, setEndpoint] = useState(endpoints[0]);
  const [body, setBody] = useState(defaultBodies[endpoints[0]]);
  const [response, setResponse] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  const send = async () => {
    setLoading(true);
    setResponse(null);
    // Simulate API call (in real app, use user's key)
    await new Promise(r => setTimeout(r, 1200));
    setResponse(JSON.stringify({
      meta: { query: "rust async programming", tier: "summary", tokens_used: 1247, sources_count: 5, duration_ms: 834 },
      results: [
        { title: "Async Programming in Rust", url: "https://rust-lang.github.io/async-book/", score: 0.94 },
        { title: "tokio.rs — async runtime for Rust", url: "https://tokio.rs", score: 0.91 },
      ],
    }, null, 2));
    setLoading(false);
  };

  const copy = async () => {
    if (!response) return;
    await navigator.clipboard.writeText(response);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold text-white">Playground</h1>
        <p className="text-sm text-white/40 mt-1">Explore the API interactively using your key.</p>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        {/* Request */}
        <div className="space-y-3">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium text-white/40 uppercase tracking-wider">POST</span>
            <select
              value={endpoint}
              onChange={e => {
                setEndpoint(e.target.value);
                setBody(defaultBodies[e.target.value]);
              }}
              className="flex-1 rounded-lg border border-white/10 bg-surface-2 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
            >
              {endpoints.map(e => <option key={e} value={e}>{e}</option>)}
            </select>
          </div>

          <textarea
            value={body}
            onChange={e => setBody(e.target.value)}
            rows={14}
            className="w-full rounded-xl border border-white/5 bg-surface-1 p-4 font-mono text-sm text-white/80 outline-none focus:border-brand-500/30 resize-none"
            spellCheck={false}
          />

          <button
            onClick={send}
            disabled={loading}
            className="flex w-full items-center justify-center gap-2 rounded-xl bg-brand-500 py-2.5 text-sm font-medium text-white hover:bg-brand-600 disabled:opacity-50 transition-colors"
          >
            {loading ? (
              <span className="animate-pulse">Sending...</span>
            ) : (
              <><Send className="h-4 w-4" /> Send Request</>
            )}
          </button>
        </div>

        {/* Response */}
        <div className="rounded-xl border border-white/5 bg-surface-1 overflow-hidden">
          <div className="flex items-center justify-between border-b border-white/5 px-4 py-3">
            <span className="text-sm font-medium text-white">Response</span>
            {response && (
              <button onClick={copy} className="flex items-center gap-1 text-xs text-white/40 hover:text-white">
                {copied ? <Check className="h-3.5 w-3.5 text-green-400" /> : <Copy className="h-3.5 w-3.5" />}
                {copied ? "Copied" : "Copy"}
              </button>
            )}
          </div>
          {response ? (
            <pre className="p-4 text-xs leading-relaxed text-white/70 overflow-auto max-h-96 font-mono">
              {response}
            </pre>
          ) : (
            <div className="flex items-center justify-center h-64 text-white/20 text-sm">
              {loading ? "Waiting for response..." : "Hit Send to see the response"}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
