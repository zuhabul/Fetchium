import { getSession, adminFetch } from '@/lib/session'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import TopBar from '@/components/layout/TopBar'
import {
  Activity,
  Building2,
  Flame,
  Ticket,
} from 'lucide-react'

function StatCard({ label, value, icon: Icon, color }: {
  label: string; value: string | number; icon: React.ElementType; color: string
}) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4 lg:rounded-xl lg:p-4">
      <div className="mb-4 flex items-start justify-between gap-3 lg:mb-3 lg:items-center">
        <span className="text-[11px] font-medium uppercase tracking-[0.18em] text-zinc-500">
          {label}
        </span>
        <div className={`flex h-8 w-8 items-center justify-center rounded-xl lg:h-7 lg:w-7 lg:rounded-lg ${color}`}>
          <Icon className="h-4 w-4 lg:h-3.5 lg:w-3.5" />
        </div>
      </div>
      <p className="break-words text-[1.75rem] font-bold leading-none text-zinc-100 lg:text-2xl">
        {value}
      </p>
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
      <div className={`w-full space-y-5 ${ADMIN_PAGE_PADDING}`}>
        <div className="flex flex-col gap-4 rounded-xl border border-zinc-800 bg-zinc-900 px-4 py-4 sm:px-5 lg:flex-row lg:items-center lg:justify-between lg:rounded-xl">
          <div className="min-w-0">
            <p className="text-sm font-medium text-zinc-100">Welcome back, {session?.name}</p>
            <p className="mt-0.5 text-xs leading-5 text-zinc-500">
              Role: <span className="text-zinc-400 capitalize">{session?.role}</span> · Fetchium production
            </p>
          </div>
          <div className="flex items-center gap-1.5 self-start rounded-full border border-emerald-500/10 bg-emerald-500/5 px-3 py-2 lg:self-center lg:px-3 lg:py-2">
            <div className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="text-xs font-medium text-emerald-400">Systems operational</span>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-3 min-[420px]:grid-cols-2 lg:gap-4 xl:grid-cols-4">
          <StatCard label="Total Orgs"     value={summary.total_orgs ?? '—'}   icon={Building2} color="bg-blue-500/20 text-blue-400" />
          <StatCard label="Open Incidents" value={summary.open_incidents ?? '—'} icon={Flame}     color="bg-red-500/20 text-red-400" />
          <StatCard label="Open Tickets"   value={summary.open_tickets ?? '—'}  icon={Ticket}    color="bg-amber-500/20 text-amber-400" />
          <StatCard label="API Version"    value={summary.version ?? '1.0.0'}   icon={Activity}  color="bg-purple-500/20 text-purple-400" />
        </div>

        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <h2 className="mb-4 text-sm font-semibold text-zinc-300">Provider Health</h2>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
            {providers.map(p => (
              <div key={p.name} className="flex min-h-11 items-center justify-between gap-3 rounded-lg bg-zinc-800/60 px-3 py-2">
                <span className="min-w-0 truncate text-xs text-zinc-400">{p.name}</span>
                <div className={`h-2 w-2 rounded-full ${statusColor[p.status] ?? statusColor.unknown}`} title={p.status} />
              </div>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
          {[
            { href: '/orgs',       label: 'Manage Orgs' },
            { href: '/incidents',  label: 'Incidents' },
            { href: '/audit',      label: 'Audit Log' },
            { href: '/system',     label: 'System Panel' },
          ].map(({ href, label }) => (
            <a key={href} href={href}
              className="flex min-h-12 items-center justify-center rounded-xl border border-zinc-800 bg-zinc-900 p-4 text-center text-sm text-zinc-400 transition-colors hover:border-zinc-600 hover:text-zinc-200">
              {label} →
            </a>
          ))}
        </div>
      </div>
    </>
  )
}
