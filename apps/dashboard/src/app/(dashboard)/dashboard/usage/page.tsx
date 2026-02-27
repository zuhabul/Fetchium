import type { Metadata } from "next";

export const metadata: Metadata = { title: "Usage — Fetchium Dashboard" };

// Mock 30-day data
const days = Array.from({ length: 30 }, (_, i) => {
  const d = new Date();
  d.setDate(d.getDate() - (29 - i));
  return {
    date: d.toLocaleDateString("en-US", { month: "short", day: "numeric" }),
    requests: Math.floor(Math.random() * 60 + 10),
  };
});

const endpoints = [
  { name: "/v1/search", count: 412, pct: 48 },
  { name: "/v1/scrape", count: 289, pct: 34 },
  { name: "/v1/research", count: 102, pct: 12 },
  { name: "/v1/youtube/search", count: 44, pct: 5 },
];

const maxReq = Math.max(...days.map(d => d.requests));

export default function UsagePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Usage Analytics</h1>
        <p className="text-sm text-white/40 mt-1">Last 30 days of API activity.</p>
      </div>

      {/* Bar chart — server-rendered SVG approximation */}
      <div className="rounded-xl border border-white/5 bg-surface-1 p-5">
        <h2 className="font-medium text-white mb-4">Requests per day</h2>
        <div className="flex items-end gap-1 h-32">
          {days.map((d, i) => (
            <div key={i} className="flex-1 flex flex-col items-center gap-1 group">
              <div
                className="w-full rounded-t bg-brand-500/60 hover:bg-brand-500 transition-colors"
                style={{ height: `${(d.requests / maxReq) * 100}%` }}
                title={`${d.date}: ${d.requests} requests`}
              />
            </div>
          ))}
        </div>
        <div className="flex justify-between mt-2 text-[10px] text-white/25">
          <span>{days[0].date}</span>
          <span>{days[14].date}</span>
          <span>{days[29].date}</span>
        </div>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        {/* By endpoint */}
        <div className="rounded-xl border border-white/5 bg-surface-1 p-5">
          <h2 className="font-medium text-white mb-4">By endpoint</h2>
          <div className="space-y-3">
            {endpoints.map(e => (
              <div key={e.name}>
                <div className="flex justify-between text-sm mb-1">
                  <span className="font-mono text-white/60 text-xs">{e.name}</span>
                  <span className="text-white/40">{e.count}</span>
                </div>
                <div className="h-1.5 w-full rounded-full bg-white/5 overflow-hidden">
                  <div className="h-full rounded-full bg-brand-500/60" style={{ width: `${e.pct}%` }} />
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Summary stats */}
        <div className="rounded-xl border border-white/5 bg-surface-1 p-5">
          <h2 className="font-medium text-white mb-4">Summary</h2>
          <div className="space-y-3">
            {[
              { label: "Total requests", value: "847" },
              { label: "Tokens used", value: "1.2M" },
              { label: "Avg latency", value: "1.2s" },
              { label: "P95 latency", value: "3.4s" },
              { label: "Success rate", value: "99.1%" },
              { label: "Quota remaining", value: "153 req" },
            ].map(s => (
              <div key={s.label} className="flex justify-between text-sm">
                <span className="text-white/40">{s.label}</span>
                <span className="font-medium text-white">{s.value}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
