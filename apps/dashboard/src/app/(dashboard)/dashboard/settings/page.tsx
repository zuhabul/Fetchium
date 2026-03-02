"use client";

import { useEffect, useState } from "react";
import { loadDashboardConfig, saveDashboardConfig } from "@/lib/client-config";

export default function SettingsPage() {
  const [apiBaseUrl, setApiBaseUrl] = useState("http://localhost:3050");
  const [apiKey, setApiKey] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    const cfg = loadDashboardConfig();
    setApiBaseUrl(cfg.apiBaseUrl);
    setApiKey(cfg.apiKey);
  }, []);

  async function save() {
    setSaving(true);
    setStatus(null);
    try {
      saveDashboardConfig({ apiBaseUrl, apiKey });
      setStatus("Saved dashboard settings.");
    } finally {
      setSaving(false);
    }
  }

  async function testHealth() {
    setTesting(true);
    setStatus(null);
    try {
      const q = encodeURIComponent(apiBaseUrl);
      const res = await fetch(`/api/health?apiBase=${q}`, { cache: "no-store" });
      const body = (await res.json()) as { status?: string; title?: string; message?: string };
      if (!res.ok) {
        setStatus(body.title || body.message || "Health check failed.");
      } else {
        setStatus(`Health check OK (${body.status || "ok"})`);
      }
    } catch (e) {
      setStatus(e instanceof Error ? e.message : "Health check failed.");
    } finally {
      setTesting(false);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-sm text-white/40 mt-1">
          Configure dashboard API connection and default key used by Usage/Playground.
        </p>
      </div>

      <div className="rounded-xl border border-white/5 bg-surface-1 divide-y divide-white/5">
        <div className="px-5 py-4">
          <label className="text-sm font-medium text-white/60 block mb-2">API Base URL</label>
          <input
            type="text"
            value={apiBaseUrl}
            onChange={(e) => setApiBaseUrl(e.target.value)}
            className="w-full rounded-lg border border-white/5 bg-white/5 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
          />
        </div>
        <div className="px-5 py-4">
          <label className="text-sm font-medium text-white/60 block mb-2">Default API Key</label>
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="fetchium_..."
            className="w-full rounded-lg border border-white/5 bg-white/5 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
          />
          <p className="text-xs text-white/30 mt-2">
            Stored locally in your browser for dashboard convenience.
          </p>
        </div>
      </div>

      <div className="flex gap-3">
        <button
          onClick={() => void save()}
          disabled={saving}
          className="rounded-lg bg-brand-500 px-4 py-2 text-sm text-white hover:bg-brand-600 disabled:opacity-60"
        >
          {saving ? "Saving..." : "Save settings"}
        </button>
        <button
          onClick={() => void testHealth()}
          disabled={testing}
          className="rounded-lg border border-white/10 px-4 py-2 text-sm text-white/70 hover:text-white disabled:opacity-60"
        >
          {testing ? "Testing..." : "Test connection"}
        </button>
      </div>

      {status && (
        <div className="rounded-xl border border-white/10 bg-white/5 p-3 text-sm text-white/80">
          {status}
        </div>
      )}
    </div>
  );
}

