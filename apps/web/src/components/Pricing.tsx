"use client";

import { useState } from "react";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import { Check, Zap, Star, ArrowRight, Building2, Rocket, TrendingUp } from "lucide-react";

/**
 * Pricing tiers — validated against market data (March 2026).
 *
 * Competitor benchmarks (verified):
 *  - Tavily:  $8.00/1K  (Project plan, $30/mo for 4K credits)
 *  - Exa:     $5.00/1K  (Neural search, pay-as-you-go)
 *  - SerpAPI: $15.00/1K (Developer plan, $75/mo for 5K searches)
 *  - Firecrawl Standard: $83/mo for 100K page scrapes ($0.83/1K)
 *  - Brave Search API: $5.00/1K
 *
 * Fetchium delivers a full pipeline (search + extract + rank + cite).
 * COGS with DDG HTML scraping + datacenter proxies: ~$0.10–0.15/1K.
 * Gross margin at $0.90/1K (Starter): ~83–89% — commercially viable.
 */

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
    rateLimit: "10 req / min",
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
    monthlyPrice: 9,
    annualPrice: 7,
    period: "/ month",
    description: "Indie developers and early-stage AI products shipping to users.",
    requestsLabel: "10,000 req / month",
    requestsMonthly: 10000,
    rateLimit: "60 req / min",
    perThousand: "$0.90",
    features: [
      "10,000 API requests per month",
      "Everything in Free",
      "YouTube intelligence API",
      "Social media research",
      "Real-time content monitoring",
      "Email support (72h SLA)",
      "Usage dashboard",
    ],
    cta: "Get Starter",
    href: (annual) =>
      `https://app.fetchium.com/register?plan=starter&billing=${annual ? "annual" : "monthly"}`,
    highlight: false,
    competitorNote: "vs Tavily: $30/mo for only 4,000 queries",
  },
  {
    id: "growth",
    name: "Growth",
    icon: TrendingUp,
    monthlyPrice: 29,
    annualPrice: 23,
    period: "/ month",
    description: "Production AI apps and teams needing reliable, scalable retrieval.",
    requestsLabel: "50,000 req / month",
    requestsMonthly: 50000,
    rateLimit: "300 req / min",
    perThousand: "$0.58",
    features: [
      "50,000 API requests per month",
      "Everything in Starter",
      "Deep research pipeline (AMRS)",
      "Cross-session learning (PIE)",
      "Adversarial content shield",
      "Priority email support (24h SLA)",
      "Advanced analytics + export",
      "Overage at $0.70 / 1K extra",
    ],
    cta: "Get Growth",
    href: (annual) =>
      `https://app.fetchium.com/register?plan=growth&billing=${annual ? "annual" : "monthly"}`,
    highlight: true,
    badge: "Most Popular",
    competitorNote: "vs Firecrawl Standard: $83/mo for 100K pages (extraction only)",
  },
  {
    id: "pro",
    name: "Pro",
    icon: Star,
    monthlyPrice: 79,
    annualPrice: 63,
    period: "/ month",
    description: "High-volume production systems and data-intensive AI pipelines.",
    requestsLabel: "200,000 req / month",
    requestsMonthly: 200000,
    rateLimit: "600 req / min",
    perThousand: "$0.40",
    features: [
      "200,000 API requests per month",
      "Everything in Growth",
      "Dedicated IP pool",
      "99.9% uptime SLA",
      "SOC 2 compliance docs",
      "Priority support (4h SLA)",
      "Slack integration",
      "Overage at $0.50 / 1K extra",
    ],
    cta: "Get Pro",
    href: (annual) =>
      `https://app.fetchium.com/register?plan=pro&billing=${annual ? "annual" : "monthly"}`,
    highlight: false,
    competitorNote: "vs SerpAPI: $500/mo for only 50K searches ($10/1K)",
  },
  {
    id: "enterprise",
    name: "Enterprise",
    icon: Building2,
    monthlyPrice: null,
    annualPrice: null,
    period: "",
    description: "Custom volume, dedicated infrastructure, compliance, and SLA guarantees.",
    requestsLabel: "500K+ req / month",
    requestsMonthly: null,
    rateLimit: "2,000+ req / min",
    perThousand: "Custom",
    features: [
      "Unlimited API requests",
      "Everything in Pro",
      "Dedicated infrastructure",
      "Custom SLA (up to 99.99%)",
      "SSO + team management",
      "On-prem / self-hosted option",
      "Dedicated Slack + Zoom support",
      "Security review + BAA available",
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

function PriceDisplay({ plan, annual }: { plan: Plan; annual: boolean }) {
  const price = annual ? plan.annualPrice : plan.monthlyPrice;

  if (price === null) {
    return (
      <div className="flex items-end gap-1">
        <span className="text-3xl font-bold tracking-tight text-slate-100">Custom</span>
      </div>
    );
  }

  return (
    <div className="flex items-end gap-1">
      {price > 0 && <span className="text-lg font-medium text-slate-500 mb-1">$</span>}
      <AnimatePresence mode="wait">
        <motion.span
          key={price}
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.2 }}
          className="text-3xl font-bold tracking-tight text-slate-100"
        >
          {price === 0 ? "Free" : price}
        </motion.span>
      </AnimatePresence>
      {price > 0 && (
        <span className="mb-1 text-sm text-slate-500">{plan.period}</span>
      )}
    </div>
  );
}

export default function Pricing() {
  const [annual, setAnnual] = useState(false);

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
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/25 bg-indigo-500/8 px-4 py-1.5 text-xs font-medium text-indigo-300">
            <Zap className="h-3.5 w-3.5" strokeWidth={2.5} />
            Transparent Pricing · No Hidden Fees
          </div>
          <h2 className="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold tracking-tight text-slate-100">
            The{" "}
            <span className="gradient-text">cheapest full-pipeline</span>
            <br className="hidden sm:block" /> search API on the market
          </h2>
          <p className="mt-4 sm:mt-5 mx-auto max-w-2xl text-sm sm:text-lg text-slate-500">
            Competitors charge $5–$15 per 1,000 queries for search only.
            Fetchium delivers search + extraction + citations + ranking from{" "}
            <span className="text-slate-300 font-semibold">$0.58 per 1,000</span> — on the Growth plan.
            No credit card required to start.
          </p>

          {/* Billing toggle */}
          <div className="mt-8 inline-flex items-center gap-2 rounded-xl border border-white/8 bg-white/3 p-1">
            <button
              onClick={() => setAnnual(false)}
              className={`rounded-lg px-4 py-2.5 text-sm font-medium transition-all duration-200 min-h-[44px] ${
                !annual ? "bg-white/10 text-white shadow-sm" : "text-slate-500 hover:text-slate-300"
              }`}
            >
              Monthly
            </button>
            <button
              onClick={() => setAnnual(true)}
              className={`flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-medium transition-all duration-200 min-h-[44px] ${
                annual ? "bg-white/10 text-white shadow-sm" : "text-slate-500 hover:text-slate-300"
              }`}
            >
              Annual
              <span className="rounded-full bg-emerald-500/15 px-1.5 py-0.5 text-[10px] font-bold text-emerald-400">
                −20%
              </span>
            </button>
          </div>
        </motion.div>

        {/* Plan cards — 5 column grid */}
        <motion.div
          className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5"
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
                      <div className="mt-1 text-[11px] text-slate-600">
                        Billed ${plan.annualPrice * 12}/yr — save $
                        {(plan.monthlyPrice - plan.annualPrice) * 12}/yr
                      </div>
                    )}

                    <p className="mt-2.5 text-[12px] leading-relaxed text-slate-500">
                      {plan.description}
                    </p>

                    {/* Request count */}
                    <div
                      className={`mt-3.5 rounded-xl border px-4 py-2.5 ${
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
                      <div className="mt-0.5 flex items-center justify-between">
                        <span className="text-[11px] text-slate-600">{plan.rateLimit}</span>
                        {plan.perThousand && plan.perThousand !== "Custom" && (
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
                    <div className="mb-4 rounded-lg border border-white/5 bg-white/[0.02] px-3 py-2">
                      <p className="text-[11px] text-slate-600 leading-snug">
                        <span className="text-emerald-400 font-semibold">vs competitor: </span>
                        {plan.competitorNote}
                      </p>
                    </div>
                  )}

                  {/* CTA */}
                  <Link
                    href={plan.href(annual)}
                    target={plan.id === "enterprise" ? undefined : "_blank"}
                    rel={plan.id === "enterprise" ? undefined : "noopener noreferrer"}
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

        {/* Market comparison proof */}
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
                  Price per 1,000 queries — full-pipeline comparison
                </h3>
                <p className="text-xs text-slate-500">
                  Verified pricing as of March 2026. Fetchium delivers search + extraction + citations; others charge for search alone.
                </p>
              </div>
              <Link
                href="/pricing"
                className="shrink-0 text-[12px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition-colors"
              >
                Full pricing details
                <ArrowRight className="h-3 w-3" />
              </Link>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
              {[
                { name: "Fetchium Growth", price: "$0.58", highlight: true, note: "search + extract + cite" },
                { name: "Fetchium Starter", price: "$0.90", highlight: true, note: "full pipeline" },
                { name: "Exa Neural", price: "$5.00", highlight: false, note: "search only" },
                { name: "Tavily Project", price: "$7.50", highlight: false, note: "search + snippets" },
                { name: "Brave Search", price: "$5.00", highlight: false, note: "search only" },
                { name: "SerpAPI Dev", price: "$15.00", highlight: false, note: "Google SERP only" },
              ].map((item) => (
                <div
                  key={item.name}
                  className={`rounded-xl p-3 text-center ${
                    item.highlight
                      ? "border border-emerald-500/25 bg-emerald-500/8"
                      : "border border-white/6 bg-white/2"
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
              Sources: Tavily pricing page (tavily.com/pricing), Exa pricing page (exa.ai/pricing), Brave Search API docs, SerpAPI pricing page — all verified March 2026.
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
