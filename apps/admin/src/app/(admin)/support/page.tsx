import { redirect } from 'next/navigation'
import Link from 'next/link'
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
    if (res.ok) tickets = await res.json()
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
      <div className="p-6 space-y-6">
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm">
            Failed to load data
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
          <div className="flex items-center justify-between p-4 border-b border-zinc-800">
            <div className="flex gap-1">
              {TABS.map((tab) => (
                <Link
                  key={tab}
                  href={`/support?tab=${tab}`}
                  className={`text-xs px-3 py-1.5 rounded-md border capitalize transition-colors ${
                    activeTab === tab
                      ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                      : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  {tab}
                </Link>
              ))}
            </div>
            <Link
              href="/support/new"
              className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
            >
              + New Ticket
            </Link>
          </div>

          <table className="w-full">
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
              {filtered.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-3 py-8 text-center text-zinc-500 text-sm">
                    No tickets found
                  </td>
                </tr>
              ) : (
                filtered.map((t) => (
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
                    <td className="px-3 py-2.5 text-sm text-zinc-400">
                      {new Date(t.created_at).toLocaleDateString('en-US', {
                        month: 'short', day: 'numeric', year: 'numeric',
                      })}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
