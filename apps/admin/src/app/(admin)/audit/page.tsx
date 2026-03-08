import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'

interface AuditEvent {
  id: string
  user_email: string | null
  user_name: string | null
  role: string | null
  target_type: string
  target_id: string | null
  action: string
  ip: string | null
  created_at: string
}

function fmt(date: string) {
  return new Date(date).toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function ActionBadge({ action }: { action: string }) {
  const isWrite = action.includes('create') || action.includes('update') || action.includes('delete') || action.includes('suspend') || action.includes('reset') || action.includes('revoke')
  return (
    <span className={`text-xs font-mono px-2 py-0.5 rounded border ${isWrite ? 'bg-amber-500/10 text-amber-400 border-amber-500/20' : 'bg-zinc-800 text-zinc-400 border-zinc-700'}`}>
      {action}
    </span>
  )
}

export default async function AuditPage({
  searchParams,
}: {
  searchParams: Promise<{ page?: string }>
}) {
  const session = await getSession()
  if (!session) redirect('/login')

  const { page = '1' } = await searchParams
  const limit = 50
  const offset = (parseInt(page) - 1) * limit

  let events: AuditEvent[] = []
  let total = 0
  try {
    const res = await adminFetch(`/internal/admin/audit?limit=${limit}&offset=${offset}`)
    if (res.ok) {
      const body = await res.json()
      events = body.data ?? []
      total = body.total ?? 0
    }
  } catch {}

  const pages = Math.ceil(total / limit)
  const currentPage = parseInt(page)

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="Audit Log" />
      <div className="p-6 space-y-4">
        <div className="flex items-center justify-between">
          <p className="text-sm text-zinc-500">{total.toLocaleString()} events total</p>
          <div className="flex gap-2">
            {currentPage > 1 && (
              <a href={`?page=${currentPage - 1}`} className="text-xs px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-lg border border-zinc-700">
                ← Prev
              </a>
            )}
            {currentPage < pages && (
              <a href={`?page=${currentPage + 1}`} className="text-xs px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-lg border border-zinc-700">
                Next →
              </a>
            )}
          </div>
        </div>

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-zinc-800">
                {['Time', 'Actor', 'Role', 'Action', 'Target', 'IP'].map(h => (
                  <th key={h} className="text-left text-xs font-medium text-zinc-500 uppercase tracking-wider px-4 py-2.5">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {events.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-4 py-10 text-center text-zinc-500 text-sm">
                    No audit events yet. Actions performed in the admin panel will appear here.
                  </td>
                </tr>
              ) : events.map(ev => (
                <tr key={ev.id} className="border-b border-zinc-800/60 hover:bg-zinc-800/30">
                  <td className="px-4 py-2.5 text-xs text-zinc-500 whitespace-nowrap">{fmt(ev.created_at)}</td>
                  <td className="px-4 py-2.5">
                    <div className="text-xs text-zinc-300">{ev.user_name ?? '—'}</div>
                    <div className="text-xs text-zinc-600">{ev.user_email ?? 'system'}</div>
                  </td>
                  <td className="px-4 py-2.5 text-xs text-zinc-500">{ev.role ?? '—'}</td>
                  <td className="px-4 py-2.5"><ActionBadge action={ev.action} /></td>
                  <td className="px-4 py-2.5 text-xs text-zinc-400">
                    <span className="text-zinc-600">{ev.target_type}</span>
                    {ev.target_id && <span className="ml-1 font-mono text-zinc-500">{ev.target_id.slice(0, 8)}…</span>}
                  </td>
                  <td className="px-4 py-2.5 text-xs font-mono text-zinc-600">{ev.ip ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {pages > 1 && (
          <p className="text-xs text-center text-zinc-600">Page {currentPage} of {pages}</p>
        )}
      </div>
    </div>
  )
}
