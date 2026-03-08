import { redirect, notFound } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import CrmAccountActions from './CrmAccountActions'

interface HealthSignal {
  label: string
  value: number
  max: number
}

interface CrmNote {
  id: string
  author: string
  body: string
  created_at: string
}

interface CrmAccountDetail {
  org_id: string
  org_name: string
  lifecycle_stage: string
  health_score: number
  arr_cents: number
  csm?: string
  churn_risk_pct: number
  notes?: CrmNote[]
  health_signals?: HealthSignal[]
}

const DEFAULT_SIGNALS: HealthSignal[] = [
  { label: 'Product Engagement', value: 0, max: 100 },
  { label: 'Support Tickets', value: 0, max: 100 },
  { label: 'Payment History', value: 0, max: 100 },
  { label: 'Feature Adoption', value: 0, max: 100 },
  { label: 'NPS Score', value: 0, max: 100 },
  { label: 'Login Frequency', value: 0, max: 100 },
]

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

function SignalBar({ signal }: { signal: HealthSignal }) {
  const pct = Math.min(100, Math.round((signal.value / signal.max) * 100))
  const color =
    pct < 40 ? 'bg-red-500' : pct <= 70 ? 'bg-amber-500' : 'bg-emerald-500'
  return (
    <div className="space-y-1">
      <div className="flex justify-between items-center">
        <span className="text-xs text-zinc-400">{signal.label}</span>
        <span className="text-xs text-zinc-500">{signal.value}/{signal.max}</span>
      </div>
      <div className="h-1.5 bg-zinc-700 rounded-full overflow-hidden">
        <div className={`h-full ${color} rounded-full transition-all`} style={{ width: `${pct}%` }} />
      </div>
    </div>
  )
}

export default async function CrmOrgPage({ params }: { params: Promise<{ orgId: string }> }) {
  const session = await getSession()
  if (!session) redirect('/login')

  const { orgId } = await params
  let account: CrmAccountDetail | null = null
  let error = false

  try {
    const res = await adminFetch(`/internal/admin/crm/accounts/${orgId}`)
    if (res.status === 404) notFound()
    if (res.ok) account = await res.json()
    else error = true
  } catch {
    error = true
  }

  const signals = account?.health_signals ?? DEFAULT_SIGNALS
  const notes = account?.notes ?? []

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title={account?.org_name ?? 'Account Detail'} />
      <div className="p-6 space-y-6">
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm">
            Failed to load data
          </div>
        )}

        {account && (
          <>
            {/* Header */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <h1 className="text-xl font-semibold text-zinc-100">{account.org_name}</h1>
                <StageBadge stage={account.lifecycle_stage} />
              </div>
              <div className="text-right">
                <p className="text-lg font-semibold text-zinc-100">
                  ${(account.arr_cents / 100).toFixed(0)}
                  <span className="text-sm font-normal text-zinc-500"> ARR</span>
                </p>
                <p className="text-xs text-zinc-500">Churn risk: {account.churn_risk_pct}%</p>
              </div>
            </div>

            {/* Health Score Breakdown */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-semibold text-zinc-100">Health Score</h2>
                <span className={`text-2xl font-bold ${
                  account.health_score < 40
                    ? 'text-red-400'
                    : account.health_score <= 70
                    ? 'text-amber-400'
                    : 'text-emerald-400'
                }`}>
                  {account.health_score}
                </span>
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                {signals.map((s) => (
                  <SignalBar key={s.label} signal={s} />
                ))}
              </div>
            </div>

            {/* CRM Actions: Stage selector, Assign CSM, Add Note */}
            <CrmAccountActions account={account} session={session} />

            {/* Notes Timeline */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
              <h2 className="text-sm font-semibold text-zinc-100 mb-4">Notes</h2>
              {notes.length === 0 ? (
                <p className="text-sm text-zinc-500">No notes yet. Add one above.</p>
              ) : (
                <div className="space-y-4">
                  {notes.map((note) => (
                    <div key={note.id} className="border-l-2 border-zinc-700 pl-4">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-xs font-medium text-zinc-300">{note.author}</span>
                        <span className="text-xs text-zinc-600">
                          {new Date(note.created_at).toLocaleDateString('en-US', {
                            month: 'short', day: 'numeric', year: 'numeric',
                          })}
                        </span>
                      </div>
                      <p className="text-sm text-zinc-400">{note.body}</p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  )
}
