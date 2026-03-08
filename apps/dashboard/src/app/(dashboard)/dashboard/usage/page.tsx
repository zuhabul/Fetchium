"use client";

import { useEffect, useState } from "react";

type UsageStats = {
  key_id: string;
  plan: string;
  requests_this_month: number;
  requests_today: number;
  tokens_this_month: number;
  monthly_limit: number | null;
  quota_remaining: number | null;
};

export default function UsagePage() {
  const [stats, setStats] = useState<UsageStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void fetchUsage();
  }, []);

  async function fetchUsage() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/usage", { cache: "no-store" });
      const body = (await res.json()) as { usage?: UsageStats; title?: string; message?: string };
      if (!res.ok) {
        setStats(null);
        setError(body.title || body.message || "Failed to fetch usage.");
        return;
      }
      setStats(body.usage || null);
    } catch (e) {
      setStats(null);
      setError(e instanceof Error ? e.message : "Failed to fetch usage.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Usage Analytics</h1>
        <p className="text-sm text-white/40 mt-1">Live usage from `/v1/usage`.</p>
      </div>

      <div className="rounded-xl border border-white/5 bg-surface-1 p-5 space-y-3">
        <p className="text-xs text-white/35">
          Usage is loaded from the authenticated session against the hosted production API.
        </p>
        <button
          onClick={() => void fetchUsage()}
          disabled={loading}
          className="rounded-lg bg-brand-500 px-4 py-2 text-sm text-white hover:bg-brand-600 disabled:opacity-60"
        >
          {loading ? "Refreshing..." : "Refresh usage"}
        </button>
      </div>

      {error && (
        <div className="rounded-xl border border-red-500/20 bg-red-500/5 p-3 text-sm text-red-300">
          {error}
        </div>
      )}

      {stats && (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard label="Plan" value={stats.plan} />
          <StatCard label="Requests Today" value={String(stats.requests_today)} />
          <StatCard label="Requests This Month" value={String(stats.requests_this_month)} />
          <StatCard
            label="Quota Remaining"
            value={stats.quota_remaining == null ? "Unlimited" : String(stats.quota_remaining)}
          />
          <StatCard
            label="Monthly Limit"
            value={stats.monthly_limit == null ? "Unlimited" : String(stats.monthly_limit)}
          />
          <StatCard label="Tokens This Month" value={stats.tokens_this_month.toLocaleString()} />
          <StatCard label="Key ID" value={stats.key_id.slice(0, 8)} />
          <StatCard
            label="Quota Used"
            value={
              stats.monthly_limit
                ? `${Math.round((stats.requests_this_month / stats.monthly_limit) * 100)}%`
                : "N/A"
            }
          />
        </div>
      )}
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-white/5 bg-surface-1 p-5">
      <div className="text-sm text-white/40">{label}</div>
      <div className="text-xl font-semibold text-white mt-1">{value}</div>
    </div>
  );
}
