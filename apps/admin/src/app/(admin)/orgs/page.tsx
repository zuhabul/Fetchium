import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Building2 } from 'lucide-react'
import FilterBar from '@/components/FilterBar'
import PaginationBar from '@/components/PaginationBar'

interface Org {
  id: string
  name: string
  slug: string
  plan: string
  status: string
  mrr_cents: number
  member_count: number
  created_at: string
}

const PLAN_BADGE: Record<string, string> = {
  free: 'bg-zinc-500/20 text-zinc-400',
  starter: 'bg-blue-500/20 text-blue-400',
  pro: 'bg-purple-500/20 text-purple-400',
  enterprise: 'bg-amber-500/20 text-amber-400',
}

const STATUS_BADGE: Record<string, string> = {
  active: 'bg-emerald-500/20 text-emerald-400',
  suspended: 'bg-red-500/20 text-red-400',
  trial: 'bg-amber-500/20 text-amber-400',
  churned: 'bg-zinc-500/20 text-zinc-400',
}

function Badge({ value, map }: { value: string; map: Record<string, string> }) {
  const cls = map[value] ?? 'bg-zinc-500/20 text-zinc-400'
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium capitalize ${cls}`}>
      {value}
    </span>
  )
}

const PAGE_SIZE = 50

export default async function OrgsPage({ searchParams }: { searchParams: Promise<Record<string, string>> }) {
  const [session, params] = await Promise.all([getSession(), searchParams])

  const page = Math.max(1, parseInt(params.page ?? '1'))
  const offset = (page - 1) * PAGE_SIZE
  const filters = { search: params.search ?? '', plan: params.plan ?? '', status: params.status ?? '' }

  const qs = new URLSearchParams({ limit: String(PAGE_SIZE), offset: String(offset) })
  if (filters.search) qs.set('search', filters.search)
  if (filters.plan) qs.set('plan', filters.plan)
  if (filters.status) qs.set('status', filters.status)

  let orgs: Org[] = []
  let total = 0
  let error: string | null = null

  try {
    if (session) {
      const res = await adminFetch(`/internal/admin/orgs?${qs}`)
      if (res.ok) {
        const data = await res.json()
        orgs = data.data ?? []
        total = data.total ?? orgs.length
      } else {
        error = `API error: ${res.status}`
      }
    }
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to fetch orgs'
  }

  return (
    <>
      <TopBar title="Organizations" subtitle={`${total} orgs total`} />
      <div className="p-6 space-y-4">
        <FilterBar
          filters={[
            { key: 'search', type: 'search', placeholder: 'Search orgs...' },
            { key: 'plan', type: 'select', options: [{ value: '', label: 'All plans' }, { value: 'free', label: 'Free' }, { value: 'starter', label: 'Starter' }, { value: 'pro', label: 'Pro' }, { value: 'enterprise', label: 'Enterprise' }] },
            { key: 'status', type: 'select', options: [{ value: '', label: 'All statuses' }, { value: 'active', label: 'Active' }, { value: 'suspended', label: 'Suspended' }, { value: 'trial', label: 'Trial' }, { value: 'churned', label: 'Churned' }] },
          ]}
          current={filters}
        />

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          {orgs.length === 0 && !error ? (
            <div className="flex flex-col items-center justify-center py-16 text-zinc-600">
              <Building2 className="w-8 h-8 mb-3" />
              <p className="text-sm">No organizations found</p>
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800">
                  {['Name', 'Plan', 'Status', 'MRR', 'Members', 'Created', 'Actions'].map(h => (
                    <th key={h} className="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/50">
                {orgs.map(org => (
                  <tr key={org.id} className="bg-zinc-900 hover:bg-zinc-800/60 transition-colors">
                    <td className="px-4 py-3 font-medium text-zinc-100">
                      <div>{org.name}</div>
                      <div className="text-xs text-zinc-500">{org.slug}</div>
                    </td>
                    <td className="px-4 py-3"><Badge value={org.plan ?? 'free'} map={PLAN_BADGE} /></td>
                    <td className="px-4 py-3"><Badge value={org.status ?? 'active'} map={STATUS_BADGE} /></td>
                    <td className="px-4 py-3 text-zinc-300">{org.mrr_cents ? `$${(org.mrr_cents / 100).toFixed(2)}` : '—'}</td>
                    <td className="px-4 py-3 text-zinc-300">{org.member_count ?? '—'}</td>
                    <td className="px-4 py-3 text-zinc-400">{org.created_at ? new Date(org.created_at).toLocaleDateString() : '—'}</td>
                    <td className="px-4 py-3">
                      <Link href={`/orgs/${org.id}`} className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors inline-block">
                        View
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {orgs.length > 0 && (
          <PaginationBar page={page} total={total} pageSize={PAGE_SIZE} shown={orgs.length} />
        )}
      </div>
    </>
  )
}
