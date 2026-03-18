'use client'

import { useState, useEffect, useCallback } from 'react'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { Activity, AlertCircle, Clock, Zap } from 'lucide-react'

type Range = '1h' | '6h' | '24h' | '7d' | '30d'

interface UsageSummary {
  total_requests: number
  error_rate: number | null
  avg_latency_ms: number | null
  top_endpoint: string | null
}

interface TopOrg {
  org_id: string
  org_name: string
  requests: number
  errors: number
  avg_latency_ms: number | null
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
      <div className="mb-3 flex items-center justify-between gap-3">
        <span className="text-xs font-medium text-zinc-500 uppercase tracking-wider">{label}</span>
        <div className={`flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-lg ${color}`}>
          <Icon className="w-3.5 h-3.5" />
        </div>
      </div>
      <p className="break-words text-xl font-bold text-zinc-100 sm:text-2xl">{value}</p>
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

function asNumber(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : null
  }
  return null
}

function asString(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value : null
}

function normalizeSummary(payload: unknown): UsageSummary {
  const data = typeof payload === 'object' && payload !== null
    ? payload as Record<string, unknown>
    : {}

  const totalRequests = asNumber(data.total_requests) ?? 0
  const errorRate = asNumber(data.error_rate)
  const errorCount = asNumber(data.error_count)

  return {
    total_requests: totalRequests,
    error_rate: errorRate ?? (
      totalRequests > 0 && errorCount != null ? (errorCount / totalRequests) * 100 : null
    ),
    avg_latency_ms: asNumber(data.avg_latency_ms),
    top_endpoint: asString(data.top_endpoint),
  }
}

function normalizeTopOrgs(payload: unknown): TopOrg[] {
  const data = typeof payload === 'object' && payload !== null
    ? payload as Record<string, unknown>
    : {}

  const rawOrgs = Array.isArray(data.orgs)
    ? data.orgs
    : Array.isArray(data.data)
      ? data.data
      : []

  return rawOrgs.map((item, index) => {
    const row = typeof item === 'object' && item !== null
      ? item as Record<string, unknown>
      : {}

    return {
      org_id: asString(row.org_id) ?? asString(row.id) ?? `org-${index}`,
      org_name: asString(row.org_name) ?? asString(row.name) ?? 'Unknown organization',
      requests: asNumber(row.requests) ?? asNumber(row.event_count) ?? 0,
      errors: asNumber(row.errors) ?? 0,
      avg_latency_ms: asNumber(row.avg_latency_ms),
    }
  })
}

function formatLatency(value: number | null) {
  return value != null ? `${value.toFixed(0)}ms` : '—'
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
      if (usageRes.ok) {
        setSummary(normalizeSummary(await usageRes.json()))
      } else {
        setSummary(normalizeSummary(null))
      }
      if (orgsRes.ok) {
        setTopOrgs(normalizeTopOrgs(await orgsRes.json()))
      } else {
        setTopOrgs([])
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch usage data')
      setSummary(normalizeSummary(null))
      setTopOrgs([])
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { fetchData(range) }, [range, fetchData])

  const RANGES: Range[] = ['1h', '6h', '24h', '7d', '30d']

  return (
    <>
      <TopBar title="Usage Explorer" subtitle="Request volume and error rates" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="grid grid-cols-3 gap-2 sm:flex sm:flex-wrap sm:items-center">
            {RANGES.map(r => (
              <button
                key={r}
                onClick={() => setRange(r)}
                className={`min-h-11 rounded-md border px-3 py-2 text-sm transition-colors sm:min-h-9 sm:py-1.5 ${
                  range === r
                    ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                    : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-300'
                }`}
              >
                {r}
              </button>
            ))}
          </div>

          <Link
            href="/usage/forensics"
            className="inline-flex min-h-11 items-center justify-center rounded-md border border-blue-500/30 bg-blue-500/20 px-3 py-2 text-sm text-blue-300 transition-colors hover:bg-blue-500/30 sm:min-h-9 sm:self-start sm:py-1.5 lg:self-auto"
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
                value={summary?.error_rate != null ? `${summary.error_rate.toFixed(1)}%` : '—'}
                icon={AlertCircle}
                color="bg-red-500/20 text-red-400"
              />
              <StatCard
                label="Avg Latency"
                value={summary?.avg_latency_ms != null ? `${summary.avg_latency_ms.toFixed(0)}ms` : '—'}
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
                <>
                  <div className="divide-y divide-zinc-800/50 lg:hidden">
                    {topOrgs.map(org => (
                      <div key={org.org_id} className="space-y-4 px-4 py-4">
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0 space-y-1">
                            <Link
                              href={`/orgs/${org.org_id}`}
                              className="block truncate text-sm font-medium text-blue-400 hover:underline"
                            >
                              {org.org_name ?? org.org_id}
                            </Link>
                            <p className="text-xs text-zinc-500">{org.org_id}</p>
                          </div>
                          <div className="rounded-lg border border-zinc-800 bg-zinc-950/70 px-3 py-2 text-right">
                            <p className="text-[11px] uppercase tracking-wider text-zinc-600">Requests</p>
                            <p className="mt-1 text-sm font-semibold text-zinc-100">
                              {org.requests.toLocaleString()}
                            </p>
                          </div>
                        </div>

                        <dl className="grid grid-cols-2 gap-3 text-sm">
                          <div>
                            <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Errors</dt>
                            <dd className="mt-1 text-zinc-300">{org.errors.toLocaleString()}</dd>
                          </div>
                          <div>
                            <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Avg Latency</dt>
                            <dd className="mt-1 text-zinc-300">{formatLatency(org.avg_latency_ms)}</dd>
                          </div>
                        </dl>
                      </div>
                    ))}
                  </div>

                  <table className="hidden w-full text-sm lg:table">
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
                          <td className="px-4 py-3 text-zinc-300">{formatLatency(org.avg_latency_ms)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </>
              )}
            </div>
          </>
        )}
      </div>
    </>
  )
}
