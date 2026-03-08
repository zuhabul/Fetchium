"use client";

import { useEffect, useMemo, useState } from "react";
import { BarChart3, Zap, Clock, CheckCircle } from "lucide-react";
import { loadRequestLogs } from "@/lib/client-config";

type UsageStats = {
  plan: string;
  requests_this_month: number;
  requests_today: number;
  monthly_limit: number | null;
  quota_remaining: number | null;
};

type RequestLog = {
  endpoint: string;
  status: number;
  latencyMs: number;
  timeIso: string;
};

export default function DashboardPage() {
  const [usage, setUsage] = useState<UsageStats | null>(null);
  const [logs, setLogs] = useState<RequestLog[]>([]);

  useEffect(() => {
    setLogs(loadRequestLogs().slice(0, 8) as RequestLog[]);
    void (async () => {
      const res = await fetch("/api/usage", { cache: "no-store" });
      if (!res.ok) return;
      const body = (await res.json()) as { usage?: UsageStats };
      setUsage(body.usage || null);
    })();
  }, []);

  const avgLatency = useMemo(() => {
    if (!logs.length) return "n/a";
    const avg = Math.round(logs.reduce((a, b) => a + b.latencyMs, 0) / logs.length);
    return `${avg}ms`;
  }, [logs]);

  const successRate = useMemo(() => {
    if (!logs.length) return "n/a";
    const ok = logs.filter((x) => x.status >= 200 && x.status < 300).length;
    return `${Math.round((ok / logs.length) * 100)}%`;
  }, [logs]);

  const quotaPct =
    usage && usage.monthly_limit
      ? Math.min(100, Math.round((usage.requests_this_month / usage.monthly_limit) * 100))
      : 0;

  const stats = [
    { label: "Requests Today", value: usage ? String(usage.requests_today) : "—", icon: Zap, color: "text-brand-400" },
    {
      label: "This Month",
      value: usage ? String(usage.requests_this_month) : "—",
      sub: usage?.monthly_limit ? `/ ${usage.monthly_limit}` : "/ unlimited",
      icon: BarChart3,
      color: "text-purple-400",
    },
    { label: "Avg Latency", value: avgLatency, icon: Clock, color: "text-yellow-400" },
    { label: "Success Rate", value: successRate, icon: CheckCircle, color: "text-green-400" },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Overview</h1>
        <p className="text-sm text-white/40 mt-1">Live usage plus recent requests from the dashboard playground.</p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((s) => (
          <div key={s.label} className="rounded-xl border border-white/5 bg-surface-1 p-5">
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm text-white/40">{s.label}</span>
              <s.icon className={`h-4 w-4 ${s.color}`} />
            </div>
            <div className="text-2xl font-bold text-white">{s.value}</div>
            <div className="text-xs text-white/30 mt-1">{s.sub || " "}</div>
          </div>
        ))}
      </div>

      <div className="rounded-xl border border-white/5 bg-surface-1 p-5">
        <div className="flex items-center justify-between mb-3">
          <span className="font-medium text-white">Monthly quota</span>
          <span className="text-sm text-white/40">
            {usage
              ? `${usage.requests_this_month} / ${usage.monthly_limit ?? "unlimited"} requests`
              : "Set API key in Settings to load"}
          </span>
        </div>
        <div className="h-2 w-full rounded-full bg-white/5 overflow-hidden">
          <div className="h-full rounded-full bg-gradient-to-r from-brand-500 to-brand-400" style={{ width: `${quotaPct}%` }} />
        </div>
      </div>

      <div className="rounded-xl border border-white/5 bg-surface-1">
        <div className="flex items-center justify-between border-b border-white/5 px-5 py-4">
          <h2 className="font-medium text-white">Recent requests</h2>
          <a href="/dashboard/playground" className="text-xs text-brand-400 hover:underline">
            Open playground →
          </a>
        </div>
        <div className="divide-y divide-white/5">
          {logs.length === 0 ? (
            <div className="px-5 py-8 text-sm text-white/30">No recent requests yet.</div>
          ) : (
            logs.map((r, i) => (
              <div key={`${r.timeIso}-${i}`} className="flex items-center gap-4 px-5 py-3.5 text-sm">
                <span className="rounded-md bg-brand-500/10 px-2 py-0.5 font-mono text-xs text-brand-300">
                  {r.endpoint}
                </span>
                <span className={`text-xs ${r.status >= 200 && r.status < 300 ? "text-green-400" : "text-red-400"}`}>
                  {r.status}
                </span>
                <span className="text-white/30 text-xs w-16 text-right">{r.latencyMs}ms</span>
                <span className="text-white/25 text-xs">{new Date(r.timeIso).toLocaleString()}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
