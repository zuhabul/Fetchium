import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { Shield, Lock, Server, Eye, CheckCircle } from "lucide-react";

export const metadata: Metadata = {
  title: "Fetchium Security — Privacy-First, Self-Hostable, Zero Telemetry",
  description:
    "Fetchium is built privacy-first: zero telemetry by default, full self-hosting, TLS everywhere, no query logging. Security practices, responsible disclosure, and compliance roadmap.",
};

const pillars = [
  {
    icon: Eye,
    title: "Zero Telemetry by Default",
    desc: "Your queries are processed in-memory and never logged, stored, or analyzed without your explicit consent. There is no usage data sent to third-party analytics or advertising systems. Ever.",
  },
  {
    icon: Server,
    title: "Full Self-Hosting",
    desc: "Every feature available in the hosted API is available in self-hosted mode. Run Fetchium on your own infrastructure — your data never leaves your network. Docker image available for all platforms.",
  },
  {
    icon: Lock,
    title: "TLS Everywhere",
    desc: "All API communication is encrypted in transit using TLS 1.3. API keys are hashed at rest using Argon2. No plaintext secrets are stored anywhere in the system.",
  },
  {
    icon: Shield,
    title: "API Key Security",
    desc: "Keys are scoped by capability (search-only, research, admin). Rotation is instant. Failed authentication is rate-limited with exponential backoff. Keys can be revoked in < 1 second.",
  },
];

const compliance = [
  { name: "GDPR (EU)", status: "Compliant", note: "Data residency in EU available. DPA available on request." },
  { name: "CCPA (California)", status: "Compliant", note: "No sale of personal data. Deletion requests honored within 30 days." },
  { name: "SOC 2 Type II", status: "Q3 2026", note: "Audit in progress. Controls documentation available to Enterprise customers." },
  { name: "ISO 27001", status: "Roadmap", note: "Planned for 2027 with SOC 2 as prerequisite." },
];

export default function SecurityPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-4xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Security</span>
          </nav>

          <div className="mb-12">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/25 bg-emerald-500/8 px-3 py-1.5 text-xs font-medium text-emerald-400">
              <Shield className="h-3.5 w-3.5" />
              Security & Privacy
            </div>
            <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-5">
              Built with{" "}
              <span className="bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent">
                privacy by default
              </span>
            </h1>
            <p className="text-base sm:text-lg text-slate-400 max-w-2xl leading-relaxed">
              Security and privacy are not features — they are constraints that shaped every architectural
              decision. Zero telemetry, full self-hosting, and TLS everywhere are defaults, not upsells.
            </p>
          </div>

          {/* Security pillars */}
          <div className="mb-12 grid sm:grid-cols-2 gap-4">
            {pillars.map((p) => {
              const Icon = p.icon;
              return (
                <div key={p.title} className="rounded-2xl border border-white/6 bg-white/[0.02] p-6">
                  <div className="mb-3 flex h-10 w-10 items-center justify-center rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                    <Icon className="h-5 w-5 text-emerald-400" strokeWidth={1.75} />
                  </div>
                  <h2 className="text-base font-semibold text-slate-100 mb-2">{p.title}</h2>
                  <p className="text-[13px] text-slate-500 leading-relaxed">{p.desc}</p>
                </div>
              );
            })}
          </div>

          {/* Data handling */}
          <section className="mb-12">
            <h2 className="text-xl font-bold mb-4">Data Handling</h2>
            <div className="space-y-3 text-[13px]">
              {[
                "API queries are processed in-memory. No query content is written to disk or database on the hosted API.",
                "API keys and user account data are stored in PostgreSQL with Argon2 hashing for secrets.",
                "Usage metrics (request counts, latency) are stored in aggregate — never linked to query content.",
                "The PIE cross-session learning feature stores patterns in a local SQLite file in self-hosted mode only.",
                "Logs retain IP addresses for 7 days for abuse prevention, then are deleted.",
                "No query content is shared with third-party services. Backend search queries to DuckDuckGo, Brave, etc. do not include your API key or user identity.",
              ].map((item) => (
                <div key={item} className="flex items-start gap-3">
                  <CheckCircle className="h-4 w-4 text-emerald-400 shrink-0 mt-0.5" strokeWidth={2} />
                  <span className="text-slate-400">{item}</span>
                </div>
              ))}
            </div>
          </section>

          {/* Compliance */}
          <section className="mb-12">
            <h2 className="text-xl font-bold mb-4">Compliance & Certifications</h2>
            <div className="space-y-3">
              {compliance.map((c) => (
                <div key={c.name} className="flex items-start gap-4 rounded-xl border border-white/6 bg-white/[0.02] p-4">
                  <div className={`shrink-0 rounded-full px-2.5 py-1 text-[11px] font-semibold ${
                    c.status === "Compliant" ? "bg-emerald-500/12 text-emerald-400" :
                    c.status.includes("2026") ? "bg-blue-500/12 text-blue-400" : "bg-slate-500/12 text-slate-500"
                  }`}>
                    {c.status}
                  </div>
                  <div>
                    <div className="text-sm font-semibold text-slate-200">{c.name}</div>
                    <div className="text-[12px] text-slate-500 mt-0.5">{c.note}</div>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* Responsible disclosure */}
          <section className="mb-12 rounded-2xl border border-amber-500/20 bg-amber-500/8 p-6">
            <h2 className="text-lg font-bold mb-2 text-amber-300">Responsible Disclosure</h2>
            <p className="text-[13px] text-slate-400 leading-relaxed mb-4">
              We welcome security researchers. If you discover a vulnerability in Fetchium, please report it
              privately. We commit to acknowledging reports within 48 hours and resolving critical issues within 7 days.
            </p>
            <div className="space-y-2 text-[13px]">
              <div><span className="text-slate-500">Email: </span><a href="mailto:security@fetchium.com" className="text-indigo-400 hover:text-indigo-300 transition-colors">security@fetchium.com</a></div>
              <div><span className="text-slate-500">PGP key: </span><span className="text-slate-400">Available on request</span></div>
              <div><span className="text-slate-500">Scope: </span><span className="text-slate-400">api.fetchium.com, app.fetchium.com, fetchium.com, and open-source crates</span></div>
              <div><span className="text-slate-500">Out of scope: </span><span className="text-slate-400">Social engineering, physical attacks, third-party services</span></div>
            </div>
          </section>

          <div className="text-center">
            <p className="text-sm text-slate-600 mb-4">Questions about security?</p>
            <Link
              href="/contact"
              className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-6 py-3 text-sm font-semibold text-white transition-all hover:shadow-[0_0_24px_rgba(99,102,241,0.4)]"
            >
              Contact Security Team
            </Link>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
