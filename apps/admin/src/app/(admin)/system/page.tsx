import { redirect } from 'next/navigation'
import Link from 'next/link'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import StatusDot from '@/components/ui/StatusDot'

interface MetricsSummary {
  uptime_seconds?: number
  requests_total?: number
  requests_errors?: number
  error_rate?: number
  providers?: Record<string, string>
  cpu_percent?: number
  ram_used_mb?: number
  ram_total_mb?: number
  disk_used_gb?: number
}

interface MetricsRealtime {
  rps?: number
  active_connections?: number
}

const PROVIDERS = ['Google', 'DDG', 'Bing', 'Brave', 'SearXNG', 'Gemini', 'Serper', 'Exa']

function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  return `${d}d ${h}h ${m}m`
}

function providerStatus(raw: string | undefined): 'ok' | 'degraded' | 'down' | 'unknown' {
  if (!raw) return 'unknown'
  const s = raw.toLowerCase()
  if (s === 'ok' || s === 'healthy') return 'ok'
  if (s === 'degraded' || s === 'slow') return 'degraded'
  if (s === 'down' || s === 'error') return 'down'
  return 'unknown'
}

const LINK_CARDS = [
  { href: '/system/config', label: 'Config Editor', desc: 'Feature flags & kill switches', color: 'text-amber-400' },
  { href: '/system/db', label: 'DB Inspector', desc: 'Browse tables, run queries', color: 'text-blue-400' },
  { href: '/system/logs', label: 'Log Stream', desc: 'Live API & admin logs', color: 'text-emerald-400' },
  { href: '/system/jobs', label: 'Job Monitor', desc: 'Background job queue', color: 'text-purple-400' },
  { href: '/system/api', label: 'API Explorer', desc: 'Route list & curl tester', color: 'text-zinc-300' },
  { href: '/audit?filter=admin_actions', label: 'Meta-Audit', desc: 'Admin action log trail', color: 'text-red-400' },
]

export default async function SystemPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  if (session.role !== 'owner') {
    return (
      <>
        <TopBar title="System" subtitle="Owner-only control panel" />
        <div className="p-6">
          <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-8 text-center">
            <p className="text-sm font-semibold text-red-400">Access denied — owner only</p>
            <p className="text-xs text-zinc-500 mt-1">This panel requires the owner role.</p>
          </div>
        </div>
      </>
    )
  }

  let summary: MetricsSummary = {}
  let realtime: MetricsRealtime = {}
  try {
    const [r1, r2] = await Promise.allSettled([
      adminFetch('/internal/admin/metrics/summary'),
      adminFetch('/internal/admin/metrics/realtime'),
    ])
    if (r1.status === 'fulfilled' && r1.value.ok) summary = await r1.value.json()
    if (r2.status === 'fulfilled' && r2.value.ok) realtime = await r2.value.json()
  } catch { /* non-fatal */ }

  const errorRatePct = summary.error_rate != null
    ? `${(summary.error_rate * 100).toFixed(2)}%` : '—'
  const cpuPct = summary.cpu_percent != null ? `${summary.cpu_percent.toFixed(1)}%` : '—'
  const ramUsed = summary.ram_used_mb != null ? `${(summary.ram_used_mb / 1024).toFixed(1)} GB` : '—'
  const ramTotal = summary.ram_total_mb != null ? `/ ${(summary.ram_total_mb / 1024).toFixed(1)} GB` : ''
  const diskUsed = summary.disk_used_gb != null ? `${summary.disk_used_gb.toFixed(1)} GB` : '—'

  async function reloadConfig() {
    'use server'
    const s = await getSession()
    if (s) await adminFetch('/internal/admin/proxy/reset', { method: 'POST' })
  }

  async function clearCache() {
    'use server'
    const s = await getSession()
    if (s) await adminFetch('/internal/admin/cache/clear', { method: 'POST' })
  }

  return (
    <>
      <TopBar title="System" subtitle="Control panel — owner only" />
      <div className="p-6 space-y-6 max-w-5xl">

        {/* 4-column stat grid */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs text-zinc-500 mb-1">CPU</p>
            <p className="text-2xl font-semibold text-zinc-100">{cpuPct}</p>
          </div>
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs text-zinc-500 mb-1">RAM Used</p>
            <p className="text-2xl font-semibold text-zinc-100">
              {ramUsed} <span className="text-sm text-zinc-500">{ramTotal}</span>
            </p>
          </div>
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs text-zinc-500 mb-1">Disk Used</p>
            <p className="text-2xl font-semibold text-zinc-100">{diskUsed}</p>
          </div>
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs text-zinc-500 mb-1">API Uptime</p>
            <p className="text-lg font-semibold text-zinc-100">
              {summary.uptime_seconds != null ? formatUptime(summary.uptime_seconds) : '—'}
            </p>
          </div>
        </div>

        {/* API Health + Database */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">API Health</p>
            <dl className="space-y-2 text-xs">
              <div className="flex justify-between">
                <dt className="text-zinc-500">Total requests</dt>
                <dd className="text-zinc-300">{summary.requests_total?.toLocaleString() ?? '—'}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Total errors</dt>
                <dd className="text-zinc-300">{summary.requests_errors?.toLocaleString() ?? '—'}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Error rate</dt>
                <dd className={`font-mono ${summary.error_rate != null && summary.error_rate > 0.05 ? 'text-red-400' : 'text-zinc-300'}`}>
                  {errorRatePct}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Req/s (live)</dt>
                <dd className="text-zinc-300">{realtime.rps ?? '—'}</dd>
              </div>
            </dl>
          </div>
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Database</p>
            <dl className="space-y-2 text-xs">
              <div className="flex justify-between">
                <dt className="text-zinc-500">admin.db</dt>
                <dd className="text-zinc-300">~2.4 MB</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">auth.db</dt>
                <dd className="text-zinc-300">~0.8 MB</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Engine</dt>
                <dd className="text-zinc-300">SQLite (PIE)</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Status</dt>
                <dd><StatusDot status="ok" showLabel={false} /></dd>
              </div>
            </dl>
          </div>
        </div>

        {/* Provider Health */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
          <p className="text-sm font-semibold text-zinc-300 mb-4">Provider Health</p>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            {PROVIDERS.map(p => {
              const raw = summary.providers?.[p.toLowerCase()]
              const status = providerStatus(raw)
              return (
                <div key={p} className="flex items-center justify-between bg-zinc-800/60 rounded-lg px-3 py-2">
                  <span className="text-xs text-zinc-400">{p}</span>
                  <StatusDot status={status} showLabel={false} />
                </div>
              )
            })}
          </div>
        </div>

        {/* Actions */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
          <p className="text-sm font-semibold text-zinc-300 mb-4">Actions</p>
          <div className="flex flex-wrap items-center gap-3">
            <form action={reloadConfig}>
              <button type="submit" className="px-4 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
                Reload Config
              </button>
            </form>
            <form action={clearCache}>
              <button type="submit" className="px-4 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
                Clear Cache
              </button>
            </form>
            <Link href="/system/logs" className="px-4 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
              View Logs
            </Link>
            <Link href="/system/jobs" className="px-4 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
              View Jobs
            </Link>
          </div>
        </div>

        {/* Link cards grid */}
        <div>
          <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Quick Access</p>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
            {LINK_CARDS.map(card => (
              <Link key={card.href} href={card.href} className="bg-zinc-900 border border-zinc-800 hover:border-zinc-700 rounded-xl p-4 transition-colors group">
                <p className={`text-sm font-semibold ${card.color} group-hover:underline`}>{card.label}</p>
                <p className="text-xs text-zinc-500 mt-1">{card.desc}</p>
              </Link>
            ))}
          </div>
        </div>

      </div>
    </>
  )
}
