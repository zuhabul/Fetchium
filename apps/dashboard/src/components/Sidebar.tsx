"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Key,
  BarChart3,
  CreditCard,
  Terminal,
  Settings,
  Zap,
  ExternalLink,
} from "lucide-react";

const nav = [
  { href: "/dashboard", icon: LayoutDashboard, label: "Overview" },
  { href: "/dashboard/keys", icon: Key, label: "API Keys" },
  { href: "/dashboard/usage", icon: BarChart3, label: "Usage" },
  { href: "/dashboard/billing", icon: CreditCard, label: "Billing" },
  { href: "/dashboard/playground", icon: Terminal, label: "Playground" },
  { href: "/dashboard/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="flex w-60 flex-col border-r border-white/5 bg-surface-1">
      {/* Logo */}
      <div className="flex h-16 items-center gap-2 border-b border-white/5 px-5">
        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-brand-500">
          <Zap className="h-3.5 w-3.5 text-white" />
        </div>
        <span className="font-semibold text-white text-sm">Fetchium</span>
      </div>

      {/* Nav */}
      <nav className="flex-1 space-y-0.5 p-3">
        {nav.map(({ href, icon: Icon, label }) => {
          const active = pathname === href || (href !== "/dashboard" && pathname.startsWith(href));
          return (
            <Link
              key={href}
              href={href}
              className={`flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors ${
                active
                  ? "bg-brand-500/10 text-brand-300"
                  : "text-white/50 hover:bg-white/5 hover:text-white"
              }`}
            >
              <Icon className={`h-4 w-4 ${active ? "text-brand-400" : ""}`} />
              {label}
            </Link>
          );
        })}
      </nav>

      {/* Bottom */}
      <div className="border-t border-white/5 p-3 space-y-0.5">
        <Link
          href="https://docs.fetchium.com"
          target="_blank"
          className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm text-white/40 hover:text-white/70"
        >
          <ExternalLink className="h-4 w-4" />
          Docs
        </Link>
      </div>
    </aside>
  );
}
