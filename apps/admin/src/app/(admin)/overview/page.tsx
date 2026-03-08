import { getSession } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import { Activity, Key, Building2, Flame } from 'lucide-react'

function StatCard({ label, value, icon: Icon, color }: {
  label: string; value: string; icon: React.ElementType; color: string
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

  return (
    <>
      <TopBar title="Overview" subtitle="Fetchium operations command center" />
      <div className="p-6 space-y-6">
        {/* Welcome banner */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-5 py-4 flex items-center justify-between">
          <div>
            <p className="text-sm font-medium text-zinc-100">
              Welcome back, {session?.name}
            </p>
            <p className="text-xs text-zinc-500 mt-0.5">
              Role: <span className="text-zinc-400 capitalize">{session?.role}</span> ·
              Admin console v1.0 · Fetchium production
            </p>
          </div>
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 bg-emerald-500 rounded-full animate-pulse" />
            <span className="text-xs text-emerald-400 font-medium">Systems operational</span>
          </div>
        </div>

        {/* KPI grid */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard label="Total Orgs"       value="—"  icon={Building2} color="bg-blue-500/20 text-blue-400" />
          <StatCard label="Active Keys"      value="—"  icon={Key}       color="bg-emerald-500/20 text-emerald-400" />
          <StatCard label="Requests / day"   value="—"  icon={Activity}  color="bg-purple-500/20 text-purple-400" />
          <StatCard label="Open Incidents"   value="0"  icon={Flame}     color="bg-red-500/20 text-red-400" />
        </div>

        {/* Provider health placeholder */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
          <h2 className="text-sm font-semibold text-zinc-300 mb-4">Provider Health</h2>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            {['Google', 'DDG', 'Bing', 'Brave', 'SearXNG', 'Gemini', 'Serper', 'Exa'].map(p => (
              <div key={p} className="flex items-center justify-between bg-zinc-800/60 rounded-lg px-3 py-2">
                <span className="text-xs text-zinc-400">{p}</span>
                <div className="w-2 h-2 bg-zinc-600 rounded-full" title="Data coming soon" />
              </div>
            ))}
          </div>
        </div>

        {/* Setup guide for new install */}
        <div className="bg-amber-500/5 border border-amber-500/20 rounded-xl p-4">
          <h3 className="text-sm font-semibold text-amber-400 mb-2">Phase 1-3 Live ✓</h3>
          <ul className="text-xs text-zinc-400 space-y-1">
            <li className="flex items-center gap-2">
              <span className="text-emerald-400">✓</span> Admin app deployed at admin.fetchium.com
            </li>
            <li className="flex items-center gap-2">
              <span className="text-emerald-400">✓</span> Session auth + TOTP 2FA active
            </li>
            <li className="flex items-center gap-2">
              <span className="text-emerald-400">✓</span> Role-based access control enforced
            </li>
            <li className="flex items-center gap-2">
              <span className="text-zinc-600">→</span> Phase 4: Org/Key/Usage management (next)
            </li>
          </ul>
        </div>
      </div>
    </>
  )
}
