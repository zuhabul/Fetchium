import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Megaphone, TrendingUp } from 'lucide-react'

interface Campaign {
  id: string
  name: string
  type: string
  status: string
  touch_count: number
  created_at: string
  updated_at: string
}

const STATUS_STYLES: Record<string, string> = {
  draft:    'bg-zinc-800 text-zinc-400 border-zinc-700',
  active:   'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
  paused:   'bg-amber-500/20 text-amber-400 border-amber-500/30',
  archived: 'bg-zinc-800 text-zinc-600 border-zinc-700',
}

function fmt(date: string) {
  return new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

export default async function CampaignsPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let campaigns: Campaign[] = []
  let total = 0

  try {
    const res = await adminFetch('/internal/admin/campaigns')
    if (res.ok) {
      const body = await res.json()
      campaigns = body.data ?? []
      total = body.total ?? 0
    }
  } catch {}

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="Campaigns" />
      <div className="p-6 space-y-6">

        {/* Stats */}
        <div className="grid grid-cols-3 gap-4">
          {[
            { label: 'Total', value: total },
            { label: 'Active', value: campaigns.filter(c => c.status === 'active').length },
            { label: 'Total Touches', value: campaigns.reduce((s, c) => s + c.touch_count, 0) },
          ].map(({ label, value }) => (
            <div key={label} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
              <p className="text-xs text-zinc-500 uppercase tracking-wider mb-1">{label}</p>
              <p className="text-2xl font-semibold text-zinc-100">{value.toLocaleString()}</p>
            </div>
          ))}
        </div>

        {/* Quick links */}
        <div className="flex gap-3">
          <Link
            href="/campaigns/funnels"
            className="flex items-center gap-2 text-xs px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-lg border border-zinc-700"
          >
            <TrendingUp className="w-3.5 h-3.5" />
            Conversion Funnels
          </Link>
        </div>

        {/* Table */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          <div className="px-4 py-3 border-b border-zinc-800 flex items-center gap-2">
            <Megaphone className="w-4 h-4 text-zinc-500" />
            <h2 className="text-sm font-semibold text-zinc-100">All Campaigns</h2>
          </div>
          <table className="w-full">
            <thead>
              <tr className="border-b border-zinc-800">
                {['Name', 'Type', 'Status', 'Touches', 'Created', ''].map(h => (
                  <th key={h} className="text-left text-xs font-medium text-zinc-500 uppercase tracking-wider px-4 py-2.5">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {campaigns.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-4 py-10 text-center text-zinc-500 text-sm">
                    No campaigns yet. Create one to start tracking attribution.
                  </td>
                </tr>
              ) : campaigns.map(c => (
                <tr key={c.id} className="border-b border-zinc-800/60 hover:bg-zinc-800/30">
                  <td className="px-4 py-3 text-sm text-zinc-200 font-medium">{c.name}</td>
                  <td className="px-4 py-3 text-xs text-zinc-500 capitalize">{c.type}</td>
                  <td className="px-4 py-3">
                    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${STATUS_STYLES[c.status] ?? STATUS_STYLES.draft}`}>
                      {c.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm text-zinc-400">{c.touch_count.toLocaleString()}</td>
                  <td className="px-4 py-3 text-xs text-zinc-600">{fmt(c.created_at)}</td>
                  <td className="px-4 py-3 text-right">
                    <Link href={`/campaigns/${c.id}`} className="text-xs text-zinc-500 hover:text-zinc-300">
                      View →
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
