"use client";

import { useEffect, useState } from "react";
import { Send, Copy, Check } from "lucide-react";
import { appendRequestLog, loadDashboardConfig } from "@/lib/client-config";

const endpoints = [
  "/v1/search",
  "/v1/scrape",
  "/v1/research",
  "/v1/youtube/search",
  "/v1/youtube/analyze",
  "/v1/social/reddit",
];

const defaultBodies: Record<string, string> = {
  "/v1/search": JSON.stringify({ query: "rust async programming", max_sources: 5, tier: "summary" }, null, 2),
  "/v1/scrape": JSON.stringify({ url: "https://doc.rust-lang.org/book/", token_budget: 3000 }, null, 2),
  "/v1/research": JSON.stringify({ query: "best practices for LLM agents in 2025", token_budget: 4000 }, null, 2),
  "/v1/youtube/search": JSON.stringify({ query: "rust programming tutorial", max_results: 5 }, null, 2),
  "/v1/youtube/analyze": JSON.stringify({ url: "https://www.youtube.com/watch?v=PkZNo7MFNFg" }, null, 2),
  "/v1/social/reddit": JSON.stringify({ query: "rustlang", max_results: 10 }, null, 2),
};

export default function PlaygroundPage() {
  const [endpoint, setEndpoint] = useState(endpoints[0]);
  const [body, setBody] = useState(defaultBodies[endpoints[0]]);
  const [response, setResponse] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [apiBase, setApiBase] = useState("http://localhost:3050");

  useEffect(() => {
    const cfg = loadDashboardConfig();
    setApiKey(cfg.apiKey);
    setApiBase(cfg.apiBaseUrl);
  }, []);

  const send = async () => {
    setLoading(true);
    setResponse(null);
    setError(null);
    try {
      const payload = JSON.parse(body);
      const res = await fetch("/api/playground", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          apiKey,
          apiBase,
          endpoint,
          payload,
        }),
      });
      const json = (await res.json()) as { status?: number; duration_ms?: number; data?: unknown; message?: string };
      if (!res.ok) {
        setError(json.message || "Request failed");
        return;
      }
      setResponse(JSON.stringify(json, null, 2));
      appendRequestLog({
        endpoint,
        status: json.status || 0,
        latencyMs: json.duration_ms || 0,
        timeIso: new Date().toISOString(),
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : "Invalid JSON or request failed");
    } finally {
      setLoading(false);
    }
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
        <p className="text-sm text-white/40 mt-1">Live API requests through dashboard proxy routes.</p>
      </div>

      <div className="grid gap-3 md:grid-cols-2">
        <input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder="fetchium_..."
          className="rounded-lg border border-white/10 bg-surface-2 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
        />
        <input
          type="text"
          value={apiBase}
          onChange={(e) => setApiBase(e.target.value)}
          placeholder="http://localhost:3050"
          className="rounded-lg border border-white/10 bg-surface-2 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
        />
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <div className="space-y-3">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium rounded bg-brand-500/10 px-2 py-0.5 text-brand-300">POST</span>
            <select
              value={endpoint}
              onChange={(e) => {
                setEndpoint(e.target.value);
                setBody(defaultBodies[e.target.value]);
              }}
              className="flex-1 rounded-lg border border-white/10 bg-surface-2 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
            >
              {endpoints.map((e) => (
                <option key={e} value={e}>
                  {e}
                </option>
              ))}
            </select>
          </div>

          <textarea
            value={body}
            onChange={(e) => setBody(e.target.value)}
            rows={14}
            className="w-full rounded-xl border border-white/5 bg-surface-1 p-4 font-mono text-sm text-white/80 outline-none focus:border-brand-500/30 resize-none"
            spellCheck={false}
          />

          <button
            onClick={() => void send()}
            disabled={loading}
            className="flex w-full items-center justify-center gap-2 rounded-xl bg-brand-500 py-2.5 text-sm font-medium text-white hover:bg-brand-600 disabled:opacity-50 transition-colors"
          >
            {loading ? <span className="animate-pulse">Sending...</span> : <><Send className="h-4 w-4" /> Send Request</>}
          </button>
        </div>

        <div className="rounded-xl border border-white/5 bg-surface-1 overflow-hidden">
          <div className="flex items-center justify-between border-b border-white/5 px-4 py-3">
            <span className="text-sm font-medium text-white">Response</span>
            {response && (
              <button onClick={() => void copy()} className="flex items-center gap-1 text-xs text-white/40 hover:text-white">
                {copied ? <Check className="h-3.5 w-3.5 text-green-400" /> : <Copy className="h-3.5 w-3.5" />}
                {copied ? "Copied" : "Copy"}
              </button>
            )}
          </div>
          {error && <div className="p-4 text-sm text-red-300">{error}</div>}
          {response ? (
            <pre className="p-4 text-xs leading-relaxed text-white/70 overflow-auto max-h-96 font-mono">{response}</pre>
          ) : !error ? (
            <div className="flex items-center justify-center h-64 text-white/20 text-sm">
              {loading ? "Waiting for response..." : "Hit Send to see the response"}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}

