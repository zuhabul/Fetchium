import Link from 'next/link'
import { notFound, redirect } from 'next/navigation'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import { ArrowLeft, MousePointerClick } from 'lucide-react'

interface TouchRow {
  id: string
  touch_type: string
  org_id: string | null
  occurred_at: string
}

interface CampaignSummary {
  campaign_id: string
  campaign_name: string
  type: string
  status: string
  touch_count: number
  unique_orgs: number
  first_touch?: string | null
  last_touch?: string | null
}

function asNumber(value: unknown): number {
  if (typeof value === 'number') return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : 0
  }
  return 0
}

function asString(value: unknown, fallback = ''): string {
  return typeof value === 'string' ? value : fallback
}

function normalizeTouches(value: unknown): TouchRow[] {
  const rows = typeof value === 'object' && value !== null && Array.isArray((value as { rows?: unknown[] }).rows)
    ? (value as { rows: unknown[] }).rows
    : []

  return rows.map((row, index) => {
    const values = Array.isArray(row) ? row : []

    return {
      id: asString(values[0], `touch-${index}`),
      touch_type: asString(values[1], 'unknown'),
      org_id: typeof values[2] === 'string' ? values[2] : null,
      occurred_at: asString(values[3], new Date(0).toISOString()),
    }
  })
}

function normalizeSummaries(value: unknown): CampaignSummary[] {
  if (!Array.isArray(value)) return []

  return value.map((row, index) => {
    const item = typeof row === 'object' && row !== null
      ? row as Record<string, unknown>
      : {}

    return {
      campaign_id: asString(item.campaign_id, `campaign-${index}`),
      campaign_name: asString(item.campaign_name, 'Untitled campaign'),
      type: asString(item.type, 'unknown'),
      status: asString(item.status, 'draft'),
      touch_count: asNumber(item.touch_count),
      unique_orgs: asNumber(item.unique_orgs),
      first_touch: typeof item.first_touch === 'string' ? item.first_touch : null,
      last_touch: typeof item.last_touch === 'string' ? item.last_touch : null,
    }
  })
}

function fmt(date?: string | null) {
  if (!date) return '—'
  return new Date(date).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
}

const STATUS_BADGE: Record<string, string> = {
  draft: 'bg-zinc-800 text-zinc-400 border-zinc-700',
  active: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
  paused: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
  archived: 'bg-zinc-800 text-zinc-600 border-zinc-700',
}

export default async function CampaignDetailPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const session = await getSession()
  if (!session) redirect('/login')

  const { id } = await params

  let touches: TouchRow[] = []
  let summary: CampaignSummary | null = null

  try {
    const [detailRes, attributionRes] = await Promise.all([
      adminFetch(`/internal/admin/campaigns/${id}`),
      adminFetch('/internal/admin/campaigns/attribution'),
    ])

    if (detailRes.status === 404) notFound()
    if (detailRes.ok) {
      const payload = await detailRes.json()
      touches = normalizeTouches(payload.touches)
    }

    if (attributionRes.ok) {
      const payload = await attributionRes.json()
      summary = normalizeSummaries(payload.data).find(campaign => campaign.campaign_id === id) ?? null
    }
  } catch {
    notFound()
  }

  if (!summary && touches.length === 0) {
    notFound()
  }

  const campaignName = summary?.campaign_name ?? `Campaign ${id.slice(0, 8)}`

  return (
    <>
      <TopBar title={campaignName} subtitle="Campaign activity and attribution touches" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>
        <Link
          href="/campaigns"
          className="inline-flex items-center gap-1.5 text-xs text-zinc-500 transition-colors hover:text-zinc-300"
        >
          <ArrowLeft className="h-3 w-3" />
          All Campaigns
        </Link>

        <div className="grid grid-cols-1 gap-3 lg:grid-cols-[minmax(0,2fr)_minmax(18rem,1fr)]">
          <div className="space-y-3 rounded-xl border border-zinc-800 bg-zinc-900 p-4">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <h2 className="truncate text-base font-semibold text-zinc-100">{campaignName}</h2>
                <p className="mt-1 text-xs capitalize text-zinc-500">{summary?.type ?? 'unknown'}</p>
              </div>
              <span
                className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium capitalize ${
                  STATUS_BADGE[summary?.status ?? 'draft'] ?? STATUS_BADGE.draft
                }`}
              >
                {summary?.status ?? 'draft'}
              </span>
            </div>

            <div className="grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
              <div className="rounded-lg bg-zinc-800/60 px-3 py-3">
                <p className="text-[11px] uppercase tracking-wider text-zinc-600">Touches</p>
                <p className="mt-2 text-lg font-semibold text-zinc-100">
                  {(summary?.touch_count ?? touches.length).toLocaleString()}
                </p>
              </div>
              <div className="rounded-lg bg-zinc-800/60 px-3 py-3">
                <p className="text-[11px] uppercase tracking-wider text-zinc-600">Touched Orgs</p>
                <p className="mt-2 text-lg font-semibold text-zinc-100">
                  {(summary?.unique_orgs ?? 0).toLocaleString()}
                </p>
              </div>
              <div className="rounded-lg bg-zinc-800/60 px-3 py-3">
                <p className="text-[11px] uppercase tracking-wider text-zinc-600">First Touch</p>
                <p className="mt-2 text-sm font-medium text-zinc-200">{fmt(summary?.first_touch)}</p>
              </div>
              <div className="rounded-lg bg-zinc-800/60 px-3 py-3">
                <p className="text-[11px] uppercase tracking-wider text-zinc-600">Last Touch</p>
                <p className="mt-2 text-sm font-medium text-zinc-200">{fmt(summary?.last_touch)}</p>
              </div>
            </div>
          </div>

          <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
            <h3 className="text-sm font-semibold text-zinc-200">Campaign ID</h3>
            <p className="mt-2 break-all font-mono text-xs text-zinc-500">{id}</p>
          </div>
        </div>

        <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900">
          <div className="flex items-center gap-2 border-b border-zinc-800 px-4 py-3">
            <MousePointerClick className="h-4 w-4 text-zinc-500" />
            <h2 className="text-sm font-semibold text-zinc-100">Recent Attribution Touches</h2>
          </div>

          {touches.length === 0 ? (
            <div className="px-4 py-10 text-center text-sm text-zinc-500">
              No attribution touches recorded for this campaign yet.
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/50 lg:hidden">
                {touches.map(touch => (
                  <div key={touch.id} className="space-y-3 px-4 py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <p className="text-sm font-medium capitalize text-zinc-100">{touch.touch_type}</p>
                        <p className="mt-1 text-xs text-zinc-500">{fmt(touch.occurred_at)}</p>
                      </div>
                      {touch.org_id ? (
                        <Link
                          href={`/orgs/${touch.org_id}`}
                          className="inline-flex min-h-11 shrink-0 items-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
                        >
                          Org
                        </Link>
                      ) : null}
                    </div>

                    <dl className="grid grid-cols-1 gap-3 text-sm">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Touch ID</dt>
                        <dd className="mt-1 break-all font-mono text-xs text-zinc-400">{touch.id}</dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Org ID</dt>
                        <dd className="mt-1 break-all text-zinc-300">{touch.org_id ?? '—'}</dd>
                      </div>
                    </dl>
                  </div>
                ))}
              </div>

              <table className="hidden w-full text-sm lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['Touch ID', 'Type', 'Org', 'Occurred At'].map(header => (
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
                  {touches.map(touch => (
                    <tr key={touch.id} className="bg-zinc-900 transition-colors hover:bg-zinc-800/60">
                      <td className="px-4 py-3 font-mono text-xs text-zinc-400">{touch.id}</td>
                      <td className="px-4 py-3 capitalize text-zinc-200">{touch.touch_type}</td>
                      <td className="px-4 py-3 text-zinc-300">
                        {touch.org_id ? (
                          <Link href={`/orgs/${touch.org_id}`} className="text-blue-400 hover:underline">
                            {touch.org_id}
                          </Link>
                        ) : '—'}
                      </td>
                      <td className="px-4 py-3 text-zinc-400">{fmt(touch.occurred_at)}</td>
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
