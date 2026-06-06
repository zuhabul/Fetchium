import { redirect } from 'next/navigation'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import ProxyActions from './ProxyActions'

interface ProxyStats {
  provider: string
  endpoint: string
  status: string
  reset_count: number
  last_reset: string | null
}

interface GeoEntry {
  country: string
  code: string
  requests: number
}

export default async function ProxyPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let stats: ProxyStats | null = null
  let geo: GeoEntry[] = []

  try {
    const [statsRes, geoRes] = await Promise.all([
      adminFetch('/internal/admin/proxy/stats'),
      adminFetch('/internal/admin/proxy/geo'),
    ])
    if (statsRes.ok) stats = await statsRes.json()
    if (geoRes.ok) {
      const body = await geoRes.json()
      geo = body.data ?? []
    }
  } catch {}

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="Proxy Operations" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-6`}>

        {/* Stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {[
            { label: 'Provider', value: stats?.provider ?? '—' },
            { label: 'Endpoint', value: stats?.endpoint ?? '—' },
            { label: 'Status', value: stats?.status ?? '—' },
            { label: 'Pool Resets', value: stats?.reset_count?.toString() ?? '0' },
          ].map(({ label, value }) => (
            <div key={label} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
              <p className="text-xs text-zinc-500 uppercase tracking-wider mb-1">{label}</p>
              <p className="text-sm font-medium text-zinc-200 truncate">{value}</p>
            </div>
          ))}
        </div>

        {stats?.last_reset && (
          <p className="text-xs text-zinc-600">
            Last reset: {new Date(stats.last_reset).toLocaleString()}
          </p>
        )}

        {/* Actions */}
        <ProxyActions sessionToken={session.sessionToken} />

        {/* Geo Distribution */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
          <div className="px-4 py-3 border-b border-zinc-800">
            <h2 className="text-sm font-semibold text-zinc-100">Geo Distribution</h2>
            <p className="text-xs text-zinc-500 mt-0.5">Country targets available via DataImpulse</p>
          </div>
          <div className="divide-y divide-zinc-800/60">
            {geo.map(g => (
              <div key={g.code} className="flex items-center justify-between px-4 py-3">
                <div className="flex items-center gap-3">
                  <span className="text-lg">{countryFlag(g.code)}</span>
                  <div>
                    <p className="text-sm text-zinc-300">{g.country}</p>
                    <p className="text-xs text-zinc-600 font-mono">{g.code.toUpperCase()}</p>
                  </div>
                </div>
                <span className="text-sm text-zinc-500">{g.requests.toLocaleString()} req</span>
              </div>
            ))}
          </div>
        </div>

      </div>
    </div>
  )
}

function countryFlag(code: string): string {
  const map: Record<string, string> = {
    us: '🇺🇸', gb: '🇬🇧', de: '🇩🇪', fr: '🇫🇷', jp: '🇯🇵', au: '🇦🇺',
    ca: '🇨🇦', br: '🇧🇷', in: '🇮🇳', sg: '🇸🇬',
  }
  return map[code] ?? '🌐'
}
