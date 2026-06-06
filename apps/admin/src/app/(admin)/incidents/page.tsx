import { getSession, adminFetch } from '@/lib/session'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { AlertTriangle, CheckCircle, Clock3, Flame, Plus } from 'lucide-react'

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

function fmt(date: string) {
  return new Date(date).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
}

function StatCard({
  label,
  value,
  icon: Icon,
  color,
}: {
  label: string
  value: number
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
      <p className="text-[1.75rem] font-bold leading-none text-zinc-100 lg:text-2xl">
        {value.toLocaleString()}
      </p>
    </div>
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
    if (res.ok) {
      const payload = await res.json()
      incidents = Array.isArray(payload)
        ? payload
        : Array.isArray(payload?.data)
          ? payload.data
          : []
    }
  } catch {}

  const filtered = status === 'all' ? incidents : incidents.filter(i => {
    if (status === 'open') return i.status !== 'resolved'
    if (status === 'resolved') return i.status === 'resolved'
    return true
  })

  const openIncidents = incidents.filter(i => i.status !== 'resolved')
  const resolvedIncidents = incidents.filter(i => i.status === 'resolved')
  const criticalIncidents = incidents.filter(i => i.severity === 'critical')

  return (
    <>
      <TopBar title="Incidents" subtitle="Observability & incident management" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>
        {openIncidents.length > 0 && (
          <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <div className="flex min-w-0 items-start gap-3">
                <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-red-400" />
                <div className="min-w-0">
                  <p className="text-sm font-medium text-red-300">
                    {openIncidents.length} active incident{openIncidents.length > 1 ? 's' : ''} — systems degraded
                  </p>
                  <p className="mt-1 truncate text-xs text-red-400">
                    Latest: {openIncidents[0].title}
                  </p>
                </div>
              </div>
              <Link
                href={`/incidents/${openIncidents[0].id}`}
                className="inline-flex min-h-11 items-center justify-center rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-300 transition-colors hover:bg-red-500/20 sm:min-h-9 sm:self-start sm:py-1.5"
              >
                Review latest
              </Link>
            </div>
          </div>
        )}

        <div className="grid grid-cols-1 gap-3 min-[420px]:grid-cols-2 xl:grid-cols-4">
          <StatCard
            label="Open"
            value={openIncidents.length}
            icon={AlertTriangle}
            color="bg-red-500/20 text-red-400"
          />
          <StatCard
            label="Resolved"
            value={resolvedIncidents.length}
            icon={CheckCircle}
            color="bg-emerald-500/20 text-emerald-400"
          />
          <StatCard
            label="Critical"
            value={criticalIncidents.length}
            icon={Flame}
            color="bg-orange-500/20 text-orange-400"
          />
          <StatCard
            label="Total"
            value={incidents.length}
            icon={Clock3}
            color="bg-blue-500/20 text-blue-400"
          />
        </div>

        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <h2 className="text-sm font-semibold text-zinc-100">Incident Queue</h2>
              <p className="mt-1 text-xs text-zinc-500">
                {filtered.length} incident{filtered.length === 1 ? '' : 's'} in the current view
              </p>
            </div>

            <Link
              href="/incidents/create"
              className="inline-flex min-h-11 items-center justify-center gap-1.5 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 sm:min-h-9 sm:self-start sm:py-1.5 lg:self-auto"
            >
              <Plus className="h-3.5 w-3.5" />
              Create Incident
            </Link>
          </div>

          <div className="mt-4 flex flex-wrap gap-2">
            {['all', 'open', 'resolved'].map(s => (
              <Link
                key={s}
                href={`?status=${s}`}
                className={`inline-flex min-h-10 items-center rounded-md border px-3 py-2 text-xs capitalize transition-colors sm:min-h-8 sm:py-1 ${
                  status === s
                    ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                    : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {s}
              </Link>
            ))}
          </div>
        </div>

        <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900">
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 gap-3">
              <CheckCircle className="w-10 h-10 text-emerald-500" />
              <p className="text-sm font-medium text-zinc-300">No incidents — systems operational</p>
              <p className="text-xs text-zinc-500">All services running normally</p>
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/60 lg:hidden">
                {filtered.map((incident, idx) => (
                  <div key={incident.id} className="space-y-4 px-4 py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 space-y-1">
                        <p className="font-mono text-[11px] uppercase tracking-wider text-zinc-600">
                          #{idx + 1}
                        </p>
                        <Link
                          href={`/incidents/${incident.id}`}
                          className="block line-clamp-2 text-sm font-medium text-blue-400 hover:text-blue-300"
                        >
                          {incident.title}
                        </Link>
                        <p className="text-xs text-zinc-500">{incident.owner}</p>
                      </div>
                      <Link
                        href={`/incidents/${incident.id}`}
                        className="inline-flex min-h-11 shrink-0 items-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
                      >
                        View
                      </Link>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <Badge
                        label={incident.severity}
                        styles={SEVERITY_STYLES[incident.severity] ?? ''}
                      />
                      <Badge
                        label={incident.status}
                        styles={STATUS_STYLES[incident.status] ?? ''}
                      />
                    </div>

                    <dl className="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Started</dt>
                        <dd className="mt-1 text-zinc-400">{fmt(incident.started_at)}</dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Duration</dt>
                        <dd className="mt-1 text-zinc-300">
                          {duration(incident.started_at, incident.resolved_at)}
                        </dd>
                      </div>
                    </dl>
                  </div>
                ))}
              </div>

              <table className="hidden w-full text-sm lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['#', 'Title', 'Severity', 'Status', 'Owner', 'Started At', 'Duration', 'Actions'].map(h => (
                      <th key={h} className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-zinc-500">
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800/60">
                  {filtered.map((incident, idx) => (
                    <tr key={incident.id} className="transition-colors hover:bg-zinc-800/30">
                      <td className="px-4 py-3 text-xs text-zinc-500">{idx + 1}</td>
                      <td className="max-w-[260px] truncate px-4 py-3 font-medium text-zinc-200">
                        {incident.title}
                      </td>
                      <td className="px-4 py-3">
                        <Badge
                          label={incident.severity}
                          styles={SEVERITY_STYLES[incident.severity] ?? ''}
                        />
                      </td>
                      <td className="px-4 py-3">
                        <Badge
                          label={incident.status}
                          styles={STATUS_STYLES[incident.status] ?? ''}
                        />
                      </td>
                      <td className="px-4 py-3 text-xs text-zinc-400">{incident.owner}</td>
                      <td className="px-4 py-3 text-xs text-zinc-400">
                        {new Date(incident.started_at).toLocaleString()}
                      </td>
                      <td className="px-4 py-3 text-xs text-zinc-400">
                        {duration(incident.started_at, incident.resolved_at)}
                      </td>
                      <td className="px-4 py-3">
                        <Link
                          href={`/incidents/${incident.id}`}
                          className="inline-block rounded-md border border-zinc-700 bg-zinc-800 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
                        >
                          View
                        </Link>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </div>
      </div>
    </>
  )
}
