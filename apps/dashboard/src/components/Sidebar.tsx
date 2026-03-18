"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  BookOpen,
  Key,
  BarChart3,
  CreditCard,
  Terminal,
  Settings,
  Zap,
  ExternalLink,
  X,
} from "lucide-react";
import { ADMIN_KEYS_ENABLED } from "@/lib/client-config";

const nav = [
  { href: "/dashboard", icon: LayoutDashboard, label: "Overview" },
  { href: "/dashboard/quickstart", icon: BookOpen, label: "Quickstart" },
  { href: "/dashboard/api", icon: Terminal, label: "API Catalog" },
  { href: "/dashboard/keys", icon: Key, label: "API Keys" },
  { href: "/dashboard/usage", icon: BarChart3, label: "Usage" },
  { href: "/dashboard/billing", icon: CreditCard, label: "Billing" },
  { href: "/dashboard/playground", icon: Terminal, label: "Playground" },
  { href: "/dashboard/settings", icon: Settings, label: "Settings" },
].filter((item) => ADMIN_KEYS_ENABLED || item.href !== "/dashboard/keys");

export default function Sidebar({
  mobileOpen = false,
  onClose,
}: {
  mobileOpen?: boolean;
  onClose?: () => void;
}) {
  const pathname = usePathname();

  return (
    <aside
      id="dashboard-sidebar"
      className={`fixed inset-y-0 left-0 z-40 flex w-60 flex-col border-r border-[var(--border-subtle)] bg-[var(--surface-base)] transition-transform duration-200 lg:static lg:translate-x-0 ${
        mobileOpen ? "translate-x-0" : "-translate-x-full"
      }`}
    >
      {/* Logo */}
      <div className="flex h-16 items-center gap-2 border-b border-[var(--border-subtle)] px-5">
        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-[var(--brand-solid)]">
          <Zap className="h-3.5 w-3.5 text-white" />
        </div>
        <span className="text-sm font-semibold text-[var(--text-primary)]">Fetchium</span>
        <button
          type="button"
          onClick={onClose}
          className="ml-auto flex h-7 w-7 items-center justify-center rounded text-[var(--text-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] lg:hidden"
          aria-label="Close navigation"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      {/* Nav */}
      <nav className="flex-1 space-y-0.5 p-3">
        {nav.map(({ href, icon: Icon, label }) => {
          const active = pathname === href || (href !== "/dashboard" && pathname.startsWith(href));
          return (
            <Link
              key={href}
              href={href}
              onClick={onClose}
              className={`flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors ${
                active
                  ? "bg-[var(--brand-soft)] text-[var(--brand-solid)]"
                  : "text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
              }`}
            >
              <Icon className={`h-4 w-4 ${active ? "text-[var(--brand-solid)]" : ""}`} />
              {label}
            </Link>
          );
        })}
      </nav>

      {/* Bottom */}
      <div className="space-y-0.5 border-t border-[var(--border-subtle)] p-3">
        <Link
          href="https://docs.fetchium.com"
          target="_blank"
          onClick={onClose}
          className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm text-[var(--text-muted)] hover:text-[var(--text-primary)]"
        >
          <ExternalLink className="h-4 w-4" />
          Docs
        </Link>
      </div>
    </aside>
  );
}
