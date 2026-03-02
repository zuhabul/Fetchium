"use client";

import Link from "next/link";
import { ArrowUpRight } from "lucide-react";
import { useEffect, useState } from "react";
import { loadDashboardConfig } from "@/lib/client-config";

type UsageStats = {
  plan: string;
  monthly_limit: number | null;
  requests_this_month: number;
};

const plans = [
  { name: "free", price: "$0", requests: "1,000 / month" },
  { name: "starter", price: "$19/mo", requests: "25,000 / month" },
  { name: "pro", price: "$79/mo", requests: "250,000 / month" },
  { name: "enterprise", price: "$299/mo", requests: "Unlimited" },
];

export default function BillingPage() {
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
      setUsage((await res.json()) as UsageStats);
    })();
  }, []);

  const currentPlan = usage?.plan?.toLowerCase() || "free";

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Billing</h1>
        <p className="text-sm text-white/40 mt-1">Manage your subscription and payment.</p>
      </div>

      <div className="rounded-xl border border-brand-500/20 bg-brand-500/5 p-5">
        <div className="flex items-start justify-between">
          <div>
            <span className="text-xs text-brand-400 font-medium uppercase tracking-wider">Current plan</span>
            <h2 className="text-xl font-bold text-white mt-1 capitalize">{currentPlan}</h2>
            <p className="text-sm text-white/40 mt-1">
              {usage
                ? `${usage.requests_this_month} / ${usage.monthly_limit ?? "unlimited"} requests`
                : "Connect API key in Settings to load live plan usage"}
            </p>
          </div>
          <span className="rounded-full bg-green-500/10 text-green-400 border border-green-500/20 px-2.5 py-0.5 text-xs font-medium">
            Active
          </span>
        </div>
      </div>

      <div>
        <h2 className="font-medium text-white mb-3">Upgrade your plan</h2>
        <div className="grid gap-3 sm:grid-cols-3">
          {plans
            .filter((p) => p.name !== currentPlan)
            .map((p) => (
              <div key={p.name} className="rounded-xl border border-white/5 bg-surface-1 p-4">
                <div className="flex justify-between items-start mb-3">
                  <div>
                    <div className="font-semibold text-white capitalize">{p.name}</div>
                    <div className="text-xs text-white/40 mt-0.5">{p.requests}</div>
                  </div>
                  <div className="text-sm font-bold text-white">{p.price}</div>
                </div>
                <Link
                  href={`https://app.fetchium.com/checkout?plan=${p.name}`}
                  className="flex items-center justify-center gap-1.5 w-full rounded-lg bg-brand-500 py-2 text-sm font-medium text-white hover:bg-brand-600 transition-colors"
                >
                  Upgrade <ArrowUpRight className="h-3.5 w-3.5" />
                </Link>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
}

