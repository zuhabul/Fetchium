"use client";

import { motion } from "framer-motion";
import Link from "next/link";
import { ArrowRight, ExternalLink } from "lucide-react";

export default function CTABand() {
  return (
    <section className="relative overflow-hidden py-16 sm:py-24 px-4">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-0 bg-gradient-to-b from-[#06070d] via-[#0a0c1e] to-[#06070d]" />
        <div className="absolute left-1/2 top-1/2 h-[600px] w-[900px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-indigo-600/8 blur-[120px]" />
        <div className="absolute left-1/4 top-1/2 h-[300px] w-[400px] -translate-y-1/2 rounded-full bg-violet-600/6 blur-[100px]" />
        {/* Grid */}
        <div
          className="absolute inset-0 opacity-40"
          style={{
            backgroundImage:
              "linear-gradient(rgba(99,102,241,0.04) 1px,transparent 1px),linear-gradient(90deg,rgba(99,102,241,0.04) 1px,transparent 1px)",
            backgroundSize: "72px 72px",
          }}
        />
      </div>

      <div className="relative mx-auto max-w-4xl text-center">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
        >
          {/* Badge */}
          <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-sm font-semibold text-emerald-300">
            <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
            Free forever · No credit card required
          </div>

          <h2 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight text-slate-100 mb-6">
            Start building with{" "}
            <span className="gradient-text-purple">Fetchium</span>
            {" "}today
          </h2>

          <p className="text-lg sm:text-xl text-slate-300 max-w-2xl mx-auto mb-10 leading-relaxed">
            Join the open beta. 1,000 free API requests per month, all features included.
            Upgrade when you need more with higher-volume Starter, Pro, and Enterprise tiers.
          </p>

          {/* CTAs */}
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-10">
            <Link
              href="https://app.fetchium.com/register"
              target="_blank"
              rel="noopener noreferrer"
              className="group inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-8 py-4 text-base font-bold text-white shadow-[0_0_40px_rgba(99,102,241,0.4)] hover:shadow-[0_0_60px_rgba(99,102,241,0.6)] transition-all duration-300 min-h-[56px]"
            >
              Get API Key — Free
              <ExternalLink className="h-5 w-5" />
            </Link>
            <Link
              href="https://docs.fetchium.com/quickstart"
              className="inline-flex items-center gap-2 rounded-xl border border-slate-700/60 bg-slate-800/40 px-8 py-4 text-base font-bold text-slate-200 hover:bg-slate-700/50 hover:text-white hover:border-slate-600/60 transition-all duration-200 min-h-[56px]"
            >
              Read the Quickstart
              <ArrowRight className="h-5 w-5" />
            </Link>
            <Link
              href="/contact"
              className="inline-flex items-center gap-2 rounded-xl px-5 py-4 text-base font-semibold text-slate-300 hover:text-slate-100 transition-colors min-h-[56px]"
            >
              Talk to Sales →
            </Link>
          </div>

          {/* Social proof stats */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 sm:gap-6 max-w-2xl mx-auto">
            {[
              { value: "1,000", label: "free req/mo", sub: "no expiry" },
              { value: "25K", label: "Starter quota", sub: "req / month" },
              { value: "17", label: "algorithms", sub: "all included" },
              { value: "12", label: "MCP tools", sub: "included" },
            ].map((stat) => (
              <div
                key={stat.label}
                className="rounded-xl border border-slate-800 bg-slate-900/60 p-4 text-center"
              >
                <div className="text-2xl sm:text-3xl font-bold gradient-text">{stat.value}</div>
                <div className="text-[14px] font-semibold text-slate-300 mt-1">{stat.label}</div>
                <div className="text-[12px] text-slate-400 mt-0.5">{stat.sub}</div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  );
}
