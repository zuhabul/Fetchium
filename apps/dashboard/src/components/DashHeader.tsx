"use client";

import { useEffect, useState } from "react";
import { Bell, LogOut, User } from "lucide-react";
import { signOut } from "next-auth/react";

type UsageStats = {
  plan: string;
  requests_this_month: number;
  monthly_limit: number | null;
};

export default function DashHeader() {
  const [usage, setUsage] = useState<UsageStats | null>(null);

  useEffect(() => {
    void (async () => {
      const res = await fetch("/api/usage", { cache: "no-store" });
      if (!res.ok) return;
      const body = (await res.json()) as { usage?: UsageStats };
      setUsage(body.usage || null);
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
        <button
          onClick={() => void signOut({ callbackUrl: "/login" })}
          className="flex h-8 items-center justify-center gap-1 rounded-lg border border-white/5 px-3 text-white/50 transition-colors hover:text-white"
        >
          <LogOut className="h-4 w-4" />
          <span className="text-xs">Sign out</span>
        </button>
      </div>
    </header>
  );
}
