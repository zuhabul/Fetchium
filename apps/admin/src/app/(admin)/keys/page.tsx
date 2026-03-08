import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Key } from 'lucide-react'
import RevokeButton from './RevokeButton'

interface ApiKey {
  id: string
  key_prefix: string
  org_id: string
  org_name: string
  plan: string
  status: string
  created_at: string
  last_used_at: string | null
  requests_this_month: number
}

interface KeysResponse {
  keys: ApiKey[]
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
  revoked: 'bg-red-500/20 text-red-400',
}

function Badge({ value, map }: { value: string; map: Record<string, string> }) {
  const cls = map[value] ?? 'bg-zinc-500/20 text-zinc-400'
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium capitalize ${cls}`}>
      {value}
    </span>
  )
}

export default async function KeysPage() {
  const session = await getSession()

  let data: KeysResponse = { keys: [], total: 0, page: 1, per_page: 50 }
  let error: string | null = null

  try {
    if (session) {
      const res = await adminFetch('/internal/admin/keys?page=1&per_page=50')
      if (res.ok) {
        data = await res.json()
      } else {
        error = `API error: ${res.status}`
      }
    }
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to fetch keys'
  }

  const keys = data.keys ?? []

  return (
    <>
      <TopBar title="API Keys" subtitle={`${data.total ?? keys.length} keys total`} />
      <div className="p-6 space-y-4">
        {/* Filter bar */}
        <div className="flex items-center gap-3 flex-wrap">
          <select className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-300 focus:outline-none focus:border-zinc-500">
            <option value="">All statuses</option>
            <option value="active">Active</option>
            <option value="revoked">Revoked</option>
          </select>
          <select className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-300 focus:outline-none focus:border-zinc-500">
            <option value="">All plans</option>
            <option value="free">Free</option>
            <option value="starter">Starter</option>
            <option value="pro">Pro</option>
            <option value="enterprise">Enterprise</option>
          </select>
        </div>

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* Table */}
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
                  {['Key ID', 'Org', 'Plan', 'Status', 'Created', 'Last Used', 'Req/Mo', 'Actions'].map(h => (
                    <th key={h} className="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/50">
                {keys.map(k => (
                  <tr key={k.id} className="bg-zinc-900 hover:bg-zinc-800/60 transition-colors">
                    <td className="px-4 py-3">
                      <span className="font-mono text-xs text-zinc-300">
                        {k.key_prefix ? `${k.key_prefix.slice(0, 8)}...` : k.id.slice(0, 8) + '...'}
                      </span>
                    </td>
                    <td className="px-4 py-3">
                      {k.org_id ? (
                        <Link href={`/orgs/${k.org_id}`} className="text-blue-400 hover:underline text-sm">
                          {k.org_name ?? k.org_id}
                        </Link>
                      ) : '—'}
                    </td>
                    <td className="px-4 py-3">
                      <Badge value={k.plan ?? 'free'} map={PLAN_BADGE} />
                    </td>
                    <td className="px-4 py-3">
                      <Badge value={k.status ?? 'active'} map={STATUS_BADGE} />
                    </td>
                    <td className="px-4 py-3 text-zinc-400">
                      {k.created_at ? new Date(k.created_at).toLocaleDateString() : '—'}
                    </td>
                    <td className="px-4 py-3 text-zinc-400">
                      {k.last_used_at ? new Date(k.last_used_at).toLocaleDateString() : '—'}
                    </td>
                    <td className="px-4 py-3 text-zinc-300">
                      {k.requests_this_month != null ? k.requests_this_month.toLocaleString() : '—'}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        {k.status !== 'revoked' && <RevokeButton keyId={k.id} />}
                        <Link
                          href={`/orgs/${k.org_id}`}
                          className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-xs px-2.5 py-1.5 rounded-md transition-colors"
                        >
                          View
                        </Link>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Pagination */}
        {keys.length > 0 && (
          <div className="flex items-center justify-between text-sm text-zinc-500">
            <span>Showing {keys.length} of {data.total ?? keys.length}</span>
            <div className="flex items-center gap-2">
              <button disabled className="bg-zinc-800 border border-zinc-700 text-zinc-500 text-sm px-3 py-1.5 rounded-md disabled:opacity-40">
                Previous
              </button>
              <button
                disabled={keys.length < (data.per_page ?? 50)}
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
