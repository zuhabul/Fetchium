import { redirect } from 'next/navigation'
import Link from 'next/link'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'

interface CrmAccount {
  org_id: string
  org_name: string
  lifecycle_stage: 'prospect' | 'trial' | 'customer' | 'expansion' | 'churned'
  health_score: number
  arr_cents: number
  csm?: string
  last_contacted?: string
  churn_risk_pct: number
}

function asNumber(value: unknown, fallback = 0) {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) return parsed
  }
  return fallback
}

function asString(value: unknown, fallback = '') {
  return typeof value === 'string' ? value : fallback
}

function normalizeAccounts(payload: unknown): CrmAccount[] {
  const body = typeof payload === 'object' && payload !== null
    ? payload as Record<string, unknown>
    : {}

  const rows = Array.isArray(body.data)
    ? body.data
    : Array.isArray(payload)
      ? payload
      : []

  return rows.map((row, index) => {
    const item = typeof row === 'object' && row !== null
      ? row as Record<string, unknown>
      : {}

    return {
      org_id: asString(item.org_id, `org-${index}`),
      org_name: asString(item.org_name, 'Unknown organization'),
      lifecycle_stage: (asString(item.lifecycle_stage, 'prospect') as CrmAccount['lifecycle_stage']),
      health_score: asNumber(item.health_score),
      arr_cents: asNumber(item.arr_cents),
      csm: asString(item.csm) || undefined,
      last_contacted: asString(item.last_contacted) || undefined,
      churn_risk_pct: asNumber(item.churn_risk_pct),
    }
  })
}

function formatCurrency(cents: number) {
  return `$${(cents / 100).toFixed(0)}`
}

function formatDate(value?: string) {
  return value
    ? new Date(value).toLocaleDateString('en-US', {
        month: 'short', day: 'numeric', year: 'numeric',
      })
    : '—'
}

function HealthScore({ score }: { score: number }) {
  const color =
    score < 40
      ? 'text-red-400'
      : score <= 70
      ? 'text-amber-400'
      : 'text-emerald-400'
  const barColor =
    score < 40 ? 'bg-red-500' : score <= 70 ? 'bg-amber-500' : 'bg-emerald-500'

  return (
    <div className="flex items-center gap-2">
      <span className={`text-sm font-medium ${color}`}>{score}</span>
      <div className="w-16 h-1.5 bg-zinc-700 rounded-full overflow-hidden">
        <div className={`h-full ${barColor} rounded-full`} style={{ width: `${score}%` }} />
      </div>
    </div>
  )
}

function StageBadge({ stage }: { stage: string }) {
  const cls: Record<string, string> = {
    prospect: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    trial: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
    customer: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    expansion: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    churned: 'bg-red-500/20 text-red-400 border-red-500/30',
  }
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${cls[stage] ?? 'bg-zinc-800 text-zinc-400 border-zinc-700'}`}>
      {stage}
    </span>
  )
}

export default async function CrmPage({
  searchParams,
}: {
  searchParams: Promise<{ stage?: string; health?: string }>
}) {
  const session = await getSession()
  if (!session) redirect('/login')

  const params = await searchParams
  const filterStage = params.stage ?? 'all'
  const filterHealth = params.health ?? 'all'

  let accounts: CrmAccount[] = []
  let error = false

  try {
    const res = await adminFetch('/internal/admin/crm/accounts')
    if (res.ok) accounts = normalizeAccounts(await res.json())
    else error = true
  } catch {
    error = true
  }

  const filtered = accounts.filter((a) => {
    const stageOk = filterStage === 'all' || a.lifecycle_stage === filterStage
    const healthOk =
      filterHealth === 'all' ||
      (filterHealth === '<40' && a.health_score < 40) ||
      (filterHealth === '40-70' && a.health_score >= 40 && a.health_score <= 70) ||
      (filterHealth === '>70' && a.health_score > 70)
    return stageOk && healthOk
  })

  const stages = ['all', 'prospect', 'trial', 'customer', 'expansion', 'churned']
  const healthRanges = ['all', '<40', '40-70', '>70']

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="CRM" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-6`}>
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm">
            Failed to load data
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
          <div className="space-y-4 border-b border-zinc-800 p-4">
            <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <h2 className="text-sm font-semibold text-zinc-100">Accounts</h2>
              <p className="text-xs text-zinc-500">{filtered.length} matching accounts</p>
            </div>

            <div className="space-y-3">
              <div className="space-y-2">
                <span className="text-[11px] uppercase tracking-wider text-zinc-500">Stage</span>
                <div className="flex flex-wrap gap-2">
                  {stages.map((s) => (
                    <Link
                      key={s}
                      href={`/crm?stage=${s}&health=${filterHealth}`}
                      className={`inline-flex min-h-10 items-center rounded-md border px-3 py-2 text-xs transition-colors sm:min-h-8 sm:py-1 ${
                        filterStage === s
                          ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                          : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                      }`}
                    >
                      {s}
                    </Link>
                  ))}
                </div>
              </div>

              <div className="space-y-2">
                <span className="text-[11px] uppercase tracking-wider text-zinc-500">Health</span>
                <div className="flex flex-wrap gap-2">
                  {healthRanges.map((h) => (
                    <Link
                      key={h}
                      href={`/crm?stage=${filterStage}&health=${h}`}
                      className={`inline-flex min-h-10 items-center rounded-md border px-3 py-2 text-xs transition-colors sm:min-h-8 sm:py-1 ${
                        filterHealth === h
                          ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                          : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                      }`}
                    >
                      {h}
                    </Link>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {filtered.length === 0 ? (
            <div className="px-4 py-12 text-center text-sm text-zinc-500">
              No accounts found
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/60 lg:hidden">
                {filtered.map((acc) => (
                  <div key={acc.org_id} className="space-y-4 px-4 py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 space-y-1">
                        <Link
                          href={`/crm/${acc.org_id}`}
                          className="block truncate text-sm font-medium text-blue-400 hover:text-blue-300"
                        >
                          {acc.org_name}
                        </Link>
                        <p className="text-xs text-zinc-500">{acc.org_id}</p>
                      </div>
                      <StageBadge stage={acc.lifecycle_stage} />
                    </div>

                    <div className="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Health</dt>
                        <dd className="mt-1">
                          <HealthScore score={acc.health_score} />
                        </dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">ARR</dt>
                        <dd className="mt-1 text-zinc-300">{formatCurrency(acc.arr_cents)}</dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">CSM</dt>
                        <dd className="mt-1 text-zinc-400">{acc.csm ?? '—'}</dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Last Contacted</dt>
                        <dd className="mt-1 text-zinc-400">{formatDate(acc.last_contacted)}</dd>
                      </div>
                      <div className="col-span-2">
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Churn Risk</dt>
                        <dd className={`mt-1 text-sm font-medium ${
                          acc.churn_risk_pct >= 70
                            ? 'text-red-400'
                            : acc.churn_risk_pct >= 40
                            ? 'text-amber-400'
                            : 'text-zinc-400'
                        }`}>
                          {acc.churn_risk_pct}%
                        </dd>
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              <table className="hidden w-full lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['Org', 'Stage', 'Health Score', 'ARR', 'CSM', 'Last Contacted', 'Churn Risk'].map((h) => (
                      <th key={h} className="text-xs font-medium text-zinc-500 uppercase tracking-wider px-3 py-2 text-left">
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((acc) => (
                    <tr key={acc.org_id} className="hover:bg-zinc-800/40 border-b border-zinc-800/60">
                      <td className="px-3 py-2.5">
                        <Link href={`/crm/${acc.org_id}`} className="text-sm text-blue-400 hover:text-blue-300">
                          {acc.org_name}
                        </Link>
                      </td>
                      <td className="px-3 py-2.5">
                        <StageBadge stage={acc.lifecycle_stage} />
                      </td>
                      <td className="px-3 py-2.5">
                        <HealthScore score={acc.health_score} />
                      </td>
                      <td className="px-3 py-2.5 text-sm text-zinc-300">
                        {formatCurrency(acc.arr_cents)}
                      </td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{acc.csm ?? '—'}</td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{formatDate(acc.last_contacted)}</td>
                      <td className="px-3 py-2.5">
                        <span className={`text-sm font-medium ${
                          acc.churn_risk_pct >= 70
                            ? 'text-red-400'
                            : acc.churn_risk_pct >= 40
                            ? 'text-amber-400'
                            : 'text-zinc-400'
                        }`}>
                          {acc.churn_risk_pct}%
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </div>
      </div>
    </div>
  )
}
