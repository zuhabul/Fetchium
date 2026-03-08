import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Building2, Plus } from 'lucide-react'

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

interface OrgsResponse {
  orgs: Org[]
  total: number
  page: number
  per_page: number
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

export default async function OrgsPage() {
  const session = await getSession()

  let data: OrgsResponse = { orgs: [], total: 0, page: 1, per_page: 50 }
  let error: string | null = null

  try {
    if (session) {
      const res = await adminFetch('/internal/admin/orgs?page=1&per_page=50')
      if (res.ok) {
        data = await res.json()
      } else {
        error = `API error: ${res.status}`
      }
    }
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to fetch orgs'
  }

  const orgs = data.orgs ?? []

  return (
    <>
      <TopBar title="Organizations" subtitle={`${data.total ?? orgs.length} orgs total`} />
      <div className="p-6 space-y-4">
        {/* Filter bar + actions */}
        <div className="flex items-center gap-3 flex-wrap">
          <input
            type="search"
            placeholder="Search orgs..."
            className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 w-56"
          />
          <select className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-300 focus:outline-none focus:border-zinc-500">
            <option value="">All plans</option>
            <option value="free">Free</option>
            <option value="starter">Starter</option>
            <option value="pro">Pro</option>
            <option value="enterprise">Enterprise</option>
          </select>
          <select className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-300 focus:outline-none focus:border-zinc-500">
            <option value="">All statuses</option>
            <option value="active">Active</option>
            <option value="suspended">Suspended</option>
            <option value="trial">Trial</option>
            <option value="churned">Churned</option>
          </select>
          <div className="flex-1" />
          {session?.role === 'owner' && (
            <button className="flex items-center gap-1.5 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors">
              <Plus className="w-3.5 h-3.5" />
              New Org
            </button>
          )}
        </div>

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* Table */}
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
                    <th key={h} className="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">
                      {h}
                    </th>
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
                    <td className="px-4 py-3">
                      <Badge value={org.plan ?? 'free'} map={PLAN_BADGE} />
                    </td>
                    <td className="px-4 py-3">
                      <Badge value={org.status ?? 'active'} map={STATUS_BADGE} />
                    </td>
                    <td className="px-4 py-3 text-zinc-300">
                      {org.mrr_cents ? `$${(org.mrr_cents / 100).toFixed(2)}` : '—'}
                    </td>
                    <td className="px-4 py-3 text-zinc-300">
                      {org.member_count ?? '—'}
                    </td>
                    <td className="px-4 py-3 text-zinc-400">
                      {org.created_at ? new Date(org.created_at).toLocaleDateString() : '—'}
                    </td>
                    <td className="px-4 py-3">
                      <Link
                        href={`/orgs/${org.id}`}
                        className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors inline-block"
                      >
                        View
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Pagination */}
        {orgs.length > 0 && (
          <div className="flex items-center justify-between text-sm text-zinc-500">
            <span>Showing {orgs.length} of {data.total ?? orgs.length}</span>
            <div className="flex items-center gap-2">
              <button
                disabled
                className="bg-zinc-800 border border-zinc-700 text-zinc-500 text-sm px-3 py-1.5 rounded-md disabled:opacity-40"
              >
                Previous
              </button>
              <button
                disabled={orgs.length < (data.per_page ?? 50)}
                className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md disabled:opacity-40 transition-colors"
              >
                Next
              </button>
            </div>
          </div>
        )}
      </div>
    </>
  )
}
