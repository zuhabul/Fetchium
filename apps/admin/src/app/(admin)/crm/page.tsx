import { redirect } from 'next/navigation'
import Link from 'next/link'
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
    if (res.ok) accounts = await res.json()
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
      <div className="p-6 space-y-6">
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm">
            Failed to load data
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
          <div className="flex flex-wrap items-center gap-3 p-4 border-b border-zinc-800">
            <h2 className="text-sm font-semibold text-zinc-100 mr-2">Accounts</h2>

            <div className="flex items-center gap-1">
              <span className="text-xs text-zinc-500 mr-1">Stage:</span>
              {stages.map((s) => (
                <Link
                  key={s}
                  href={`/crm?stage=${s}&health=${filterHealth}`}
                  className={`text-xs px-2.5 py-1 rounded-md border transition-colors ${
                    filterStage === s
                      ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                      : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  {s}
                </Link>
              ))}
            </div>

            <div className="flex items-center gap-1">
              <span className="text-xs text-zinc-500 mr-1">Health:</span>
              {healthRanges.map((h) => (
                <Link
                  key={h}
                  href={`/crm?stage=${filterStage}&health=${h}`}
                  className={`text-xs px-2.5 py-1 rounded-md border transition-colors ${
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

          <table className="w-full">
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
              {filtered.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-3 py-8 text-center text-zinc-500 text-sm">
                    No accounts found
                  </td>
                </tr>
              ) : (
                filtered.map((acc) => (
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
                      ${(acc.arr_cents / 100).toFixed(0)}
                    </td>
                    <td className="px-3 py-2.5 text-sm text-zinc-400">{acc.csm ?? '—'}</td>
                    <td className="px-3 py-2.5 text-sm text-zinc-400">
                      {acc.last_contacted
                        ? new Date(acc.last_contacted).toLocaleDateString('en-US', {
                            month: 'short', day: 'numeric', year: 'numeric',
                          })
                        : '—'}
                    </td>
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
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
