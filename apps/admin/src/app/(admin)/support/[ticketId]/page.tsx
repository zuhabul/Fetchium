import { redirect, notFound } from 'next/navigation'
import Link from 'next/link'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import TicketActions from './TicketActions'

interface TicketNote {
  id: string
  author: string
  body: string
  internal: boolean
  created_at: string
}

interface TicketDetail {
  id: string
  subject: string
  org_id: string
  org_name: string
  status: string
  priority: string
  assignee?: string
  sla_due_at?: string
  created_at: string
  notes: TicketNote[]
}

function asString(value: unknown, fallback = '') {
  return typeof value === 'string' ? value : fallback
}

function asBoolean(value: unknown, fallback = false) {
  return typeof value === 'boolean' ? value : fallback
}

function normalizeNotes(value: unknown): TicketNote[] {
  if (!Array.isArray(value)) return []

  return value.map((note, index) => {
    const item = typeof note === 'object' && note !== null
      ? note as Record<string, unknown>
      : {}

    return {
      id: asString(item.id, `note-${index}`),
      author: asString(item.author, 'Unknown'),
      body: asString(item.body),
      internal: asBoolean(item.internal),
      created_at: asString(item.created_at, new Date(0).toISOString()),
    }
  })
}

function normalizeTicket(payload: unknown): TicketDetail | null {
  const body = typeof payload === 'object' && payload !== null
    ? payload as Record<string, unknown>
    : {}

  const rawTicket = typeof body.data === 'object' && body.data !== null
    ? body.data as Record<string, unknown>
    : typeof payload === 'object' && payload !== null
      ? payload as Record<string, unknown>
      : null

  if (!rawTicket) return null

  return {
    id: asString(rawTicket.id),
    subject: asString(rawTicket.subject, 'Ticket'),
    org_id: asString(rawTicket.org_id),
    org_name: asString(rawTicket.org_name, 'Unknown organization'),
    status: asString(rawTicket.status, 'open'),
    priority: asString(rawTicket.priority, 'normal'),
    assignee: asString(rawTicket.assignee ?? rawTicket.assignee_id) || undefined,
    sla_due_at: asString(rawTicket.sla_due_at) || undefined,
    created_at: asString(rawTicket.created_at, new Date(0).toISOString()),
    notes: normalizeNotes(body.notes ?? rawTicket.notes),
  }
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

function StatusBadge({ status }: { status: string }) {
  const cls: Record<string, string> = {
    open: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    pending: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
    resolved: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    closed: 'bg-zinc-800 text-zinc-400 border-zinc-700',
  }
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${cls[status] ?? cls.closed}`}>
      {status}
    </span>
  )
}

function SlaStatus({ sla_due_at }: { sla_due_at?: string }) {
  if (!sla_due_at) return <span className="text-zinc-500 text-sm">No SLA set</span>
  const msLeft = new Date(sla_due_at).getTime() - Date.now()
  if (msLeft <= 0) return <span className="text-sm font-semibold text-red-400">BREACHED</span>
  const hLeft = Math.floor(msLeft / 3600000)
  const mLeft = Math.floor((msLeft % 3600000) / 60000)
  const color = hLeft < 2 ? 'text-red-400' : hLeft < 8 ? 'text-amber-400' : 'text-emerald-400'
  return <span className={`text-sm font-medium ${color}`}>{hLeft}h {mLeft}m remaining</span>
}

function fmt(date: string) {
  return new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

function SidebarRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex justify-between items-start py-2 border-b border-zinc-800/60 last:border-0 gap-2">
      <span className="text-xs text-zinc-500 uppercase tracking-wider font-medium shrink-0">{label}</span>
      <span className="text-sm text-zinc-300 text-right">{children}</span>
    </div>
  )
}

export default async function TicketDetailPage({
  params,
}: {
  params: Promise<{ ticketId: string }>
}) {
  const session = await getSession()
  if (!session) redirect('/login')

  const { ticketId } = await params
  let ticket: TicketDetail | null = null
  let error = false

  try {
    const res = await adminFetch(`/internal/admin/support/tickets/${ticketId}`)
    if (res.status === 404) notFound()
    if (res.ok) ticket = normalizeTicket(await res.json())
    else error = true
  } catch {
    error = true
  }

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title={ticket?.subject ?? 'Ticket'} />
      <div className={ADMIN_PAGE_PADDING}>
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm mb-6">
            Failed to load data
          </div>
        )}

        {ticket && (
          <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:gap-6">
            {/* Sidebar — shown first on mobile, right on desktop */}
            <div className="order-first lg:order-last lg:flex-[35] lg:sticky lg:top-6 space-y-4 min-w-0">
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
                <h2 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-3">
                  Ticket Info
                </h2>
                <SidebarRow label="Status"><StatusBadge status={ticket.status} /></SidebarRow>
                <SidebarRow label="Priority"><PriorityBadge priority={ticket.priority} /></SidebarRow>
                <SidebarRow label="Assignee">{ticket.assignee ?? <span className="text-zinc-500">Unassigned</span>}</SidebarRow>
                <SidebarRow label="Org">
                  <Link href={`/orgs/${ticket.org_id}`} className="text-blue-400 hover:text-blue-300">
                    {ticket.org_name}
                  </Link>
                </SidebarRow>
                <SidebarRow label="SLA"><SlaStatus sla_due_at={ticket.sla_due_at} /></SidebarRow>
                <SidebarRow label="Created">{fmt(ticket.created_at)}</SidebarRow>
              </div>
            </div>

            {/* Thread — full width on mobile, 65% on desktop */}
            <div className="order-last lg:order-first lg:flex-[65] space-y-4 min-w-0">
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
                <h1 className="text-base font-semibold text-zinc-100 mb-1">{ticket.subject}</h1>
                <div className="flex flex-wrap items-center gap-2">
                  <StatusBadge status={ticket.status} />
                  <PriorityBadge priority={ticket.priority} />
                  <span className="text-xs text-zinc-600">#{ticket.id.slice(0, 8)}</span>
                </div>
              </div>

              {/* Notes thread */}
              <div className="space-y-3">
                {ticket.notes.length === 0 ? (
                  <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6 text-center text-zinc-500 text-sm">
                    No messages yet
                  </div>
                ) : (
                  ticket.notes.map((note) => (
                    <div
                      key={note.id}
                      className={`bg-zinc-900 border rounded-xl p-4 ${
                        note.internal ? 'border-zinc-700 opacity-90' : 'border-zinc-800'
                      }`}
                    >
                      <div className="flex flex-wrap items-center gap-2 mb-2">
                        <span className="text-xs font-medium text-zinc-300">{note.author}</span>
                        {note.internal && (
                          <span className="text-xs px-1.5 py-0.5 rounded bg-zinc-700 text-zinc-400 border border-zinc-600">
                            internal
                          </span>
                        )}
                        <span className="text-xs text-zinc-600 ml-auto">{fmt(note.created_at)}</span>
                      </div>
                      <p className="text-sm text-zinc-300 whitespace-pre-wrap">{note.body}</p>
                    </div>
                  ))
                )}
              </div>

              {/* Add note + actions */}
              <TicketActions ticketId={ticketId} ticket={ticket} />
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
