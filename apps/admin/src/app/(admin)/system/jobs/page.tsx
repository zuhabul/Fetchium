import { redirect } from 'next/navigation'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'

interface AuditEvent {
  id: string
  user_email: string | null
  user_name: string | null
  target_type: string
  action: string
  created_at: string
}

interface Summary { total_orgs?: number; open_incidents?: number; open_tickets?: number }

function timeAgo(date: string) {
  const ms = Date.now() - new Date(date).getTime()
  if (ms < 60000) return Math.floor(ms / 1000) + 's ago'
  if (ms < 3600000) return Math.floor(ms / 60000) + 'm ago'
  return Math.floor(ms / 3600000) + 'h ago'
}

export default async function JobsPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let events: AuditEvent[] = []
  let summary: Summary = {}

  try {
    const [jobsRes, summaryRes] = await Promise.all([
      adminFetch('/internal/admin/system/jobs'),
      adminFetch('/internal/admin/metrics/summary'),
    ])
    if (jobsRes.ok) { const b = await jobsRes.json(); events = b.jobs ?? [] }
    if (summaryRes.ok) summary = await summaryRes.json()
  } catch {}

  return (
    <>
      <TopBar title="Activity Monitor" subtitle="Real-time admin activity" />
      <div className={`${ADMIN_PAGE_PADDING} max-w-5xl space-y-6`}>
        <div className="grid grid-cols-3 gap-4">
          {[['Total Orgs', summary.total_orgs], ['Open Incidents', summary.open_incidents], ['Open Tickets', summary.open_tickets]].map(([label, val]) => (
            <div key={label as string} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
              <p className="text-xs text-zinc-500 mb-1">{label as string}</p>
              <p className="text-2xl font-semibold text-zinc-100">{val ?? '—'}</p>
            </div>
          ))}
        </div>
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          <div className="px-4 py-3 border-b border-zinc-800">
            <p className="text-sm font-semibold text-zinc-300">Recent Admin Activity</p>
            <p className="text-xs text-zinc-600 mt-0.5">Last 50 actions from the audit log</p>
          </div>
          {events.length === 0 ? (
            <div className="px-4 py-10 text-center text-zinc-500 text-sm">No activity recorded yet.</div>
          ) : (
            <table className="w-full text-xs">
              <thead>
                <tr className="border-b border-zinc-800">
                  {['Actor', 'Action', 'Target', 'When'].map(h => (
                    <th key={h} className="px-4 py-2.5 text-left font-medium text-zinc-500">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {events.map(ev => (
                  <tr key={ev.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/20">
                    <td className="px-4 py-3">
                      <p className="text-zinc-300">{ev.user_name ?? 'system'}</p>
                      <p className="text-zinc-600">{ev.user_email ?? ''}</p>
                    </td>
                    <td className="px-4 py-3 font-mono text-zinc-400">{ev.action}</td>
                    <td className="px-4 py-3 text-zinc-500">{ev.target_type}</td>
                    <td className="px-4 py-3 text-zinc-600">{timeAgo(ev.created_at)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </>
  )
}
