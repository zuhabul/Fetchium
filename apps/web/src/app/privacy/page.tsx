import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Privacy Policy — Fetchium",
  description: "Fetchium Privacy Policy: what data we collect, how we use it, and your rights.",
};

export default function PrivacyPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />
      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-3xl">
          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Privacy Policy</span>
          </nav>

          <h1 className="text-3xl sm:text-4xl font-bold mb-2">Privacy Policy</h1>
          <p className="text-sm text-slate-500 mb-8">Last updated: March 3, 2026</p>

          <div className="prose prose-invert prose-sm max-w-none space-y-6 text-slate-400 leading-relaxed">

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">1. Overview</h2>
              <p>Fetchium (&quot;we,&quot; &quot;us,&quot; or &quot;our&quot;) is committed to protecting your privacy. This policy describes what information we collect from users of fetchium.com and the Fetchium API, how we use it, and your rights.</p>
              <p className="mt-3">The core principle: <strong className="text-slate-300">your search queries are yours</strong>. We do not log, store, sell, or analyze query content by default.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">2. What We Collect</h2>
              <div className="space-y-3">
                <div>
                  <h3 className="text-sm font-semibold text-slate-300">Account data</h3>
                  <p>Email address and hashed password when you create an account. We use this only for authentication and communication.</p>
                </div>
                <div>
                  <h3 className="text-sm font-semibold text-slate-300">Usage metrics</h3>
                  <p>Aggregate request counts, latency percentiles, and error rates — stored per API key. Query content is never stored with these metrics.</p>
                </div>
                <div>
                  <h3 className="text-sm font-semibold text-slate-300">Server logs</h3>
                  <p>IP addresses and request timestamps are retained for 7 days for abuse prevention, then permanently deleted. Query content is never written to logs.</p>
                </div>
                <div>
                  <h3 className="text-sm font-semibold text-slate-300">Payment data</h3>
                  <p>Billing information is handled entirely by Stripe. We never store credit card numbers or CVVs.</p>
                </div>
              </div>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">3. What We Do Not Collect</h2>
              <ul className="list-disc pl-5 space-y-1">
                <li>The content of your search queries</li>
                <li>The content of pages extracted via the API</li>
                <li>Behavioral analytics or tracking pixels</li>
                <li>Data from third-party advertising networks</li>
                <li>Keystroke or mouse movement data</li>
              </ul>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">4. How We Use Your Data</h2>
              <ul className="list-disc pl-5 space-y-1">
                <li>Providing and improving the Fetchium API</li>
                <li>Sending transactional emails (account confirmation, billing receipts)</li>
                <li>Preventing abuse and enforcing rate limits</li>
                <li>Providing customer support when you contact us</li>
              </ul>
              <p className="mt-3">We do not sell your data. We do not share your data with advertising networks.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">5. Third-Party Services</h2>
              <p>We use the following third-party services. Each has its own privacy policy:</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li><strong className="text-slate-300">Stripe</strong> — payment processing</li>
                <li><strong className="text-slate-300">Resend</strong> — transactional email</li>
                <li><strong className="text-slate-300">Cloudflare</strong> — CDN and DDoS protection (IP addresses may be logged)</li>
              </ul>
              <p className="mt-3">When you use the Fetchium API, your queries are dispatched to search backends (DuckDuckGo, Brave Search, etc.). These backends do not receive your API key or account identity.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">6. Your Rights</h2>
              <p>Under GDPR and CCPA, you have the right to:</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li>Access the personal data we hold about you</li>
                <li>Request deletion of your account and associated data</li>
                <li>Object to processing for marketing purposes</li>
                <li>Data portability (export your usage history)</li>
              </ul>
              <p className="mt-3">To exercise these rights, email <a href="mailto:privacy@fetchium.com" className="text-indigo-400 hover:text-indigo-300">privacy@fetchium.com</a>. We will respond within 30 days.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">7. Self-Hosting</h2>
              <p>If you self-host Fetchium, this Privacy Policy does not apply — you control all data processing. See the <Link href="https://docs.fetchium.com/self-hosting/docker" className="text-indigo-400 hover:text-indigo-300">self-hosting documentation</Link>.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">8. Changes to This Policy</h2>
              <p>We will notify registered users by email before making material changes to this policy. The updated policy will be posted to this page with a revised date.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">9. Contact</h2>
              <p>For privacy questions: <a href="mailto:privacy@fetchium.com" className="text-indigo-400 hover:text-indigo-300">privacy@fetchium.com</a></p>
            </section>
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}
