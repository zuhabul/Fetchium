import { redirect } from 'next/navigation'
import Link from 'next/link'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'

interface Subscription {
  org_id: string
  org_name: string
  plan: string
  status: 'active' | 'trialing' | 'past_due' | 'canceled'
  mrr_cents: number
  period_end: string
  provider_id: string
}

interface BillingOverview {
  total_mrr_cents: number
  active_subscriptions: number
  failed_payments: number
  trial_conversion_pct: number
  subscriptions: Subscription[]
}

function StatusBadge({ status }: { status: string }) {
  const cls: Record<string, string> = {
    active: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    trialing: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
    past_due: 'bg-red-500/20 text-red-400 border-red-500/30',
    canceled: 'bg-red-500/20 text-red-400 border-red-500/30',
  }
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${cls[status] ?? 'bg-zinc-800 text-zinc-400 border-zinc-700'}`}>
      {status.replace('_', ' ')}
    </span>
  )
}

function KpiCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
      <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">{label}</p>
      <p className="text-2xl font-semibold text-zinc-100">{value}</p>
    </div>
  )
}

export default async function BillingPage({
  searchParams,
}: {
  searchParams: Promise<{ status?: string }>
}) {
  const session = await getSession()
  if (!session) redirect('/login')

  const params = await searchParams
  const filterStatus = params.status ?? 'all'

  let data: BillingOverview | null = null
  let error = false

  try {
    const res = await adminFetch('/internal/admin/billing')
    if (res.ok) data = await res.json()
    else error = true
  } catch {
    error = true
  }

  const subs = data?.subscriptions ?? []
  const filtered = filterStatus === 'all' ? subs : subs.filter((s) => s.status === filterStatus)

  const statuses = ['all', 'active', 'trialing', 'past_due', 'canceled']

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="Billing" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-6`}>
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm">
            Failed to load data
          </div>
        )}

        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <KpiCard
            label="Total MRR"
            value={`$${((data?.total_mrr_cents ?? 0) / 100).toFixed(2)}`}
          />
          <KpiCard
            label="Active Subscriptions"
            value={String(data?.active_subscriptions ?? 0)}
          />
          <KpiCard
            label="Failed Payments"
            value={String(data?.failed_payments ?? 0)}
          />
          <KpiCard
            label="Trial Conversion"
            value={`${(data?.trial_conversion_pct ?? 0).toFixed(1)}%`}
          />
        </div>

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
          <div className="flex items-center justify-between p-4 border-b border-zinc-800">
            <h2 className="text-sm font-semibold text-zinc-100">Subscriptions</h2>
            <div className="flex gap-1">
              {statuses.map((s) => (
                <Link
                  key={s}
                  href={`/billing?status=${s}`}
                  className={`text-xs px-3 py-1 rounded-md border transition-colors ${
                    filterStatus === s
                      ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                      : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  {s.replace('_', ' ')}
                </Link>
              ))}
            </div>
          </div>
          <table className="w-full">
            <thead>
              <tr className="border-b border-zinc-800">
                {['Org', 'Plan', 'Status', 'MRR', 'Period End', 'Provider ID'].map((h) => (
                  <th key={h} className="text-xs font-medium text-zinc-500 uppercase tracking-wider px-3 py-2 text-left">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-3 py-8 text-center text-zinc-500 text-sm">
                    No subscriptions found
                  </td>
                </tr>
              ) : (
                filtered.map((sub) => (
                  <tr key={sub.org_id} className="hover:bg-zinc-800/40 border-b border-zinc-800/60">
                    <td className="px-3 py-2.5">
                      <Link href={`/billing/${sub.org_id}`} className="text-sm text-blue-400 hover:text-blue-300">
                        {sub.org_name}
                      </Link>
                    </td>
                    <td className="px-3 py-2.5 text-sm text-zinc-300">{sub.plan}</td>
                    <td className="px-3 py-2.5"><StatusBadge status={sub.status} /></td>
                    <td className="px-3 py-2.5 text-sm text-zinc-300">
                      ${(sub.mrr_cents / 100).toFixed(2)}
                    </td>
                    <td className="px-3 py-2.5 text-sm text-zinc-400">
                      {new Date(sub.period_end).toLocaleDateString('en-US', {
                        month: 'short', day: 'numeric', year: 'numeric',
                      })}
                    </td>
                    <td className="px-3 py-2.5 text-sm text-zinc-500 font-mono">
                      {sub.provider_id.slice(0, 20)}…
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
