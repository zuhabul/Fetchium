"use client";

import { useEffect, useState } from "react";
import { loadDashboardConfig } from "@/lib/client-config";

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
  const [apiKey, setApiKey] = useState("");
  const [apiBase, setApiBase] = useState("http://localhost:3050");
  const [stats, setStats] = useState<UsageStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const cfg = loadDashboardConfig();
    setApiKey(cfg.apiKey);
    setApiBase(cfg.apiBaseUrl);
    if (cfg.apiKey) {
      void fetchUsage(cfg.apiKey, cfg.apiBaseUrl);
    }
  }, []);

  async function fetchUsage(key = apiKey, base = apiBase) {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/usage", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ apiKey: key, apiBase: base }),
      });
      const body = (await res.json()) as UsageStats & { title?: string; message?: string };
      if (!res.ok) {
        setStats(null);
        setError(body.title || body.message || "Failed to fetch usage.");
        return;
      }
      setStats(body);
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
        <div className="grid gap-3 md:grid-cols-2">
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="fetchium_..."
            className="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
          />
          <input
            type="text"
            value={apiBase}
            onChange={(e) => setApiBase(e.target.value)}
            placeholder="http://localhost:3050"
            className="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
          />
        </div>
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
          <StatCard label="Monthly Limit" value={stats.monthly_limit == null ? "Unlimited" : String(stats.monthly_limit)} />
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

