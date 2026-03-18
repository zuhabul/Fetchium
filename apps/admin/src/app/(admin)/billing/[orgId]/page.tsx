import { redirect, notFound } from 'next/navigation'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import BillingActions from './BillingActions'

interface BillingDetail {
  org_id: string
  org_name: string
  plan: string
  status: string
  mrr_cents: number
  period_start: string
  period_end: string
  trial_end?: string
  cancel_at?: string
  provider_id: string
}

interface Invoice {
  id: string
  amount_cents: number
  status: string
  due_date: string
  paid_date?: string
}

interface CreditEntry {
  id: string
  delta_cents: number
  reason: string
  created_at: string
}

function StatusBadge({ status }: { status: string }) {
  const cls: Record<string, string> = {
    active: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    trialing: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
    past_due: 'bg-red-500/20 text-red-400 border-red-500/30',
    canceled: 'bg-red-500/20 text-red-400 border-red-500/30',
    paid: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    open: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    failed: 'bg-red-500/20 text-red-400 border-red-500/30',
  }
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${cls[status] ?? 'bg-zinc-800 text-zinc-400 border-zinc-700'}`}>
      {status.replace('_', ' ')}
    </span>
  )
}

function fmt(date?: string) {
  if (!date) return '—'
  return new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

function DetailRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex justify-between items-center py-2 border-b border-zinc-800/60 last:border-0">
      <span className="text-xs text-zinc-500 uppercase tracking-wider font-medium">{label}</span>
      <span className="text-sm text-zinc-300">{value}</span>
    </div>
  )
}

export default async function BillingOrgPage({ params }: { params: Promise<{ orgId: string }> }) {
  const session = await getSession()
  if (!session) redirect('/login')

  const { orgId } = await params

  let billing: BillingDetail | null = null
  let invoices: Invoice[] = []
  let credits: CreditEntry[] = []
  let error = false

  try {
    const [billingRes, invoicesRes, creditsRes] = await Promise.all([
      adminFetch(`/internal/admin/billing/${orgId}`),
      adminFetch(`/internal/admin/billing/${orgId}/invoices`),
      adminFetch(`/internal/admin/billing/${orgId}/credits`),
    ])
    if (billingRes.status === 404) notFound()
    if (billingRes.ok) billing = await billingRes.json()
    else error = true
    if (invoicesRes.ok) invoices = await invoicesRes.json()
    if (creditsRes.ok) credits = await creditsRes.json()
  } catch {
    error = true
  }

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title={billing?.org_name ?? 'Billing Detail'} />
      <div className={`${ADMIN_PAGE_PADDING} space-y-6`}>
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm">
            Failed to load data
          </div>
        )}

        {billing && (
          <>
            {/* Header */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <h1 className="text-xl font-semibold text-zinc-100">{billing.org_name}</h1>
                <span className="text-sm text-zinc-500 bg-zinc-800 border border-zinc-700 rounded-md px-2 py-0.5">
                  {billing.plan}
                </span>
                <StatusBadge status={billing.status} />
              </div>
              <p className="text-lg font-semibold text-zinc-100">
                ${(billing.mrr_cents / 100).toFixed(2)}<span className="text-sm font-normal text-zinc-500">/mo</span>
              </p>
            </div>

            {/* Subscription Details */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
              <h2 className="text-sm font-semibold text-zinc-100 mb-3">Subscription Details</h2>
              <DetailRow label="Plan" value={billing.plan} />
              <DetailRow label="Status" value={<StatusBadge status={billing.status} />} />
              <DetailRow label="Period Start" value={fmt(billing.period_start)} />
              <DetailRow label="Period End" value={fmt(billing.period_end)} />
              <DetailRow label="Trial End" value={fmt(billing.trial_end)} />
              <DetailRow label="Cancel At" value={fmt(billing.cancel_at)} />
              <DetailRow
                label="Provider ID"
                value={<span className="font-mono text-xs text-zinc-400">{billing.provider_id}</span>}
              />
            </div>

            {/* Action Buttons + Modals */}
            <BillingActions orgId={orgId} />

            {/* Invoices */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
              <div className="px-4 py-3 border-b border-zinc-800">
                <h2 className="text-sm font-semibold text-zinc-100">Invoices</h2>
              </div>
              <table className="w-full">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['ID', 'Amount', 'Status', 'Due Date', 'Paid Date'].map((h) => (
                      <th key={h} className="text-xs font-medium text-zinc-500 uppercase tracking-wider px-3 py-2 text-left">
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {invoices.length === 0 ? (
                    <tr><td colSpan={5} className="px-3 py-6 text-center text-zinc-500 text-sm">No invoices</td></tr>
                  ) : invoices.map((inv) => (
                    <tr key={inv.id} className="hover:bg-zinc-800/40 border-b border-zinc-800/60">
                      <td className="px-3 py-2.5 text-xs font-mono text-zinc-500">{inv.id.slice(0, 16)}…</td>
                      <td className="px-3 py-2.5 text-sm text-zinc-300">${(inv.amount_cents / 100).toFixed(2)}</td>
                      <td className="px-3 py-2.5"><StatusBadge status={inv.status} /></td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{fmt(inv.due_date)}</td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{fmt(inv.paid_date)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Credits Ledger */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
              <div className="px-4 py-3 border-b border-zinc-800">
                <h2 className="text-sm font-semibold text-zinc-100">Credits Ledger</h2>
              </div>
              <table className="w-full">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['Amount', 'Reason', 'Created At'].map((h) => (
                      <th key={h} className="text-xs font-medium text-zinc-500 uppercase tracking-wider px-3 py-2 text-left">
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {credits.length === 0 ? (
                    <tr><td colSpan={3} className="px-3 py-6 text-center text-zinc-500 text-sm">No credits issued</td></tr>
                  ) : credits.map((c) => (
                    <tr key={c.id} className="hover:bg-zinc-800/40 border-b border-zinc-800/60">
                      <td className="px-3 py-2.5 text-sm text-zinc-300">
                        <span className={c.delta_cents >= 0 ? 'text-emerald-400' : 'text-red-400'}>
                          {c.delta_cents >= 0 ? '+' : ''}${(c.delta_cents / 100).toFixed(2)}
                        </span>
                      </td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{c.reason}</td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{fmt(c.created_at)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        )}
      </div>
    </div>
  )
}
