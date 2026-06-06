import Link from "next/link";

export default function RegisterPage() {
  return (
    <main className="min-h-screen bg-[linear-gradient(180deg,#020617,#0f172a)] px-6 py-16 text-white">
      <div className="mx-auto max-w-3xl rounded-[28px] border border-white/10 bg-white/[0.03] p-8 shadow-[0_24px_80px_rgba(2,6,23,0.45)]">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-sky-300">
          Hosted Access
        </p>
        <h1 className="mt-4 text-3xl font-semibold tracking-tight">
          Account creation is handled during Fetchium onboarding.
        </h1>
        <p className="mt-4 text-sm leading-7 text-slate-300">
          The hosted dashboard does not offer anonymous self-signup yet. Access is provisioned with
          a real Fetchium API key, then dashboard sessions are created through secure sign-in on the
          login page.
        </p>

        <div className="mt-8 flex flex-wrap gap-3">
          <Link
            href="/login"
            className="inline-flex items-center rounded-xl bg-sky-500 px-5 py-3 text-sm font-semibold text-slate-950 transition hover:bg-sky-400"
          >
            Go to Login
          </Link>
          <Link
            href="mailto:founders@fetchium.com?subject=Fetchium%20account%20access"
            className="inline-flex items-center rounded-xl border border-white/10 px-5 py-3 text-sm font-semibold text-white/80 transition hover:text-white"
          >
            Request Access
          </Link>
        </div>
      </div>
    </main>
  );
}
