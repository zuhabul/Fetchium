import Link from "next/link";
import { Check, ArrowUpRight } from "lucide-react";
import type { Metadata } from "next";

export const metadata: Metadata = { title: "Billing — HyperSearchX Dashboard" };

const plans = [
  { name: "Free", price: "$0", requests: "1,000 / month", current: true },
  { name: "Starter", price: "$19/mo", requests: "25,000 / month", current: false },
  { name: "Pro", price: "$79/mo", requests: "250,000 / month", current: false },
  { name: "Enterprise", price: "$299/mo", requests: "Unlimited", current: false },
];

export default function BillingPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Billing</h1>
        <p className="text-sm text-white/40 mt-1">Manage your subscription and payment.</p>
      </div>

      {/* Current plan */}
      <div className="rounded-xl border border-brand-500/20 bg-brand-500/5 p-5">
        <div className="flex items-start justify-between">
          <div>
            <span className="text-xs text-brand-400 font-medium uppercase tracking-wider">Current plan</span>
            <h2 className="text-xl font-bold text-white mt-1">Free</h2>
            <p className="text-sm text-white/40 mt-1">1,000 requests per month · 60 req/min</p>
          </div>
          <span className="rounded-full bg-green-500/10 text-green-400 border border-green-500/20 px-2.5 py-0.5 text-xs font-medium">
            Active
          </span>
        </div>
      </div>

      {/* Upgrade options */}
      <div>
        <h2 className="font-medium text-white mb-3">Upgrade your plan</h2>
        <div className="grid gap-3 sm:grid-cols-3">
          {plans.filter(p => !p.current).map(p => (
            <div key={p.name} className="rounded-xl border border-white/5 bg-surface-1 p-4">
              <div className="flex justify-between items-start mb-3">
                <div>
                  <div className="font-semibold text-white">{p.name}</div>
                  <div className="text-xs text-white/40 mt-0.5">{p.requests}</div>
                </div>
                <div className="text-sm font-bold text-white">{p.price}</div>
              </div>
              <Link
                href={`https://app.hypersearchx.com/checkout?plan=${p.name.toLowerCase()}`}
                className="flex items-center justify-center gap-1.5 w-full rounded-lg bg-brand-500 py-2 text-sm font-medium text-white hover:bg-brand-600 transition-colors"
              >
                Upgrade <ArrowUpRight className="h-3.5 w-3.5" />
              </Link>
            </div>
          ))}
        </div>
      </div>

      {/* Invoice history */}
      <div className="rounded-xl border border-white/5 bg-surface-1">
        <div className="border-b border-white/5 px-5 py-4">
          <h2 className="font-medium text-white">Invoice history</h2>
        </div>
        <div className="py-12 text-center text-sm text-white/30">
          No invoices yet — you&apos;re on the free plan.
        </div>
      </div>
    </div>
  );
}
