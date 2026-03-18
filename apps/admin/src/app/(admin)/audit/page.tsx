import { redirect } from 'next/navigation'
import Link from 'next/link'
import { Activity, Shield, UserRound } from 'lucide-react'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
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
  return new Date(date).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function is_write_action(action: string) {
  return (
    action.includes('create') ||
    action.includes('update') ||
    action.includes('delete') ||
    action.includes('suspend') ||
    action.includes('reset') ||
    action.includes('revoke')
  )
}

function ActionBadge({ action }: { action: string }) {
  return (
    <span
      className={`rounded border px-2 py-0.5 font-mono text-xs ${
        is_write_action(action)
          ? 'border-amber-500/20 bg-amber-500/10 text-amber-400'
          : 'border-zinc-700 bg-zinc-800 text-zinc-400'
      }`}
    >
      {action}
    </span>
  )
}

function StatCard({
  label,
  value,
  icon: Icon,
  color,
}: {
  label: string
  value: string
  icon: React.ElementType
  color: string
}) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
      <div className="mb-3 flex items-center justify-between gap-3">
        <span className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">{label}</span>
        <div className={`flex h-8 w-8 items-center justify-center rounded-lg ${color}`}>
          <Icon className="h-4 w-4" />
        </div>
      </div>
      <p className="text-[1.75rem] font-bold leading-none text-zinc-100 lg:text-2xl">{value}</p>
    </div>
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
  const currentPage = Math.max(parseInt(page) || 1, 1)
  const limit = 50
  const offset = (currentPage - 1) * limit

  let events: AuditEvent[] = []
  let total = 0
  try {
    const res = await adminFetch(`/internal/admin/audit?limit=${limit}&offset=${offset}`)
    if (res.ok) {
      const body = await res.json()
      events = Array.isArray(body?.data) ? body.data : []
      total = typeof body?.total === 'number' ? body.total : 0
    }
  } catch {}

  const pages = Math.max(Math.ceil(total / limit), 1)
  const writeActions = events.filter(ev => is_write_action(ev.action)).length
  const uniqueActors = new Set(events.map(ev => ev.user_email ?? ev.user_name ?? 'system')).size
  const rangeStart = total === 0 ? 0 : offset + 1
  const rangeEnd = total === 0 ? 0 : Math.min(offset + events.length, total)

  return (
    <div className="flex min-h-full flex-col">
      <TopBar title="Audit Log" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>
        <div className="grid grid-cols-1 gap-3 min-[420px]:grid-cols-2 xl:grid-cols-4">
          <StatCard
            label="Events"
            value={total.toLocaleString()}
            icon={Activity}
            color="bg-blue-500/20 text-blue-400"
          />
          <StatCard
            label="This Page"
            value={events.length.toLocaleString()}
            icon={Shield}
            color="bg-emerald-500/20 text-emerald-400"
          />
          <StatCard
            label="Write Actions"
            value={writeActions.toLocaleString()}
            icon={Activity}
            color="bg-amber-500/20 text-amber-400"
          />
          <StatCard
            label="Actors"
            value={uniqueActors.toLocaleString()}
            icon={UserRound}
            color="bg-purple-500/20 text-purple-400"
          />
        </div>

        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <h2 className="text-sm font-semibold text-zinc-100">Event Stream</h2>
              <p className="mt-1 text-xs text-zinc-500">
                {total.toLocaleString()} events across the admin surface
              </p>
            </div>

            <div className="flex flex-wrap gap-2">
              {currentPage > 1 && (
                <Link
                  href={`?page=${currentPage - 1}`}
                  className="inline-flex min-h-11 items-center justify-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 sm:min-h-9 sm:py-1.5"
                >
                  Previous
                </Link>
              )}
              {currentPage < pages && (
                <Link
                  href={`?page=${currentPage + 1}`}
                  className="inline-flex min-h-11 items-center justify-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 sm:min-h-9 sm:py-1.5"
                >
                  Next
                </Link>
              )}
            </div>
          </div>

          <div className="mt-4 flex flex-col gap-1 text-xs text-zinc-500 sm:flex-row sm:items-center sm:justify-between">
            <p>
              Showing {rangeStart.toLocaleString()}-{rangeEnd.toLocaleString()} of{' '}
              {total.toLocaleString()}
            </p>
            {pages > 1 && <p>Page {currentPage} of {pages}</p>}
          </div>
        </div>

        <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900">
          {events.length === 0 ? (
            <div className="px-4 py-14 text-center">
              <p className="text-sm text-zinc-400">
                No audit events yet. Actions performed in the admin panel will appear here.
              </p>
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/60 lg:hidden">
                {events.map(ev => (
                  <div key={ev.id} className="space-y-4 px-4 py-4">
                    <div className="space-y-1">
                      <p className="text-xs text-zinc-500">{fmt(ev.created_at)}</p>
                      <p className="text-sm font-medium text-zinc-200">
                        {ev.user_name ?? 'System'}
                      </p>
                      <p className="break-all text-xs text-zinc-500">
                        {ev.user_email ?? 'system'}
                      </p>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <ActionBadge action={ev.action} />
                      <span className="rounded border border-zinc-700 bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
                        {ev.role ?? 'no role'}
                      </span>
                    </div>

                    <dl className="grid grid-cols-1 gap-3 text-sm min-[460px]:grid-cols-2">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Target</dt>
                        <dd className="mt-1 break-all text-zinc-400">
                          <span className="text-zinc-500">{ev.target_type}</span>
                          {ev.target_id && (
                            <span className="ml-1 font-mono text-zinc-400">{ev.target_id}</span>
                          )}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">IP</dt>
                        <dd className="mt-1 font-mono text-zinc-400">{ev.ip ?? '—'}</dd>
                      </div>
                    </dl>
                  </div>
                ))}
              </div>

              <table className="hidden w-full text-sm lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['Time', 'Actor', 'Role', 'Action', 'Target', 'IP'].map(h => (
                      <th
                        key={h}
                        className="px-4 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-zinc-500"
                      >
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {events.map(ev => (
                    <tr key={ev.id} className="border-b border-zinc-800/60 hover:bg-zinc-800/30">
                      <td className="whitespace-nowrap px-4 py-2.5 text-xs text-zinc-500">
                        {fmt(ev.created_at)}
                      </td>
                      <td className="px-4 py-2.5">
                        <div className="text-xs text-zinc-300">{ev.user_name ?? '—'}</div>
                        <div className="text-xs text-zinc-600">{ev.user_email ?? 'system'}</div>
                      </td>
                      <td className="px-4 py-2.5 text-xs text-zinc-500">{ev.role ?? '—'}</td>
                      <td className="px-4 py-2.5">
                        <ActionBadge action={ev.action} />
                      </td>
                      <td className="px-4 py-2.5 text-xs text-zinc-400">
                        <span className="text-zinc-600">{ev.target_type}</span>
                        {ev.target_id && (
                          <span className="ml-1 font-mono text-zinc-500">
                            {ev.target_id.slice(0, 8)}…
                          </span>
                        )}
                      </td>
                      <td className="px-4 py-2.5 text-xs font-mono text-zinc-600">
                        {ev.ip ?? '—'}
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
