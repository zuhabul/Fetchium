import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Users } from 'lucide-react'
import FilterBar from '@/components/FilterBar'
import PaginationBar from '@/components/PaginationBar'

interface User {
  id: string
  email: string
  org_id: string
  org_name: string
  role: string
  is_active: boolean
  last_login_at: string | null
  created_at: string
}

const STATUS_BADGE: Record<string, string> = {
  active: 'bg-emerald-500/20 text-emerald-400',
  suspended: 'bg-red-500/20 text-red-400',
}

const ROLE_BADGE: Record<string, string> = {
  owner: 'bg-red-500/20 text-red-400',
  admin: 'bg-amber-500/20 text-amber-400',
  member: 'bg-blue-500/20 text-blue-400',
  viewer: 'bg-zinc-500/20 text-zinc-400',
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

export default async function UsersPage({ searchParams }: { searchParams: Promise<Record<string, string>> }) {
  const [session, params] = await Promise.all([getSession(), searchParams])

  const page = Math.max(1, parseInt(params.page ?? '1'))
  const offset = (page - 1) * PAGE_SIZE
  const filters = { search: params.search ?? '', status: params.status ?? '' }

  const qs = new URLSearchParams({ limit: String(PAGE_SIZE), offset: String(offset) })
  if (filters.search) qs.set('search', filters.search)
  if (filters.status) qs.set('status', filters.status)

  let users: User[] = []
  let total = 0
  let error: string | null = null

  try {
    if (session) {
      const res = await adminFetch(`/internal/admin/users?${qs}`)
      if (res.ok) {
        const data = await res.json()
        users = data.data ?? []
        total = data.total ?? users.length
      } else {
        error = `API error: ${res.status}`
      }
    }
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to fetch users'
  }

  return (
    <>
      <TopBar title="Users" subtitle={`${total} users total`} />
      <div className="p-6 space-y-4">
        <FilterBar
          filters={[
            { key: 'search', type: 'search', placeholder: 'Search users...' },
            { key: 'status', type: 'select', options: [{ value: '', label: 'All statuses' }, { value: 'active', label: 'Active' }, { value: 'suspended', label: 'Suspended' }] },
          ]}
          current={filters}
        />

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          {users.length === 0 && !error ? (
            <div className="flex flex-col items-center justify-center py-16 text-zinc-600">
              <Users className="w-8 h-8 mb-3" />
              <p className="text-sm">No users found</p>
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800">
                  {['Email', 'Org', 'Role', 'Status', 'Last Active', 'Created', 'Actions'].map(h => (
                    <th key={h} className="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/50">
                {users.map(user => (
                  <tr key={user.id} className="bg-zinc-900 hover:bg-zinc-800/60 transition-colors">
                    <td className="px-4 py-3 text-zinc-100 font-medium">{user.email}</td>
                    <td className="px-4 py-3">
                      {user.org_id ? (
                        <Link href={`/orgs/${user.org_id}`} className="text-blue-400 hover:underline text-sm">
                          {user.org_name ?? user.org_id}
                        </Link>
                      ) : '—'}
                    </td>
                    <td className="px-4 py-3"><Badge value={user.role ?? 'member'} map={ROLE_BADGE} /></td>
                    <td className="px-4 py-3"><Badge value={user.is_active ? 'active' : 'suspended'} map={STATUS_BADGE} /></td>
                    <td className="px-4 py-3 text-zinc-400">{user.last_login_at ? new Date(user.last_login_at).toLocaleDateString() : '—'}</td>
                    <td className="px-4 py-3 text-zinc-400">{user.created_at ? new Date(user.created_at).toLocaleDateString() : '—'}</td>
                    <td className="px-4 py-3">
                      <Link href={`/users/${user.id}`} className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors inline-block">
                        View
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {users.length > 0 && (
          <PaginationBar page={page} total={total} pageSize={PAGE_SIZE} shown={users.length} />
        )}
      </div>
    </>
  )
}
