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
          <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-emerald-500/25 bg-emerald-500/8 px-4 py-1.5 text-xs font-medium text-emerald-400">
            <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
            Free forever · No credit card required
          </div>

          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight text-slate-100 mb-5">
            Start building with{" "}
            <span className="gradient-text-purple">Fetchium</span>
            {" "}today
          </h2>

          <p className="text-base sm:text-lg text-slate-500 max-w-2xl mx-auto mb-8 leading-relaxed">
            Join the open beta. 1,000 free API requests per month, all features included.
            Upgrade when you need more — from{" "}
            <span className="text-slate-300 font-semibold">$9/month</span>{" "}for 10,000 requests.
          </p>

          {/* CTAs */}
          <div className="flex flex-col sm:flex-row items-center justify-center gap-3 mb-10">
            <Link
              href="https://app.fetchium.com/register"
              target="_blank"
              rel="noopener noreferrer"
              className="group inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-8 py-4 text-sm font-semibold text-white shadow-[0_0_40px_rgba(99,102,241,0.4)] hover:shadow-[0_0_60px_rgba(99,102,241,0.6)] transition-all duration-300 min-h-[52px]"
            >
              Get API Key — Free
              <ExternalLink className="h-4 w-4" />
            </Link>
            <Link
              href="/docs/quickstart"
              className="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-white/3 px-8 py-4 text-sm font-semibold text-slate-300 hover:bg-white/8 hover:text-white hover:border-white/15 transition-all duration-200 min-h-[52px]"
            >
              Read the Quickstart
              <ArrowRight className="h-4 w-4" />
            </Link>
            <Link
              href="/contact"
              className="inline-flex items-center gap-2 rounded-xl px-5 py-4 text-sm font-medium text-slate-500 hover:text-slate-300 transition-colors min-h-[52px]"
            >
              Talk to Sales →
            </Link>
          </div>

          {/* Social proof stats */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 sm:gap-6 max-w-2xl mx-auto">
            {[
              { value: "1,000", label: "free req/mo", sub: "no expiry" },
              { value: "$9", label: "entry plan", sub: "10K req/mo" },
              { value: "17", label: "algorithms", sub: "all included" },
              { value: "~500ms", label: "P50 latency", sub: "search mode" },
            ].map((stat) => (
              <div
                key={stat.label}
                className="rounded-xl border border-white/5 bg-white/[0.02] p-4 text-center"
              >
                <div className="text-xl sm:text-2xl font-bold gradient-text">{stat.value}</div>
                <div className="text-[12px] font-medium text-slate-400 mt-0.5">{stat.label}</div>
                <div className="text-[10px] text-slate-700 mt-0.5">{stat.sub}</div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  );
}
