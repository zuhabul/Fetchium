import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Terms of Service — Fetchium",
  description: "Fetchium Terms of Service — acceptable use, API limits, intellectual property, and liability.",
};

export default function TermsPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />
      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-3xl">
          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Terms of Service</span>
          </nav>

          <h1 className="text-3xl sm:text-4xl font-bold mb-2">Terms of Service</h1>
          <p className="text-sm text-slate-500 mb-8">Last updated: March 3, 2026</p>

          <div className="prose prose-invert prose-sm max-w-none space-y-6 text-slate-400 leading-relaxed">

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">1. Acceptance of Terms</h2>
              <p>By accessing or using the Fetchium API, dashboard, or website, you agree to these Terms of Service. If you do not agree, do not use Fetchium.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">2. Acceptable Use</h2>
              <p>You may use Fetchium to build applications, conduct research, and retrieve information from publicly available sources. You may not:</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li>Use Fetchium to collect personal data without appropriate legal basis</li>
                <li>Attempt to circumvent rate limits or bypass authentication</li>
                <li>Resell API access without a partnership agreement</li>
                <li>Use the API for activities that violate applicable laws</li>
                <li>Interfere with the operation of the Fetchium infrastructure</li>
                <li>Use Fetchium to generate or distribute spam, malware, or harmful content</li>
              </ul>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">3. API Keys and Accounts</h2>
              <p>API keys are your responsibility. Keep them secure. You are responsible for all API calls made using your keys. If you believe a key has been compromised, rotate it immediately in the dashboard.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">4. Billing and Refunds</h2>
              <p>Paid plans are billed monthly or annually in advance. Usage is capped at your plan limit; you will not be charged overage unless you opt into the overage feature. Refunds are available within 14 days of an initial subscription purchase. No refunds for usage already consumed.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">5. Intellectual Property</h2>
              <p>Content retrieved via the Fetchium API is subject to the copyright of the original authors and sites. Fetchium does not grant any rights to the content retrieved. You are responsible for complying with the terms of service of the underlying sources.</p>
              <p className="mt-3">The Fetchium name, logo, and brand are trademarks of Fetchium. The Fetchium source code is licensed under the terms stated in the GitHub repository.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">6. Availability and SLA</h2>
              <p>Free and Starter plans have no uptime SLA. Growth plan targets 99.9% uptime. Pro and Enterprise plans have contractual SLAs. We will use commercially reasonable efforts to maintain availability for all plans.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">7. Limitation of Liability</h2>
              <p>TO THE MAXIMUM EXTENT PERMITTED BY LAW, FETCHIUM&apos;S TOTAL LIABILITY FOR ANY CLAIM SHALL NOT EXCEED THE AMOUNT YOU PAID IN THE 3 MONTHS PRECEDING THE CLAIM. WE ARE NOT LIABLE FOR INDIRECT, CONSEQUENTIAL, OR PUNITIVE DAMAGES.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">8. Termination</h2>
              <p>You may cancel your account at any time. We may terminate accounts for material violation of these terms. Upon termination, your API keys are revoked immediately.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">9. Governing Law</h2>
              <p>These terms are governed by the laws of England and Wales. Disputes shall be resolved in the courts of England and Wales.</p>
            </section>

            <section>
              <h2 className="text-lg font-semibold text-slate-200 mb-3">10. Contact</h2>
              <p>For legal matters: <a href="mailto:legal@fetchium.com" className="text-indigo-400 hover:text-indigo-300">legal@fetchium.com</a></p>
            </section>
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}
