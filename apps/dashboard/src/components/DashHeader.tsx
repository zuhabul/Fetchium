"use client";

import { useEffect, useState } from "react";
import { Bell, User } from "lucide-react";
import { loadDashboardConfig } from "@/lib/client-config";

type UsageStats = {
  plan: string;
  requests_this_month: number;
  monthly_limit: number | null;
};

export default function DashHeader() {
  const [usage, setUsage] = useState<UsageStats | null>(null);

  useEffect(() => {
    const cfg = loadDashboardConfig();
    if (!cfg.apiKey) return;
    void (async () => {
      const res = await fetch("/api/usage", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ apiKey: cfg.apiKey, apiBase: cfg.apiBaseUrl }),
      });
      if (!res.ok) return;
      const body = (await res.json()) as UsageStats;
      setUsage(body);
    })();
  }, []);

  const plan = usage?.plan || "unconfigured";
  const usageText =
    usage == null
      ? "Configure API key in Settings to load usage"
      : `${usage.requests_this_month} / ${usage.monthly_limit ?? "unlimited"} requests this month`;

  return (
    <header className="flex h-16 items-center justify-between border-b border-white/5 px-6">
      <div className="flex items-center gap-2">
        <span className="text-xs font-medium rounded-full bg-brand-500/10 text-brand-300 px-2.5 py-0.5 border border-brand-500/20 capitalize">
          {plan} plan
        </span>
        <span className="text-xs text-white/30">{usageText}</span>
      </div>

      <div className="flex items-center gap-3">
        <button className="flex h-8 w-8 items-center justify-center rounded-lg border border-white/5 text-white/40 hover:text-white transition-colors">
          <Bell className="h-4 w-4" />
        </button>
        <button className="flex h-8 w-8 items-center justify-center rounded-full bg-brand-500/20 text-brand-300">
          <User className="h-4 w-4" />
        </button>
      </div>
    </header>
  );
}

