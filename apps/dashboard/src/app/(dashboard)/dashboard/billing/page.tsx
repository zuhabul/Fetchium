"use client";

import Link from "next/link";
import { ArrowUpRight } from "lucide-react";
import { useEffect, useState } from "react";

type UsageStats = {
  plan: string;
  monthly_limit: number | null;
  requests_this_month: number;
};

export default function BillingPage() {
  const [usage, setUsage] = useState<UsageStats | null>(null);

  useEffect(() => {
    void (async () => {
      const res = await fetch("/api/usage", { cache: "no-store" });
      if (!res.ok) return;
      const body = (await res.json()) as { usage?: UsageStats };
      setUsage(body.usage || null);
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

      <div className="rounded-xl border border-white/5 bg-surface-1 p-5">
        <h2 className="font-medium text-white">Plan management</h2>
        <p className="mt-2 max-w-2xl text-sm leading-6 text-white/45">
          Hosted billing changes are handled outside this dashboard to avoid broken checkout or
          stale pricing flows. Review the current pricing page or contact the Fetchium team for plan
          changes on a production workspace.
        </p>
        <div className="mt-4 flex flex-wrap gap-3">
          <Link
            href="https://fetchium.com/pricing"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1.5 rounded-lg bg-brand-500 px-4 py-2 text-sm font-medium text-white hover:bg-brand-600 transition-colors"
          >
            View pricing <ArrowUpRight className="h-3.5 w-3.5" />
          </Link>
          <Link
            href="mailto:founders@fetchium.com?subject=Fetchium%20plan%20change"
            className="inline-flex items-center gap-1.5 rounded-lg border border-white/10 px-4 py-2 text-sm font-medium text-white/70 hover:text-white transition-colors"
          >
            Contact billing <ArrowUpRight className="h-3.5 w-3.5" />
          </Link>
        </div>
      </div>
    </div>
  );
}
