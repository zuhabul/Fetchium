"use client";

import Link from "next/link";
import { useCallback, useEffect, useRef, useState } from "react";
import { Send, Copy, Check, ExternalLink, Loader2, Clock, Hash, Shield, RefreshCw, ChevronDown, X } from "lucide-react";
import { endpointByPath, playgroundEndpoints, resolveEndpointPath } from "@/lib/dashboard-catalog";
import type { DashboardEndpoint } from "@/lib/dashboard-catalog";
import {
  DASHBOARD_CARD,
  DASHBOARD_CARD_PADDED,
  DASHBOARD_PAGE_HEADER,
  DASHBOARD_PAGE_LEAD,
  DASHBOARD_PAGE_STACK,
  DASHBOARD_PANEL,
  DASHBOARD_PANEL_HEADER,
  DASHBOARD_PANEL_ROW,
} from "@/lib/dashboard-layout";

/* ─── Types ──────────────────────────────────────────────────────────── */

type PlaygroundResponse = {
  status: number;
  duration_ms: number;
  rate_limit: {
    limit: string | null;
    remaining: string | null;
    reset: string | null;
  };
  data: unknown;
};

type ResponseTab = "body" | "meta" | "headers";

/* ─── Page ───────────────────────────────────────────────────────────── */

export default function PlaygroundPage() {
  const [endpoint, setEndpoint] = useState(playgroundEndpoints[0].path);
  const [body, setBody] = useState(playgroundEndpoints[0].sampleBody || "{}");
  const [pathParam, setPathParam] = useState("");
  const [response, setResponse] = useState<PlaygroundResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [copiedCurl, setCopiedCurl] = useState(false);
  const [activeTab, setActiveTab] = useState<ResponseTab>("body");

  // Endpoint picker
  const [sheetOpen, setSheetOpen] = useState(false);
  // Guide collapse on mobile
  const [guideOpen, setGuideOpen] = useState(false);

  // Job polling
  const [jobId, setJobId] = useState<string | null>(null);
  const [polling, setPolling] = useState(false);

  const selectedEndpoint = endpointByPath(endpoint);
  const isGetMethod = selectedEndpoint?.method === "GET" || selectedEndpoint?.supportsBody === false;
  const requiresPathParam = selectedEndpoint?.requiresPathParam;

  function selectEndpoint(path: string) {
    const next = endpointByPath(path);
    setEndpoint(path);
    setBody(next?.sampleBody || "{}");
    setPathParam("");
    setResponse(null);
    setError(null);
    setJobId(null);
    setSheetOpen(false);
  }

  const send = useCallback(async () => {
    setLoading(true);
    setResponse(null);
    setError(null);
    setJobId(null);
    try {
      let resolvedEndpoint = endpoint;
      if (requiresPathParam && pathParam.trim()) {
        resolvedEndpoint = endpoint.replace(`:${requiresPathParam}`, encodeURIComponent(pathParam.trim()));
      } else if (requiresPathParam && !pathParam.trim()) {
        setError(`Path parameter "${requiresPathParam}" is required.`);
        setLoading(false);
        return;
      }

      const payload = isGetMethod ? undefined : JSON.parse(body);
      const res = await fetch("/api/playground", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          endpoint: resolvedEndpoint,
          method: selectedEndpoint?.method || "POST",
          payload,
        }),
      });
      const json = (await res.json()) as PlaygroundResponse;
      if (!res.ok) {
        setError((json as unknown as { message?: string }).message || "Request failed");
        return;
      }
      setResponse(json);
      setActiveTab("body");

      // Check if this is a job submission response
      const data = json.data as Record<string, unknown> | undefined;
      if (data && typeof data === "object") {
        const meta = data.meta as Record<string, unknown> | undefined;
        const jid = (data.job_id as string) || (meta?.job_id as string);
        if (jid && json.status === 202) {
          setJobId(jid);
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Invalid JSON or request failed");
    } finally {
      setLoading(false);
    }
  }, [endpoint, body, pathParam, isGetMethod, requiresPathParam, selectedEndpoint]);

  const pollJob = useCallback(async () => {
    if (!jobId) return;
    setPolling(true);
    try {
      const res = await fetch("/api/playground", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          endpoint: `/v1/jobs/${jobId}`,
          method: "GET",
        }),
      });
      const json = (await res.json()) as PlaygroundResponse;
      setResponse(json);
      setActiveTab("body");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Job polling failed");
    } finally {
      setPolling(false);
    }
  }, [jobId]);

  const copyResponse = async () => {
    if (!response) return;
    const text = typeof response.data === "string" ? response.data : JSON.stringify(response.data, null, 2);
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const copyCurl = async () => {
    if (!selectedEndpoint?.sampleCurl) return;
    await navigator.clipboard.writeText(selectedEndpoint.sampleCurl);
    setCopiedCurl(true);
    setTimeout(() => setCopiedCurl(false), 2000);
  };

  return (
    <div className={DASHBOARD_PAGE_STACK}>
      {/* ── Header: endpoint selector + send button ── */}
      <div className="space-y-3">
        <div>
          <h1 className="text-xl font-bold text-[var(--text-primary)] sm:text-2xl">Playground</h1>
          <p className="mt-1 text-sm text-[var(--text-muted)]">
            Live API console through the authenticated dashboard proxy.
          </p>
        </div>

        {/* Endpoint selector trigger */}
        <button
          type="button"
          onClick={() => setSheetOpen(true)}
          className="flex w-full items-center gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2.5 text-left transition-colors hover:border-[var(--brand-border)] hover:bg-[var(--surface-hover)]"
        >
          <span className={`inline-flex flex-shrink-0 items-center rounded-md px-2 py-0.5 text-xs font-medium ${
            isGetMethod ? "bg-emerald-500/10 text-emerald-300" : "bg-brand-500/10 text-brand-300"
          }`}>
            {selectedEndpoint?.method || "POST"}
          </span>
          <span className="min-w-0 flex-1 truncate font-mono text-sm text-[var(--text-primary)]">
            {endpoint}
          </span>
          <ChevronDown className="h-4 w-4 flex-shrink-0 text-[var(--text-faint)]" />
        </button>

        {/* Send button — always visible, prominent on mobile */}
        <button
          onClick={() => void send()}
          disabled={loading}
          className="inline-flex w-full items-center justify-center gap-2 rounded-xl bg-brand-500 px-4 py-3 text-sm font-medium text-white transition-colors hover:bg-brand-600 disabled:opacity-50 sm:w-auto sm:py-2.5"
        >
          {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Send className="h-4 w-4" />}
          {loading ? "Sending..." : "Send Request"}
        </button>
      </div>

      {/* Endpoint picker bottom sheet */}
      <EndpointSheet
        open={sheetOpen}
        selected={endpoint}
        onSelect={selectEndpoint}
        onClose={() => setSheetOpen(false)}
      />

      {/* ── Request body / path param ── */}
      <section className={`${DASHBOARD_CARD_PADDED} space-y-3`}>
        <h2 className="text-sm font-medium text-[var(--text-primary)]">
          {isGetMethod ? "Request parameters" : "Request body"}
        </h2>

        {requiresPathParam && (
          <div>
            <label className="mb-1.5 block text-xs font-medium text-[var(--text-muted)]">{requiresPathParam}</label>
            <input
              type="text"
              value={pathParam}
              onChange={(e) => setPathParam(e.target.value)}
              placeholder={`Enter ${requiresPathParam}...`}
              className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-faint)] focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
            />
          </div>
        )}

        {!isGetMethod && (
          <textarea
            value={body}
            onChange={(e) => setBody(e.target.value)}
            rows={8}
            className={`w-full ${DASHBOARD_CARD} p-3 font-mono text-xs leading-relaxed text-[var(--text-secondary)] outline-none resize-none focus:border-brand-500/30 sm:p-4 sm:text-sm sm:rows-14`}
            spellCheck={false}
          />
        )}
      </section>

      {/* ── Response panel ── */}
      <section className={`${DASHBOARD_CARD} overflow-hidden`}>
        {/* Response header */}
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-[var(--border-subtle)] px-3 py-3 sm:px-5">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-[var(--text-primary)]">Response</span>
            {response && (
              <>
                <span className={`rounded-md px-1.5 py-0.5 text-[11px] font-medium ${
                  response.status >= 200 && response.status < 300
                    ? "bg-emerald-500/10 text-emerald-300"
                    : response.status === 202
                      ? "bg-purple-500/10 text-purple-300"
                      : "bg-red-500/10 text-red-300"
                }`}>
                  {response.status}
                </span>
                <span className="text-[11px] text-[var(--text-faint)]">{response.duration_ms}ms</span>
              </>
            )}
          </div>
          <div className="flex items-center gap-2">
            {jobId && (
              <button
                onClick={() => void pollJob()}
                disabled={polling}
                className="inline-flex items-center gap-1 rounded-lg border border-purple-500/30 px-2 py-1 text-[11px] text-purple-300 hover:bg-purple-500/10"
              >
                {polling ? <Loader2 className="h-3 w-3 animate-spin" /> : <RefreshCw className="h-3 w-3" />}
                Poll
              </button>
            )}
            {response && (
              <button onClick={() => void copyResponse()} className="flex items-center gap-1 text-[11px] text-[var(--text-muted)] hover:text-[var(--text-primary)]">
                {copied ? <Check className="h-3 w-3 text-green-400" /> : <Copy className="h-3 w-3" />}
                {copied ? "Copied" : "Copy"}
              </button>
            )}
          </div>
        </div>

        {/* Tabs */}
        {response && (
          <div className="flex border-b border-[var(--border-subtle)]">
            {(["body", "meta", "headers"] as ResponseTab[]).map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={`flex-1 px-2 py-2 text-center text-[11px] font-medium transition-colors sm:flex-none sm:px-4 sm:text-xs ${
                  activeTab === tab
                    ? "border-b-2 border-brand-500 text-brand-400"
                    : "text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                }`}
              >
                {tab === "headers" ? "Limits" : tab === "body" ? "Body" : "Meta"}
              </button>
            ))}
          </div>
        )}

        {error && <div className="p-3 text-sm text-[var(--danger-text)] sm:p-4">{error}</div>}

        {response ? (
          <div className="max-h-[50vh] overflow-auto sm:max-h-[32rem]">
            {activeTab === "body" && (
              <pre className="whitespace-pre-wrap break-all p-3 font-mono text-[11px] leading-relaxed text-[var(--text-secondary)] sm:break-normal sm:p-5 sm:text-xs">
                {typeof response.data === "string"
                  ? response.data
                  : JSON.stringify(response.data, null, 2)}
              </pre>
            )}
            {activeTab === "meta" && (
              <div className="space-y-2 p-3 sm:space-y-3 sm:p-5">
                <MetaRow icon={Hash} label="Status" value={String(response.status)} />
                <MetaRow icon={Clock} label="Duration" value={`${response.duration_ms}ms`} />
                {jobId && <MetaRow icon={RefreshCw} label="Job ID" value={jobId} />}
              </div>
            )}
            {activeTab === "headers" && (
              <div className="space-y-2 p-3 sm:space-y-3 sm:p-5">
                <MetaRow icon={Shield} label="Limit" value={response.rate_limit.limit || "—"} />
                <MetaRow icon={Shield} label="Left" value={response.rate_limit.remaining || "—"} />
                <MetaRow icon={Clock} label="Reset" value={response.rate_limit.reset || "—"} />
              </div>
            )}
          </div>
        ) : !error ? (
          <div className="flex min-h-32 items-center justify-center px-4 text-center text-sm text-[var(--text-faint)] sm:min-h-56">
            {loading ? "Waiting for response..." : "Send a request to inspect the live response"}
          </div>
        ) : null}
      </section>

      {/* ── Endpoint guide (collapsible on mobile) ── */}
      <section className={`${DASHBOARD_CARD} overflow-hidden`}>
        <button
          type="button"
          onClick={() => setGuideOpen(!guideOpen)}
          className="flex w-full items-center justify-between gap-3 px-3 py-3 text-left sm:px-5 sm:py-4"
        >
          <div className="min-w-0">
            <h2 className="text-sm font-medium text-[var(--text-primary)]">
              {selectedEndpoint?.label || "Endpoint"} guide
            </h2>
            <p className="mt-0.5 truncate text-xs text-[var(--text-faint)]">
              {selectedEndpoint?.description || "Select an endpoint to see details."}
            </p>
          </div>
          <div className="flex items-center gap-2">
            {selectedEndpoint?.docsHref && (
              <Link
                href={selectedEndpoint.docsHref}
                target="_blank"
                rel="noopener noreferrer"
                onClick={(e) => e.stopPropagation()}
                className="hidden items-center gap-1 text-xs text-brand-400 hover:underline sm:inline-flex"
              >
                Docs <ExternalLink className="h-3 w-3" />
              </Link>
            )}
            <ChevronDown className={`h-4 w-4 text-[var(--text-faint)] transition-transform ${guideOpen ? "rotate-180" : ""}`} />
          </div>
        </button>

        {guideOpen && (
          <div className="space-y-4 border-t border-[var(--border-subtle)] px-3 py-4 sm:px-5">
            {/* Meta tiles */}
            <div className="grid grid-cols-3 gap-2 sm:gap-3">
              <MetaTile label="Category" value={selectedEndpoint?.category || "—"} />
              <MetaTile label="Method" value={selectedEndpoint?.method || "POST"} />
              <MetaTile label="Path" value={selectedEndpoint?.path || endpoint} mono />
            </div>

            {/* Capability badges */}
            <div className="flex flex-wrap gap-2">
              {selectedEndpoint?.playgroundSupported && <Badge label="Playground" color="bg-brand-500/10 text-brand-300" />}
              {selectedEndpoint?.asyncVariant && <Badge label="Async" color="bg-purple-500/10 text-purple-300" />}
              {selectedEndpoint?.pollingRoute && <Badge label="Pollable" color="bg-amber-500/10 text-amber-300" />}
            </div>

            {/* Docs link (mobile) */}
            {selectedEndpoint?.docsHref && (
              <Link
                href={selectedEndpoint.docsHref}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 text-xs text-brand-400 hover:underline sm:hidden"
              >
                Open documentation <ExternalLink className="h-3 w-3" />
              </Link>
            )}

            {/* Curl snippet */}
            {selectedEndpoint?.sampleCurl && (
              <div className="space-y-2">
                <div className="flex items-center justify-between gap-3">
                  <span className="text-[10px] uppercase tracking-[0.18em] text-white/30">Direct curl</span>
                  <button onClick={() => void copyCurl()} className="inline-flex items-center gap-1 rounded-lg border border-[var(--border-subtle)] px-2 py-1 text-[11px] text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]">
                    <Copy className="h-3 w-3" />
                    {copiedCurl ? "Copied" : "Copy"}
                  </button>
                </div>
                <pre className="max-h-40 overflow-auto whitespace-pre-wrap break-all rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-3 text-[11px] leading-relaxed text-[var(--text-secondary)] sm:max-h-56 sm:break-normal sm:p-4 sm:text-xs">
                  {selectedEndpoint.sampleCurl}
                </pre>
              </div>
            )}
          </div>
        )}
      </section>
    </div>
  );
}

/* ─── Sub-components ─────────────────────────────────────────────────── */

function MetaTile({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-2 py-2 sm:px-3 sm:py-3">
      <div className="text-[10px] text-[var(--text-faint)] sm:text-xs">{label}</div>
      <div className={`mt-0.5 truncate text-xs font-medium text-[var(--text-primary)] sm:mt-1 sm:text-sm ${mono ? "font-mono" : ""}`}>
        {value}
      </div>
    </div>
  );
}

function Badge({ label, color }: { label: string; color: string }) {
  return (
    <span className={`inline-flex rounded-full border border-transparent px-2 py-0.5 text-[10px] font-medium sm:px-2.5 sm:text-xs ${color}`}>
      {label}
    </span>
  );
}

function MetaRow({ icon: Icon, label, value }: { icon: typeof Clock; label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2 sm:gap-3 sm:px-4 sm:py-3">
      <div className="flex items-center gap-1.5 sm:gap-2">
        <Icon className="h-3 w-3 text-[var(--text-faint)] sm:h-3.5 sm:w-3.5" />
        <span className="text-xs text-[var(--text-muted)] sm:text-sm">{label}</span>
      </div>
      <span className="truncate font-mono text-xs text-[var(--text-primary)] sm:text-sm">{value}</span>
    </div>
  );
}

/* ─── Endpoint Bottom Sheet ──────────────────────────────────────────── */

const CATEGORY_LABELS: Record<string, string> = {
  "Core API": "Core",
  Research: "Research",
  Media: "Media",
  Social: "Social",
  "Async Jobs": "Jobs",
};

function EndpointSheet({
  open,
  selected,
  onSelect,
  onClose,
}: {
  open: boolean;
  selected: string;
  onSelect: (path: string) => void;
  onClose: () => void;
}) {
  const backdropRef = useRef<HTMLDivElement>(null);
  const [search, setSearch] = useState("");

  useEffect(() => {
    if (open) {
      document.body.style.overflow = "hidden";
      setSearch("");
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [open]);

  if (!open) return null;

  const filtered = search.trim()
    ? playgroundEndpoints.filter(
        (ep) =>
          ep.label.toLowerCase().includes(search.toLowerCase()) ||
          ep.path.toLowerCase().includes(search.toLowerCase()),
      )
    : playgroundEndpoints;

  const grouped = new Map<string, typeof playgroundEndpoints>();
  for (const ep of filtered) {
    const list = grouped.get(ep.category) || [];
    list.push(ep);
    grouped.set(ep.category, list);
  }

  return (
    <div
      ref={backdropRef}
      className="fixed inset-0 z-50 flex items-end justify-center sm:items-center"
      onClick={(e) => {
        if (e.target === backdropRef.current) onClose();
      }}
    >
      <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" />

      <div className="relative flex max-h-[85vh] w-full flex-col overflow-hidden rounded-t-2xl border border-[var(--border-subtle)] bg-[var(--surface-base)] shadow-2xl sm:max-w-lg sm:rounded-2xl">
        {/* Grab handle */}
        <div className="flex justify-center pb-1 pt-3 sm:hidden">
          <div className="h-1 w-10 rounded-full bg-[var(--text-faint)]/30" />
        </div>

        {/* Header */}
        <div className="flex items-center justify-between gap-3 border-b border-[var(--border-subtle)] px-4 py-3">
          <h2 className="text-base font-semibold text-[var(--text-primary)]">Select endpoint</h2>
          <button
            onClick={onClose}
            className="rounded-lg p-1.5 text-[var(--text-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Search */}
        <div className="border-b border-[var(--border-subtle)] px-4 py-3">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search endpoints..."
            autoFocus
            className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-faint)] focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
          />
        </div>

        {/* List */}
        <div className="flex-1 overflow-y-auto overscroll-contain px-2 py-2">
          {filtered.length === 0 && (
            <div className="px-3 py-6 text-center text-sm text-[var(--text-faint)]">
              No endpoints match your search.
            </div>
          )}

          {Array.from(grouped.entries()).map(([category, endpoints]) => (
            <div key={category} className="mb-2">
              <div className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--text-faint)]">
                {CATEGORY_LABELS[category] || category}
              </div>
              {endpoints.map((ep) => {
                const isActive = ep.path === selected;
                return (
                  <button
                    key={ep.path}
                    onClick={() => onSelect(ep.path)}
                    className={`flex w-full items-start gap-3 rounded-lg px-3 py-3 text-left transition-colors ${
                      isActive
                        ? "bg-brand-500/10 border border-brand-500/20"
                        : "border border-transparent hover:bg-[var(--surface-hover)]"
                    }`}
                  >
                    <span className={`mt-0.5 inline-flex flex-shrink-0 rounded-md px-1.5 py-0.5 text-[10px] font-bold ${
                      ep.method === "GET"
                        ? "bg-emerald-500/15 text-emerald-400"
                        : "bg-brand-500/15 text-brand-400"
                    }`}>
                      {ep.method}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-[var(--text-primary)]">{ep.label}</span>
                        {isActive && <Check className="h-3.5 w-3.5 flex-shrink-0 text-brand-400" />}
                      </div>
                      <span className="mt-0.5 block truncate font-mono text-xs text-[var(--text-muted)]">
                        {ep.path}
                      </span>
                    </div>
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
