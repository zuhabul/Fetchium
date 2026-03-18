"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { ArrowRight, BarChart3, CheckCircle, Clock, Rocket, Zap } from "lucide-react";
import { dashboardEndpoints } from "@/lib/dashboard-catalog";
import {
  DASHBOARD_CARD_PADDED,
  DASHBOARD_PAGE_HEADER,
  DASHBOARD_PAGE_LEAD,
  DASHBOARD_PAGE_STACK,
  DASHBOARD_PANEL,
  DASHBOARD_PANEL_EMPTY,
  DASHBOARD_PANEL_HEADER,
  DASHBOARD_PANEL_ROW,
} from "@/lib/dashboard-layout";

type OverviewSummary = {
  key_id: string;
  plan: string;
  requests_today: number;
  requests_this_month: number;
  tokens_this_month: number;
  monthly_limit: number | null;
  quota_remaining: number | null;
  avg_latency_ms_7d?: number | null;
  success_rate_7d?: number | null;
};

type OverviewRecentRequest = {
  endpoint: string;
  status: number;
  duration_ms: number;
  tokens_used: number;
  created_at: string;
};

type OverviewEndpointStat = {
  endpoint: string;
  requests: number;
  last_seen_at: string;
};

type OverviewTimeseriesPoint = {
  date: string;
  requests: number;
};

type OverviewResponse = {
  summary: OverviewSummary;
  recent_requests: OverviewRecentRequest[];
  top_endpoints: OverviewEndpointStat[];
  timeseries: OverviewTimeseriesPoint[];
  title?: string;
  message?: string;
};

export default function DashboardPage() {
  const [overview, setOverview] = useState<OverviewResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void loadOverview();
  }, []);

  async function loadOverview() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/dashboard/overview", { cache: "no-store" });
      const body = (await res.json()) as OverviewResponse;
      if (!res.ok) {
        setOverview(null);
        setError(body.title || body.message || "Failed to load dashboard overview.");
        return;
      }
      setOverview(body);
    } catch (err) {
      setOverview(null);
      setError(err instanceof Error ? err.message : "Failed to load dashboard overview.");
    } finally {
      setLoading(false);
    }
  }

  const summary = overview?.summary || null;
  const quotaPct =
    summary && summary.monthly_limit
      ? Math.min(100, Math.round((summary.requests_this_month / summary.monthly_limit) * 100))
      : 0;
  const recentRequests = overview?.recent_requests || [];
  const topEndpoints = overview?.top_endpoints || [];
  const trend = overview?.timeseries || [];
  const trendTotal = useMemo(
    () => trend.reduce((sum, point) => sum + point.requests, 0),
    [trend],
  );

  const stats = [
    {
      label: "Requests Today",
      value: summary ? String(summary.requests_today) : "—",
      sub: "Current day",
      icon: Zap,
      color: "text-brand-400",
    },
    {
      label: "This Month",
      value: summary ? String(summary.requests_this_month) : "—",
      sub: summary?.monthly_limit ? `/ ${summary.monthly_limit}` : "/ unlimited",
      icon: BarChart3,
      color: "text-purple-400",
    },
    {
      label: "Avg Latency",
      value: summary?.avg_latency_ms_7d != null ? `${summary.avg_latency_ms_7d}ms` : "n/a",
      sub: "Last 7 days",
      icon: Clock,
      color: "text-yellow-400",
    },
    {
      label: "Success Rate",
      value: summary?.success_rate_7d != null ? `${summary.success_rate_7d}%` : "n/a",
      sub: "Last 7 days",
      icon: CheckCircle,
      color: "text-green-400",
    },
  ];

  return (
    <div className={DASHBOARD_PAGE_STACK}>
      <div className={DASHBOARD_PAGE_HEADER}>
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">Overview</h1>
          <p className={DASHBOARD_PAGE_LEAD}>
            Live request telemetry and quota data for the authenticated API key.
          </p>
        </div>

        <div className="grid gap-3 sm:grid-cols-2 lg:hidden">
          <div className={DASHBOARD_CARD_PADDED}>
            <p className="text-[11px] uppercase tracking-[0.18em] text-[var(--text-faint)]">
              Remaining quota
            </p>
            <p className="mt-2 text-lg font-semibold text-[var(--text-primary)]">
              {summary?.quota_remaining ?? "—"}
            </p>
          </div>
          <div className={DASHBOARD_CARD_PADDED}>
            <p className="text-[11px] uppercase tracking-[0.18em] text-[var(--text-faint)]">
              Current plan
            </p>
            <p className="mt-2 text-lg font-semibold capitalize text-[var(--text-primary)]">
              {summary?.plan ?? "unconfigured"}
            </p>
          </div>
        </div>
      </div>

      {error && (
        <div className="rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-sm text-[var(--danger-text)]">
          {error}
        </div>
      )}

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((s) => (
          <div key={s.label} className={DASHBOARD_CARD_PADDED}>
            <div className="mb-3 flex items-center justify-between">
              <span className="text-sm text-[var(--text-muted)]">{s.label}</span>
              <s.icon className={`h-4 w-4 ${s.color}`} />
            </div>
            <div className="text-2xl font-bold text-[var(--text-primary)]">{loading ? "…" : s.value}</div>
            <div className="mt-1 text-xs text-[var(--text-faint)]">{s.sub}</div>
          </div>
        ))}
      </div>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.05fr)_minmax(320px,0.95fr)]">
        <section className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4 text-brand-400" />
            <h2 className="font-medium text-[var(--text-primary)]">Usage trend</h2>
          </div>
          <p className="text-sm leading-6 text-[var(--text-muted)]">
            Daily request volume over the last 14 days. Total requests in window:{" "}
            <span className="font-medium text-[var(--text-primary)]">{loading ? "…" : trendTotal}</span>
          </p>
          {loading ? (
            <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-sm text-[var(--text-faint)]">
              Loading trend data…
            </div>
          ) : trend.length === 0 ? (
            <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-sm text-[var(--text-faint)]">
              No request trend available yet.
            </div>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2">
              {trend.slice(-6).map((point) => (
                <div
                  key={point.date}
                  className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-4 py-3"
                >
                  <div className="text-xs text-[var(--text-faint)]">{point.date}</div>
                  <div className="mt-1 text-base font-semibold text-[var(--text-primary)]">
                    {point.requests} requests
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>

        <section className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div>
            <h2 className="font-medium text-[var(--text-primary)]">Top endpoints</h2>
            <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">
              Most-used endpoints over the recent activity window.
            </p>
          </div>
          <div className="space-y-3">
            {loading ? (
              <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-sm text-[var(--text-faint)]">
                Loading endpoint activity…
              </div>
            ) : topEndpoints.length === 0 ? (
              <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-sm text-[var(--text-faint)]">
                No endpoint activity recorded yet.
              </div>
            ) : (
              topEndpoints.map((endpoint) => (
                <div
                  key={`${endpoint.endpoint}-${endpoint.last_seen_at}`}
                  className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-4 py-3"
                >
                  <div className="flex items-center justify-between gap-3">
                    <span className="text-sm font-medium text-[var(--text-primary)]">
                      {endpoint.endpoint}
                    </span>
                    <span className="rounded-full border border-[var(--border-strong)] px-2 py-0.5 text-xs text-[var(--text-muted)]">
                      {endpoint.requests} req
                    </span>
                  </div>
                  <div className="mt-1 text-xs text-[var(--text-faint)]">
                    Last seen {new Date(endpoint.last_seen_at).toLocaleString()}
                  </div>
                </div>
              ))
            )}
          </div>
        </section>
      </div>

      <div className={DASHBOARD_CARD_PADDED}>
        <div className="mb-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <span className="font-medium text-[var(--text-primary)]">Monthly quota</span>
          <span className="text-sm text-[var(--text-muted)]">
            {summary
              ? `${summary.requests_this_month} / ${summary.monthly_limit ?? "unlimited"} requests`
              : loading
                ? "Loading usage data…"
                : "Set API key in Settings to load"}
          </span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-[var(--surface-hover)]">
          <div
            className="h-full rounded-full bg-gradient-to-r from-brand-500 to-brand-400"
            style={{ width: `${quotaPct}%` }}
          />
        </div>
      </div>

      <div className={DASHBOARD_PANEL}>
        <div className={DASHBOARD_PANEL_HEADER}>
          <h2 className="font-medium text-[var(--text-primary)]">Recent requests</h2>
          <a href="/dashboard/playground" className="text-xs text-brand-400 hover:underline">
            Open playground →
          </a>
        </div>
        <div className="divide-y divide-white/5">
          {loading ? (
            <div className={DASHBOARD_PANEL_EMPTY}>Loading request activity…</div>
          ) : recentRequests.length === 0 ? (
            <div className={DASHBOARD_PANEL_EMPTY}>No request activity yet for this key.</div>
          ) : (
            recentRequests.map((r, i) => (
              <div key={`${r.created_at}-${i}`}>
                <div className={`hidden items-center gap-4 text-sm sm:flex ${DASHBOARD_PANEL_ROW}`}>
                  <span className="rounded-md bg-brand-500/10 px-2 py-0.5 font-mono text-xs text-brand-300">
                    {r.endpoint}
                  </span>
                  <span
                    className={`text-xs ${
                      r.status >= 200 && r.status < 300 ? "text-green-400" : "text-red-400"
                    }`}
                  >
                    {r.status}
                  </span>
                  <span className="w-16 text-right text-xs text-[var(--text-faint)]">{r.duration_ms}ms</span>
                  <span className="w-20 text-right text-xs text-[var(--text-faint)]">
                    {r.tokens_used.toLocaleString()}
                  </span>
                  <span className="text-xs text-[var(--text-faint)]">
                    {new Date(r.created_at).toLocaleString()}
                  </span>
                </div>

                <article className={`${DASHBOARD_PANEL_ROW} sm:hidden`}>
                  <div className="flex flex-col gap-3">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="rounded-md bg-brand-500/10 px-2 py-0.5 font-mono text-xs text-brand-300">
                        {r.endpoint}
                      </span>
                      <span
                        className={`text-xs ${
                          r.status >= 200 && r.status < 300 ? "text-green-400" : "text-red-400"
                        }`}
                      >
                        {r.status}
                      </span>
                    </div>
                    <div className="flex items-center justify-between gap-3 text-xs text-[var(--text-faint)]">
                      <span>{r.duration_ms}ms</span>
                      <span>{r.tokens_used.toLocaleString()} tokens</span>
                    </div>
                    <div className="text-right text-xs text-[var(--text-faint)]">
                      {new Date(r.created_at).toLocaleString()}
                    </div>
                  </div>
                </article>
              </div>
            ))
          )}
        </div>
      </div>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.05fr)_minmax(320px,0.95fr)]">
        <section className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div className="flex items-center gap-2">
            <Rocket className="h-4 w-4 text-brand-400" />
            <h2 className="font-medium text-[var(--text-primary)]">Get to first value</h2>
          </div>
          <p className="text-sm leading-6 text-[var(--text-muted)]">
            The fastest path is Search, then Usage, then deeper endpoint testing in Playground.
          </p>
          <div className="grid gap-3 sm:grid-cols-3">
            {[
              {
                href: "/dashboard/quickstart",
                title: "Quickstart",
                copy: "Follow the first-request workflow.",
              },
              {
                href: "/dashboard/api",
                title: "API Catalog",
                copy: "Review endpoint coverage and examples.",
              },
              {
                href: "/dashboard/playground",
                title: "Playground",
                copy: "Send live requests through the proxy.",
              },
            ].map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 transition-colors hover:border-[var(--brand-border)] hover:bg-[var(--surface-hover)]"
              >
                <div className="flex items-center justify-between gap-3">
                  <span className="text-sm font-medium text-[var(--text-primary)]">{item.title}</span>
                  <ArrowRight className="h-4 w-4 text-brand-400" />
                </div>
                <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">{item.copy}</p>
              </Link>
            ))}
          </div>
        </section>

        <section className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div>
            <h2 className="font-medium text-[var(--text-primary)]">Suggested endpoints</h2>
            <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">
              Good next calls once your session is active.
            </p>
          </div>
          <div className="space-y-3">
            {dashboardEndpoints.slice(0, 4).map((endpoint) => (
              <div
                key={endpoint.path}
                className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-4 py-3"
              >
                <div className="flex items-center justify-between gap-3">
                  <span className="text-sm font-medium text-[var(--text-primary)]">{endpoint.label}</span>
                  <span className="rounded-full border border-[var(--border-strong)] px-2 py-0.5 text-xs text-[var(--text-muted)]">
                    {endpoint.method}
                  </span>
                </div>
                <div className="mt-1 font-mono text-xs text-brand-300">{endpoint.path}</div>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
}
