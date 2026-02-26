"use client";

import { Bell, User } from "lucide-react";

export default function DashHeader() {
  return (
    <header className="flex h-16 items-center justify-between border-b border-white/5 px-6">
      <div className="flex items-center gap-2">
        <span className="text-xs font-medium rounded-full bg-brand-500/10 text-brand-300 px-2.5 py-0.5 border border-brand-500/20">
          Free plan
        </span>
        <span className="text-xs text-white/30">847 / 1,000 requests this month</span>
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
