"use client";

import { useEffect, useMemo, useState } from "react";
import { Plus, Copy, Trash2, Check } from "lucide-react";
import { ADMIN_KEYS_ENABLED } from "@/lib/client-config";

type KeyInfo = {
  id: string;
  name: string;
  key_preview: string;
  plan: string;
  created_at: string;
  last_used_at?: string | null;
};

type KeysResponse = {
  keys: KeyInfo[];
  count: number;
};

type CreateKeyResponse = {
  key: string;
  id: string;
  name: string;
  plan: string;
  created_at: string;
  warning: string;
};

const plans = ["free", "starter", "pro", "enterprise"] as const;

export default function KeysPage() {
  const [keys, setKeys] = useState<KeyInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [newPlan, setNewPlan] = useState<(typeof plans)[number]>("free");
  const [newKey, setNewKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const sorted = useMemo(
    () =>
      [...keys].sort((a, b) => {
        return (b.created_at || "").localeCompare(a.created_at || "");
      }),
    [keys],
  );

  async function loadKeys() {
    if (!ADMIN_KEYS_ENABLED) {
      setLoading(false);
      setKeys([]);
      setError("API key management is disabled on the hosted dashboard.");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/keys", { cache: "no-store" });
      const json = (await res.json()) as KeysResponse | { title?: string; message?: string };
      if (!res.ok) {
        setError((json as { title?: string }).title || "Failed to load API keys");
        setKeys([]);
        return;
      }
      setKeys((json as KeysResponse).keys || []);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load API keys");
      setKeys([]);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadKeys();
  }, []);

  async function createKey() {
    setSubmitting(true);
    setError(null);
    try {
      const res = await fetch("/api/keys", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: newName || "Dashboard key", plan: newPlan }),
      });
      const json = (await res.json()) as CreateKeyResponse | { title?: string; message?: string };
      if (!res.ok) {
        setError((json as { title?: string }).title || "Failed to create key");
        return;
      }
      setNewKey((json as CreateKeyResponse).key);
      setCreating(false);
      setNewName("");
      setNewPlan("free");
      await loadKeys();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create key");
    } finally {
      setSubmitting(false);
    }
  }

  async function revokeKey(id: string) {
    if (!confirm("Revoke this key? This cannot be undone.")) return;
    setError(null);
    try {
      const res = await fetch(`/api/keys/${encodeURIComponent(id)}`, { method: "DELETE" });
      if (!res.ok) {
        const json = (await res.json()) as { title?: string };
        setError(json.title || "Failed to revoke key");
        return;
      }
      await loadKeys();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to revoke key");
    }
  }

  async function copy(text: string) {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">API Keys</h1>
          <p className="text-sm text-white/40 mt-1">
            Real-time key management via `/v1/keys` admin endpoints.
          </p>
        </div>
        {ADMIN_KEYS_ENABLED && (
          <button
            onClick={() => setCreating(true)}
            className="flex items-center gap-2 rounded-xl bg-brand-500 px-4 py-2 text-sm font-medium text-white hover:bg-brand-600 transition-colors"
          >
            <Plus className="h-4 w-4" />
            Create key
          </button>
        )}
      </div>

      {error && (
        <div className="rounded-xl border border-red-500/20 bg-red-500/5 p-3 text-sm text-red-300">
          {error}
        </div>
      )}

      {newKey && (
        <div className="rounded-xl border border-green-500/20 bg-green-500/5 p-4">
          <div className="flex items-start justify-between gap-4">
            <div>
              <p className="text-sm font-medium text-green-400 mb-1">New API key created</p>
              <p className="text-xs text-white/50 mb-3">Copy it now — it won&apos;t be shown again.</p>
              <code className="font-mono text-sm text-white/80 break-all">{newKey}</code>
            </div>
            <button
              onClick={() => copy(newKey)}
              className="shrink-0 flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-1.5 text-xs text-white/60 hover:text-white"
            >
              {copied ? <Check className="h-3.5 w-3.5 text-green-400" /> : <Copy className="h-3.5 w-3.5" />}
              {copied ? "Copied!" : "Copy"}
            </button>
          </div>
          <button onClick={() => setNewKey(null)} className="mt-3 text-xs text-white/30 hover:text-white/50">
            I&apos;ve saved this key ✓
          </button>
        </div>
      )}

      {!ADMIN_KEYS_ENABLED ? (
        <div className="rounded-xl border border-amber-500/20 bg-amber-500/5 p-4 text-sm text-amber-200">
          API key management is disabled on the hosted dashboard until authenticated admin access exists.
        </div>
      ) : creating && !newKey ? (
        <div className="rounded-xl border border-white/10 bg-surface-2 p-4">
          <h3 className="font-medium text-white mb-3">Create new key</h3>
          <div className="flex gap-3">
            <input
              autoFocus
              type="text"
              placeholder="Key name (e.g. Production)"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && void createKey()}
              className="flex-1 rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder-white/30 outline-none focus:border-brand-500/50"
            />
            <select
              value={newPlan}
              onChange={(e) => setNewPlan(e.target.value as (typeof plans)[number])}
              className="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white outline-none focus:border-brand-500/50"
            >
              {plans.map((plan) => (
                <option key={plan} value={plan}>
                  {plan}
                </option>
              ))}
            </select>
            <button
              onClick={() => void createKey()}
              disabled={submitting}
              className="rounded-lg bg-brand-500 px-4 py-2 text-sm font-medium text-white hover:bg-brand-600 disabled:opacity-60"
            >
              {submitting ? "Creating..." : "Create"}
            </button>
            <button
              onClick={() => setCreating(false)}
              className="rounded-lg border border-white/10 px-3 py-2 text-sm text-white/50 hover:text-white"
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}

      <div className="rounded-xl border border-white/5 bg-surface-1 divide-y divide-white/5">
        {loading ? (
          <div className="py-12 text-center text-white/30 text-sm">Loading keys...</div>
        ) : sorted.length === 0 ? (
          <div className="py-12 text-center text-white/30 text-sm">No API keys yet.</div>
        ) : (
          sorted.map((k) => (
            <div key={k.id} className="flex items-center gap-4 px-5 py-4">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-medium text-white text-sm">{k.name}</span>
                  <span className="rounded-full bg-brand-500/10 text-brand-300 border border-brand-500/20 px-2 py-0.5 text-xs">
                    {k.plan}
                  </span>
                </div>
                <div className="font-mono text-xs text-white/40">{k.key_preview}</div>
              </div>
              <div className="text-xs text-white/30 hidden sm:block">
                <div>Created {new Date(k.created_at).toLocaleString()}</div>
                <div>
                  Last used{" "}
                  {k.last_used_at ? new Date(k.last_used_at).toLocaleString() : "Never"}
                </div>
              </div>
              <button
                onClick={() => void copy(k.key_preview)}
                className="flex h-8 w-8 items-center justify-center rounded-lg border border-white/5 text-white/40 hover:text-white transition-colors"
                title="Copy preview"
              >
                <Copy className="h-3.5 w-3.5" />
              </button>
              <button
                onClick={() => void revokeKey(k.id)}
                className="flex h-8 w-8 items-center justify-center rounded-lg border border-white/5 text-white/30 hover:text-red-400 transition-colors"
                title="Revoke key"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
