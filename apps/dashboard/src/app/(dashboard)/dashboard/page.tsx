import { BarChart3, Zap, Clock, CheckCircle } from "lucide-react";
import type { Metadata } from "next";

export const metadata: Metadata = { title: "Overview — HyperSearchX Dashboard" };

const stats = [
  { label: "Requests Today", value: "47", change: "+12%", icon: Zap, color: "text-brand-400" },
  { label: "This Month", value: "847", sub: "/ 1,000 free", icon: BarChart3, color: "text-purple-400" },
  { label: "Avg Latency", value: "1.2s", change: "P95: 3.4s", icon: Clock, color: "text-yellow-400" },
  { label: "Success Rate", value: "99.1%", change: "last 7 days", icon: CheckCircle, color: "text-green-400" },
];

const recentRequests = [
  { endpoint: "/v1/search", query: "rust async programming", status: 200, latency: "834ms", time: "2 min ago" },
  { endpoint: "/v1/scrape", query: "https://doc.rust-lang.org", status: 200, latency: "1.2s", time: "5 min ago" },
  { endpoint: "/v1/research", query: "LLM agent architectures 2025", status: 200, latency: "8.1s", time: "12 min ago" },
  { endpoint: "/v1/search", query: "tokio vs async-std", status: 200, latency: "612ms", time: "1 hr ago" },
  { endpoint: "/v1/scrape", query: "https://crates.io/crates/axum", status: 200, latency: "945ms", time: "2 hr ago" },
];

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Overview</h1>
        <p className="text-sm text-white/40 mt-1">Welcome back. Here&apos;s your API usage summary.</p>
      </div>

      {/* Stats */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((s) => (
          <div key={s.label} className="rounded-xl border border-white/5 bg-surface-1 p-5">
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm text-white/40">{s.label}</span>
              <s.icon className={`h-4 w-4 ${s.color}`} />
            </div>
            <div className="text-2xl font-bold text-white">{s.value}</div>
            <div className="text-xs text-white/30 mt-1">{s.change ?? s.sub}</div>
          </div>
        ))}
      </div>

      {/* Quota bar */}
      <div className="rounded-xl border border-white/5 bg-surface-1 p-5">
        <div className="flex items-center justify-between mb-3">
          <span className="font-medium text-white">Monthly quota</span>
          <span className="text-sm text-white/40">847 / 1,000 requests</span>
        </div>
        <div className="h-2 w-full rounded-full bg-white/5 overflow-hidden">
          <div className="h-full rounded-full bg-gradient-to-r from-brand-500 to-brand-400" style={{ width: "84.7%" }} />
        </div>
        <div className="flex justify-between mt-2 text-xs text-white/30">
          <span>15.3% remaining</span>
          <a href="/dashboard/billing" className="text-brand-400 hover:underline">Upgrade plan →</a>
        </div>
      </div>

      {/* Recent requests */}
      <div className="rounded-xl border border-white/5 bg-surface-1">
        <div className="flex items-center justify-between border-b border-white/5 px-5 py-4">
          <h2 className="font-medium text-white">Recent requests</h2>
          <a href="/dashboard/usage" className="text-xs text-brand-400 hover:underline">View all →</a>
        </div>
        <div className="divide-y divide-white/5">
          {recentRequests.map((r, i) => (
            <div key={i} className="flex items-center gap-4 px-5 py-3.5 text-sm">
              <span className="rounded-md bg-brand-500/10 px-2 py-0.5 font-mono text-xs text-brand-300">
                {r.endpoint}
              </span>
              <span className="flex-1 truncate text-white/60">{r.query}</span>
              <span className="text-green-400 text-xs">{r.status}</span>
              <span className="text-white/30 text-xs w-14 text-right">{r.latency}</span>
              <span className="text-white/25 text-xs w-20 text-right">{r.time}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
