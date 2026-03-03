import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Cookie Policy — Fetchium",
  description: "Fetchium Cookie Policy: what cookies we use and how to manage them.",
};

export default function CookiesPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />
      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-3xl">
          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Cookie Policy</span>
          </nav>

          <h1 className="text-3xl sm:text-4xl font-bold mb-2">Cookie Policy</h1>
          <p className="text-sm text-slate-500 mb-8">Last updated: March 3, 2026</p>

          <div className="prose prose-invert prose-sm max-w-none space-y-6 text-slate-400 leading-relaxed">

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">What cookies we use</h2>
              <p>Fetchium uses a minimal set of cookies — only what is necessary for the site to function and for your session to be secure. We do not use advertising cookies or third-party tracking pixels.</p>

              <div className="mt-4 space-y-4">
                <div className="rounded-xl border border-white/6 bg-white/[0.02] p-4">
                  <h3 className="text-sm font-semibold text-emerald-400 mb-1">Essential cookies</h3>
                  <p className="text-[13px]">Session authentication (HttpOnly, Secure, SameSite=Strict). Required for dashboard login. Duration: session. No personal data beyond your session ID.</p>
                </div>
                <div className="rounded-xl border border-white/6 bg-white/[0.02] p-4">
                  <h3 className="text-sm font-semibold text-blue-400 mb-1">Preference cookies</h3>
                  <p className="text-[13px]">Theme preference (dark/light), billing toggle state. Stored in localStorage, not transmitted to our servers. Duration: indefinite (until cleared).</p>
                </div>
                <div className="rounded-xl border border-white/6 bg-white/[0.02] p-4">
                  <h3 className="text-sm font-semibold text-slate-500 mb-1">Analytics cookies</h3>
                  <p className="text-[13px]">We do not use analytics cookies or tracking pixels. We use server-side aggregate metrics only.</p>
                </div>
              </div>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">Managing cookies</h2>
              <p>You can delete cookies at any time through your browser settings. Deleting the session cookie will log you out of the dashboard. Preference cookies do not affect functionality if deleted.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">Contact</h2>
              <p>For cookie-related questions: <a href="mailto:privacy@fetchium.com" className="text-indigo-400 hover:text-indigo-300">privacy@fetchium.com</a></p>
            </section>
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}
