'use client'

import { useState, useEffect, useCallback } from 'react'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Activity, AlertCircle, Clock, Zap } from 'lucide-react'

type Range = '1h' | '6h' | '24h' | '7d' | '30d'

interface UsageSummary {
  total_requests: number
  error_rate: number
  avg_latency_ms: number
  top_endpoint: string
}

interface TopOrg {
  org_id: string
  org_name: string
  requests: number
  errors: number
  avg_latency_ms: number
}

const RANGE_MS: Record<Range, number> = {
  '1h': 60 * 60 * 1000,
  '6h': 6 * 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
  '7d': 7 * 24 * 60 * 60 * 1000,
  '30d': 30 * 24 * 60 * 60 * 1000,
}

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

function Skeleton() {
  return (
    <div className="animate-pulse space-y-4">
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {[...Array(4)].map((_, i) => (
          <div key={i} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 h-24" />
        ))}
      </div>
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl h-48" />
    </div>
  )
}

export default function UsagePage() {
  const [range, setRange] = useState<Range>('24h')
  const [summary, setSummary] = useState<UsageSummary | null>(null)
  const [topOrgs, setTopOrgs] = useState<TopOrg[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchData = useCallback(async (r: Range) => {
    setLoading(true)
    setError(null)
    const now = Date.now()
    const from = now - RANGE_MS[r]
    try {
      const [usageRes, orgsRes] = await Promise.all([
        fetch(`/api/admin/usage?from=${from}&to=${now}`),
        fetch('/api/admin/usage/top-orgs'),
      ])
      if (usageRes.ok) setSummary(await usageRes.json())
      if (orgsRes.ok) setTopOrgs((await orgsRes.json()).orgs ?? [])
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch usage data')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { fetchData(range) }, [range, fetchData])

  const RANGES: Range[] = ['1h', '6h', '24h', '7d', '30d']

  return (
    <>
      <TopBar title="Usage Explorer" subtitle="Request volume and error rates" />
      <div className="p-6 space-y-5">
        {/* Time range selector */}
        <div className="flex items-center gap-1.5">
          {RANGES.map(r => (
            <button
              key={r}
              onClick={() => setRange(r)}
              className={`px-3 py-1.5 text-sm rounded-md border transition-colors ${
                range === r
                  ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                  : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-300'
              }`}
            >
              {r}
            </button>
          ))}
          <div className="flex-1" />
          <Link
            href="/usage/forensics"
            className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
          >
            Request Forensics
          </Link>
        </div>

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {loading ? <Skeleton /> : (
          <>
            {/* Stat cards */}
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
              <StatCard
                label="Total Requests"
                value={summary ? summary.total_requests.toLocaleString() : '—'}
                icon={Activity}
                color="bg-blue-500/20 text-blue-400"
              />
              <StatCard
                label="Error Rate"
                value={summary ? `${summary.error_rate.toFixed(1)}%` : '—'}
                icon={AlertCircle}
                color="bg-red-500/20 text-red-400"
              />
              <StatCard
                label="Avg Latency"
                value={summary ? `${summary.avg_latency_ms.toFixed(0)}ms` : '—'}
                icon={Clock}
                color="bg-amber-500/20 text-amber-400"
              />
              <StatCard
                label="Top Endpoint"
                value={summary?.top_endpoint ?? '—'}
                icon={Zap}
                color="bg-purple-500/20 text-purple-400"
              />
            </div>

            {/* Top orgs table */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
              <div className="px-4 py-3 border-b border-zinc-800">
                <h2 className="text-sm font-semibold text-zinc-300">Top Orgs by Volume</h2>
              </div>
              {topOrgs.length === 0 ? (
                <div className="py-12 text-center text-sm text-zinc-600">No data available</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-zinc-800">
                      {['Org', 'Requests', 'Errors', 'Avg Latency'].map(h => (
                        <th key={h} className="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">
                          {h}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-zinc-800/50">
                    {topOrgs.map(org => (
                      <tr key={org.org_id} className="bg-zinc-900 hover:bg-zinc-800/60 transition-colors">
                        <td className="px-4 py-3">
                          <Link href={`/orgs/${org.org_id}`} className="text-blue-400 hover:underline">
                            {org.org_name ?? org.org_id}
                          </Link>
                        </td>
                        <td className="px-4 py-3 text-zinc-300">{org.requests.toLocaleString()}</td>
                        <td className="px-4 py-3 text-zinc-300">{org.errors.toLocaleString()}</td>
                        <td className="px-4 py-3 text-zinc-300">{org.avg_latency_ms.toFixed(0)}ms</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </>
        )}
      </div>
    </>
  )
}
