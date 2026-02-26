"use client";

import { useState } from "react";
import { Plus, Copy, Trash2, Eye, EyeOff, Check } from "lucide-react";
import type { Metadata } from "next";

// Note: metadata can't be used in client components; move to a server wrapper if needed.

const mockKeys = [
  {
    id: "k_1",
    name: "Production",
    preview: "hsx_a1b2...****",
    plan: "Free",
    created: "Jan 15, 2026",
    lastUsed: "2 min ago",
  },
  {
    id: "k_2",
    name: "Development",
    preview: "hsx_c3d4...****",
    plan: "Free",
    created: "Jan 20, 2026",
    lastUsed: "1 day ago",
  },
];

export default function KeysPage() {
  const [keys, setKeys] = useState(mockKeys);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [newKey, setNewKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const createKey = () => {
    // In real app: POST /v1/keys
    const fakeKey = "hsx_" + Array.from(crypto.getRandomValues(new Uint8Array(32)))
      .map(b => b.toString(16).padStart(2, "0")).join("");
    setNewKey(fakeKey);
    setKeys(prev => [...prev, {
      id: "k_" + Date.now(),
      name: newName || "Unnamed key",
      preview: fakeKey.slice(0, 12) + "...****",
      plan: "Free",
      created: "Just now",
      lastUsed: "Never",
    }]);
    setNewName("");
    setCreating(false);
  };

  const copy = async (text: string) => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const revoke = (id: string) => {
    if (confirm("Revoke this key? This cannot be undone.")) {
      setKeys(prev => prev.filter(k => k.id !== id));
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">API Keys</h1>
          <p className="text-sm text-white/40 mt-1">Manage your API keys. Keys are shown once — store them safely.</p>
        </div>
        <button
          onClick={() => setCreating(true)}
          className="flex items-center gap-2 rounded-xl bg-brand-500 px-4 py-2 text-sm font-medium text-white hover:bg-brand-600 transition-colors"
        >
          <Plus className="h-4 w-4" />
          Create key
        </button>
      </div>

      {/* New key revealed */}
      {newKey && (
        <div className="rounded-xl border border-green-500/20 bg-green-500/5 p-4">
          <div className="flex items-start justify-between gap-4">
            <div>
              <p className="text-sm font-medium text-green-400 mb-1">New API key created</p>
              <p className="text-xs text-white/50 mb-3">Copy it now — it won&apos;t be shown again.</p>
              <code className="font-mono text-sm text-white/80 break-all">{newKey}</code>
            </div>
            <button onClick={() => copy(newKey)} className="shrink-0 flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-1.5 text-xs text-white/60 hover:text-white">
              {copied ? <Check className="h-3.5 w-3.5 text-green-400" /> : <Copy className="h-3.5 w-3.5" />}
              {copied ? "Copied!" : "Copy"}
            </button>
          </div>
          <button onClick={() => setNewKey(null)} className="mt-3 text-xs text-white/30 hover:text-white/50">
            I&apos;ve saved this key ✓
          </button>
        </div>
      )}

      {/* Create form */}
      {creating && !newKey && (
        <div className="rounded-xl border border-white/10 bg-surface-2 p-4">
          <h3 className="font-medium text-white mb-3">Create new key</h3>
          <div className="flex gap-3">
            <input
              autoFocus
              type="text"
              placeholder="Key name (e.g. Production)"
              value={newName}
              onChange={e => setNewName(e.target.value)}
              onKeyDown={e => e.key === "Enter" && createKey()}
              className="flex-1 rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder-white/30 outline-none focus:border-brand-500/50"
            />
            <button onClick={createKey} className="rounded-lg bg-brand-500 px-4 py-2 text-sm font-medium text-white hover:bg-brand-600">
              Create
            </button>
            <button onClick={() => setCreating(false)} className="rounded-lg border border-white/10 px-3 py-2 text-sm text-white/50 hover:text-white">
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Key list */}
      <div className="rounded-xl border border-white/5 bg-surface-1 divide-y divide-white/5">
        {keys.length === 0 ? (
          <div className="py-12 text-center text-white/30 text-sm">
            No API keys yet. Create one to get started.
          </div>
        ) : (
          keys.map(k => (
            <div key={k.id} className="flex items-center gap-4 px-5 py-4">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-medium text-white text-sm">{k.name}</span>
                  <span className="rounded-full bg-brand-500/10 text-brand-300 border border-brand-500/20 px-2 py-0.5 text-xs">
                    {k.plan}
                  </span>
                </div>
                <div className="font-mono text-xs text-white/40">{k.preview}</div>
              </div>
              <div className="text-xs text-white/30 hidden sm:block">
                <div>Created {k.created}</div>
                <div>Last used {k.lastUsed}</div>
              </div>
              <button
                onClick={() => copy(k.preview)}
                className="flex h-8 w-8 items-center justify-center rounded-lg border border-white/5 text-white/40 hover:text-white transition-colors"
                title="Copy preview"
              >
                <Copy className="h-3.5 w-3.5" />
              </button>
              <button
                onClick={() => revoke(k.id)}
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
