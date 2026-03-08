import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Users } from 'lucide-react'

interface User {
  id: string
  email: string
  org_id: string
  org_name: string
  role: string
  status: string
  last_active_at: string | null
  created_at: string
}

interface UsersResponse {
  users: User[]
  total: number
  page: number
  per_page: number
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

export default async function UsersPage() {
  const session = await getSession()

  let data: UsersResponse = { users: [], total: 0, page: 1, per_page: 50 }
  let error: string | null = null

  try {
    if (session) {
      const res = await adminFetch('/internal/admin/users?page=1&per_page=50')
      if (res.ok) {
        data = await res.json()
      } else {
        error = `API error: ${res.status}`
      }
    }
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to fetch users'
  }

  const users = data.users ?? []

  return (
    <>
      <TopBar title="Users" subtitle={`${data.total ?? users.length} users total`} />
      <div className="p-6 space-y-4">
        {/* Filter bar */}
        <div className="flex items-center gap-3 flex-wrap">
          <input
            type="search"
            placeholder="Search users..."
            className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 w-56"
          />
          <select className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-300 focus:outline-none focus:border-zinc-500">
            <option value="">All statuses</option>
            <option value="active">Active</option>
            <option value="suspended">Suspended</option>
          </select>
        </div>

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* Table */}
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
                    <th key={h} className="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">
                      {h}
                    </th>
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
                    <td className="px-4 py-3">
                      <Badge value={user.role ?? 'member'} map={ROLE_BADGE} />
                    </td>
                    <td className="px-4 py-3">
                      <Badge value={user.status ?? 'active'} map={STATUS_BADGE} />
                    </td>
                    <td className="px-4 py-3 text-zinc-400">
                      {user.last_active_at ? new Date(user.last_active_at).toLocaleDateString() : '—'}
                    </td>
                    <td className="px-4 py-3 text-zinc-400">
                      {user.created_at ? new Date(user.created_at).toLocaleDateString() : '—'}
                    </td>
                    <td className="px-4 py-3">
                      <Link
                        href={`/users/${user.id}`}
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
        {users.length > 0 && (
          <div className="flex items-center justify-between text-sm text-zinc-500">
            <span>Showing {users.length} of {data.total ?? users.length}</span>
            <div className="flex items-center gap-2">
              <button disabled className="bg-zinc-800 border border-zinc-700 text-zinc-500 text-sm px-3 py-1.5 rounded-md disabled:opacity-40">
                Previous
              </button>
              <button
                disabled={users.length < (data.per_page ?? 50)}
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
