import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { Mail, MessageSquare, Building2, Shield } from "lucide-react";

export const metadata: Metadata = {
  title: "Contact Fetchium — Sales, Support, and Security",
  description:
    "Get in touch with Fetchium for enterprise sales, technical support, security reports, or general enquiries.",
};

const channels = [
  {
    icon: MessageSquare,
    title: "General Support",
    desc: "Questions about the API, features, or getting started.",
    action: "hello@fetchium.com",
    href: "mailto:hello@fetchium.com",
    color: "text-indigo-400",
    bg: "bg-indigo-500/10 border-indigo-500/20",
  },
  {
    icon: Building2,
    title: "Enterprise Sales",
    desc: "Custom pricing, dedicated infrastructure, SLA guarantees, and compliance.",
    action: "enterprise@fetchium.com",
    href: "mailto:enterprise@fetchium.com",
    color: "text-violet-400",
    bg: "bg-violet-500/10 border-violet-500/20",
  },
  {
    icon: Shield,
    title: "Security",
    desc: "Responsible disclosure of vulnerabilities or security concerns.",
    action: "security@fetchium.com",
    href: "mailto:security@fetchium.com",
    color: "text-emerald-400",
    bg: "bg-emerald-500/10 border-emerald-500/20",
  },
  {
    icon: Mail,
    title: "Press & Partnerships",
    desc: "Media enquiries, partnership proposals, and integration opportunities.",
    action: "press@fetchium.com",
    href: "mailto:press@fetchium.com",
    color: "text-blue-400",
    bg: "bg-blue-500/10 border-blue-500/20",
  },
];

export default function ContactPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />

      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-4xl">

          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400 transition-colors">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Contact</span>
          </nav>

          <div className="mb-12">
            <h1 className="text-3xl sm:text-4xl font-bold tracking-tight mb-4">
              Get in{" "}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-400 bg-clip-text text-transparent">
                touch
              </span>
            </h1>
            <p className="text-base text-slate-400 max-w-xl">
              We respond to all emails within 48 hours. For Enterprise enquiries, we aim to respond within 4 hours on business days.
            </p>
          </div>

          {/* Contact cards */}
          <div className="grid sm:grid-cols-2 gap-4 mb-12">
            {channels.map((c) => {
              const Icon = c.icon;
              return (
                <a
                  key={c.title}
                  href={c.href}
                  className={`group rounded-2xl border ${c.bg} p-6 transition-all hover:scale-[1.01] hover:shadow-[0_8px_32px_rgba(0,0,0,0.3)]`}
                >
                  <div className={`mb-3 flex h-10 w-10 items-center justify-center rounded-xl border bg-white/5 ${c.bg}`}>
                    <Icon className={`h-5 w-5 ${c.color}`} strokeWidth={1.75} />
                  </div>
                  <h2 className="text-base font-semibold text-slate-100 mb-1.5">{c.title}</h2>
                  <p className="text-[13px] text-slate-500 mb-3 leading-relaxed">{c.desc}</p>
                  <span className={`text-[13px] font-medium ${c.color} group-hover:underline underline-offset-2`}>
                    {c.action}
                  </span>
                </a>
              );
            })}
          </div>

          {/* Community */}
          <div className="rounded-2xl border border-white/6 bg-white/[0.02] p-6">
            <h2 className="text-lg font-bold mb-3">Community & Self-service</h2>
            <div className="grid sm:grid-cols-3 gap-3">
              {[
                { label: "Discord Community", desc: "Developer support + announcements", href: "https://discord.gg/fetchium" },
                { label: "GitHub Issues", desc: "Bug reports + feature requests", href: "https://github.com/zuhabul/Fetchium/issues" },
                { label: "Documentation", desc: "API reference, quickstart, guides", href: "https://docs.fetchium.com" },
              ].map((l) => (
                <Link key={l.label} href={l.href} target={l.href.startsWith("http") ? "_blank" : undefined} rel={l.href.startsWith("http") ? "noopener noreferrer" : undefined} className="rounded-xl border border-white/6 bg-white/2 p-4 hover:bg-white/5 transition-all group">
                  <div className="text-[13px] font-medium text-slate-300 group-hover:text-white">{l.label}</div>
                  <div className="text-[11px] text-slate-600 mt-0.5">{l.desc}</div>
                </Link>
              ))}
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
