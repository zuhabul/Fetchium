"use client";

import { useEffect, useMemo, useState } from "react";
import {
  DEFAULT_API_BASE,
  loadDashboardConfig,
  normalize_api_base,
  normalize_api_key,
  saveDashboardConfig,
  validate_api_base,
  validate_api_key,
} from "@/lib/client-config";

type Status = {
  tone: "success" | "error" | "info";
  message: string;
};

export default function SettingsPage() {
  const [apiBaseUrl, setApiBaseUrl] = useState(DEFAULT_API_BASE);
  const [apiKey, setApiKey] = useState("");
  const [status, setStatus] = useState<Status | null>(null);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [showKey, setShowKey] = useState(false);

  useEffect(() => {
    const cfg = loadDashboardConfig();
    setApiBaseUrl(cfg.apiBaseUrl);
    setApiKey(cfg.apiKey);
  }, []);

  const baseError = useMemo(() => validate_api_base(apiBaseUrl), [apiBaseUrl]);
  const keyError = useMemo(() => validate_api_key(apiKey), [apiKey]);
  const canSubmit = !baseError && !keyError;

  async function save() {
    const normalizedBase = normalize_api_base(apiBaseUrl);
    const normalizedKey = normalize_api_key(apiKey);
    const nextBaseError = validate_api_base(normalizedBase);
    const nextKeyError = validate_api_key(normalizedKey);

    if (nextBaseError || nextKeyError) {
      setStatus({
        tone: "error",
        message: nextBaseError || nextKeyError || "Fix the invalid settings first.",
      });
      return;
    }

    setSaving(true);
    setStatus(null);
    try {
      saveDashboardConfig({ apiBaseUrl: normalizedBase, apiKey: normalizedKey });
      setApiBaseUrl(normalizedBase);
      setApiKey(normalizedKey);
      setStatus({
        tone: "success",
        message: "Settings saved locally for this browser.",
      });
    } finally {
      setSaving(false);
    }
  }

  async function testHealth() {
    const normalizedBase = normalize_api_base(apiBaseUrl);
    const normalizedKey = normalize_api_key(apiKey);
    const nextBaseError = validate_api_base(normalizedBase);
    const nextKeyError = validate_api_key(normalizedKey);

    if (nextBaseError || nextKeyError) {
      setStatus({
        tone: "error",
        message: nextBaseError || nextKeyError || "Fix the invalid settings first.",
      });
      return;
    }

    setTesting(true);
    setStatus(null);
    try {
      const q = encodeURIComponent(normalizedBase);
      const [healthRes, usageRes] = await Promise.all([
        fetch(`/api/health?apiBase=${q}`, { cache: "no-store" }),
        fetch("/api/usage", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ apiKey: normalizedKey, apiBase: normalizedBase }),
        }),
      ]);

      const healthBody = (await healthRes.json()) as { status?: string; title?: string; message?: string };
      const usageBody = (await usageRes.json()) as { error?: string; title?: string; message?: string; plan?: string };

      if (!healthRes.ok) {
        setStatus({
          tone: "error",
          message: healthBody.title || healthBody.message || "API health check failed.",
        });
        return;
      }

      if (!usageRes.ok) {
        setStatus({
          tone: "error",
          message: usageBody.title || usageBody.message || "API key validation failed.",
        });
        return;
      }

      setStatus({
        tone: "success",
        message: `Connection verified. API is healthy and the key is valid${usageBody.plan ? ` (${usageBody.plan} plan)` : ""}.`,
      });
    } catch (e) {
      setStatus({
        tone: "error",
        message: e instanceof Error ? e.message : "Health check failed.",
      });
    } finally {
      setTesting(false);
    }
  }

  function resetDefaults() {
    setApiBaseUrl(DEFAULT_API_BASE);
    setStatus({
      tone: "info",
      message: "API base reset to the production default.",
    });
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-sm text-white/40 mt-1">
          Configure the API connection and key used by dashboard usage and playground requests.
        </p>
      </div>

      <div className="rounded-xl border border-white/5 bg-surface-1 divide-y divide-white/5">
        <div className="px-5 py-4">
          <label className="text-sm font-medium text-white/60 block mb-2">API Base URL</label>
          <input
            type="url"
            value={apiBaseUrl}
            onChange={(e) => setApiBaseUrl(e.target.value)}
            spellCheck={false}
            autoCapitalize="off"
            autoCorrect="off"
            placeholder={DEFAULT_API_BASE}
            className="w-full rounded-lg border border-white/5 bg-white/5 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
          />
          <div className="mt-2 flex items-center justify-between gap-3">
            <p className="text-xs text-white/30">
              Production default is <span className="font-mono text-white/60">{DEFAULT_API_BASE}</span>.
            </p>
            <button
              type="button"
              onClick={resetDefaults}
              className="text-xs font-medium text-brand-300 hover:text-brand-200 transition-colors"
            >
              Reset default
            </button>
          </div>
          {baseError && <p className="mt-2 text-xs text-red-400">{baseError}</p>}
        </div>
        <div className="px-5 py-4">
          <label className="text-sm font-medium text-white/60 block mb-2">Default API Key</label>
          <div className="flex gap-2">
            <input
              type={showKey ? "text" : "password"}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              spellCheck={false}
              autoCapitalize="off"
              autoCorrect="off"
              placeholder="fetchium_..."
              className="w-full rounded-lg border border-white/5 bg-white/5 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
            />
            <button
              type="button"
              onClick={() => setShowKey((value) => !value)}
              className="rounded-lg border border-white/10 px-3 py-2 text-xs font-medium text-white/70 hover:text-white transition-colors"
            >
              {showKey ? "Hide" : "Show"}
            </button>
          </div>
          <p className="text-xs text-white/30 mt-2">
            Stored locally in this browser only. It is never written by the dashboard to a server-side user profile.
          </p>
          {keyError && <p className="mt-2 text-xs text-red-400">{keyError}</p>}
        </div>
      </div>

      <div className="flex gap-3">
        <button
          onClick={() => void save()}
          disabled={saving || !canSubmit}
          className="rounded-lg bg-brand-500 px-4 py-2 text-sm text-white hover:bg-brand-600 disabled:opacity-60"
        >
          {saving ? "Saving..." : "Save settings"}
        </button>
        <button
          onClick={() => void testHealth()}
          disabled={testing || !canSubmit}
          className="rounded-lg border border-white/10 px-4 py-2 text-sm text-white/70 hover:text-white disabled:opacity-60"
        >
          {testing ? "Testing..." : "Verify connection"}
        </button>
      </div>

      {status && (
        <div
          className={`rounded-xl border p-3 text-sm ${
            status.tone === "success"
              ? "border-emerald-500/20 bg-emerald-500/10 text-emerald-100"
              : status.tone === "error"
                ? "border-red-500/20 bg-red-500/10 text-red-100"
                : "border-white/10 bg-white/5 text-white/80"
          }`}
        >
          {status.message}
        </div>
      )}
    </div>
  );
}
