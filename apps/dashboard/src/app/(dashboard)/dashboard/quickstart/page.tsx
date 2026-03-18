"use client";

import Link from "next/link";
import {
  Copy,
  ArrowRight,
  CheckCircle2,
  Rocket,
  TerminalSquare,
  AlertTriangle,
  XCircle,
  Loader2,
  Play,
  Clock,
  Activity,
  Zap,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import {
  DASHBOARD_CARD_PADDED,
  DASHBOARD_PAGE_HEADER,
  DASHBOARD_PAGE_LEAD,
  DASHBOARD_PAGE_STACK,
} from "@/lib/dashboard-layout";
import { dashboardEndpoints } from "@/lib/dashboard-catalog";

/* ─── Types ──────────────────────────────────────────────────────────── */

type SessionState = {
  plan?: string;
  keyId?: string;
  apiKeyPreview?: string;
};

type QuickstartStatus = {
  session: { plan: string; key_id: string };
  connectivity: { api_reachable: boolean; usage_check_ok: boolean };
  first_success: {
    has_successful_request: boolean;
    first_success_at?: string;
    first_success_endpoint?: string;
  };
  recent_activity: {
    request_count_7d: number;
    last_request_at?: string;
    last_request_status?: number;
  };
  recommended_next_steps: string[];
};

type StarterResult = {
  status: number;
  duration_ms: number;
  data: unknown;
};

type OnboardingState =
  | "loading"
  | "session_missing"
  | "api_unreachable"
  | "no_traffic"
  | "first_success"
  | "has_failures"
  | "active";

/* ─── Helpers ────────────────────────────────────────────────────────── */

function deriveState(
  session: SessionState | null,
  qs: QuickstartStatus | null,
  apiError: boolean,
): OnboardingState {
  if (!session?.keyId) return "session_missing";
  if (apiError || (qs && !qs.connectivity.api_reachable)) return "api_unreachable";
  if (!qs) return "loading";
  if (!qs.first_success.has_successful_request && qs.recent_activity.request_count_7d === 0)
    return "no_traffic";
  if (
    qs.recent_activity.last_request_status &&
    qs.recent_activity.last_request_status >= 400
  )
    return "has_failures";
  if (qs.first_success.has_successful_request) return qs.recent_activity.request_count_7d > 0 ? "active" : "first_success";
  return "no_traffic";
}

function formatTime(iso?: string): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

const STEP_MAP: Record<string, { href: string; title: string; description: string }> = {
  send_first_request: {
    href: "/dashboard/playground",
    title: "Send your first request",
    description: "Use the live playground to verify end-to-end connectivity.",
  },
  playground_search: {
    href: "/dashboard/playground",
    title: "Try Search in Playground",
    description: "Run a federated search to test ranking and citations.",
  },
  try_fetch: {
    href: "/dashboard/playground",
    title: "Try Fetch or Scrape",
    description: "Extract structured content from a URL with a token budget.",
  },
  try_research: {
    href: "/dashboard/playground",
    title: "Try Research",
    description: "Run multi-step research synthesis with source-backed output.",
  },
  usage_check: {
    href: "/dashboard/usage",
    title: "Check usage and quota",
    description: "Confirm plan limits and current request consumption.",
  },
  check_api_health: {
    href: "/dashboard/usage",
    title: "Check API health",
    description: "Recent requests are failing — verify connectivity and quota.",
  },
};

/* ─── Page ───────────────────────────────────────────────────────────── */

export default function QuickstartPage() {
  const [session, setSession] = useState<SessionState | null>(null);
  const [quickstart, setQuickstart] = useState<QuickstartStatus | null>(null);
  const [apiError, setApiError] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const [starterLoading, setStarterLoading] = useState(false);
  const [starterResult, setStarterResult] = useState<StarterResult | null>(null);
  const [starterError, setStarterError] = useState<string | null>(null);

  const loadQuickstart = useCallback(async () => {
    try {
      const res = await fetch("/api/dashboard/quickstart", { cache: "no-store" });
      if (!res.ok) {
        setApiError(true);
        return;
      }
      const body = await res.json();
      setQuickstart(body.quickstart ?? body);
      setApiError(false);
    } catch {
      setApiError(true);
    }
  }, []);

  useEffect(() => {
    void (async () => {
      const res = await fetch("/api/auth/session", { cache: "no-store" });
      if (!res.ok) return;
      const body = (await res.json()) as SessionState;
      setSession(body);
    })();
    void loadQuickstart();
  }, [loadQuickstart]);

  async function copy(text: string, key: string) {
    await navigator.clipboard.writeText(text);
    setCopied(key);
    window.setTimeout(() => setCopied(null), 1500);
  }

  async function runStarterRequest() {
    setStarterLoading(true);
    setStarterResult(null);
    setStarterError(null);
    try {
      const res = await fetch("/api/playground", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          endpoint: "/v1/search",
          payload: { query: "hello world", max_sources: 3, tier: "key_facts" },
        }),
      });
      const body = (await res.json()) as StarterResult;
      setStarterResult(body);
      // Refresh quickstart status after the request
      void loadQuickstart();
    } catch (err) {
      setStarterError(err instanceof Error ? err.message : "Request failed");
    } finally {
      setStarterLoading(false);
    }
  }

  const state = deriveState(session, quickstart, apiError);
  const firstRequest = dashboardEndpoints.find((ep) => ep.path === "/v1/search");
  const usageRequest = dashboardEndpoints.find((ep) => ep.path === "/v1/usage");

  return (
    <div className={DASHBOARD_PAGE_STACK}>
      {/* Header */}
      <div className={DASHBOARD_PAGE_HEADER}>
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">Quickstart</h1>
          <p className={DASHBOARD_PAGE_LEAD}>
            Go from authenticated dashboard access to a working first request.
          </p>
        </div>
        <StatusBanner state={state} quickstart={quickstart} />
      </div>

      {/* Workspace + Recommended Steps */}
      <section className="grid gap-4 xl:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
        <div className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div>
            <h2 className="text-lg font-semibold text-[var(--text-primary)]">Your workspace</h2>
            <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">
              Confirm the current authenticated context before sending live requests.
            </p>
          </div>
          <div className="grid gap-3 sm:grid-cols-3">
            <InfoTile label="Plan" value={session?.plan || quickstart?.session.plan || "—"} loading={state === "loading"} />
            <InfoTile label="Key ID" value={session?.keyId || quickstart?.session.key_id || "—"} loading={state === "loading"} />
            <InfoTile label="Key Preview" value={session?.apiKeyPreview || "—"} loading={state === "loading"} />
          </div>
          {quickstart && (
            <div className="grid gap-3 sm:grid-cols-3">
              <InfoTile
                label="Requests (7d)"
                value={String(quickstart.recent_activity.request_count_7d)}
              />
              <InfoTile
                label="Last request"
                value={formatTime(quickstart.recent_activity.last_request_at)}
              />
              <InfoTile
                label="Last status"
                value={
                  quickstart.recent_activity.last_request_status
                    ? String(quickstart.recent_activity.last_request_status)
                    : "No requests yet"
                }
              />
            </div>
          )}
        </div>

        <div className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div>
            <h2 className="text-lg font-semibold text-[var(--text-primary)]">Recommended next</h2>
            <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">
              {state === "loading"
                ? "Checking your account status..."
                : state === "no_traffic"
                  ? "Send your first request to get started."
                  : state === "has_failures"
                    ? "Recent failures detected — check connectivity."
                    : "Based on your actual usage."}
            </p>
          </div>
          <div className="space-y-3">
            {quickstart?.recommended_next_steps.map((step) => {
              const info = STEP_MAP[step];
              if (!info) return null;
              return (
                <ActionLink
                  key={step}
                  href={info.href}
                  title={info.title}
                  copy={info.description}
                />
              );
            })}
            {(!quickstart || quickstart.recommended_next_steps.length === 0) && (
              <>
                <ActionLink href="/dashboard/playground" title="Send your first request" copy="Use the live playground with your authenticated session." />
                <ActionLink href="/dashboard/api" title="Review endpoint catalog" copy="See supported endpoints, docs, and sample payloads." />
                <ActionLink href="/dashboard/usage" title="Check live quota" copy="Confirm plan limits and request consumption." />
              </>
            )}
          </div>
        </div>
      </section>

      {/* First-success milestone */}
      {quickstart?.first_success.has_successful_request && (
        <section className={`${DASHBOARD_CARD_PADDED} space-y-3`}>
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-emerald-500/10">
              <CheckCircle2 className="h-4 w-4 text-emerald-400" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-[var(--text-primary)]">First successful request</h2>
              <p className="text-xs text-[var(--text-muted)]">
                {quickstart.first_success.first_success_endpoint && (
                  <span className="font-mono text-brand-300">{quickstart.first_success.first_success_endpoint}</span>
                )}
                {quickstart.first_success.first_success_at && (
                  <span> at {formatTime(quickstart.first_success.first_success_at)}</span>
                )}
              </p>
            </div>
          </div>
        </section>
      )}

      {/* One-click starter request */}
      {state !== "loading" && state !== "session_missing" && (
        <section className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <h2 className="text-lg font-semibold text-[var(--text-primary)]">Quick test</h2>
              <p className="mt-1 text-sm text-[var(--text-muted)]">
                Run a sample Search request through the dashboard proxy to verify everything works.
              </p>
            </div>
            <button
              onClick={() => void runStarterRequest()}
              disabled={starterLoading}
              className="inline-flex items-center gap-2 rounded-lg bg-brand-500 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-brand-600 disabled:opacity-50"
            >
              {starterLoading ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Play className="h-4 w-4" />
              )}
              {starterLoading ? "Running..." : "Run starter request"}
            </button>
          </div>

          {starterResult && (
            <div
              className={`rounded-lg border p-4 ${
                starterResult.status >= 200 && starterResult.status < 300
                  ? "border-emerald-500/20 bg-emerald-500/5"
                  : "border-red-500/20 bg-red-500/5"
              }`}
            >
              <div className="mb-2 flex items-center gap-3">
                {starterResult.status >= 200 && starterResult.status < 300 ? (
                  <CheckCircle2 className="h-4 w-4 text-emerald-400" />
                ) : (
                  <XCircle className="h-4 w-4 text-red-400" />
                )}
                <span className="text-sm font-medium text-[var(--text-primary)]">
                  {starterResult.status >= 200 && starterResult.status < 300
                    ? "Request succeeded"
                    : `Request failed (${starterResult.status})`}
                </span>
                <span className="text-xs text-[var(--text-muted)]">
                  {starterResult.duration_ms}ms
                </span>
              </div>
              <pre className="max-h-48 overflow-auto rounded bg-[var(--surface-sunken)] p-3 text-xs leading-relaxed text-[var(--text-secondary)]">
                {typeof starterResult.data === "string"
                  ? starterResult.data
                  : JSON.stringify(starterResult.data, null, 2)}
              </pre>
            </div>
          )}

          {starterError && (
            <div className="rounded-lg border border-red-500/20 bg-red-500/5 p-4">
              <div className="flex items-center gap-2">
                <XCircle className="h-4 w-4 text-red-400" />
                <span className="text-sm text-red-300">{starterError}</span>
              </div>
            </div>
          )}
        </section>
      )}

      {/* Step-by-step guide */}
      <section className="grid gap-4 lg:grid-cols-3">
        {[
          {
            icon: Rocket,
            title: "1. Verify access",
            copy: "You are signed in with a server-validated session. No browser-stored raw key is required for dashboard actions.",
            done: state !== "loading" && state !== "session_missing",
          },
          {
            icon: TerminalSquare,
            title: "2. Hit Search first",
            copy: "Search is the fastest way to verify end-to-end auth, quota, latency, and response shape.",
            done: quickstart?.first_success.has_successful_request ?? false,
          },
          {
            icon: ArrowRight,
            title: "3. Expand from there",
            copy: "Once Search works, move to Fetch, Estimate, Research, media, and social endpoints.",
            done: (quickstart?.recent_activity.request_count_7d ?? 0) > 5,
          },
        ].map((item) => (
          <div key={item.title} className={`${DASHBOARD_CARD_PADDED} relative`}>
            <div className="flex items-center justify-between">
              <item.icon className="h-5 w-5 text-brand-400" />
              {item.done && <CheckCircle2 className="h-4 w-4 text-emerald-400" />}
            </div>
            <h2 className="mt-4 text-base font-semibold text-[var(--text-primary)]">{item.title}</h2>
            <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">{item.copy}</p>
          </div>
        ))}
      </section>

      {/* Curl snippets */}
      {firstRequest && usageRequest && (
        <section className="grid gap-4 xl:grid-cols-2">
          <SnippetCard
            title="Direct API first request"
            copy="Use this from your terminal or CI job."
            value={firstRequest.sampleCurl}
            copyKey="search"
            copied={copied}
            onCopy={copy}
          />
          <SnippetCard
            title="Inspect usage and quota"
            copy="This is the fastest way to confirm the current key state."
            value={usageRequest.sampleCurl}
            copyKey="usage"
            copied={copied}
            onCopy={copy}
          />
        </section>
      )}

      {/* Next endpoints */}
      <section className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-lg font-semibold text-[var(--text-primary)]">Next endpoints to try</h2>
            <p className="mt-1 text-sm text-[var(--text-muted)]">
              Use these when the first Search call is already working.
            </p>
          </div>
          <Link href="/dashboard/api" className="text-sm text-brand-400 hover:underline">
            Open full catalog →
          </Link>
        </div>

        <div className="grid gap-3 lg:grid-cols-2">
          {dashboardEndpoints.slice(1, 5).map((endpoint) => (
            <div
              key={endpoint.path}
              className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4"
            >
              <div className="flex items-center justify-between gap-3">
                <div>
                  <p className="text-sm font-medium text-[var(--text-primary)]">{endpoint.label}</p>
                  <p className="mt-1 font-mono text-xs text-brand-300">{endpoint.path}</p>
                </div>
                <span className="rounded-full border border-[var(--border-strong)] px-2.5 py-0.5 text-xs text-[var(--text-muted)]">
                  {endpoint.method}
                </span>
              </div>
              <p className="mt-3 text-sm leading-6 text-[var(--text-muted)]">{endpoint.description}</p>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

/* ─── Status Banner ──────────────────────────────────────────────────── */

function StatusBanner({
  state,
  quickstart,
}: {
  state: OnboardingState;
  quickstart: QuickstartStatus | null;
}) {
  const configs: Record<
    OnboardingState,
    { icon: typeof CheckCircle2; label: string; color: string; bg: string; description: string }
  > = {
    loading: {
      icon: Loader2,
      label: "Checking...",
      color: "text-[var(--text-muted)]",
      bg: "border-[var(--border-subtle)] bg-[var(--surface-raised)]",
      description: "Verifying session and API connectivity.",
    },
    session_missing: {
      icon: XCircle,
      label: "Session expired",
      color: "text-red-400",
      bg: "border-red-500/20 bg-red-500/5",
      description: "Your session is missing or expired. Please sign in again.",
    },
    api_unreachable: {
      icon: AlertTriangle,
      label: "API unreachable",
      color: "text-amber-400",
      bg: "border-amber-500/20 bg-amber-500/5",
      description: "Unable to reach the Fetchium API. Check connectivity or try again.",
    },
    no_traffic: {
      icon: Zap,
      label: "Ready to start",
      color: "text-brand-400",
      bg: "border-brand-500/20 bg-brand-500/5",
      description: "Your session is active but no requests have been sent yet. Send your first request below.",
    },
    has_failures: {
      icon: AlertTriangle,
      label: "Recent failures",
      color: "text-amber-400",
      bg: "border-amber-500/20 bg-amber-500/5",
      description: "Your last request returned an error. Check the details below and retry.",
    },
    first_success: {
      icon: CheckCircle2,
      label: "First request complete",
      color: "text-emerald-400",
      bg: "border-emerald-500/20 bg-[var(--success-soft)]",
      description: "You have completed a successful request. Explore more endpoints to unlock the full API.",
    },
    active: {
      icon: Activity,
      label: "Active",
      color: "text-emerald-400",
      bg: "border-emerald-500/20 bg-[var(--success-soft)]",
      description: `Session active with ${quickstart?.recent_activity.request_count_7d ?? 0} requests in the last 7 days.`,
    },
  };

  const cfg = configs[state];

  return (
    <div className={`${DASHBOARD_CARD_PADDED} space-y-3 lg:max-w-md ${cfg.bg}`}>
      <div className={`inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-medium ${cfg.color} ${cfg.bg}`}>
        <cfg.icon className={`h-3.5 w-3.5 ${state === "loading" ? "animate-spin" : ""}`} />
        {cfg.label}
      </div>
      <p className="text-sm leading-6 text-[var(--text-muted)]">{cfg.description}</p>
    </div>
  );
}

/* ─── Sub-components ─────────────────────────────────────────────────── */

function InfoTile({ label, value, loading }: { label: string; value: string; loading?: boolean }) {
  return (
    <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-4 py-3">
      <div className="text-xs text-[var(--text-faint)]">{label}</div>
      {loading ? (
        <div className="mt-1 h-5 w-24 animate-pulse rounded bg-[var(--surface-sunken)]" />
      ) : (
        <div className="mt-1 break-all text-sm font-medium text-[var(--text-primary)]">{value}</div>
      )}
    </div>
  );
}

function ActionLink({
  href,
  title,
  copy,
}: {
  href: string;
  title: string;
  copy: string;
}) {
  return (
    <Link
      href={href}
      className="block rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 transition-colors hover:border-[var(--brand-border)] hover:bg-[var(--surface-hover)]"
    >
      <div className="flex items-center justify-between gap-3">
        <p className="text-sm font-medium text-[var(--text-primary)]">{title}</p>
        <ArrowRight className="h-4 w-4 text-brand-400" />
      </div>
      <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">{copy}</p>
    </Link>
  );
}

function SnippetCard({
  title,
  copy,
  value,
  copyKey,
  copied,
  onCopy,
}: {
  title: string;
  copy: string;
  value?: string;
  copyKey: string;
  copied: string | null;
  onCopy: (text: string, key: string) => Promise<void>;
}) {
  if (!value) return null;

  return (
    <div className="overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-base)]">
      <div className="flex items-center justify-between gap-3 border-b border-[var(--border-subtle)] px-4 py-4 sm:px-5">
        <div>
          <h2 className="text-sm font-medium text-[var(--text-primary)]">{title}</h2>
          <p className="mt-1 text-xs text-[var(--text-faint)]">{copy}</p>
        </div>
        <button
          onClick={() => void onCopy(value, copyKey)}
          className="inline-flex items-center gap-1 rounded-lg border border-[var(--border-subtle)] px-3 py-1.5 text-xs text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
        >
          <Copy className="h-3.5 w-3.5" />
          {copied === copyKey ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="overflow-x-auto p-4 text-xs leading-relaxed text-[var(--text-secondary)] sm:p-5">
        {value}
      </pre>
    </div>
  );
}
