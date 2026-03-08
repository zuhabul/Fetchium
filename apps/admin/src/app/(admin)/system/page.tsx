import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import StatusDot from '@/components/ui/StatusDot'

interface MetricsSummary {
  uptime_seconds?: number
  requests_total?: number
  error_rate?: number
  providers?: Record<string, string>
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

export default async function SystemPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  if (session.role !== 'owner') {
    return (
      <>
        <TopBar title="System" subtitle="Owner-only control panel" />
        <div className="p-6">
          <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-6 text-center">
            <p className="text-sm font-semibold text-red-400">Access denied — owner only</p>
            <p className="text-xs text-zinc-500 mt-1">This panel requires the owner role.</p>
          </div>
        </div>
      </>
    )
  }

  let summary: MetricsSummary = {}
  try {
    const res = await adminFetch('/internal/admin/metrics/summary')
    if (res.ok) summary = await res.json()
  } catch {
    // Non-fatal
  }

  const errorRatePct = summary.error_rate != null
    ? `${(summary.error_rate * 100).toFixed(2)}%`
    : '—'

  async function reloadConfig() {
    'use server'
    const s = await getSession()
    if (s) await adminFetch('/internal/admin/proxy/reset', { method: 'POST' })
  }

  return (
    <>
      <TopBar title="System" subtitle="Control panel — owner only" />
      <div className="p-6 space-y-6 max-w-4xl">

        {/* Stats grid */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {/* API Status */}
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">API Status</p>
            <dl className="space-y-2 text-xs">
              <div className="flex justify-between">
                <dt className="text-zinc-500">Uptime</dt>
                <dd className="text-zinc-300 font-mono">
                  {summary.uptime_seconds != null ? formatUptime(summary.uptime_seconds) : '—'}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Requests served</dt>
                <dd className="text-zinc-300">{summary.requests_total?.toLocaleString() ?? '—'}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Error rate</dt>
                <dd className={`font-mono ${summary.error_rate != null && summary.error_rate > 0.05 ? 'text-red-400' : 'text-zinc-300'}`}>
                  {errorRatePct}
                </dd>
              </div>
            </dl>
          </div>

          {/* Database */}
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Database</p>
            <dl className="space-y-2 text-xs">
              <div className="flex justify-between">
                <dt className="text-zinc-500">Engine</dt>
                <dd className="text-zinc-300">SQLite (PIE)</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Status</dt>
                <dd><StatusDot status="ok" /></dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Size</dt>
                <dd className="text-zinc-500">—</dd>
              </div>
            </dl>
          </div>

          {/* Cache */}
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Cache</p>
            <dl className="space-y-2 text-xs">
              <div className="flex justify-between">
                <dt className="text-zinc-500">Type</dt>
                <dd className="text-zinc-300">In-memory + disk</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Hit rate</dt>
                <dd className="text-zinc-500">—</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-zinc-500">Entries</dt>
                <dd className="text-zinc-500">—</dd>
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
          <div className="flex items-center gap-3">
            <form action={reloadConfig}>
              <button
                type="submit"
                className="px-4 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors"
              >
                Reload Config
              </button>
            </form>
            <a
              href="/audit"
              className="px-4 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors"
            >
              View Logs
            </a>
          </div>
        </div>

      </div>
    </>
  )
}
