import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import { Activity, Key, Building2, Flame, Ticket } from 'lucide-react'

function StatCard({ label, value, icon: Icon, color }: {
  label: string; value: string | number; icon: React.ElementType; color: string
}) {
  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
      <div className="flex items-center justify-between mb-3">
        <span className="text-xs font-medium text-zinc-500 uppercase tracking-wider">{label}</span>
        <div className={`w-7 h-7 rounded-lg flex items-center justify-center ${color}`}>
          <Icon className="w-3.5 h-3.5" />
        </div>
      </div>
      <p className="text-2xl font-bold text-zinc-100">{value}</p>
    </div>
  )
}

export default async function OverviewPage() {
  const session = await getSession()

  let summary: Record<string, number> = {}
  let providerHealth: Array<{ name: string; status: string }> = []

  try {
    const [summaryRes, healthRes] = await Promise.all([
      adminFetch('/internal/admin/metrics/summary'),
      adminFetch('/internal/admin/metrics/providers'),
    ])
    if (summaryRes.ok) summary = await summaryRes.json()
    if (healthRes.ok) {
      const hbody = await healthRes.json()
      providerHealth = hbody.data ?? []
    }
  } catch {}

  const providers = providerHealth.length > 0
    ? providerHealth
    : ['Google', 'DDG', 'Bing', 'Brave', 'SearXNG', 'Gemini', 'Serper', 'Exa'].map(n => ({ name: n, status: 'unknown' }))

  const statusColor: Record<string, string> = {
    ok:       'bg-emerald-500',
    degraded: 'bg-amber-500',
    down:     'bg-red-500',
    unknown:  'bg-zinc-600',
  }

  return (
    <>
      <TopBar title="Overview" subtitle="Fetchium operations command center" />
      <div className="p-6 space-y-6">
        {/* Welcome banner */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-5 py-4 flex items-center justify-between">
          <div>
            <p className="text-sm font-medium text-zinc-100">Welcome back, {session?.name}</p>
            <p className="text-xs text-zinc-500 mt-0.5">
              Role: <span className="text-zinc-400 capitalize">{session?.role}</span> · Fetchium production
            </p>
          </div>
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 bg-emerald-500 rounded-full animate-pulse" />
            <span className="text-xs text-emerald-400 font-medium">Systems operational</span>
          </div>
        </div>

        {/* KPI grid */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard label="Total Orgs"     value={summary.total_orgs ?? '—'}   icon={Building2} color="bg-blue-500/20 text-blue-400" />
          <StatCard label="Open Incidents" value={summary.open_incidents ?? '—'} icon={Flame}     color="bg-red-500/20 text-red-400" />
          <StatCard label="Open Tickets"   value={summary.open_tickets ?? '—'}  icon={Ticket}    color="bg-amber-500/20 text-amber-400" />
          <StatCard label="API Version"    value={summary.version ?? '1.0.0'}   icon={Activity}  color="bg-purple-500/20 text-purple-400" />
        </div>

        {/* Provider health */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
          <h2 className="text-sm font-semibold text-zinc-300 mb-4">Provider Health</h2>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            {providers.map(p => (
              <div key={p.name} className="flex items-center justify-between bg-zinc-800/60 rounded-lg px-3 py-2">
                <span className="text-xs text-zinc-400">{p.name}</span>
                <div className={`w-2 h-2 rounded-full ${statusColor[p.status] ?? statusColor.unknown}`} title={p.status} />
              </div>
            ))}
          </div>
        </div>

        {/* Quick links */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {[
            { href: '/orgs',       label: 'Manage Orgs' },
            { href: '/incidents',  label: 'Incidents' },
            { href: '/audit',      label: 'Audit Log' },
            { href: '/system',     label: 'System Panel' },
          ].map(({ href, label }) => (
            <a key={href} href={href}
              className="bg-zinc-900 border border-zinc-800 hover:border-zinc-600 rounded-xl p-4 text-sm text-zinc-400 hover:text-zinc-200 transition-colors text-center">
              {label} →
            </a>
          ))}
        </div>
      </div>
    </>
  )
}
