"use client";

import Link from "next/link";
import { ArrowUpRight } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  DASHBOARD_ALERT,
  DASHBOARD_CARD_PADDED,
  DASHBOARD_PAGE_HEADER,
  DASHBOARD_PAGE_LEAD,
  DASHBOARD_PAGE_STACK,
} from "@/lib/dashboard-layout";

type BillingOrganization = {
  id: string;
  name: string;
  slug?: string;
  plan?: string;
  status?: string;
  owner_email?: string | null;
};

type BillingSubscription = {
  plan: string;
  status: string;
  current_period_start?: string | null;
  current_period_end?: string | null;
  self_serve_manageable?: boolean;
  billing_profile_linked?: boolean;
};

type BillingUsage = {
  requests_this_month: number;
  monthly_limit: number | null;
  quota_remaining: number | null;
};

type BillingInvoice = {
  id: string;
  amount: number;
  currency: string;
  status: string;
  due_date?: string | null;
  paid_at?: string | null;
  created_at: string;
};

type BillingCredits = {
  balance_cents: number;
};

type BillingActions = {
  can_upgrade: boolean;
  can_downgrade: boolean;
  can_open_portal: boolean;
  requires_sales_contact: boolean;
  message?: string;
};

type BillingResponse = {
  organization: BillingOrganization | null;
  subscription: BillingSubscription;
  payment_method: unknown;
  usage: BillingUsage;
  credits: BillingCredits;
  invoices: BillingInvoice[];
  actions: BillingActions;
  title?: string;
  message?: string;
};

export default function BillingPage() {
  const [billing, setBilling] = useState<BillingResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void loadBilling();
  }, []);

  async function loadBilling() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/dashboard/billing", { cache: "no-store" });
      const body = (await res.json()) as BillingResponse;
      if (!res.ok) {
        setBilling(null);
        setError(body.title || body.message || "Failed to load billing.");
        return;
      }
      setBilling(body);
    } catch (err) {
      setBilling(null);
      setError(err instanceof Error ? err.message : "Failed to load billing.");
    } finally {
      setLoading(false);
    }
  }

  const subscription = billing?.subscription;
  const usage = billing?.usage;
  const invoices = billing?.invoices || [];
  const credits = billing?.credits;
  const actions = billing?.actions;
  const currentPlan = subscription?.plan?.toLowerCase() || "free";
  const quotaUsedPct = useMemo(() => {
    if (!usage?.monthly_limit) return null;
    return Math.min(100, Math.round((usage.requests_this_month / usage.monthly_limit) * 100));
  }, [usage]);

  return (
    <div className={DASHBOARD_PAGE_STACK}>
      <div className={DASHBOARD_PAGE_HEADER}>
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">Billing</h1>
          <p className={DASHBOARD_PAGE_LEAD}>
            Subscription state, invoice history, and billing readiness for the authenticated workspace.
          </p>
        </div>

        <div className={`${DASHBOARD_CARD_PADDED} space-y-3 lg:max-w-md`}>
          <p className="text-sm leading-6 text-[var(--text-muted)]">
            Billing is now loaded from the billing read model instead of being inferred only from usage.
          </p>
          <div className="flex flex-col gap-3 sm:flex-row">
            <BillingLink href="https://fetchium.com/pricing" primary>
              View pricing
            </BillingLink>
            <BillingLink href="mailto:founders@fetchium.com?subject=Fetchium%20billing">
              Contact billing
            </BillingLink>
          </div>
        </div>
      </div>

      {error && (
        <div className={`${DASHBOARD_ALERT} border-red-500/20 bg-red-500/10 text-[var(--danger-text)]`}>
          {error}
        </div>
      )}

      {actions?.message && (
        <div
          className={`${DASHBOARD_ALERT} ${
            subscription?.billing_profile_linked
              ? "border-amber-500/20 bg-amber-500/10 text-amber-200"
              : "border-[var(--border-subtle)] bg-[var(--surface-raised)] text-[var(--text-secondary)]"
          }`}
        >
          {actions.message}
        </div>
      )}

      <section className="rounded-xl border border-[var(--brand-border)] bg-[var(--brand-soft)] p-4 sm:p-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <div className="flex flex-wrap items-center gap-3">
              <span className="text-xs font-medium uppercase tracking-wider text-[var(--brand-solid)]">
                Current plan
              </span>
              <span className="rounded-full border border-emerald-500/20 bg-[var(--success-soft)] px-2.5 py-0.5 text-xs font-medium text-[var(--success-text)]">
                {loading ? "Loading" : subscription?.status || "Unknown"}
              </span>
            </div>
            <h2 className="mt-3 text-2xl font-semibold capitalize text-[var(--text-primary)]">
              {loading ? "Loading…" : currentPlan}
            </h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-[var(--text-muted)]">
              {billing?.organization
                ? `Billing profile linked to ${billing.organization.name}.`
                : "No linked customer billing profile is available for this API key yet."}
            </p>
          </div>

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:min-w-[21rem]">
            <SummaryTile label="This month" value={usage ? String(usage.requests_this_month) : "—"} />
            <SummaryTile
              label="Monthly limit"
              value={usage ? (usage.monthly_limit == null ? "Unlimited" : String(usage.monthly_limit)) : "—"}
            />
          </div>
        </div>

        <div className="mt-5 space-y-3">
          <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <span className="text-sm font-medium text-[var(--text-primary)]">Plan usage</span>
            <span className="text-sm text-[var(--text-muted)]">
              {usage
                ? `${usage.requests_this_month} / ${usage.monthly_limit ?? "unlimited"} requests`
                : "Usage unavailable"}
            </span>
          </div>
          <div className="h-2.5 overflow-hidden rounded-full bg-[var(--surface-hover)]">
            <div
              className="h-full rounded-full bg-gradient-to-r from-brand-500 to-brand-300"
              style={{ width: `${quotaUsedPct ?? 0}%` }}
            />
          </div>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1.05fr)_minmax(320px,0.95fr)]">
        <div className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div>
            <h2 className="font-medium text-[var(--text-primary)]">Subscription</h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-[var(--text-muted)]">
              Billing truth comes from the billing read model. Usage is shown separately for plan fit.
            </p>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <InfoTile title="Workspace" copy={billing?.organization?.name || "No linked organization"} />
            <InfoTile title="Subscription status" copy={subscription?.status || "Unknown"} />
            <InfoTile
              title="Current period start"
              copy={subscription?.current_period_start ? new Date(subscription.current_period_start).toLocaleString() : "Unavailable"}
            />
            <InfoTile
              title="Current period end"
              copy={subscription?.current_period_end ? new Date(subscription.current_period_end).toLocaleString() : "Unavailable"}
            />
          </div>
        </div>

        <div className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
          <div>
            <h2 className="font-medium text-[var(--text-primary)]">Credits and actions</h2>
            <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">
              Self-serve checkout is only shown when a billing profile supports it.
            </p>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <SummaryTile
              label="Credit balance"
              value={credits ? `$${(credits.balance_cents / 100).toFixed(2)}` : "—"}
            />
            <SummaryTile
              label="Quota remaining"
              value={usage ? (usage.quota_remaining == null ? "Unlimited" : String(usage.quota_remaining)) : "—"}
            />
          </div>

          <div className="space-y-3">
            <BillingLink href="https://fetchium.com/pricing" primary block>
              Review pricing
            </BillingLink>
            <BillingLink href="mailto:founders@fetchium.com?subject=Fetchium%20plan%20change" block>
              Request plan change
            </BillingLink>
          </div>
        </div>
      </section>

      <section className={`${DASHBOARD_CARD_PADDED} space-y-4`}>
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-lg font-semibold text-[var(--text-primary)]">Invoices</h2>
            <p className="mt-1 text-sm text-[var(--text-muted)]">
              Historical invoices for the linked billing profile.
            </p>
          </div>
        </div>

        {loading ? (
          <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-sm text-[var(--text-faint)]">
            Loading invoices…
          </div>
        ) : invoices.length === 0 ? (
          <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-sm text-[var(--text-faint)]">
            No invoices are available for this billing profile yet.
          </div>
        ) : (
          <div className="space-y-3">
            {invoices.map((invoice) => (
              <div
                key={invoice.id}
                className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] px-4 py-3"
              >
                <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                  <div>
                    <div className="text-sm font-medium text-[var(--text-primary)]">{invoice.id}</div>
                    <div className="mt-1 text-xs text-[var(--text-faint)]">
                      Created {new Date(invoice.created_at).toLocaleString()}
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center gap-3 text-sm">
                    <span className="text-[var(--text-secondary)]">
                      {(invoice.amount / 100).toLocaleString(undefined, {
                        style: "currency",
                        currency: invoice.currency.toUpperCase(),
                      })}
                    </span>
                    <span className="rounded-full border border-[var(--border-strong)] px-2 py-0.5 text-xs text-[var(--text-muted)]">
                      {invoice.status}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function SummaryTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4">
      <div className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-faint)]">{label}</div>
      <div className="mt-2 text-lg font-semibold text-[var(--text-primary)]">{value}</div>
    </div>
  );
}

function InfoTile({ title, copy }: { title: string; copy: string }) {
  return (
    <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4">
      <div className="text-sm font-medium text-[var(--text-primary)]">{title}</div>
      <p className="mt-2 break-words text-sm leading-6 text-[var(--text-muted)]">{copy}</p>
    </div>
  );
}

function BillingLink({
  href,
  children,
  primary = false,
  block = false,
}: {
  href: string;
  children: React.ReactNode;
  primary?: boolean;
  block?: boolean;
}) {
  return (
    <Link
      href={href}
      target={href.startsWith("http") ? "_blank" : undefined}
      rel={href.startsWith("http") ? "noopener noreferrer" : undefined}
      className={`inline-flex items-center justify-center gap-1.5 rounded-lg px-4 py-2.5 text-sm font-medium transition-colors ${
        block ? "w-full" : ""
      } ${
        primary
          ? "bg-brand-500 text-white hover:bg-brand-600"
          : "border border-[var(--border-subtle)] text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
      }`}
    >
      {children}
      <ArrowUpRight className="h-3.5 w-3.5" />
    </Link>
  );
}
