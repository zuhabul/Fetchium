"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { FormEvent, useMemo, useState } from "react";
import { signIn } from "next-auth/react";
import { ArrowRight, KeyRound, ShieldCheck } from "lucide-react";

export default function LoginForm({ callbackUrl }: { callbackUrl: string }) {
  const router = useRouter();
  const [apiKey, setApiKey] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const apiKeyError = useMemo(() => {
    const value = apiKey.trim();
    if (!value) return null;
    if (!value.startsWith("fetchium_")) return "API key must start with fetchium_.";
    if (value.length < 16) return "API key looks too short.";
    return null;
  }, [apiKey]);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setError(null);

    const validation = await fetch("/api/auth/validate-key", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        apiKey: apiKey.trim(),
      }),
    });

    const validationBody = (await validation.json()) as { message?: string };
    if (!validation.ok) {
      setSubmitting(false);
      setError(validationBody.message || "Sign-in failed. The API key was rejected.");
      return;
    }

    const result = await signIn("credentials", {
      apiKey: apiKey.trim(),
      redirect: false,
      callbackUrl,
    });

    setSubmitting(false);

    if (!result || result.error) {
      setError(
        "The API key was accepted by the API, but dashboard session creation failed. Retry once. If it persists, this is a dashboard auth bug rather than an invalid key.",
      );
      return;
    }

    router.push(result.url || callbackUrl);
    router.refresh();
  }

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,rgba(14,165,233,0.18),transparent_35%),linear-gradient(180deg,#020617,#0f172a)] px-4 py-8 sm:px-6 sm:py-10 lg:px-8 lg:py-12">
      <div className="mx-auto grid min-h-[calc(100vh-4rem)] max-w-6xl items-start gap-8 lg:min-h-[calc(100vh-6rem)] lg:grid-cols-[minmax(0,1.05fr)_minmax(360px,0.92fr)] lg:items-center lg:gap-10">
        <section className="max-w-2xl pt-2 sm:pt-4 lg:pt-0">
          <div className="inline-flex items-center gap-2 rounded-full border border-sky-400/20 bg-sky-400/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-sky-200 sm:text-xs">
            <ShieldCheck className="h-3.5 w-3.5" />
            Hosted Dashboard Access
          </div>
          <h1 className="mt-5 text-3xl font-semibold tracking-tight text-white sm:mt-6 sm:text-4xl lg:text-5xl">
            Sign in with your Fetchium API key.
          </h1>
          <p className="mt-4 max-w-xl text-sm leading-7 text-slate-300 sm:mt-5">
            The hosted dashboard now uses secure session auth. Your API key is validated
            server-side against the production API, then kept behind an authenticated cookie-backed
            session instead of browser-local configuration.
          </p>

          <div className="mt-6 grid gap-3 sm:mt-8 sm:grid-cols-2 xl:grid-cols-3">
            <FeatureCard
              title="Real validation"
              body="Login succeeds only when the API key is accepted by the live Fetchium API."
            />
            <FeatureCard
              title="Protected routes"
              body="Dashboard and proxy actions require an authenticated session before they run."
            />
            <FeatureCard
              title="Production API only"
              body="Hosted traffic is locked to ***REMOVED*** with no local override."
            />
          </div>
        </section>

        <section className="rounded-[28px] border border-white/10 bg-slate-950/75 p-5 shadow-[0_32px_120px_rgba(2,6,23,0.6)] backdrop-blur sm:p-6 lg:p-8">
          <div className="mb-6 flex items-center gap-3">
            <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-sky-500/15 text-sky-300">
              <KeyRound className="h-5 w-5" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">Dashboard Sign In</h2>
              <p className="text-sm text-slate-400">Use a provisioned `fetchium_` API key.</p>
            </div>
          </div>

          <form className="space-y-5" onSubmit={(event) => void onSubmit(event)}>
            <div>
              <label htmlFor="apiKey" className="mb-2 block text-sm font-medium text-slate-200">
                API key
              </label>
              <input
                id="apiKey"
                type="password"
                value={apiKey}
                onChange={(event) => setApiKey(event.target.value)}
                placeholder="fetchium_..."
                spellCheck={false}
                autoCapitalize="off"
                autoCorrect="off"
                className="w-full rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-white outline-none transition focus:border-sky-400/40"
              />
              {apiKeyError && <p className="mt-2 text-xs text-red-300">{apiKeyError}</p>}
            </div>

            {error && (
              <div className="rounded-2xl border border-red-400/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">
                {error}
              </div>
            )}

            <button
              type="submit"
              disabled={submitting || !!apiKeyError || !apiKey.trim()}
              className="inline-flex w-full items-center justify-center gap-2 rounded-2xl bg-sky-500 px-5 py-3 text-sm font-semibold text-slate-950 transition hover:bg-sky-400 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {submitting ? "Signing in..." : "Sign in"}
              {!submitting && <ArrowRight className="h-4 w-4" />}
            </button>
          </form>

          <div className="mt-6 rounded-2xl border border-white/8 bg-white/[0.03] px-4 py-4 text-sm text-slate-300">
            <p className="font-medium text-white">Need account provisioning?</p>
            <p className="mt-1 text-slate-400">
              If you do not have a Fetchium API key yet, contact the Fetchium team to enable hosted
              access for your workspace.
            </p>
            <div className="mt-4 flex flex-col gap-3 sm:flex-row sm:flex-wrap">
              <Link
                href="mailto:founders@fetchium.com?subject=Fetchium%20dashboard%20access"
                className="inline-flex items-center justify-center rounded-xl border border-white/10 px-4 py-2 text-sm font-medium text-white/80 transition hover:text-white"
              >
                Contact Access Team
              </Link>
              <Link
                href="https://docs.fetchium.com/quickstart"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center justify-center rounded-xl border border-white/10 px-4 py-2 text-sm font-medium text-white/60 transition hover:text-white"
              >
                Quickstart
              </Link>
            </div>
          </div>
        </section>
      </div>
    </main>
  );
}

function FeatureCard({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-3xl border border-white/8 bg-white/[0.04] p-4 sm:p-5">
      <h2 className="text-sm font-semibold text-white">{title}</h2>
      <p className="mt-2 text-sm leading-6 text-slate-400">{body}</p>
    </div>
  );
}
