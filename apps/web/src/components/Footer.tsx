"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { Zap, Github, Twitter, Circle } from "lucide-react";

const nav: Record<string, { label: string; href: string }[]> = {
  Product: [
    { label: "Search API", href: "/product/search" },
    { label: "Extract API", href: "/product/extract" },
    { label: "Research API", href: "/product/research" },
    { label: "MCP Tools", href: "/product/mcp" },
    { label: "Pricing", href: "/pricing" },
    { label: "Changelog", href: "/changelog" },
    { label: "Roadmap", href: "/roadmap" },
    { label: "Status", href: "/status" },
  ],
  Developers: [
    { label: "Documentation", href: "https://docs.fetchium.com" },
    { label: "API Reference", href: "https://docs.fetchium.com/api/search" },
    { label: "Quickstart", href: "https://docs.fetchium.com/quickstart" },
    { label: "TypeScript SDK", href: "https://docs.fetchium.com/sdk/typescript" },
    { label: "Python SDK", href: "https://docs.fetchium.com/sdk/python" },
    { label: "Algorithm Docs", href: "https://docs.fetchium.com/algorithms" },
  ],
  Compare: [
    { label: "vs Tavily", href: "/compare/tavily" },
    { label: "vs Exa", href: "/compare/exa" },
    { label: "vs SerpAPI", href: "/compare/serpapi" },
    { label: "vs Firecrawl", href: "/compare/firecrawl" },
    { label: "vs Perplexity", href: "/compare/perplexity" },
  ],
  Company: [
    { label: "About", href: "/about" },
    { label: "Blog", href: "/blog" },
    { label: "Security", href: "/security" },
    { label: "Contact", href: "/contact" },
    { label: "GitHub", href: "https://github.com/zuhabul/Fetchium" },
  ],
  Legal: [
    { label: "Privacy Policy", href: "/privacy" },
    { label: "Terms of Service", href: "/terms" },
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
    <footer className="relative overflow-hidden border-t border-slate-900 pt-12 sm:pt-20 pb-8 sm:pb-10 px-4">
      {/* Background glow */}
      <div className="pointer-events-none absolute bottom-0 left-1/2 h-[300px] w-[800px] -translate-x-1/2 rounded-full bg-indigo-500/5 blur-[120px]" />

      <div className="relative mx-auto max-w-7xl">
        {/* Main grid */}
        <div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-7">
          {/* Brand column — spans 2 */}
          <motion.div
            className="lg:col-span-2 sm:col-span-2"
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

            <p className="mt-4 max-w-[240px] text-[13px] sm:text-[14px] leading-relaxed text-slate-300">
              The search API that thinks. 17 algorithms, 17+ federated
              backends, and docs at docs.fetchium.com.
            </p>

            {/* Status indicator */}
            <div className="mt-4 sm:mt-5 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/10 px-3 py-1.5 text-[12px] font-semibold text-indigo-300">
              <Circle
                className="h-2 w-2 fill-indigo-400 text-indigo-400"
                style={{ animation: "pulse 2s ease-in-out infinite" }}
              />
              Documentation on docs.fetchium.com
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
                    className="flex h-11 w-11 items-center justify-center rounded-xl border border-slate-700/60 bg-slate-900/40 text-slate-400 transition-all duration-200 hover:border-indigo-500/40 hover:bg-indigo-500/10 hover:text-indigo-300"
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
              <h4 className="mb-3 sm:mb-4 text-[13px] font-bold uppercase tracking-widest text-slate-300">
                {section}
              </h4>
              <ul className="space-y-2 sm:space-y-3">
                {links.map((link) => (
                  <li key={link.label}>
                    <Link
                      href={link.href}
                      target={link.href.startsWith("http") ? "_blank" : undefined}
                      rel={link.href.startsWith("http") ? "noopener noreferrer" : undefined}
                      className="text-[14px] text-slate-400 transition-colors duration-150 hover:text-slate-200 font-medium"
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
          className="mt-10 sm:mt-16 flex flex-col items-center justify-between gap-3 border-t border-slate-900 pt-6 sm:pt-8 sm:flex-row"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.3, duration: 0.5 }}
        >
          <p className="text-[13px] sm:text-[14px] text-slate-400">
            &copy; 2026 Fetchium. All rights reserved.
          </p>

          {/* Tech badges */}
          <div className="flex flex-wrap items-center gap-2 sm:gap-3">
            {["Rust", "1,100+ tests", "17 algos", "12 MCP tools"].map(
              (badge) => (
                <span
                  key={badge}
                  className="rounded-md border border-slate-700 bg-slate-900/60 px-2.5 py-1 text-[12px] sm:text-[13px] font-semibold text-slate-300"
                >
                  {badge}
                </span>
              )
            )}
          </div>

          <p className="text-[13px] sm:text-[14px] text-slate-400">
            Built with Rust + Next.js
          </p>
        </motion.div>
      </div>
    </footer>
  );
}
