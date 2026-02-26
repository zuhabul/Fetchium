"use client";

import { useState } from "react";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import { Check, Zap, Star, ArrowRight, Building2 } from "lucide-react";

interface Plan {
  name: string;
  icon: React.ElementType;
  monthlyPrice: number | null;
  annualPrice: number | null;
  period: string;
  description: string;
  requestsLabel: string;
  rateLimit: string;
  features: string[];
  cta: string;
  href: (annual: boolean) => string;
  highlight: boolean;
  badge?: string;
  badgeColor?: string;
}

const plans: Plan[] = [
  {
    name: "Free",
    icon: Zap,
    monthlyPrice: 0,
    annualPrice: 0,
    period: "forever",
    description: "Evaluation, personal projects, and exploration.",
    requestsLabel: "1,000 req / month",
    rateLimit: "60 req / min",
    features: [
      "All 11 search backends",
      "5-layer CEP extraction",
      "HyperFusion ranking (8-signal)",
      "Token budget management",
      "Evidence graphs + citations",
      "Community support (Discord)",
    ],
    cta: "Start Free",
    href: () => "https://app.hypersearchx.zuhabul.com/register",
    highlight: false,
  },
  {
    name: "Starter",
    icon: Star,
    monthlyPrice: 19,
    annualPrice: 15,
    period: "/ month",
    description: "Indie developers shipping AI-powered products.",
    requestsLabel: "25,000 req / month",
    rateLimit: "200 req / min",
    features: [
      "Everything in Free",
      "25,000 API requests / month",
      "YouTube intelligence",
      "Social media research",
      "Real-time monitoring",
      "Email support (48h SLA)",
    ],
    cta: "Start Starter",
    href: (annual) =>
      `https://app.hypersearchx.zuhabul.com/register?plan=starter&billing=${annual ? "annual" : "monthly"}`,
    highlight: false,
  },
  {
    name: "Pro",
    icon: Zap,
    monthlyPrice: 79,
    annualPrice: 63,
    period: "/ month",
    description: "Teams and production AI applications.",
    requestsLabel: "250,000 req / month",
    rateLimit: "500 req / min",
    features: [
      "Everything in Starter",
      "250,000 API requests / month",
      "PIE cross-session learning",
      "AMRS deep research pipeline",
      "Priority support (4h SLA)",
      "Usage analytics dashboard",
    ],
    cta: "Start Pro",
    href: (annual) =>
      `https://app.hypersearchx.zuhabul.com/register?plan=pro&billing=${annual ? "annual" : "monthly"}`,
    highlight: true,
    badge: "Most Popular",
  },
  {
    name: "Enterprise",
    icon: Building2,
    monthlyPrice: null,
    annualPrice: null,
    period: "",
    description: "Dedicated infrastructure, custom SLAs, full control.",
    requestsLabel: "Unlimited",
    rateLimit: "2,000+ req / min",
    features: [
      "Everything in Pro",
      "Unlimited API requests",
      "Dedicated infrastructure",
      "SLA guarantees (99.9% uptime)",
      "SSO + team management",
      "Dedicated Slack channel",
    ],
    cta: "Contact Sales",
    href: () => "mailto:enterprise@hypersearchx.zuhabul.com",
    highlight: false,
    badge: "Custom",
  },
];

const containerVariants = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.08 } },
};

const cardVariants = {
  hidden: { opacity: 0, y: 28, scale: 0.97 },
  visible: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { duration: 0.55, ease: [0.22, 1, 0.36, 1] as [number, number, number, number] },
  },
};

function PriceDisplay({
  plan,
  annual,
}: {
  plan: Plan;
  annual: boolean;
}) {
  const price = annual ? plan.annualPrice : plan.monthlyPrice;

  if (price === null) {
    return (
      <div className="flex items-end gap-1">
        <span className="text-4xl font-bold tracking-tight text-slate-100">
          Custom
        </span>
      </div>
    );
  }

  return (
    <div className="flex items-end gap-1">
      <span className="text-lg font-medium text-slate-500">$</span>
      <AnimatePresence mode="wait">
        <motion.span
          key={price}
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.2 }}
          className="text-4xl font-bold tracking-tight text-slate-100"
        >
          {price}
        </motion.span>
      </AnimatePresence>
      {price > 0 && (
        <span className="mb-1.5 text-sm text-slate-500">{plan.period}</span>
      )}
      {price === 0 && (
        <span className="mb-1.5 text-sm text-slate-500">{plan.period}</span>
      )}
    </div>
  );
}

export default function Pricing() {
  const [annual, setAnnual] = useState(false);

  return (
    <section id="pricing" className="relative overflow-hidden py-16 sm:py-28 px-4">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/2 top-1/2 h-[700px] w-[900px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-indigo-500/4 blur-[150px]" />
      </div>

      <div className="relative mx-auto max-w-7xl">
        {/* Header */}
        <motion.div
          className="mb-10 sm:mb-14 text-center"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-4 py-1.5 text-xs font-medium text-indigo-300">
            <Zap className="h-3.5 w-3.5" strokeWidth={2.5} />
            Transparent Pricing
          </div>
          <h2 className="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold tracking-tight text-slate-100">
            Simple, transparent{" "}
            <span className="gradient-text">pricing</span>
          </h2>
          <p className="mt-4 sm:mt-5 mx-auto max-w-xl text-sm sm:text-lg text-slate-500">
            Start free. Upgrade when you need more. No surprise charges, no
            vendor lock-in — self-host for free anytime.
          </p>

          {/* Billing toggle */}
          <div className="mt-8 inline-flex items-center gap-2 sm:gap-3 rounded-xl border border-white/8 bg-white/3 p-1">
            <button
              onClick={() => setAnnual(false)}
              className={`rounded-lg px-4 py-2.5 text-sm font-medium transition-all duration-200 min-h-[44px] ${
                !annual
                  ? "bg-white/10 text-white shadow-sm"
                  : "text-slate-500 hover:text-slate-300"
              }`}
            >
              Monthly
            </button>
            <button
              onClick={() => setAnnual(true)}
              className={`flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-medium transition-all duration-200 min-h-[44px] ${
                annual
                  ? "bg-white/10 text-white shadow-sm"
                  : "text-slate-500 hover:text-slate-300"
              }`}
            >
              Annual
              <span className="rounded-full bg-emerald-500/15 px-1.5 py-0.5 text-[10px] font-bold text-emerald-400">
                −20%
              </span>
            </button>
          </div>
        </motion.div>

        {/* Plan cards */}
        <motion.div
          className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4"
          variants={containerVariants}
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-60px" }}
        >
          {plans.map((plan) => {
            const Icon = plan.icon;
            return (
              <motion.div
                key={plan.name}
                variants={cardVariants}
                className={`group relative flex flex-col overflow-hidden rounded-2xl transition-all duration-300 ${
                  plan.highlight
                    ? "border border-indigo-500/40 bg-gradient-to-b from-[rgba(99,102,241,0.08)] to-[rgba(13,17,23,0.9)] shadow-[0_0_40px_rgba(99,102,241,0.15),0_24px_48px_rgba(0,0,0,0.4)]"
                    : "glass-card"
                }`}
              >
                {/* Popular badge */}
                {plan.badge && (
                  <div
                    className={`absolute -top-px left-0 right-0 h-px ${
                      plan.highlight
                        ? "bg-gradient-to-r from-transparent via-indigo-500 to-transparent"
                        : "bg-gradient-to-r from-transparent via-white/10 to-transparent"
                    }`}
                  />
                )}
                {plan.highlight && (
                  <div className="flex items-center justify-center py-2.5 text-[11px] font-bold uppercase tracking-widest text-indigo-300">
                    <Zap className="mr-1.5 h-3 w-3" strokeWidth={3} />
                    Most Popular
                  </div>
                )}

                <div className={`flex flex-1 flex-col p-6 ${plan.highlight ? "" : "pt-6"}`}>
                  {/* Plan header */}
                  <div className="mb-6">
                    <div className="mb-3 flex items-center gap-2.5">
                      <div
                        className={`flex h-8 w-8 items-center justify-center rounded-lg ${
                          plan.highlight
                            ? "bg-indigo-500/20 shadow-[0_0_12px_rgba(99,102,241,0.3)]"
                            : "bg-white/6 border border-white/8"
                        }`}
                      >
                        <Icon
                          className={`h-4 w-4 ${plan.highlight ? "text-indigo-300" : "text-slate-400"}`}
                          strokeWidth={1.75}
                        />
                      </div>
                      <span
                        className={`text-[13px] font-semibold ${
                          plan.highlight ? "text-indigo-300" : "text-slate-300"
                        }`}
                      >
                        {plan.name}
                      </span>
                    </div>

                    <PriceDisplay plan={plan} annual={annual} />

                    {annual && plan.monthlyPrice !== null && plan.monthlyPrice > 0 && plan.annualPrice !== null && (
                      <div className="mt-1.5 text-[11px] text-slate-600">
                        Billed ${plan.annualPrice * 12}/year — save $
                        {(plan.monthlyPrice - plan.annualPrice) * 12}/yr
                      </div>
                    )}

                    <p className="mt-3 text-[13px] leading-relaxed text-slate-500">
                      {plan.description}
                    </p>

                    {/* Request count pill */}
                    <div
                      className={`mt-4 rounded-xl border px-4 py-2.5 text-center ${
                        plan.highlight
                          ? "border-indigo-500/25 bg-indigo-500/8"
                          : "border-white/6 bg-white/3"
                      }`}
                    >
                      <div
                        className={`text-[13px] font-semibold ${
                          plan.highlight ? "text-indigo-200" : "text-slate-200"
                        }`}
                      >
                        {plan.requestsLabel}
                      </div>
                      <div className="mt-0.5 text-[11px] text-slate-600">
                        {plan.rateLimit}
                      </div>
                    </div>
                  </div>

                  {/* Feature list */}
                  <ul className="mb-7 flex-1 space-y-2.5">
                    {plan.features.map((f) => (
                      <li key={f} className="flex items-start gap-2.5">
                        <div
                          className={`mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full ${
                            plan.highlight
                              ? "bg-indigo-500/20"
                              : "bg-emerald-500/12"
                          }`}
                        >
                          <Check
                            className={`h-2.5 w-2.5 ${
                              plan.highlight ? "text-indigo-300" : "text-emerald-400"
                            }`}
                            strokeWidth={3}
                          />
                        </div>
                        <span className="text-[13px] text-slate-400">{f}</span>
                      </li>
                    ))}
                  </ul>

                  {/* CTA */}
                  <Link
                    href={plan.href(annual)}
                    target={plan.name === "Enterprise" ? undefined : "_blank"}
                    rel="noopener noreferrer"
                    className={`group/btn flex items-center justify-center gap-2 rounded-xl py-3 text-[13px] font-semibold transition-all duration-200 min-h-[44px] ${
                      plan.highlight
                        ? "bg-gradient-to-r from-indigo-500 to-violet-600 text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]"
                        : "border border-white/10 bg-white/4 text-slate-300 hover:bg-white/8 hover:text-white"
                    }`}
                  >
                    {plan.cta}
                    <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover/btn:translate-x-0.5" />
                  </Link>
                </div>
              </motion.div>
            );
          })}
        </motion.div>

        {/* Self-hosting note */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.5 }}
          className="mt-10 flex flex-col items-center gap-3 text-center"
        >
          <p className="text-[13px] text-slate-600">
            Or{" "}
            <Link
              href="https://github.com/hypersearchx/hypersearchx"
              target="_blank"
              rel="noopener noreferrer"
              className="text-indigo-400 underline underline-offset-2 transition-colors hover:text-indigo-300"
            >
              self-host for free
            </Link>{" "}
            with no request limits, no API keys required, full source code.
            MIT OR Apache-2.0 licensed.
          </p>

          {/* Value comparison */}
          <div className="mt-2 inline-block rounded-xl border border-white/6 bg-white/2 px-5 py-3">
            <span className="text-[13px] text-slate-400">
              <span className="font-semibold text-slate-100">
                vs Firecrawl Pro ($599/mo):
              </span>{" "}
              Get more for{" "}
              <span className="font-bold text-emerald-400">87% less cost</span>{" "}
              with features Firecrawl doesn&apos;t offer at any price.
            </span>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
