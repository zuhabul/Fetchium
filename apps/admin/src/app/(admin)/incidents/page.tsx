import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { AlertTriangle, CheckCircle, Plus } from 'lucide-react'

interface Incident {
  id: string
  title: string
  severity: 'critical' | 'high' | 'medium' | 'low'
  status: 'investigating' | 'identified' | 'monitoring' | 'resolved'
  owner: string
  started_at: string
  resolved_at?: string
}

const SEVERITY_STYLES: Record<string, string> = {
  critical: 'bg-red-500/20 text-red-400 border-red-500/30',
  high: 'bg-orange-500/20 text-orange-400 border-orange-500/30',
  medium: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
  low: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
}

const STATUS_STYLES: Record<string, string> = {
  investigating: 'bg-red-500/20 text-red-400 border-red-500/30',
  identified: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
  monitoring: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  resolved: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
}

function duration(start: string, end?: string): string {
  const ms = new Date(end || Date.now()).getTime() - new Date(start).getTime()
  const h = Math.floor(ms / 3600000)
  const m = Math.floor((ms % 3600000) / 60000)
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

function Badge({ label, styles }: { label: string; styles: string }) {
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${styles}`}>
      {label}
    </span>
  )
}

export default async function IncidentsPage({
  searchParams,
}: {
  searchParams: Promise<{ status?: string }>
}) {
  const session = await getSession()
  const { status = 'all' } = await searchParams

  let incidents: Incident[] = []
  try {
    const res = await adminFetch('/internal/admin/incidents')
    if (res.ok) incidents = await res.json()
  } catch {}

  const filtered = status === 'all' ? incidents : incidents.filter(i => {
    if (status === 'open') return i.status !== 'resolved'
    if (status === 'resolved') return i.status === 'resolved'
    return true
  })

  const openIncidents = incidents.filter(i => i.status !== 'resolved')

  return (
    <>
      <TopBar title="Incidents" subtitle="Observability & incident management" />
      <div className="p-6 space-y-5">
        {/* Active incident banner */}
        {openIncidents.length > 0 && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 flex items-center gap-3">
            <AlertTriangle className="w-4 h-4 text-red-400 shrink-0" />
            <p className="text-sm text-red-300 font-medium">
              {openIncidents.length} active incident{openIncidents.length > 1 ? 's' : ''} — systems degraded
            </p>
            <span className="ml-auto text-xs text-red-400">
              Latest: {openIncidents[0].title}
            </span>
          </div>
        )}

        {/* Toolbar */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            {['all', 'open', 'resolved'].map(s => (
              <Link
                key={s}
                href={`?status=${s}`}
                className={`text-xs px-3 py-1.5 rounded-md border transition-colors capitalize ${
                  status === s
                    ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                    : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {s}
              </Link>
            ))}
          </div>
          <Link
            href="/incidents/create"
            className="flex items-center gap-1.5 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            Create Incident
          </Link>
        </div>

        {/* Table */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 gap-3">
              <CheckCircle className="w-10 h-10 text-emerald-500" />
              <p className="text-sm font-medium text-zinc-300">No incidents — systems operational</p>
              <p className="text-xs text-zinc-500">All services running normally</p>
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800">
                  {['#', 'Title', 'Severity', 'Status', 'Owner', 'Started At', 'Duration', 'Actions'].map(h => (
                    <th key={h} className="text-left text-xs font-medium text-zinc-500 uppercase tracking-wider px-4 py-3">
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60">
                {filtered.map((inc, idx) => (
                  <tr key={inc.id} className="hover:bg-zinc-800/30 transition-colors">
                    <td className="px-4 py-3 text-xs text-zinc-500">{idx + 1}</td>
                    <td className="px-4 py-3 text-zinc-200 font-medium max-w-[200px] truncate">{inc.title}</td>
                    <td className="px-4 py-3">
                      <Badge label={inc.severity} styles={SEVERITY_STYLES[inc.severity] ?? ''} />
                    </td>
                    <td className="px-4 py-3">
                      <Badge label={inc.status} styles={STATUS_STYLES[inc.status] ?? ''} />
                    </td>
                    <td className="px-4 py-3 text-zinc-400 text-xs">{inc.owner}</td>
                    <td className="px-4 py-3 text-zinc-400 text-xs">{new Date(inc.started_at).toLocaleString()}</td>
                    <td className="px-4 py-3 text-zinc-400 text-xs">
                      {duration(inc.started_at, inc.resolved_at)}
                    </td>
                    <td className="px-4 py-3">
                      <Link
                        href={`/incidents/${inc.id}`}
                        className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
                      >
                        View →
                      </Link>
                    </td>
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
