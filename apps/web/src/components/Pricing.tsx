"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { Check, Zap, Star, ArrowRight, Rocket, TrendingUp } from "lucide-react";

interface Plan {
  id: string;
  name: string;
  icon: React.ElementType;
  monthlyPrice: number | null;
  annualPrice: number | null;
  period: string;
  description: string;
  requestsLabel: string;
  requestsMonthly: number | null;
  rateLimit: string;
  perThousand: string | null;
  features: string[];
  cta: string;
  href: (annual: boolean) => string;
  highlight: boolean;
  badge?: string;
  competitorNote?: string;
}

const plans: Plan[] = [
  {
    id: "free",
    name: "Free",
    icon: Zap,
    monthlyPrice: 0,
    annualPrice: 0,
    period: "forever",
    description: "Explore the API, build a prototype, or run personal projects.",
    requestsLabel: "1,000 req / month",
    requestsMonthly: 1000,
    rateLimit: "60 req / min",
    perThousand: null,
    features: [
      "1,000 API requests per month",
      "All 11 search backends",
      "5-layer CEP content extraction",
      "HyperFusion 8-signal ranking",
      "Token budget management",
      "Evidence graphs + citations",
      "MCP protocol tools",
      "Community support (Discord)",
    ],
    cta: "Start for Free",
    href: () => "https://app.fetchium.com/register",
    highlight: false,
  },
  {
    id: "starter",
    name: "Starter",
    icon: Rocket,
    monthlyPrice: null,
    annualPrice: null,
    period: "",
    description: "First paid tier in the current API configuration.",
    requestsLabel: "25,000 req / month",
    requestsMonthly: 25000,
    rateLimit: "200 req / min",
    perThousand: null,
    features: [
      "25,000 API requests per month",
      "Everything in Free",
      "YouTube intelligence API",
      "Social media research",
      "Async jobs + usage tracking",
      "Admin key management",
      "Usage dashboard",
    ],
    cta: "Contact Sales",
    href: () => "/contact",
    highlight: false,
    competitorNote: "Current auth limits: 200 req/min and 25,000 req/month",
  },
  {
    id: "pro",
    name: "Pro",
    icon: TrendingUp,
    monthlyPrice: null,
    annualPrice: null,
    period: "",
    description: "Higher-volume production usage in the current API configuration.",
    requestsLabel: "250,000 req / month",
    requestsMonthly: 250000,
    rateLimit: "500 req / min",
    perThousand: null,
    features: [
      "250,000 API requests per month",
      "Everything in Starter",
      "Deep research pipeline (AMRS)",
      "Cross-session learning (PIE)",
      "Adversarial content shield",
      "Higher monthly quota",
      "Higher per-minute limits",
      "Best fit for production workloads",
    ],
    cta: "Contact Sales",
    href: () => "/contact",
    highlight: true,
    badge: "Most Popular",
    competitorNote: "Current auth limits: 500 req/min and 250,000 req/month",
  },
  {
    id: "enterprise",
    name: "Enterprise",
    icon: Star,
    monthlyPrice: null,
    annualPrice: null,
    period: "",
    description: "Custom volume and support for teams that need enterprise handling.",
    requestsLabel: "Unlimited / custom",
    requestsMonthly: null,
    rateLimit: "2,000 req / min",
    perThousand: null,
    features: [
      "Unlimited API requests",
      "Everything in Pro",
      "Custom commercial terms",
      "Priority onboarding",
      "Dedicated support channel",
      "Deployment guidance",
      "Security review on request",
    ],
    cta: "Contact Sales",
    href: () => "/contact",
    highlight: false,
    badge: "Custom",
  },
];

const containerVariants = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.07 } },
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

function PriceDisplay({ plan }: { plan: Plan }) {
  const price = plan.monthlyPrice;

  if (price === null) {
    return (
      <div className="flex items-end gap-1">
        <span className="text-3xl font-bold tracking-tight text-slate-100">
          {plan.id === "starter" || plan.id === "pro" ? "Contact" : "Custom"}
        </span>
      </div>
    );
  }

  return (
    <div className="flex items-end gap-1">
      {price > 0 && <span className="text-lg font-medium text-slate-500 mb-1">$</span>}
      <span className="text-3xl font-bold tracking-tight text-slate-100">
        {price === 0 ? "Free" : price}
      </span>
      {price > 0 && (
        <span className="mb-1 text-sm text-slate-500">{plan.period}</span>
      )}
    </div>
  );
}

export default function Pricing() {
  return (
    <section id="pricing" className="relative overflow-hidden py-16 sm:py-28 px-4">
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
          <div className="mb-5 inline-flex items-center gap-2 rounded-full border border-indigo-500/30 bg-indigo-500/10 px-4 py-2 text-sm font-semibold text-indigo-200">
            <Zap className="h-4 w-4" strokeWidth={2.5} />
            Current API Tiers
          </div>
          <h2 className="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-bold tracking-tight text-slate-100">
            Plans synced to the{" "}
            <span className="gradient-text">current auth configuration</span>
          </h2>
          <p className="mt-5 sm:mt-6 mx-auto max-w-2xl text-base sm:text-xl text-slate-400 leading-relaxed">
            Free tier limits are sourced from the API auth layer. Paid plan names and
            request ceilings below reflect the current codebase; contact sales for
            commercial pricing details.
          </p>
        </motion.div>

        {/* Plan cards */}
        <motion.div
          className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4"
          variants={containerVariants}
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-60px" }}
        >
          {plans.map((plan) => {
            const Icon = plan.icon;
            return (
              <motion.div
                key={plan.id}
                variants={cardVariants}
                className={`group relative flex flex-col overflow-hidden rounded-2xl transition-all duration-300 ${
                  plan.highlight
                    ? "border border-indigo-500/40 bg-gradient-to-b from-[rgba(99,102,241,0.08)] to-[rgba(13,17,23,0.9)] shadow-[0_0_40px_rgba(99,102,241,0.15),0_24px_48px_rgba(0,0,0,0.4)] lg:scale-105"
                    : "glass-card"
                }`}
              >
                {/* Top accent line */}
                <div
                  className={`absolute -top-px left-0 right-0 h-px ${
                    plan.highlight
                      ? "bg-gradient-to-r from-transparent via-indigo-500 to-transparent"
                      : "bg-gradient-to-r from-transparent via-white/10 to-transparent"
                  }`}
                />

                {/* Popular badge */}
                {plan.highlight && (
                  <div className="flex items-center justify-center py-2.5 text-[11px] font-bold uppercase tracking-widest text-indigo-300">
                    <Zap className="mr-1.5 h-3 w-3" strokeWidth={3} />
                    Most Popular
                  </div>
                )}

                <div className={`flex flex-1 flex-col p-5 ${plan.highlight ? "" : "pt-5"}`}>
                  {/* Plan header */}
                  <div className="mb-5">
                    <div className="mb-3 flex items-center gap-2.5">
                      <div
                        className={`flex h-8 w-8 items-center justify-center rounded-lg ${
                          plan.highlight
                            ? "bg-indigo-500/20 shadow-[0_0_12px_rgba(99,102,241,0.3)]"
                            : "bg-slate-800/60 border border-slate-700/50"
                        }`}
                      >
                        <Icon className={`h-4 w-4 ${plan.highlight ? "text-indigo-300" : "text-slate-400"}`} strokeWidth={1.75} />
                      </div>
                      <span
                        className={`text-[13px] font-semibold ${
                          plan.highlight ? "text-indigo-300" : "text-slate-300"
                        }`}
                      >
                        {plan.name}
                      </span>
                    </div>

                    <PriceDisplay plan={plan} />

                    <p className="mt-2.5 text-[12px] leading-relaxed text-slate-400">
                      {plan.description}
                    </p>

                    {/* Request count */}
                    <div
                      className={`mt-3.5 rounded-xl border px-4 py-2.5 ${
                        plan.highlight
                          ? "border-indigo-500/25 bg-indigo-500/8"
                          : "border-slate-700/50 bg-slate-900/50"
                      }`}
                    >
                      <div
                        className={`text-[13px] font-semibold ${
                          plan.highlight ? "text-indigo-200" : "text-slate-200"
                        }`}
                      >
                        {plan.requestsLabel}
                      </div>
                      <div className="mt-0.5 flex items-center justify-between">
                        <span className="text-[11px] text-slate-400">{plan.rateLimit}</span>
                        {plan.perThousand && (
                          <span className="text-[11px] font-semibold text-emerald-400">
                            {plan.perThousand}/1K
                          </span>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Feature list */}
                  <ul className="mb-6 flex-1 space-y-2">
                    {plan.features.map((f) => (
                      <li key={f} className="flex items-start gap-2">
                        <div
                          className={`mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full ${
                            plan.highlight ? "bg-indigo-500/20" : "bg-emerald-500/12"
                          }`}
                        >
                          <Check
                            className={`h-2.5 w-2.5 ${
                              plan.highlight ? "text-indigo-300" : "text-emerald-400"
                            }`}
                            strokeWidth={3}
                          />
                        </div>
                        <span className="text-[12px] text-slate-400">{f}</span>
                      </li>
                    ))}
                  </ul>

                  {/* Competitor note */}
                  {plan.competitorNote && (
                    <div className="mb-4 rounded-lg border border-slate-700/40 bg-slate-900/50 px-3 py-2">
                      <p className="text-[11px] text-slate-400 leading-snug">
                        <span className="text-emerald-400 font-semibold">source: </span>
                        {plan.competitorNote}
                      </p>
                    </div>
                  )}

                  {/* CTA */}
                  <Link
                    href={plan.href(false)}
                    className={`group/btn flex items-center justify-center gap-2 rounded-xl py-3 text-[13px] font-semibold transition-all duration-200 min-h-[44px] ${
                      plan.highlight
                        ? "bg-gradient-to-r from-indigo-500 to-violet-600 text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]"
                        : "border border-slate-600/60 bg-slate-800/50 text-slate-200 hover:bg-slate-700/60 hover:text-white"
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

        {/* Plan note */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.5 }}
          className="mt-10 sm:mt-12"
        >
          <div className="rounded-2xl border border-indigo-500/12 bg-gradient-to-r from-indigo-500/5 to-violet-500/5 p-5 sm:p-7">
            <div className="flex flex-col sm:flex-row items-start sm:items-center gap-4 sm:gap-6 mb-5">
              <div className="flex-1">
                <h3 className="text-sm sm:text-base font-semibold text-slate-200 mb-1">
                  What is verified here
                </h3>
                <p className="text-xs text-slate-500">
                  This section is intentionally conservative. Request quotas, rate limits, and
                  free-tier availability are taken from the current API auth code. If you need
                  a signed commercial quote, use the contact flow.
                </p>
              </div>
              <Link
                href="/contact"
                className="shrink-0 text-[12px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition-colors"
              >
                Contact sales
                <ArrowRight className="h-3 w-3" />
              </Link>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
              {[
                { name: "Free", price: "1,000/mo", highlight: true, note: "60 req/min" },
                { name: "Starter", price: "25,000/mo", highlight: false, note: "200 req/min" },
                { name: "Pro", price: "250,000/mo", highlight: false, note: "500 req/min" },
                { name: "Enterprise", price: "Unlimited", highlight: false, note: "2,000 req/min" },
              ].map((item) => (
                <div
                  key={item.name}
                  className={`rounded-xl p-3 text-center ${
                    item.highlight
                      ? "border border-emerald-500/25 bg-emerald-500/8"
                      : "border border-slate-700/50 bg-slate-900/50"
                  }`}
                >
                  <div
                    className={`text-lg font-bold ${
                      item.highlight ? "text-emerald-400" : "text-slate-400"
                    }`}
                  >
                    {item.price}
                  </div>
                  <div className={`text-[11px] font-medium mt-0.5 ${item.highlight ? "text-emerald-300" : "text-slate-500"}`}>
                    {item.name}
                  </div>
                  <div className="text-[10px] text-slate-600 mt-0.5">{item.note}</div>
                </div>
              ))}
            </div>

            <p className="mt-4 text-[11px] text-slate-600 text-center">
              Source: current API auth configuration in the Fetchium codebase.
            </p>
          </div>
        </motion.div>

        {/* FAQs */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.3, duration: 0.5 }}
          className="mt-8 text-center"
        >
          <p className="text-[13px] text-slate-600">
            Questions about pricing?{" "}
            <Link href="/contact" className="text-indigo-400 hover:text-indigo-300 underline underline-offset-2">
              Talk to us
            </Link>
            {" "}or see the{" "}
            <Link href="/pricing" className="text-indigo-400 hover:text-indigo-300 underline underline-offset-2">
              full pricing FAQ
            </Link>
            . All plans include all 17 algorithms, all backends, and evidence graphs — only scale differs.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
