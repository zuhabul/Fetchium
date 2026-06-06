"use client";

import { useCallback, useEffect, useState } from "react";
import { signOut } from "next-auth/react";
import { DEFAULT_API_BASE } from "@/lib/client-config";
import {
  DASHBOARD_ALERT,
  DASHBOARD_CARD_PADDED,
  DASHBOARD_PAGE_HEADER,
  DASHBOARD_PAGE_LEAD,
  DASHBOARD_PAGE_STACK,
  DASHBOARD_PANEL,
  DASHBOARD_PANEL_HEADER,
  DASHBOARD_PANEL_ROW,
} from "@/lib/dashboard-layout";
import {
  CheckCircle2,
  Loader2,
  Save,
  Shield,
  Bell,
  Settings2,
  Wifi,
  LogOut,
  Building2,
} from "lucide-react";

/* ─── Types ──────────────────────────────────────────────────────────── */

type ConnectionStatus = {
  tone: "success" | "error" | "info";
  message: string;
};

type SessionState = {
  plan?: string;
  keyId?: string;
  apiKeyPreview?: string;
};

type Preferences = {
  workspace_name: string;
  email_updates: boolean;
  incident_alerts: boolean;
  changelog_notifications: boolean;
  default_search_tier: string;
  default_max_sources: number;
  theme: string;
  updated_at: string;
};

type SettingsData = {
  workspace: {
    name: string;
    plan: string;
    key_id: string;
  };
  session: {
    key_id: string;
    api_key_preview: string;
    api_base: string;
    created_at: string;
    last_used_at: string | null;
  };
  preferences: Preferences;
};

/* ─── Page ───────────────────────────────────────────────────────────── */

export default function SettingsPage() {
  const [session, setSession] = useState<SessionState | null>(null);
  const [settings, setSettings] = useState<SettingsData | null>(null);
  const [loadError, setLoadError] = useState(false);
  const [connStatus, setConnStatus] = useState<ConnectionStatus | null>(null);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  // Editable form state
  const [workspaceName, setWorkspaceName] = useState("");
  const [emailUpdates, setEmailUpdates] = useState(true);
  const [incidentAlerts, setIncidentAlerts] = useState(false);
  const [changelogNotifications, setChangelogNotifications] = useState(true);
  const [defaultTier, setDefaultTier] = useState("summary");
  const [defaultMaxSources, setDefaultMaxSources] = useState(5);
  const [theme, setTheme] = useState("system");

  const loadSettings = useCallback(async () => {
    try {
      const res = await fetch("/api/dashboard/settings", { cache: "no-store" });
      if (!res.ok) {
        setLoadError(true);
        return;
      }
      const body = await res.json();
      const data: SettingsData = body.settings ?? body;
      setSettings(data);
      // Hydrate form
      setWorkspaceName(data.preferences.workspace_name);
      setEmailUpdates(data.preferences.email_updates);
      setIncidentAlerts(data.preferences.incident_alerts);
      setChangelogNotifications(data.preferences.changelog_notifications);
      setDefaultTier(data.preferences.default_search_tier);
      setDefaultMaxSources(data.preferences.default_max_sources);
      setTheme(data.preferences.theme);
      setLoadError(false);
    } catch {
      setLoadError(true);
    }
  }, []);

  useEffect(() => {
    void (async () => {
      const res = await fetch("/api/auth/session", { cache: "no-store" });
      if (!res.ok) return;
      const body = (await res.json()) as SessionState;
      setSession(body);
    })();
    void loadSettings();
  }, [loadSettings]);

  async function saveSettings() {
    setSaving(true);
    setSaved(false);
    try {
      const res = await fetch("/api/dashboard/settings", {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          workspace_name: workspaceName,
          email_updates: emailUpdates,
          incident_alerts: incidentAlerts,
          changelog_notifications: changelogNotifications,
          default_search_tier: defaultTier,
          default_max_sources: defaultMaxSources,
          theme,
        }),
      });
      if (res.ok) {
        const body = await res.json();
        const data: SettingsData | undefined = body.settings;
        if (data) setSettings({ ...settings!, preferences: data as unknown as Preferences });
        setSaved(true);
        setTimeout(() => setSaved(false), 2000);
      }
    } catch {
      // silent
    } finally {
      setSaving(false);
    }
  }

  async function testHealth() {
    setTesting(true);
    setConnStatus(null);
    try {
      const q = encodeURIComponent(DEFAULT_API_BASE);
      const [healthRes, usageRes] = await Promise.all([
        fetch(`/api/health?apiBase=${q}`, { cache: "no-store" }),
        fetch("/api/usage", { cache: "no-store" }),
      ]);

      const healthBody = (await healthRes.json()) as { status?: string; title?: string; message?: string };
      const usageBody = (await usageRes.json()) as { error?: string; title?: string; message?: string; usage?: { plan?: string } };

      if (!healthRes.ok) {
        setConnStatus({ tone: "error", message: healthBody.title || healthBody.message || "API health check failed." });
        return;
      }
      if (!usageRes.ok) {
        setConnStatus({ tone: "error", message: usageBody.title || usageBody.message || "API key validation failed." });
        return;
      }
      setConnStatus({
        tone: "success",
        message: `Connection verified. API is healthy and the session is valid${usageBody.usage?.plan ? ` (${usageBody.usage.plan} plan)` : ""}.`,
      });
    } catch (e) {
      setConnStatus({ tone: "error", message: e instanceof Error ? e.message : "Health check failed." });
    } finally {
      setTesting(false);
    }
  }

  const hasChanges =
    settings &&
    (workspaceName !== settings.preferences.workspace_name ||
      emailUpdates !== settings.preferences.email_updates ||
      incidentAlerts !== settings.preferences.incident_alerts ||
      changelogNotifications !== settings.preferences.changelog_notifications ||
      defaultTier !== settings.preferences.default_search_tier ||
      defaultMaxSources !== settings.preferences.default_max_sources ||
      theme !== settings.preferences.theme);

  return (
    <div className={DASHBOARD_PAGE_STACK}>
      {/* Header */}
      <div className={DASHBOARD_PAGE_HEADER}>
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">Settings</h1>
          <p className={DASHBOARD_PAGE_LEAD}>
            Manage your workspace, preferences, and session.
          </p>
        </div>
        <div className="flex flex-col gap-3 sm:flex-row">
          <button
            onClick={() => void saveSettings()}
            disabled={saving || !hasChanges}
            className="inline-flex items-center gap-2 rounded-lg bg-brand-500 px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-brand-600 disabled:opacity-50"
          >
            {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : saved ? <CheckCircle2 className="h-4 w-4" /> : <Save className="h-4 w-4" />}
            {saving ? "Saving..." : saved ? "Saved" : "Save changes"}
          </button>
          <button
            onClick={() => void signOut({ callbackUrl: "/login" })}
            className="inline-flex items-center gap-2 rounded-lg border border-red-500/30 px-4 py-2.5 text-sm text-red-400 transition-colors hover:bg-red-500/10"
          >
            <LogOut className="h-4 w-4" />
            Sign out
          </button>
        </div>
      </div>

      {loadError && (
        <div className={`${DASHBOARD_ALERT} border-amber-500/20 bg-amber-500/5 text-amber-300`}>
          Unable to load settings from the API. Showing session-only data.
        </div>
      )}

      {/* Workspace */}
      <section className={DASHBOARD_PANEL}>
        <div className={DASHBOARD_PANEL_HEADER}>
          <div className="flex items-center gap-2">
            <Building2 className="h-4 w-4 text-brand-400" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">Workspace</h2>
          </div>
        </div>
        <div className={DASHBOARD_PANEL_ROW}>
          <div className="grid gap-4 sm:grid-cols-2">
            <div>
              <label className="mb-1.5 block text-xs font-medium text-[var(--text-muted)]">Workspace name</label>
              <input
                type="text"
                value={workspaceName}
                onChange={(e) => setWorkspaceName(e.target.value)}
                placeholder="My workspace"
                className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-faint)] focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
              />
            </div>
            <div>
              <label className="mb-1.5 block text-xs font-medium text-[var(--text-muted)]">Plan</label>
              <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-sunken)] px-3 py-2 text-sm text-[var(--text-primary)]">
                {settings?.workspace.plan || session?.plan || "—"}
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Session & Security */}
      <section className={DASHBOARD_PANEL}>
        <div className={DASHBOARD_PANEL_HEADER}>
          <div className="flex items-center gap-2">
            <Shield className="h-4 w-4 text-brand-400" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">Session & Security</h2>
          </div>
        </div>
        <div className={DASHBOARD_PANEL_ROW}>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <InfoTile label="Key ID" value={settings?.session.key_id || session?.keyId || "—"} />
            <InfoTile label="Key Preview" value={settings?.session.api_key_preview || session?.apiKeyPreview || "—"} />
            <InfoTile label="Created" value={formatTime(settings?.session.created_at)} />
            <InfoTile label="Last used" value={formatTime(settings?.session.last_used_at ?? undefined)} />
          </div>
        </div>
        <div className={DASHBOARD_PANEL_ROW}>
          <div className="space-y-2">
            <p className="text-xs text-[var(--text-faint)]">
              The full API key is never re-exposed after sign-in. The dashboard operates through the authenticated server-side session.
            </p>
          </div>
        </div>
      </section>

      {/* API Defaults */}
      <section className={DASHBOARD_PANEL}>
        <div className={DASHBOARD_PANEL_HEADER}>
          <div className="flex items-center gap-2">
            <Settings2 className="h-4 w-4 text-brand-400" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">API Defaults</h2>
          </div>
        </div>
        <div className={DASHBOARD_PANEL_ROW}>
          <div className="grid gap-4 sm:grid-cols-3">
            <div>
              <label className="mb-1.5 block text-xs font-medium text-[var(--text-muted)]">Default search tier</label>
              <select
                value={defaultTier}
                onChange={(e) => setDefaultTier(e.target.value)}
                className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2 text-sm text-[var(--text-primary)] focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
              >
                <option value="key_facts">Key Facts (~200 tokens)</option>
                <option value="summary">Summary (~1000 tokens)</option>
                <option value="detailed">Detailed (~5000 tokens)</option>
                <option value="complete">Complete (full)</option>
              </select>
            </div>
            <div>
              <label className="mb-1.5 block text-xs font-medium text-[var(--text-muted)]">Default max sources</label>
              <input
                type="number"
                min={1}
                max={20}
                value={defaultMaxSources}
                onChange={(e) => setDefaultMaxSources(Math.max(1, Math.min(20, Number(e.target.value) || 1)))}
                className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2 text-sm text-[var(--text-primary)] focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
              />
            </div>
            <div>
              <label className="mb-1.5 block text-xs font-medium text-[var(--text-muted)]">Theme</label>
              <select
                value={theme}
                onChange={(e) => setTheme(e.target.value)}
                className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-2 text-sm text-[var(--text-primary)] focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
              >
                <option value="system">System</option>
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>
          </div>
        </div>
        <div className={DASHBOARD_PANEL_ROW}>
          <div>
            <label className="mb-1.5 block text-xs font-medium text-[var(--text-muted)]">Production API base</label>
            <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-sunken)] px-3 py-2 text-sm text-[var(--text-primary)]">
              {DEFAULT_API_BASE}
            </div>
            <p className="mt-1.5 text-xs text-[var(--text-faint)]">
              Locked to the production API in hosted mode.
            </p>
          </div>
        </div>
      </section>

      {/* Notifications */}
      <section className={DASHBOARD_PANEL}>
        <div className={DASHBOARD_PANEL_HEADER}>
          <div className="flex items-center gap-2">
            <Bell className="h-4 w-4 text-brand-400" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">Notifications</h2>
          </div>
        </div>
        <div className={DASHBOARD_PANEL_ROW}>
          <div className="space-y-4">
            <ToggleRow
              label="Product updates"
              description="Receive emails about new features, improvements, and releases."
              checked={emailUpdates}
              onChange={setEmailUpdates}
            />
            <ToggleRow
              label="Incident alerts"
              description="Get notified about service incidents and maintenance windows."
              checked={incidentAlerts}
              onChange={setIncidentAlerts}
            />
            <ToggleRow
              label="Changelog notifications"
              description="Receive updates when new changelog entries are published."
              checked={changelogNotifications}
              onChange={setChangelogNotifications}
            />
          </div>
        </div>
      </section>

      {/* Connection Diagnostics */}
      <section className={DASHBOARD_PANEL}>
        <div className={DASHBOARD_PANEL_HEADER}>
          <div className="flex items-center gap-2">
            <Wifi className="h-4 w-4 text-brand-400" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">Connection Diagnostics</h2>
          </div>
          <button
            onClick={() => void testHealth()}
            disabled={testing}
            className="inline-flex items-center gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-1.5 text-xs text-[var(--text-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] disabled:opacity-60"
          >
            {testing ? <Loader2 className="h-3 w-3 animate-spin" /> : <Wifi className="h-3 w-3" />}
            {testing ? "Testing..." : "Verify connection"}
          </button>
        </div>
        {connStatus && (
          <div className={DASHBOARD_PANEL_ROW}>
            <div
              className={`${DASHBOARD_ALERT} ${
                connStatus.tone === "success"
                  ? "border-emerald-500/20 bg-[var(--success-soft)] text-[var(--success-text)]"
                  : connStatus.tone === "error"
                    ? "border-red-500/20 bg-red-500/10 text-[var(--danger-text)]"
                    : "border-[var(--border-subtle)] bg-[var(--surface-raised)] text-[var(--text-secondary)]"
              }`}
            >
              {connStatus.message}
            </div>
          </div>
        )}
        <div className={DASHBOARD_PANEL_ROW}>
          <div className="space-y-3">
            <NoteTile
              title="Session security"
              copy="The full API key is not re-exposed after sign-in. The dashboard operates through the authenticated session instead."
            />
            <NoteTile
              title="Connectivity checks"
              copy="Verify connection runs both the production health check and usage validation so the session can be tested end-to-end."
            />
          </div>
        </div>
      </section>

      {/* Last saved indicator */}
      {settings?.preferences.updated_at && (
        <p className="text-xs text-[var(--text-faint)]">
          Settings last updated: {formatTime(settings.preferences.updated_at)}
        </p>
      )}
    </div>
  );
}

/* ─── Helpers ────────────────────────────────────────────────────────── */

function formatTime(iso?: string): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

/* ─── Sub-components ─────────────────────────────────────────────────── */

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-3 py-3">
      <div className="text-xs text-[var(--text-faint)]">{label}</div>
      <div className="mt-1 break-all text-sm font-medium text-[var(--text-primary)]">{value}</div>
    </div>
  );
}

function NoteTile({ title, copy }: { title: string; copy: string }) {
  return (
    <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4">
      <div className="text-sm font-medium text-[var(--text-primary)]">{title}</div>
      <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">{copy}</p>
    </div>
  );
}

function ToggleRow({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (val: boolean) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div className="min-w-0">
        <p className="text-sm font-medium text-[var(--text-primary)]">{label}</p>
        <p className="mt-0.5 text-xs leading-5 text-[var(--text-muted)]">{description}</p>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative mt-0.5 inline-flex h-5 w-9 flex-shrink-0 cursor-pointer items-center rounded-full transition-colors ${
          checked ? "bg-brand-500" : "bg-[var(--surface-sunken)]"
        }`}
      >
        <span
          className={`inline-block h-3.5 w-3.5 rounded-full bg-white shadow transition-transform ${
            checked ? "translate-x-4" : "translate-x-0.5"
          }`}
        />
      </button>
    </div>
  );
}
