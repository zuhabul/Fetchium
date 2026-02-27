"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { Zap, Github, Twitter, Circle } from "lucide-react";

const nav: Record<string, { label: string; href: string }[]> = {
  Product: [
    { label: "Features", href: "#features" },
    { label: "Pricing", href: "#pricing" },
    { label: "Changelog", href: "/changelog" },
    { label: "Roadmap", href: "/roadmap" },
    { label: "Status", href: "/status" },
  ],
  Developers: [
    { label: "Documentation", href: "/docs" },
    { label: "API Reference", href: "/docs/api" },
    { label: "TypeScript SDK", href: "https://www.npmjs.com/package/@fetchium/sdk" },
    { label: "Python SDK", href: "https://pypi.org/project/fetchium" },
    { label: "Algorithm Docs", href: "/docs/algorithms" },
  ],
  Company: [
    { label: "About", href: "/about" },
    { label: "Blog", href: "/blog" },
    { label: "Discord", href: "https://discord.gg/fetchium" },
    { label: "GitHub", href: "https://github.com/zuhabul/Fetchium" },
    { label: "Contact", href: "mailto:hello@fetchium.com" },
  ],
  Legal: [
    { label: "Privacy Policy", href: "/privacy" },
    { label: "Terms of Service", href: "/terms" },
    { label: "Security", href: "/security" },
    { label: "Cookie Policy", href: "/cookies" },
  ],
};

const socialLinks = [
  {
    label: "GitHub",
    href: "https://github.com/zuhabul/Fetchium",
    icon: Github,
  },
  {
    label: "Twitter / X",
    href: "https://twitter.com/fetchium",
    icon: Twitter,
  },
];

export default function Footer() {
  return (
    <footer className="relative overflow-hidden border-t border-white/5 pt-12 sm:pt-20 pb-8 sm:pb-10 px-4">
      {/* Background glow */}
      <div className="pointer-events-none absolute bottom-0 left-1/2 h-[300px] w-[800px] -translate-x-1/2 rounded-full bg-indigo-500/5 blur-[120px]" />

      <div className="relative mx-auto max-w-7xl">
        {/* Main grid */}
        <div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-6">
          {/* Brand column — spans 2 */}
          <motion.div
            className="lg:col-span-2"
            initial={{ opacity: 0, y: 16 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            {/* Logo */}
            <Link
              href="/"
              className="group inline-flex items-center gap-2.5 font-semibold text-white"
            >
              <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 shadow-[0_0_16px_rgba(99,102,241,0.35)] transition-shadow group-hover:shadow-[0_0_24px_rgba(99,102,241,0.55)]">
                <Zap className="h-4 w-4 text-white" strokeWidth={2.5} />
              </div>
              <span className="text-[15px] tracking-tight text-slate-100">
                Fetchi
                <span className="text-indigo-400">um</span>
              </span>
            </Link>

            <p className="mt-4 max-w-[220px] text-[12px] sm:text-[13px] leading-relaxed text-slate-500">
              The search API that thinks. 17 novel algorithms. 11 federated
              backends. AI-ready context in under 200ms.
            </p>

            {/* Status indicator */}
            <div className="mt-4 sm:mt-5 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/8 px-3 py-1.5 text-[11px] font-medium text-emerald-400">
              <Circle
                className="h-2 w-2 fill-emerald-400 text-emerald-400"
                style={{ animation: "pulse 2s ease-in-out infinite" }}
              />
              All systems operational
            </div>

            {/* Social links */}
            <div className="mt-4 sm:mt-6 flex gap-2">
              {socialLinks.map((s) => {
                const Icon = s.icon;
                return (
                  <Link
                    key={s.label}
                    href={s.href}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label={s.label}
                    className="flex h-11 w-11 items-center justify-center rounded-xl border border-white/8 bg-white/3 text-slate-500 transition-all duration-200 hover:border-indigo-500/30 hover:bg-indigo-500/10 hover:text-indigo-300"
                  >
                    <Icon className="h-4 w-4" />
                  </Link>
                );
              })}
            </div>
          </motion.div>

          {/* Navigation columns */}
          {Object.entries(nav).map(([section, links], colIdx) => (
            <motion.div
              key={section}
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.08 * colIdx, duration: 0.5 }}
            >
              <h4 className="mb-3 sm:mb-4 text-[11px] sm:text-[12px] font-semibold uppercase tracking-widest text-slate-500">
                {section}
              </h4>
              <ul className="space-y-2 sm:space-y-2.5">
                {links.map((link) => (
                  <li key={link.label}>
                    <Link
                      href={link.href}
                      target={link.href.startsWith("http") ? "_blank" : undefined}
                      rel={link.href.startsWith("http") ? "noopener noreferrer" : undefined}
                      className="text-[12px] sm:text-[13px] text-slate-600 transition-colors duration-150 hover:text-slate-200"
                    >
                      {link.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </motion.div>
          ))}
        </div>

        {/* Bottom bar */}
        <motion.div
          className="mt-10 sm:mt-16 flex flex-col items-center justify-between gap-3 border-t border-white/5 pt-6 sm:pt-8 sm:flex-row"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.3, duration: 0.5 }}
        >
          <p className="text-[11px] sm:text-[12px] text-slate-700">
            &copy; 2026 Fetchium. All rights reserved.
          </p>

          {/* Tech badges */}
          <div className="flex flex-wrap items-center gap-2 sm:gap-3">
            {["Rust", "930+ tests", "17 algos", "< 200ms"].map(
              (badge) => (
                <span
                  key={badge}
                  className="rounded-md border border-white/6 bg-white/2 px-2 py-1 text-[10px] sm:text-[11px] font-medium text-slate-700"
                >
                  {badge}
                </span>
              )
            )}
          </div>

          <p className="text-[11px] sm:text-[12px] text-slate-700">
            Built with Rust + Next.js
          </p>
        </motion.div>
      </div>
    </footer>
  );
}
