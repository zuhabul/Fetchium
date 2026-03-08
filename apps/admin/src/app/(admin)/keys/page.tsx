import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Key } from 'lucide-react'
import RevokeButton from './RevokeButton'
import FilterBar from '@/components/FilterBar'
import PaginationBar from '@/components/PaginationBar'

interface ApiKey {
  id: string
  key_prefix: string
  org_id: string
  org_name: string
  plan: string
  active: boolean
  created_at: string
  last_used_at: string | null
  requests_this_month: number
}

const PLAN_BADGE: Record<string, string> = {
  free: 'bg-zinc-500/20 text-zinc-400',
  starter: 'bg-blue-500/20 text-blue-400',
  pro: 'bg-purple-500/20 text-purple-400',
  enterprise: 'bg-amber-500/20 text-amber-400',
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

export default async function KeysPage({ searchParams }: { searchParams: Promise<Record<string, string>> }) {
  const [session, params] = await Promise.all([getSession(), searchParams])
  if (!session) redirect('/login')

  const page = Math.max(1, parseInt(params.page ?? '1'))
  const offset = (page - 1) * PAGE_SIZE
  const filters = { status: params.status ?? '', plan: params.plan ?? '' }

  const qs = new URLSearchParams({ limit: String(PAGE_SIZE), offset: String(offset) })
  if (filters.status) qs.set('status', filters.status)
  if (filters.plan) qs.set('plan', filters.plan)

  let keys: ApiKey[] = []
  let total = 0
  let error: string | null = null

  try {
    const res = await adminFetch(`/internal/admin/keys?${qs}`)
    if (res.ok) {
      const data = await res.json()
      keys = data.data ?? []
      total = data.total ?? keys.length
    } else {
      error = `API error: ${res.status}`
    }
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to fetch keys'
  }

  return (
    <>
      <TopBar title="API Keys" subtitle={`${total} keys total`} />
      <div className="p-6 space-y-4">
        <FilterBar
          filters={[
            { key: 'status', type: 'select', options: [{ value: '', label: 'All statuses' }, { value: 'active', label: 'Active' }, { value: 'revoked', label: 'Revoked' }] },
            { key: 'plan', type: 'select', options: [{ value: '', label: 'All plans' }, { value: 'free', label: 'Free' }, { value: 'starter', label: 'Starter' }, { value: 'pro', label: 'Pro' }, { value: 'enterprise', label: 'Enterprise' }] },
          ]}
          current={filters}
        />

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          {keys.length === 0 && !error ? (
            <div className="flex flex-col items-center justify-center py-16 text-zinc-600">
              <Key className="w-8 h-8 mb-3" />
              <p className="text-sm">No API keys found</p>
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800">
                  {['Key Prefix', 'Org', 'Plan', 'Status', 'Created', 'Last Used', 'Actions'].map(h => (
                    <th key={h} className="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/50">
                {keys.map(k => (
                  <tr key={k.id} className="bg-zinc-900 hover:bg-zinc-800/60 transition-colors">
                    <td className="px-4 py-3">
                      <span className="font-mono text-xs text-zinc-300">{k.key_prefix}…</span>
                    </td>
                    <td className="px-4 py-3">
                      {k.org_id ? (
                        <Link href={`/orgs/${k.org_id}`} className="text-blue-400 hover:underline text-sm">
                          {k.org_name ?? k.org_id}
                        </Link>
                      ) : '—'}
                    </td>
                    <td className="px-4 py-3"><Badge value={k.plan ?? 'free'} map={PLAN_BADGE} /></td>
                    <td className="px-4 py-3">
                      <span className={`text-xs px-1.5 py-0.5 rounded ${k.active ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'}`}>
                        {k.active ? 'active' : 'revoked'}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-zinc-400">{k.created_at ? new Date(k.created_at).toLocaleDateString() : '—'}</td>
                    <td className="px-4 py-3 text-zinc-400">{k.last_used_at ? new Date(k.last_used_at).toLocaleDateString() : '—'}</td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        {k.active && <RevokeButton keyId={k.id} />}
                        <Link href={`/orgs/${k.org_id}`} className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-xs px-2.5 py-1.5 rounded-md transition-colors">
                          Org
                        </Link>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {keys.length > 0 && (
          <PaginationBar page={page} total={total} pageSize={PAGE_SIZE} shown={keys.length} />
        )}
      </div>
    </>
  )
}
