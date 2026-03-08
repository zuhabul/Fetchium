"use client";

import { useEffect, useState } from "react";
import { signOut } from "next-auth/react";
import { DEFAULT_API_BASE } from "@/lib/client-config";

type Status = {
  tone: "success" | "error" | "info";
  message: string;
};

type SessionState = {
  plan?: string;
  keyId?: string;
  apiKeyPreview?: string;
};

export default function SettingsPage() {
  const [session, setSession] = useState<SessionState | null>(null);
  const [status, setStatus] = useState<Status | null>(null);
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    void (async () => {
      const res = await fetch("/api/auth/session", { cache: "no-store" });
      if (!res.ok) return;
      const body = (await res.json()) as {
        plan?: string;
        keyId?: string;
        apiKeyPreview?: string;
      };
      setSession(body);
    })();
  }, []);

  async function testHealth() {
    setTesting(true);
    setStatus(null);
    try {
      const q = encodeURIComponent(DEFAULT_API_BASE);
      const [healthRes, usageRes] = await Promise.all([
        fetch(`/api/health?apiBase=${q}`, { cache: "no-store" }),
        fetch("/api/usage", { cache: "no-store" }),
      ]);

      const healthBody = (await healthRes.json()) as { status?: string; title?: string; message?: string };
      const usageBody = (await usageRes.json()) as {
        error?: string;
        title?: string;
        message?: string;
        usage?: { plan?: string };
      };

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
        message: `Connection verified. API is healthy and the session is valid${usageBody.usage?.plan ? ` (${usageBody.usage.plan} plan)` : ""}.`,
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

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-sm text-white/40 mt-1">
          Review the authenticated dashboard session and verify production connectivity.
        </p>
      </div>

      <div className="rounded-xl border border-white/5 bg-surface-1 divide-y divide-white/5">
        <div className="px-5 py-4">
          <label className="text-sm font-medium text-white/60 block mb-2">Production API Base</label>
          <div className="w-full rounded-lg border border-white/5 bg-white/5 px-3 py-2 text-sm text-white">
            {DEFAULT_API_BASE}
          </div>
          <p className="mt-2 text-xs text-white/30">
            Hosted dashboard traffic is locked to the production API and cannot be changed from the browser.
          </p>
        </div>
        <div className="px-5 py-4">
          <label className="text-sm font-medium text-white/60 block mb-2">Authenticated Key</label>
          <div className="grid gap-3 md:grid-cols-3">
            <InfoTile label="Plan" value={session?.plan || "Loading"} />
            <InfoTile label="Key ID" value={session?.keyId || "Loading"} />
            <InfoTile label="Key Preview" value={session?.apiKeyPreview || "Loading"} />
          </div>
          <p className="text-xs text-white/30 mt-3">
            The full API key is not re-exposed in the dashboard after sign-in. Session auth now gates all dashboard actions.
          </p>
        </div>
      </div>

      <div className="flex gap-3">
        <button
          onClick={() => void testHealth()}
          disabled={testing}
          className="rounded-lg border border-white/10 px-4 py-2 text-sm text-white/70 hover:text-white disabled:opacity-60"
        >
          {testing ? "Testing..." : "Verify connection"}
        </button>
        <button
          onClick={() => void signOut({ callbackUrl: "/login" })}
          className="rounded-lg bg-brand-500 px-4 py-2 text-sm text-white hover:bg-brand-600"
        >
          Sign out
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

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-white/5 bg-white/5 px-3 py-3">
      <div className="text-xs text-white/35">{label}</div>
      <div className="mt-1 break-all text-sm font-medium text-white">{value}</div>
    </div>
  );
}
