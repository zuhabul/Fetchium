"use client";

import { motion } from "framer-motion";
import { Zap, Shield, Cpu, Clock, FlaskConical, Lock } from "lucide-react";

const items = [
  { icon: Zap,         label: "17 novel algorithms",    sub: "not available anywhere else" },
  { icon: Clock,       label: "12 MCP tools",           sub: "search, fetch, research, YouTube, and social workflows" },
  { icon: FlaskConical,label: "11+ search backends",     sub: "federated in a single call" },
  { icon: Shield,      label: "563+ unit tests",         sub: "production-grade reliability" },
  { icon: Cpu,         label: "Built in Rust",           sub: "zero GC pauses, safe concurrency" },
  { icon: Lock,        label: "Zero telemetry",          sub: "your queries stay private" },
];

export default function TrustBar() {
  return (
    <section className="relative border-y border-slate-900 bg-gradient-to-r from-[#06070d] via-[#0a0c18] to-[#06070d] py-8 sm:py-10 overflow-hidden">
      {/* Subtle glow */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/2 top-0 h-px w-full -translate-x-1/2 bg-gradient-to-r from-transparent via-indigo-500/20 to-transparent" />
        <div className="absolute left-1/2 bottom-0 h-px w-full -translate-x-1/2 bg-gradient-to-r from-transparent via-indigo-500/10 to-transparent" />
      </div>

      <div className="relative mx-auto max-w-7xl px-4 sm:px-6">
        {/* Eyebrow */}
        <motion.p
          className="text-center text-[14px] font-bold uppercase tracking-widest text-slate-400 mb-6 sm:mb-8"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
        >
          Why developers choose Fetchium
        </motion.p>

        {/* Items grid */}
        <motion.div
          className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3 sm:gap-4"
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-40px" }}
          variants={{ hidden: {}, visible: { transition: { staggerChildren: 0.07 } } }}
        >
          {items.map((item) => {
            const Icon = item.icon;
            return (
              <motion.div
                key={item.label}
                variants={{
                  hidden: { opacity: 0, y: 16 },
                  visible: { opacity: 1, y: 0, transition: { duration: 0.45, ease: [0.22, 1, 0.36, 1] } },
                }}
                className="flex flex-col items-center gap-2 rounded-xl border border-slate-800 bg-slate-900/50 px-3 py-4 text-center hover:border-indigo-500/30 hover:bg-indigo-500/6 transition-all duration-200"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-indigo-500/12 border border-indigo-500/20">
                  <Icon className="h-5 w-5 text-indigo-400" strokeWidth={1.75} />
                </div>
                <div>
                  <div className="text-[15px] sm:text-[16px] font-bold text-slate-100">{item.label}</div>
                  <div className="mt-1.5 text-[13px] sm:text-[14px] text-slate-400 leading-snug">{item.sub}</div>
                </div>
              </motion.div>
            );
          })}
        </motion.div>

        {/* Bottom CTA strip */}
        <motion.div
          className="mt-6 sm:mt-8 flex flex-col sm:flex-row items-center justify-center gap-3 sm:gap-6 text-center"
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.5 }}
        >
          <p className="text-[16px] sm:text-[17px] text-slate-200 max-w-lg leading-relaxed">
            A search API with{" "}
            <span className="text-slate-100 font-semibold">federated backends</span>,{" "}
            <span className="text-slate-100 font-semibold">neural ranking</span>, and{" "}
            <span className="text-slate-100 font-semibold">cross-session learning</span>{" "}
            built into a single retrieval pipeline.
          </p>
          <a
            href="#compare"
            className="shrink-0 inline-flex items-center gap-1.5 rounded-lg border border-indigo-500/30 bg-indigo-500/10 px-4 py-2.5 text-[14px] font-semibold text-indigo-200 hover:bg-indigo-500/18 hover:text-indigo-100 transition-all"
          >
            See comparison
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </a>
        </motion.div>
      </div>
    </section>
  );
}
