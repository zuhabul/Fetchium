import { redirect } from 'next/navigation'
import Link from 'next/link'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'

interface Ticket {
  id: string
  subject: string
  org_id: string
  org_name: string
  priority: 'low' | 'normal' | 'high' | 'urgent'
  status: 'open' | 'pending' | 'resolved' | 'closed'
  assignee?: string
  sla_due_at?: string
  created_at: string
}

function asString(value: unknown, fallback = '') {
  return typeof value === 'string' ? value : fallback
}

function normalizeTickets(payload: unknown): Ticket[] {
  const body = typeof payload === 'object' && payload !== null
    ? payload as Record<string, unknown>
    : {}

  const rows = Array.isArray(body.data)
    ? body.data
    : Array.isArray(payload)
      ? payload
      : []

  return rows.map((row, index) => {
    const item = typeof row === 'object' && row !== null
      ? row as Record<string, unknown>
      : {}

    return {
      id: asString(item.id, `ticket-${index}`),
      subject: asString(item.subject, 'Untitled ticket'),
      org_id: asString(item.org_id),
      org_name: asString(item.org_name, 'Unknown organization'),
      priority: (asString(item.priority, 'normal') as Ticket['priority']),
      status: (asString(item.status, 'open') as Ticket['status']),
      assignee: asString(item.assignee ?? item.assignee_id) || undefined,
      sla_due_at: asString(item.sla_due_at) || undefined,
      created_at: asString(item.created_at, new Date(0).toISOString()),
    }
  })
}

function formatTicketDate(value: string) {
  return new Date(value).toLocaleDateString('en-US', {
    month: 'short', day: 'numeric', year: 'numeric',
  })
}

function PriorityBadge({ priority }: { priority: string }) {
  const cls: Record<string, string> = {
    urgent: 'bg-red-500/20 text-red-400 border-red-500/30',
    high: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
    normal: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    low: 'bg-zinc-800 text-zinc-400 border-zinc-700',
  }
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${cls[priority] ?? cls.low}`}>
      {priority}
    </span>
  )
}

function SlaCountdown({ sla_due_at }: { sla_due_at?: string }) {
  if (!sla_due_at) return <span className="text-zinc-600 text-sm">—</span>
  const msLeft = new Date(sla_due_at).getTime() - Date.now()
  if (msLeft <= 0) {
    return <span className="text-xs font-semibold text-red-400">BREACHED</span>
  }
  const hLeft = Math.floor(msLeft / 3600000)
  const mLeft = Math.floor((msLeft % 3600000) / 60000)
  const color = hLeft < 2 ? 'text-red-400' : hLeft < 8 ? 'text-amber-400' : 'text-zinc-400'
  return (
    <span className={`text-xs font-medium ${color}`}>
      {hLeft}h {mLeft}m remaining
    </span>
  )
}

const TABS = ['all', 'open', 'pending', 'urgent', 'unassigned'] as const
type Tab = typeof TABS[number]

export default async function SupportPage({
  searchParams,
}: {
  searchParams: Promise<{ tab?: string }>
}) {
  const session = await getSession()
  if (!session) redirect('/login')

  const params = await searchParams
  const activeTab = (params.tab ?? 'all') as Tab

  let tickets: Ticket[] = []
  let error = false

  try {
    const res = await adminFetch('/internal/admin/support/tickets')
    if (res.ok) tickets = normalizeTickets(await res.json())
    else error = true
  } catch {
    error = true
  }

  const filtered = tickets.filter((t) => {
    if (activeTab === 'all') return true
    if (activeTab === 'open') return t.status === 'open'
    if (activeTab === 'pending') return t.status === 'pending'
    if (activeTab === 'urgent') return t.priority === 'urgent'
    if (activeTab === 'unassigned') return !t.assignee
    return true
  })

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="Support" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-6`}>
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm">
            Failed to load data
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
          <div className="space-y-4 border-b border-zinc-800 p-4">
            <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <div>
                <h2 className="text-sm font-semibold text-zinc-100">Tickets</h2>
                <p className="mt-1 text-xs text-zinc-500">{filtered.length} matching conversations</p>
              </div>
              <Link
                href="/support/new"
                className="inline-flex min-h-11 items-center justify-center rounded-md border border-blue-500/30 bg-blue-500/20 px-3 py-2 text-sm text-blue-300 transition-colors hover:bg-blue-500/30 sm:min-h-9 sm:self-start sm:py-1.5 lg:self-auto"
              >
                + New Ticket
              </Link>
            </div>

            <div className="flex flex-wrap gap-2">
              {TABS.map((tab) => (
                <Link
                  key={tab}
                  href={`/support?tab=${tab}`}
                  className={`inline-flex min-h-10 items-center rounded-md border px-3 py-2 text-xs capitalize transition-colors sm:min-h-8 sm:py-1 ${
                    activeTab === tab
                      ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                      : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  {tab}
                </Link>
              ))}
            </div>
          </div>

          {filtered.length === 0 ? (
            <div className="px-4 py-12 text-center text-sm text-zinc-500">
              No tickets found
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/60 lg:hidden">
                {filtered.map((t) => (
                  <div key={t.id} className="space-y-4 px-4 py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 space-y-1">
                        <p className="font-mono text-[11px] uppercase tracking-wider text-zinc-600">
                          #{t.id.slice(0, 8)}
                        </p>
                        <Link
                          href={`/support/${t.id}`}
                          className="block line-clamp-2 text-sm font-medium text-blue-400 hover:text-blue-300"
                        >
                          {t.subject}
                        </Link>
                        <p className="truncate text-xs text-zinc-500">{t.org_name}</p>
                      </div>
                      <PriorityBadge priority={t.priority} />
                    </div>

                    <dl className="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Assignee</dt>
                        <dd className="mt-1 text-zinc-400">{t.assignee ?? 'Unassigned'}</dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Created</dt>
                        <dd className="mt-1 text-zinc-400">{formatTicketDate(t.created_at)}</dd>
                      </div>
                      <div className="col-span-2">
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">SLA</dt>
                        <dd className="mt-1"><SlaCountdown sla_due_at={t.sla_due_at} /></dd>
                      </div>
                    </dl>
                  </div>
                ))}
              </div>

              <table className="hidden w-full lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['#ID', 'Subject', 'Org', 'Priority', 'Assignee', 'SLA', 'Created'].map((h) => (
                      <th key={h} className="text-xs font-medium text-zinc-500 uppercase tracking-wider px-3 py-2 text-left">
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((t) => (
                    <tr key={t.id} className="hover:bg-zinc-800/40 border-b border-zinc-800/60">
                      <td className="px-3 py-2.5 text-xs font-mono text-zinc-500">#{t.id.slice(0, 8)}</td>
                      <td className="px-3 py-2.5">
                        <Link href={`/support/${t.id}`} className="text-sm text-blue-400 hover:text-blue-300 line-clamp-1">
                          {t.subject}
                        </Link>
                      </td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{t.org_name}</td>
                      <td className="px-3 py-2.5"><PriorityBadge priority={t.priority} /></td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{t.assignee ?? <span className="text-zinc-600">Unassigned</span>}</td>
                      <td className="px-3 py-2.5"><SlaCountdown sla_due_at={t.sla_due_at} /></td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{formatTicketDate(t.created_at)}</td>
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
