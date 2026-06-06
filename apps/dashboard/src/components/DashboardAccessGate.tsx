"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { usePathname } from "next/navigation";
import { ShieldCheck, Settings, KeyRound, ArrowRight } from "lucide-react";
import {
  loadDashboardConfig,
  normalize_api_base,
  normalize_api_key,
  validate_api_base,
  validate_api_key,
} from "@/lib/client-config";

type GateState = "loading" | "ready" | "setup_required";

export default function DashboardAccessGate({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [state, setState] = useState<GateState>("loading");
  const [reason, setReason] = useState<string | null>(null);

  useEffect(() => {
    const cfg = loadDashboardConfig();
    const apiKey = normalize_api_key(cfg.apiKey);
    const apiBase = normalize_api_base(cfg.apiBaseUrl);
    const keyError = validate_api_key(apiKey);
    const baseError = validate_api_base(apiBase);

    if (pathname === "/dashboard/settings") {
      setState("ready");
      return;
    }

    if (keyError || baseError) {
      setReason(keyError || baseError || "Complete setup to access the dashboard.");
      setState("setup_required");
      return;
    }

    void (async () => {
      try {
        const res = await fetch("/api/usage", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ apiKey, apiBase }),
        });

        if (!res.ok) {
          const body = (await res.json()) as { message?: string; title?: string };
          setReason(body.title || body.message || "Configure a valid API key to continue.");
          setState("setup_required");
          return;
        }

        setState("ready");
      } catch {
        setReason("We could not verify your API key. Update your settings to continue.");
        setState("setup_required");
      }
    })();
  }, [pathname]);

  if (state === "loading") {
    return (
      <div className="flex h-full min-h-[60vh] items-center justify-center">
        <div className="rounded-2xl border border-white/10 bg-white/[0.03] px-6 py-5 text-sm text-white/60">
          Verifying dashboard access...
        </div>
      </div>
    );
  }

  if (state === "setup_required") {
    return (
      <div className="mx-auto flex min-h-[70vh] w-full max-w-4xl items-center justify-center">
        <div className="w-full rounded-3xl border border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.96),rgba(2,6,23,0.98))] p-8 shadow-[0_30px_90px_rgba(0,0,0,0.45)]">
          <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-brand-500/20 bg-brand-500/10 px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.18em] text-brand-200">
            <ShieldCheck className="h-3.5 w-3.5" />
            Account Required
          </div>

          <h1 className="text-3xl font-bold tracking-tight text-white">
            Create your Fetchium account and connect a valid API key first.
          </h1>
          <p className="mt-4 max-w-2xl text-sm leading-7 text-white/55">
            Hosted dashboard actions are intentionally blocked until a real account has been created
            and a working `fetchium_` API key has been verified. This prevents anonymous or
            misconfigured access to production workflows.
          </p>

          {reason && (
            <div className="mt-6 rounded-2xl border border-amber-500/20 bg-amber-500/8 px-4 py-3 text-sm text-amber-100">
              {reason}
            </div>
          )}

          <div className="mt-8 grid gap-4 md:grid-cols-3">
            <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-4">
              <KeyRound className="mb-3 h-5 w-5 text-brand-300" />
              <h2 className="text-sm font-semibold text-white">1. Create account</h2>
              <p className="mt-2 text-sm text-white/45">
                Register for a Fetchium account and generate your API key from the main app.
              </p>
            </div>
            <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-4">
              <Settings className="mb-3 h-5 w-5 text-brand-300" />
              <h2 className="text-sm font-semibold text-white">2. Open Settings</h2>
              <p className="mt-2 text-sm text-white/45">
                Save the production API base and your personal `fetchium_` key in dashboard settings.
              </p>
            </div>
            <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-4">
              <ShieldCheck className="mb-3 h-5 w-5 text-brand-300" />
              <h2 className="text-sm font-semibold text-white">3. Verify access</h2>
              <p className="mt-2 text-sm text-white/45">
                The dashboard unlocks only after the API key and production connection validate.
              </p>
            </div>
          </div>

          <div className="mt-8 flex flex-col gap-3 sm:flex-row">
            <Link
              href="https://app.fetchium.com/register"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-2 rounded-xl bg-brand-500 px-5 py-3 text-sm font-semibold text-white hover:bg-brand-600 transition-colors"
            >
              Create Account
              <ArrowRight className="h-4 w-4" />
            </Link>
            <Link
              href="/dashboard/settings"
              className="inline-flex items-center justify-center gap-2 rounded-xl border border-white/10 px-5 py-3 text-sm font-semibold text-white/75 hover:text-white transition-colors"
            >
              Open Settings
            </Link>
            <Link
              href="https://docs.fetchium.com/quickstart"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-2 rounded-xl border border-white/10 px-5 py-3 text-sm font-semibold text-white/55 hover:text-white/80 transition-colors"
            >
              Quickstart
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
