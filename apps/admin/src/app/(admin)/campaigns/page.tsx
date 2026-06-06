import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Megaphone, MousePointerClick, TrendingUp } from 'lucide-react'

interface Campaign {
  id: string
  name: string
  type: string
  status: string
  touch_count: number
  created_at: string
  updated_at: string
}

const STATUS_STYLES: Record<string, string> = {
  draft:    'bg-zinc-800 text-zinc-400 border-zinc-700',
  active:   'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
  paused:   'bg-amber-500/20 text-amber-400 border-amber-500/30',
  archived: 'bg-zinc-800 text-zinc-600 border-zinc-700',
}

function fmt(date: string) {
  return new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
      <p className="mb-1 text-[11px] uppercase tracking-[0.18em] text-zinc-500">{label}</p>
      <p className="break-words text-[1.75rem] font-bold leading-none text-zinc-100 lg:text-2xl">
        {value.toLocaleString()}
      </p>
    </div>
  )
}

export default async function CampaignsPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let campaigns: Campaign[] = []
  let total = 0

  try {
    const res = await adminFetch('/internal/admin/campaigns')
    if (res.ok) {
      const body = await res.json()
      campaigns = body.data ?? []
      total = body.total ?? 0
    }
  } catch {}

  const activeCount = campaigns.filter(c => c.status === 'active').length
  const totalTouches = campaigns.reduce((sum, campaign) => sum + campaign.touch_count, 0)

  return (
    <>
      <TopBar title="Campaigns" subtitle={`${total} campaigns tracked`} />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>
        <div className="grid grid-cols-1 gap-3 min-[420px]:grid-cols-2 xl:grid-cols-3">
          <StatCard label="Total" value={total} />
          <StatCard label="Active" value={activeCount} />
          <StatCard label="Total Touches" value={totalTouches} />
        </div>

        <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap">
          <Link
            href="/campaigns/funnels"
            className="inline-flex min-h-11 items-center justify-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 sm:justify-start"
          >
            <TrendingUp className="h-4 w-4" />
            Conversion Funnels
          </Link>
        </div>

        <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900">
          <div className="flex items-center gap-2 border-b border-zinc-800 px-4 py-3">
            <Megaphone className="h-4 w-4 text-zinc-500" />
            <h2 className="text-sm font-semibold text-zinc-100">All Campaigns</h2>
          </div>

          {campaigns.length === 0 ? (
            <div className="px-4 py-12 text-center text-sm text-zinc-500">
              No campaigns yet. Create one to start tracking attribution.
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/50 lg:hidden">
                {campaigns.map(campaign => (
                  <div key={campaign.id} className="space-y-4 px-4 py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 space-y-1">
                        <p className="truncate text-sm font-medium text-zinc-100">
                          {campaign.name}
                        </p>
                        <p className="text-xs capitalize text-zinc-500">{campaign.type}</p>
                      </div>
                      <Link
                        href={`/campaigns/${campaign.id}`}
                        className="inline-flex min-h-11 shrink-0 items-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
                      >
                        View
                      </Link>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <span
                        className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium capitalize ${
                          STATUS_STYLES[campaign.status] ?? STATUS_STYLES.draft
                        }`}
                      >
                        {campaign.status}
                      </span>
                    </div>

                    <dl className="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">
                          Touches
                        </dt>
                        <dd className="mt-1 flex items-center gap-2 text-zinc-300">
                          <MousePointerClick className="h-3.5 w-3.5 text-zinc-500" />
                          {campaign.touch_count.toLocaleString()}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">
                          Created
                        </dt>
                        <dd className="mt-1 text-zinc-400">{fmt(campaign.created_at)}</dd>
                      </div>
                    </dl>
                  </div>
                ))}
              </div>

              <table className="hidden w-full text-sm lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['Name', 'Type', 'Status', 'Touches', 'Created', 'Actions'].map(header => (
                      <th
                        key={header}
                        className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-zinc-500"
                      >
                        {header}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800/50">
                  {campaigns.map(campaign => (
                    <tr
                      key={campaign.id}
                      className="bg-zinc-900 transition-colors hover:bg-zinc-800/60"
                    >
                      <td className="px-4 py-3 font-medium text-zinc-100">{campaign.name}</td>
                      <td className="px-4 py-3 text-xs capitalize text-zinc-500">
                        {campaign.type}
                      </td>
                      <td className="px-4 py-3">
                        <span
                          className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium capitalize ${
                            STATUS_STYLES[campaign.status] ?? STATUS_STYLES.draft
                          }`}
                        >
                          {campaign.status}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-400">
                        {campaign.touch_count.toLocaleString()}
                      </td>
                      <td className="px-4 py-3 text-xs text-zinc-600">
                        {fmt(campaign.created_at)}
                      </td>
                      <td className="px-4 py-3">
                        <Link
                          href={`/campaigns/${campaign.id}`}
                          className="inline-block rounded-md border border-zinc-700 bg-zinc-800 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
                        >
                          View
                        </Link>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </div>
      </div>
    </>
  )
}
