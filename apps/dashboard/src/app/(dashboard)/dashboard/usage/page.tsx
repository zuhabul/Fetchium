"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  BarChart3,
  CheckCircle,
  Clock,
  RefreshCw,
  AlertTriangle,
  Zap,
  Activity,
  Loader2,
  Shield,
} from "lucide-react";
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

/* ─── Types ──────────────────────────────────────────────────────────── */

type UsageAnalyticsSummary = {
  key_id: string;
  plan: string;
  requests_today: number;
  requests_this_month: number;
  tokens_this_month: number;
  monthly_limit: number | null;
  quota_remaining: number | null;
  requests_per_minute_limit: number;
};

type UsageTimeseriesPoint = {
  date: string;
  requests: number;
  tokens?: number;
};

type EndpointUsageBreakdown = {
  endpoint: string;
  requests: number;
  tokens_used: number;
  error_count: number;
};

type UsageHealthSummary = {
  success_rate: number;
  error_count: number;
};

type UsageAnalytics = {
  summary: UsageAnalyticsSummary;
  timeseries: UsageTimeseriesPoint[];
  endpoint_breakdown: EndpointUsageBreakdown[];
  health: UsageHealthSummary;
};

/* ─── Page ───────────────────────────────────────────────────────────── */

export default function UsagePage() {
  const [analytics, setAnalytics] = useState<UsageAnalytics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchUsage = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/dashboard/usage", { cache: "no-store" });
      const body = await res.json();
      if (!res.ok) {
        setError(body.title || body.message || "Failed to fetch usage analytics.");
        return;
      }
      setAnalytics(body.usage ?? body);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to fetch usage.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void fetchUsage();
  }, [fetchUsage]);

  const summary = analytics?.summary;
  const quotaPct = useMemo(() => {
    if (!summary?.monthly_limit) return null;
    return Math.min(100, Math.round((summary.requests_this_month / summary.monthly_limit) * 100));
  }, [summary]);

  const trendMax = useMemo(() => {
    if (!analytics?.timeseries.length) return 1;
    return Math.max(...analytics.timeseries.map((p) => p.requests), 1);
  }, [analytics]);

  return (
    <div className={DASHBOARD_PAGE_STACK}>
      {/* Header */}
      <div className={DASHBOARD_PAGE_HEADER}>
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">Usage Analytics</h1>
          <p className={DASHBOARD_PAGE_LEAD}>
            Quota, trends, endpoint breakdown, and request health for the authenticated key.
          </p>
        </div>
        <button
          onClick={() => void fetchUsage()}
          disabled={loading}
          className="inline-flex items-center gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-4 py-2.5 text-sm text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] disabled:opacity-60"
        >
          {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          {loading ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      {error && (
        <div className="rounded-xl border border-red-500/20 bg-red-500/5 p-3 text-sm text-[var(--danger-text)] sm:p-4">
          {error}
        </div>
      )}

      {/* KPI Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard icon={Zap} label="Requests Today" value={summary ? String(summary.requests_today) : "—"} sub="Current day" color="text-brand-400" loading={loading} />
        <KpiCard icon={BarChart3} label="This Month" value={summary ? String(summary.requests_this_month) : "—"} sub={summary?.monthly_limit ? `/ ${summary.monthly_limit.toLocaleString()}` : "/ unlimited"} color="text-purple-400" loading={loading} />
        <KpiCard icon={CheckCircle} label="Success Rate" value={analytics?.health ? `${analytics.health.success_rate}%` : "—"} sub="Last 30 days" color="text-green-400" loading={loading} />
        <KpiCard icon={Shield} label="Rate Limit" value={summary ? `${summary.requests_per_minute_limit}/min` : "—"} sub={`${summary?.plan || "—"} plan`} color="text-yellow-400" loading={loading} />
      </div>

      {/* Quota Bar */}
      {summary && (
        <div className={DASHBOARD_CARD_PADDED}>
          <div className="mb-3 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <span className="text-sm font-medium text-[var(--text-primary)]">Monthly quota</span>
            <span className="text-sm text-[var(--text-muted)]">
              {summary.requests_this_month.toLocaleString()} / {summary.monthly_limit?.toLocaleString() ?? "unlimited"} requests
            </span>
          </div>
          <div className="h-2.5 overflow-hidden rounded-full bg-[var(--surface-hover)]">
            <div
              className={`h-full rounded-full transition-all ${
                (quotaPct ?? 0) > 90
                  ? "bg-gradient-to-r from-red-500 to-red-400"
                  : (quotaPct ?? 0) > 70
                    ? "bg-gradient-to-r from-amber-500 to-amber-400"
                    : "bg-gradient-to-r from-brand-500 to-brand-400"
              }`}
              style={{ width: `${quotaPct ?? 0}%` }}
            />
          </div>
          <div className="mt-3 grid gap-3 sm:grid-cols-4">
            <MiniStat label="Quota used" value={quotaPct != null ? `${quotaPct}%` : "N/A"} />
            <MiniStat label="Remaining" value={summary.quota_remaining != null ? summary.quota_remaining.toLocaleString() : "Unlimited"} />
            <MiniStat label="Tokens this month" value={summary.tokens_this_month.toLocaleString()} />
            <MiniStat label="Key ID" value={summary.key_id.length > 12 ? `${summary.key_id.slice(0, 8)}...` : summary.key_id} />
          </div>
        </div>
      )}

      {/* Trend + Health */}
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(300px,0.8fr)]">
        {/* Usage Trend */}
        <section className={DASHBOARD_PANEL}>
          <div className={DASHBOARD_PANEL_HEADER}>
            <div className="flex items-center gap-2">
              <Activity className="h-4 w-4 text-brand-400" />
              <h2 className="text-sm font-semibold text-[var(--text-primary)]">Usage trend (30 days)</h2>
            </div>
          </div>
          <div className={DASHBOARD_PANEL_ROW}>
            {loading ? (
              <div className="text-sm text-[var(--text-faint)]">Loading trend data...</div>
            ) : !analytics?.timeseries.length ? (
              <div className="text-sm text-[var(--text-faint)]">No request trend available yet.</div>
            ) : (
              <div className="space-y-2">
                {analytics.timeseries.map((point) => (
                  <div key={point.date} className="flex items-center gap-3">
                    <span className="w-20 flex-shrink-0 text-xs text-[var(--text-faint)]">{point.date.slice(5)}</span>
                    <div className="flex-1">
                      <div
                        className="h-5 rounded bg-brand-500/20"
                        style={{ width: `${Math.max(2, (point.requests / trendMax) * 100)}%` }}
                      >
                        <div
                          className="h-full rounded bg-brand-500/60"
                          style={{ width: "100%" }}
                        />
                      </div>
                    </div>
                    <span className="w-14 text-right text-xs font-medium text-[var(--text-primary)]">{point.requests}</span>
                    {point.tokens != null && (
                      <span className="w-20 text-right text-xs text-[var(--text-faint)]">{point.tokens.toLocaleString()} tok</span>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>

        {/* Request Health */}
        <section className={DASHBOARD_PANEL}>
          <div className={DASHBOARD_PANEL_HEADER}>
            <div className="flex items-center gap-2">
              {analytics?.health && analytics.health.error_count > 0 ? (
                <AlertTriangle className="h-4 w-4 text-amber-400" />
              ) : (
                <CheckCircle className="h-4 w-4 text-emerald-400" />
              )}
              <h2 className="text-sm font-semibold text-[var(--text-primary)]">Request health (30d)</h2>
            </div>
          </div>
          <div className={DASHBOARD_PANEL_ROW}>
            {loading ? (
              <div className="text-sm text-[var(--text-faint)]">Loading health data...</div>
            ) : !analytics?.health ? (
              <div className="text-sm text-[var(--text-faint)]">No health data available yet.</div>
            ) : (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-[var(--text-muted)]">Success rate</span>
                  <span className={`text-lg font-bold ${analytics.health.success_rate >= 99 ? "text-emerald-400" : analytics.health.success_rate >= 95 ? "text-amber-400" : "text-red-400"}`}>
                    {analytics.health.success_rate}%
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-[var(--text-muted)]">Errors (30d)</span>
                  <span className={`text-lg font-bold ${analytics.health.error_count === 0 ? "text-emerald-400" : "text-red-400"}`}>
                    {analytics.health.error_count}
                  </span>
                </div>
                <div className="h-2 overflow-hidden rounded-full bg-[var(--surface-hover)]">
                  <div
                    className="h-full rounded-full bg-emerald-500"
                    style={{ width: `${analytics.health.success_rate}%` }}
                  />
                </div>
              </div>
            )}
          </div>
        </section>
      </div>

      {/* Endpoint Breakdown */}
      <section className={DASHBOARD_PANEL}>
        <div className={DASHBOARD_PANEL_HEADER}>
          <div className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4 text-brand-400" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">Endpoint breakdown (30 days)</h2>
          </div>
        </div>
        {loading ? (
          <div className={DASHBOARD_PANEL_EMPTY}>Loading endpoint breakdown...</div>
        ) : !analytics?.endpoint_breakdown.length ? (
          <div className={DASHBOARD_PANEL_EMPTY}>No endpoint usage recorded yet.</div>
        ) : (
          <div className="divide-y divide-white/5">
            {/* Header row */}
            <div className={`${DASHBOARD_PANEL_ROW} hidden sm:block`}>
              <div className="grid grid-cols-[1fr_80px_100px_80px] gap-4 text-xs font-medium uppercase tracking-wider text-[var(--text-faint)]">
                <span>Endpoint</span>
                <span className="text-right">Requests</span>
                <span className="text-right">Tokens</span>
                <span className="text-right">Errors</span>
              </div>
            </div>
            {analytics.endpoint_breakdown.map((ep) => (
              <div key={ep.endpoint} className={DASHBOARD_PANEL_ROW}>
                <div className="grid gap-2 sm:grid-cols-[1fr_80px_100px_80px] sm:gap-4">
                  <span className="font-mono text-sm text-brand-300">{ep.endpoint}</span>
                  <span className="text-right text-sm font-medium text-[var(--text-primary)]">{ep.requests.toLocaleString()}</span>
                  <span className="text-right text-sm text-[var(--text-muted)]">{ep.tokens_used.toLocaleString()}</span>
                  <span className={`text-right text-sm ${ep.error_count > 0 ? "text-red-400" : "text-[var(--text-faint)]"}`}>
                    {ep.error_count}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

/* ─── Sub-components ─────────────────────────────────────────────────── */

function KpiCard({
  icon: Icon,
  label,
  value,
  sub,
  color,
  loading,
}: {
  icon: typeof Zap;
  label: string;
  value: string;
  sub: string;
  color: string;
  loading: boolean;
}) {
  return (
    <div className={DASHBOARD_CARD_PADDED}>
      <div className="mb-3 flex items-center justify-between">
        <span className="text-sm text-[var(--text-muted)]">{label}</span>
        <Icon className={`h-4 w-4 ${color}`} />
      </div>
      <div className="text-2xl font-bold text-[var(--text-primary)]">{loading ? "..." : value}</div>
      <div className="mt-1 text-xs text-[var(--text-faint)]">{sub}</div>
    </div>
  );
}

function MiniStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-4 py-3">
      <div className="text-xs text-[var(--text-faint)]">{label}</div>
      <div className="mt-1 text-sm font-medium text-[var(--text-primary)]">{value}</div>
    </div>
  );
}
