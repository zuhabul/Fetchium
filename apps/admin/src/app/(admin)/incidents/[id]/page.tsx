import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import { notFound } from 'next/navigation'
import { Bot } from 'lucide-react'

interface TimelineEvent {
  id: string
  type: 'update' | 'mitigation' | 'resolution' | 'created'
  body: string
  actor: string
  created_at: string
}

interface IncidentDetail {
  id: string
  title: string
  severity: 'critical' | 'high' | 'medium' | 'low'
  status: 'investigating' | 'identified' | 'monitoring' | 'resolved'
  owner: string
  started_at: string
  resolved_at?: string
  affected_org_count: number
  impacted_endpoints: string[]
  timeline: TimelineEvent[]
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

const EVENT_TYPE_STYLES: Record<string, string> = {
  created: 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30',
  update: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  mitigation: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
  resolution: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
}

function Badge({ label, styles }: { label: string; styles: string }) {
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${styles}`}>
      {label}
    </span>
  )
}

export default async function IncidentDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params
  const session = await getSession()

  let incident: IncidentDetail | null = null
  try {
    const res = await adminFetch(`/internal/admin/incidents/${id}`)
    if (res.status === 404) notFound()
    if (res.ok) incident = await res.json()
  } catch {}

  if (!incident) {
    return (
      <>
        <TopBar title="Incident" />
        <div className="p-6 text-zinc-400 text-sm">Failed to load incident.</div>
      </>
    )
  }

  return (
    <>
      <TopBar title={`INC-${id.slice(0, 8).toUpperCase()}`} subtitle={incident.title} />
      <div className="p-6 space-y-5 max-w-4xl">
        {/* Header */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 flex flex-wrap items-center gap-3">
          <h2 className="text-base font-semibold text-zinc-100 flex-1">{incident.title}</h2>
          <Badge label={incident.severity} styles={SEVERITY_STYLES[incident.severity] ?? ''} />
          <Badge label={incident.status} styles={STATUS_STYLES[incident.status] ?? ''} />
          <span className="text-xs text-zinc-500">Owner: <span className="text-zinc-300">{incident.owner}</span></span>
          <span className="text-xs text-zinc-500">Started: <span className="text-zinc-300">{new Date(incident.started_at).toLocaleString()}</span></span>
        </div>

        {/* Impact */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
          <h3 className="text-sm font-semibold text-zinc-300">Impact Summary</h3>
          <div className="flex items-center gap-6">
            <div>
              <p className="text-xs text-zinc-500">Affected Orgs</p>
              <p className="text-2xl font-bold text-zinc-100">{incident.affected_org_count}</p>
            </div>
          </div>
          {incident.impacted_endpoints.length > 0 && (
            <div>
              <p className="text-xs text-zinc-500 mb-2">Impacted Endpoints</p>
              <div className="flex flex-wrap gap-2">
                {incident.impacted_endpoints.map(ep => (
                  <span key={ep} className="text-xs font-mono bg-zinc-800 text-zinc-300 px-2 py-1 rounded-md border border-zinc-700">
                    {ep}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Timeline */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-4">
          <h3 className="text-sm font-semibold text-zinc-300">Timeline</h3>
          {incident.timeline.length === 0 ? (
            <p className="text-xs text-zinc-500">No timeline events yet.</p>
          ) : (
            <ol className="space-y-3">
              {incident.timeline.map(ev => (
                <li key={ev.id} className="flex gap-3">
                  <div className="flex flex-col items-center">
                    <div className="w-2 h-2 rounded-full bg-zinc-600 mt-1.5 shrink-0" />
                    <div className="w-px flex-1 bg-zinc-800 mt-1" />
                  </div>
                  <div className="pb-3 flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <Badge label={ev.type} styles={EVENT_TYPE_STYLES[ev.type] ?? EVENT_TYPE_STYLES.update} />
                      <span className="text-xs text-zinc-500">{ev.actor}</span>
                      <span className="text-xs text-zinc-600">{new Date(ev.created_at).toLocaleString()}</span>
                    </div>
                    <p className="text-sm text-zinc-300">{ev.body}</p>
                  </div>
                </li>
              ))}
            </ol>
          )}

          {/* Add Update form */}
          <form action={`/api/admin/incidents/${id}/timeline`} method="POST" className="border-t border-zinc-800 pt-4 space-y-3">
            <p className="text-xs font-medium text-zinc-400">Add Update</p>
            <select name="event_type" className="w-full bg-zinc-800 border border-zinc-700 rounded-md text-sm text-zinc-300 px-3 py-2">
              <option value="update">Update</option>
              <option value="mitigation">Mitigation</option>
              <option value="resolution">Resolution</option>
            </select>
            <textarea
              name="body"
              rows={3}
              placeholder="Describe what happened, what was done, or current status..."
              className="w-full bg-zinc-800 border border-zinc-700 rounded-md text-sm text-zinc-300 px-3 py-2 placeholder:text-zinc-600 resize-none"
            />
            <button type="submit" className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors">
              Post Update
            </button>
          </form>
        </div>

        {/* Actions */}
        <div className="flex gap-2">
          <form action={`/api/admin/incidents/${id}/severity`} method="POST">
            <button type="submit" className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors">
              Change Severity
            </button>
          </form>
          {incident.status !== 'resolved' && (
            <form action={`/api/admin/incidents/${id}/resolve`} method="POST">
              <button type="submit" className="bg-emerald-500/20 hover:bg-emerald-500/30 border border-emerald-500/30 text-emerald-400 text-sm px-3 py-1.5 rounded-md transition-colors">
                Resolve Incident
              </button>
            </form>
          )}
        </div>

        {/* Postmortem */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
          <h3 className="text-sm font-semibold text-zinc-300">Postmortem</h3>
          <p className="text-xs text-zinc-500">
            Generate an AI-powered postmortem report summarizing the incident timeline, root cause, and remediation steps.
          </p>
          <button className="flex items-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors">
            <Bot className="w-3.5 h-3.5" />
            Generate AI Postmortem
          </button>
          <div className="bg-zinc-800/50 border border-zinc-700/50 rounded-lg p-3 text-xs text-zinc-500 italic">
            Postmortem will appear here after generation…
          </div>
        </div>
      </div>
    </>
  )
}
