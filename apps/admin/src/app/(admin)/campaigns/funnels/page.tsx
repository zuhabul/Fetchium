import Link from 'next/link'
import { redirect } from 'next/navigation'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import { ArrowLeft, TrendingUp } from 'lucide-react'

interface FunnelStage {
  name: string
  count: number
}

interface AttributionRow {
  campaign_id: string
  campaign_name: string
  type: string
  status: string
  touch_count: number
  unique_orgs: number
}

function asNumber(value: unknown): number {
  if (typeof value === 'number') return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : 0
  }
  return 0
}

function normalizeStages(value: unknown): FunnelStage[] {
  const stages = typeof value === 'object' && value !== null && Array.isArray((value as { stages?: unknown[] }).stages)
    ? (value as { stages: unknown[] }).stages
    : []

  return stages.map((stage, index) => {
    const item = typeof stage === 'object' && stage !== null
      ? stage as Record<string, unknown>
      : {}

    return {
      name: typeof item.name === 'string' ? item.name : `Stage ${index + 1}`,
      count: asNumber(item.count),
    }
  })
}

function normalizeAttributionRows(value: unknown): AttributionRow[] {
  if (!Array.isArray(value)) return []

  return value.map((row, index) => {
    const item = typeof row === 'object' && row !== null
      ? row as Record<string, unknown>
      : {}

    return {
      campaign_id: typeof item.campaign_id === 'string' ? item.campaign_id : `campaign-${index}`,
      campaign_name: typeof item.campaign_name === 'string' ? item.campaign_name : 'Untitled campaign',
      type: typeof item.type === 'string' ? item.type : 'unknown',
      status: typeof item.status === 'string' ? item.status : 'draft',
      touch_count: asNumber(item.touch_count),
      unique_orgs: asNumber(item.unique_orgs),
    }
  })
}

const STATUS_BADGE: Record<string, string> = {
  draft: 'bg-zinc-800 text-zinc-400 border-zinc-700',
  active: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
  paused: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
  archived: 'bg-zinc-800 text-zinc-600 border-zinc-700',
}

export default async function CampaignFunnelsPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let stages: FunnelStage[] = []
  let topCampaigns: AttributionRow[] = []

  try {
    const [funnelRes, attributionRes] = await Promise.all([
      adminFetch('/internal/admin/campaigns/funnel'),
      adminFetch('/internal/admin/campaigns/attribution'),
    ])

    if (funnelRes.ok) {
      const payload = await funnelRes.json()
      stages = normalizeStages(payload.data)
    }

    if (attributionRes.ok) {
      const payload = await attributionRes.json()
      topCampaigns = normalizeAttributionRows(payload.data).slice(0, 8)
    }
  } catch {}

  const maxCount = Math.max(...stages.map(stage => stage.count), 1)

  return (
    <>
      <TopBar title="Campaign Funnels" subtitle="Attribution and conversion flow" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>
        <Link
          href="/campaigns"
          className="inline-flex items-center gap-1.5 text-xs text-zinc-500 transition-colors hover:text-zinc-300"
        >
          <ArrowLeft className="h-3 w-3" />
          All Campaigns
        </Link>

        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="mb-4 flex items-center gap-2">
            <TrendingUp className="h-4 w-4 text-zinc-500" />
            <h2 className="text-sm font-semibold text-zinc-100">Funnel Overview</h2>
          </div>

          {stages.length === 0 ? (
            <p className="text-sm text-zinc-500">No funnel data available yet.</p>
          ) : (
            <div className="space-y-3">
              {stages.map((stage, index) => {
                const width = `${Math.max((stage.count / maxCount) * 100, stage.count > 0 ? 10 : 0)}%`

                return (
                  <div key={`${stage.name}-${index}`} className="space-y-2">
                    <div className="flex items-center justify-between gap-3">
                      <div className="min-w-0">
                        <p className="text-sm font-medium text-zinc-200">{stage.name}</p>
                      </div>
                      <span className="text-sm font-semibold text-zinc-100">
                        {stage.count.toLocaleString()}
                      </span>
                    </div>
                    <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                      <div
                        className="h-full rounded-full bg-zinc-300 transition-[width]"
                        style={{ width }}
                      />
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </div>

        <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900">
          <div className="border-b border-zinc-800 px-4 py-3">
            <h2 className="text-sm font-semibold text-zinc-100">Top Campaigns by Touch Volume</h2>
          </div>

          {topCampaigns.length === 0 ? (
            <div className="px-4 py-10 text-center text-sm text-zinc-500">
              No attribution data available yet.
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/50 lg:hidden">
                {topCampaigns.map(campaign => (
                  <div key={campaign.campaign_id} className="space-y-4 px-4 py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 space-y-1">
                        <p className="truncate text-sm font-medium text-zinc-100">
                          {campaign.campaign_name}
                        </p>
                        <p className="text-xs capitalize text-zinc-500">{campaign.type}</p>
                      </div>
                      <Link
                        href={`/campaigns/${campaign.campaign_id}`}
                        className="inline-flex min-h-11 shrink-0 items-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
                      >
                        View
                      </Link>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <span
                        className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium capitalize ${
                          STATUS_BADGE[campaign.status] ?? STATUS_BADGE.draft
                        }`}
                      >
                        {campaign.status}
                      </span>
                    </div>

                    <dl className="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Touches</dt>
                        <dd className="mt-1 text-zinc-300">{campaign.touch_count.toLocaleString()}</dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Touched Orgs</dt>
                        <dd className="mt-1 text-zinc-300">{campaign.unique_orgs.toLocaleString()}</dd>
                      </div>
                    </dl>
                  </div>
                ))}
              </div>

              <table className="hidden w-full text-sm lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['Campaign', 'Type', 'Status', 'Touches', 'Touched Orgs', 'Actions'].map(header => (
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
                  {topCampaigns.map(campaign => (
                    <tr key={campaign.campaign_id} className="bg-zinc-900 transition-colors hover:bg-zinc-800/60">
                      <td className="px-4 py-3 font-medium text-zinc-100">{campaign.campaign_name}</td>
                      <td className="px-4 py-3 text-xs capitalize text-zinc-500">{campaign.type}</td>
                      <td className="px-4 py-3">
                        <span
                          className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium capitalize ${
                            STATUS_BADGE[campaign.status] ?? STATUS_BADGE.draft
                          }`}
                        >
                          {campaign.status}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-zinc-300">{campaign.touch_count.toLocaleString()}</td>
                      <td className="px-4 py-3 text-zinc-300">{campaign.unique_orgs.toLocaleString()}</td>
                      <td className="px-4 py-3">
                        <Link
                          href={`/campaigns/${campaign.campaign_id}`}
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
