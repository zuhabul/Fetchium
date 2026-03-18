"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Bell, LogOut, Menu, User, Settings, Key, BarChart3, ChevronRight, X, CheckCircle2, AlertTriangle, Info } from "lucide-react";
import { signOut } from "next-auth/react";
import Link from "next/link";
import ThemeToggle from "@/components/ThemeToggle";

/* ─── Types ──────────────────────────────────────────────────────────── */

type UsageStats = {
  plan: string;
  key_id?: string;
  requests_this_month: number;
  requests_today?: number;
  monthly_limit: number | null;
  quota_remaining?: number | null;
};

type SessionData = {
  plan?: string;
  keyId?: string;
  apiKeyPreview?: string;
  user?: { name?: string; id?: string };
};

type Notification = {
  id: string;
  type: "info" | "warning" | "success";
  title: string;
  message: string;
  time: string;
  read: boolean;
};

/* ─── Header ─────────────────────────────────────────────────────────── */

export default function DashHeader() {
  const [usage, setUsage] = useState<UsageStats | null>(null);
  const [session, setSession] = useState<SessionData | null>(null);
  const [notifOpen, setNotifOpen] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);
  const [notifications, setNotifications] = useState<Notification[]>([]);

  const notifRef = useRef<HTMLDivElement>(null);
  const profileRef = useRef<HTMLDivElement>(null);

  // Load usage + session
  useEffect(() => {
    void (async () => {
      const [usageRes, sessionRes] = await Promise.all([
        fetch("/api/usage", { cache: "no-store" }),
        fetch("/api/auth/session", { cache: "no-store" }),
      ]);
      if (usageRes.ok) {
        const body = (await usageRes.json()) as { usage?: UsageStats };
        setUsage(body.usage || null);
      }
      if (sessionRes.ok) {
        const body = (await sessionRes.json()) as SessionData;
        setSession(body);
      }
    })();
  }, []);

  // Generate notifications from usage data
  useEffect(() => {
    if (!usage) return;
    const notifs: Notification[] = [];
    const now = new Date().toISOString();

    // Quota warning
    if (usage.monthly_limit && usage.quota_remaining != null) {
      const pct = ((usage.monthly_limit - usage.quota_remaining) / usage.monthly_limit) * 100;
      if (pct >= 90) {
        notifs.push({
          id: "quota-critical",
          type: "warning",
          title: "Quota nearly exhausted",
          message: `${usage.quota_remaining.toLocaleString()} requests remaining this month (${Math.round(pct)}% used).`,
          time: now,
          read: false,
        });
      } else if (pct >= 75) {
        notifs.push({
          id: "quota-warning",
          type: "warning",
          title: "Quota usage high",
          message: `${Math.round(pct)}% of monthly quota used. ${usage.quota_remaining.toLocaleString()} requests remaining.`,
          time: now,
          read: false,
        });
      }
    }

    // Active session
    notifs.push({
      id: "session-active",
      type: "success",
      title: "Session active",
      message: `Authenticated with ${usage.plan} plan. ${usage.requests_this_month.toLocaleString()} requests this month.`,
      time: now,
      read: true,
    });

    setNotifications(notifs);
  }, [usage]);

  // Close popovers on outside click
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (notifRef.current && !notifRef.current.contains(e.target as Node)) {
        setNotifOpen(false);
      }
      if (profileRef.current && !profileRef.current.contains(e.target as Node)) {
        setProfileOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const plan = usage?.plan || session?.plan || "unconfigured";
  const usageText =
    usage == null
      ? "Configure API key in Settings to load usage"
      : `${usage.requests_this_month} / ${usage.monthly_limit ?? "unlimited"} requests this month`;

  const unreadCount = notifications.filter((n) => !n.read).length;
  const keyId = session?.keyId || usage?.key_id || "";
  const keyPreview = session?.apiKeyPreview || "";
  const initials = plan.slice(0, 1).toUpperCase();

  function toggleSidebar() {
    window.dispatchEvent(new CustomEvent("fetchium:toggle-dashboard-sidebar"));
  }

  function markAllRead() {
    setNotifications((prev) => prev.map((n) => ({ ...n, read: true })));
  }

  return (
    <header className="flex min-h-16 items-center justify-between gap-3 border-b border-[var(--border-subtle)] bg-[var(--surface-base)] px-4 sm:px-6">
      {/* Left: hamburger + plan info */}
      <div className="flex min-w-0 flex-1 items-center gap-3">
        <button
          type="button"
          onClick={toggleSidebar}
          className="flex h-9 w-9 items-center justify-center rounded-lg border border-[var(--border-subtle)] text-[var(--text-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] lg:hidden"
          aria-label="Toggle navigation"
          aria-controls="dashboard-sidebar"
        >
          <Menu className="h-4 w-4" />
        </button>
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <span className="rounded-full border border-[var(--brand-border)] bg-[var(--brand-soft)] px-2.5 py-0.5 text-xs font-medium capitalize text-[var(--brand-solid)]">
              {plan} plan
            </span>
          </div>
          <span className="mt-1 block truncate text-xs text-[var(--text-faint)]">{usageText}</span>
        </div>
      </div>

      {/* Right: actions */}
      <div className="flex items-center gap-2 sm:gap-3">
        <ThemeToggle />

        {/* ── Notification Bell ── */}
        <div ref={notifRef} className="relative">
          <button
            type="button"
            onClick={() => { setNotifOpen(!notifOpen); setProfileOpen(false); }}
            aria-label="Notifications"
            className="relative flex h-8 w-8 items-center justify-center rounded-lg border border-[var(--border-subtle)] text-[var(--text-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
          >
            <Bell className="h-4 w-4" />
            {unreadCount > 0 && (
              <span className="absolute -right-1 -top-1 flex h-4 w-4 items-center justify-center rounded-full bg-red-500 text-[10px] font-bold text-white">
                {unreadCount}
              </span>
            )}
          </button>

          {notifOpen && (
            <div className="absolute right-0 top-full z-50 mt-2 w-80 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-base)] shadow-2xl sm:w-96">
              <div className="flex items-center justify-between border-b border-[var(--border-subtle)] px-4 py-3">
                <h3 className="text-sm font-semibold text-[var(--text-primary)]">Notifications</h3>
                {unreadCount > 0 && (
                  <button
                    onClick={markAllRead}
                    className="text-xs text-brand-400 hover:underline"
                  >
                    Mark all read
                  </button>
                )}
              </div>
              <div className="max-h-80 overflow-y-auto">
                {notifications.length === 0 ? (
                  <div className="px-4 py-8 text-center text-sm text-[var(--text-faint)]">
                    No notifications
                  </div>
                ) : (
                  notifications.map((notif) => (
                    <div
                      key={notif.id}
                      className={`flex gap-3 border-b border-[var(--border-subtle)] px-4 py-3 last:border-b-0 ${
                        notif.read ? "opacity-60" : ""
                      }`}
                    >
                      <div className="mt-0.5 flex-shrink-0">
                        {notif.type === "warning" && <AlertTriangle className="h-4 w-4 text-amber-400" />}
                        {notif.type === "success" && <CheckCircle2 className="h-4 w-4 text-emerald-400" />}
                        {notif.type === "info" && <Info className="h-4 w-4 text-brand-400" />}
                      </div>
                      <div className="min-w-0 flex-1">
                        <p className="text-sm font-medium text-[var(--text-primary)]">{notif.title}</p>
                        <p className="mt-0.5 text-xs leading-relaxed text-[var(--text-muted)]">{notif.message}</p>
                      </div>
                      {!notif.read && (
                        <div className="mt-1.5 h-2 w-2 flex-shrink-0 rounded-full bg-brand-500" />
                      )}
                    </div>
                  ))
                )}
              </div>
            </div>
          )}
        </div>

        {/* ── Profile Button ── */}
        <div ref={profileRef} className="relative">
          <button
            type="button"
            onClick={() => { setProfileOpen(!profileOpen); setNotifOpen(false); }}
            aria-label="Profile menu"
            className="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--brand-soft)] text-sm font-bold text-[var(--brand-solid)] transition-opacity hover:opacity-80"
          >
            {initials}
          </button>

          {profileOpen && (
            <div className="absolute right-0 top-full z-50 mt-2 w-72 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-base)] shadow-2xl">
              {/* Profile info */}
              <div className="border-b border-[var(--border-subtle)] px-4 py-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--brand-soft)] text-sm font-bold text-[var(--brand-solid)]">
                    {initials}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium capitalize text-[var(--text-primary)]">{plan} plan</p>
                    {keyId && (
                      <p className="truncate text-xs text-[var(--text-muted)]">{keyId}</p>
                    )}
                    {keyPreview && (
                      <p className="truncate font-mono text-[10px] text-[var(--text-faint)]">{keyPreview}</p>
                    )}
                  </div>
                </div>
              </div>

              {/* Menu items */}
              <div className="py-1">
                <ProfileLink href="/dashboard" icon={BarChart3} label="Overview" onClick={() => setProfileOpen(false)} />
                <ProfileLink href="/dashboard/settings" icon={Settings} label="Settings" onClick={() => setProfileOpen(false)} />
                <ProfileLink href="/dashboard/keys" icon={Key} label="API Keys" onClick={() => setProfileOpen(false)} />
                <ProfileLink href="/dashboard/usage" icon={BarChart3} label="Usage" onClick={() => setProfileOpen(false)} />
              </div>

              {/* Sign out */}
              <div className="border-t border-[var(--border-subtle)] py-1">
                <button
                  onClick={() => void signOut({ callbackUrl: "/login" })}
                  className="flex w-full items-center gap-3 px-4 py-2.5 text-left text-sm text-red-400 transition-colors hover:bg-red-500/5"
                >
                  <LogOut className="h-4 w-4" />
                  Sign out
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Sign out (desktop shortcut) */}
        <button
          onClick={() => void signOut({ callbackUrl: "/login" })}
          className="hidden h-8 items-center justify-center gap-1 rounded-lg border border-[var(--border-subtle)] px-3 text-[var(--text-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] sm:flex"
          aria-label="Sign out"
        >
          <LogOut className="h-4 w-4" />
          <span className="text-xs">Sign out</span>
        </button>
      </div>
    </header>
  );
}

/* ─── Sub-components ─────────────────────────────────────────────────── */

function ProfileLink({
  href,
  icon: Icon,
  label,
  onClick,
}: {
  href: string;
  icon: typeof Settings;
  label: string;
  onClick: () => void;
}) {
  return (
    <Link
      href={href}
      onClick={onClick}
      className="flex items-center justify-between gap-3 px-4 py-2.5 text-sm text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
    >
      <div className="flex items-center gap-3">
        <Icon className="h-4 w-4 text-[var(--text-faint)]" />
        {label}
      </div>
      <ChevronRight className="h-3.5 w-3.5 text-[var(--text-faint)]" />
    </Link>
  );
}
