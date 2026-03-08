'use client'

import { useState, useEffect, useRef } from 'react'
import TopBar from '@/components/layout/TopBar'

interface RequestRow {
  id: number
  time: string
  org: string
  key: string
  endpoint: string
  status: number
  latency: number
  tokens: number
  rateLimited: boolean
}

interface RequestDetail extends RequestRow {
  provider: string
  cacheHit: boolean
  searchMs: number
  rankMs: number
  outputMs: number
  tokenLimit: number
}

const ENDPOINTS = ['/v1/search', '/v1/scrape', '/v1/research', '/v1/health']
const ORGS = ['Acme Corp', 'TechStart', 'DevCo', 'Startup Inc']
const STATUSES = [200, 200, 200, 200, 429, 500]
const PROVIDERS = ['Google SERP', 'DDG', 'Bing', 'Brave', 'SearXNG']
const KEYS = ['fch_abc', 'fch_xyz', 'fch_def', 'fch_qrs']

let _reqId = 0
function pick<T>(arr: T[]): T { return arr[Math.floor(Math.random() * arr.length)] }

function makeRow(): RequestRow {
  const now = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return {
    id: ++_reqId,
    time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`,
    org: pick(ORGS),
    key: pick(KEYS) + '…',
    endpoint: pick(ENDPOINTS),
    status: pick(STATUSES),
    latency: Math.floor(Math.random() * 8000) + 50,
    tokens: Math.floor(Math.random() * 4000) + 100,
    rateLimited: Math.random() < 0.1,
  }
}

function makeDetail(row: RequestRow): RequestDetail {
  return {
    ...row,
    provider: pick(PROVIDERS),
    cacheHit: Math.random() < 0.3,
    searchMs: Math.floor(row.latency * 0.5),
    rankMs: Math.floor(row.latency * 0.2),
    outputMs: Math.floor(row.latency * 0.3),
    tokenLimit: 10000,
  }
}

const STATUS_COLOR: Record<number, string> = { 200: 'text-zinc-400', 429: 'text-amber-400', 500: 'text-red-400' }

export default function RequestInspectorPage() {
  const [rows, setRows] = useState<RequestRow[]>([])
  const [frozen, setFrozen] = useState(false)
  const [selected, setSelected] = useState<RequestDetail | null>(null)
  const [endpointFilter, setEndpointFilter] = useState('')
  const [statusFilter, setStatusFilter] = useState('')
  const [minLatency, setMinLatency] = useState('')
  const bufferRef = useRef<RequestRow[]>([])
  const frozenRef = useRef(false)

  useEffect(() => { frozenRef.current = frozen }, [frozen])

  useEffect(() => {
    const interval = setInterval(() => {
      const row = makeRow()
      bufferRef.current = [row, ...bufferRef.current].slice(0, 100)
      if (!frozenRef.current) setRows([...bufferRef.current])
    }, 500)
    return () => clearInterval(interval)
  }, [])

  function handleFreeze() {
    if (frozen) {
      setFrozen(false)
      setRows([...bufferRef.current])
    } else {
      setFrozen(true)
    }
  }

  const visible = rows.filter(r => {
    if (endpointFilter && r.endpoint !== endpointFilter) return false
    if (statusFilter && String(r.status) !== statusFilter) return false
    if (minLatency && r.latency < Number(minLatency)) return false
    return true
  })

  return (
    <>
      <TopBar title="Request Inspector" subtitle="Live API request stream" />
      <div className="p-6 space-y-4 max-w-full">

        {/* Filters + Freeze */}
        <div className="flex flex-wrap items-center gap-2">
          <select value={endpointFilter} onChange={e => setEndpointFilter(e.target.value)}
            className="bg-zinc-900 border border-zinc-800 rounded-md px-3 py-1.5 text-xs text-zinc-300 focus:outline-none">
            <option value="">All endpoints</option>
            {ENDPOINTS.map(e => <option key={e} value={e}>{e}</option>)}
          </select>
          <select value={statusFilter} onChange={e => setStatusFilter(e.target.value)}
            className="bg-zinc-900 border border-zinc-800 rounded-md px-3 py-1.5 text-xs text-zinc-300 focus:outline-none">
            <option value="">All statuses</option>
            <option value="200">200</option>
            <option value="429">429</option>
            <option value="500">500</option>
          </select>
          <input type="number" placeholder="Min latency (ms)" value={minLatency}
            onChange={e => setMinLatency(e.target.value)}
            className="bg-zinc-900 border border-zinc-800 rounded-md px-3 py-1.5 text-xs text-zinc-300 focus:outline-none w-36 placeholder-zinc-600" />
          <div className="flex-1" />
          <button onClick={handleFreeze}
            className={`px-3 py-1.5 text-xs font-medium border rounded-md transition-colors ${frozen ? 'bg-amber-500/20 border-amber-500/30 text-amber-400' : 'bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700'}`}>
            {frozen ? 'Frozen — Click to Resume' : 'Freeze'}
          </button>
        </div>

        <div className="flex gap-4">
          {/* Request table */}
          <div className="flex-1 min-w-0 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            <div className="overflow-x-auto max-h-[calc(100vh-280px)] overflow-y-auto">
              <table className="w-full text-xs">
                <thead className="sticky top-0 bg-zinc-900 border-b border-zinc-800">
                  <tr>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">Time</th>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">Org</th>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">Key</th>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">Endpoint</th>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">Status</th>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">Latency</th>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">Tokens</th>
                    <th className="px-3 py-2.5 text-left font-medium text-zinc-500">RL</th>
                  </tr>
                </thead>
                <tbody>
                  {visible.map(r => (
                    <tr key={r.id} onClick={() => setSelected(makeDetail(r))}
                      className={`border-b border-zinc-800/40 hover:bg-zinc-800/30 cursor-pointer ${selected?.id === r.id ? 'bg-zinc-800/50' : ''}`}>
                      <td className="px-3 py-2 font-mono text-zinc-600">{r.time}</td>
                      <td className="px-3 py-2 text-zinc-300">{r.org}</td>
                      <td className="px-3 py-2 font-mono text-zinc-500">{r.key}</td>
                      <td className="px-3 py-2 font-mono text-zinc-400">{r.endpoint}</td>
                      <td className={`px-3 py-2 font-mono font-medium ${STATUS_COLOR[r.status] ?? 'text-zinc-400'}`}>{r.status}</td>
                      <td className="px-3 py-2 font-mono text-zinc-400">{r.latency}ms</td>
                      <td className="px-3 py-2 text-zinc-500">{r.tokens}</td>
                      <td className="px-3 py-2">{r.rateLimited ? <span className="text-red-400">✓</span> : <span className="text-zinc-700">✗</span>}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {visible.length === 0 && (
                <div className="text-center py-12 text-xs text-zinc-600">No requests match filters…</div>
              )}
            </div>
          </div>

          {/* Detail drawer */}
          {selected && (
            <div className="w-72 flex-shrink-0 bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-4 h-fit">
              <div className="flex items-center justify-between">
                <p className="text-sm font-semibold text-zinc-300">Request Detail</p>
                <button onClick={() => setSelected(null)} className="text-zinc-600 hover:text-zinc-400 text-xs">✕</button>
              </div>

              <dl className="space-y-2 text-xs">
                <div className="flex justify-between">
                  <dt className="text-zinc-500">Endpoint</dt>
                  <dd className="font-mono text-zinc-300">{selected.endpoint}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-zinc-500">Status</dt>
                  <dd className={`font-mono font-medium ${STATUS_COLOR[selected.status] ?? 'text-zinc-400'}`}>{selected.status}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-zinc-500">Provider</dt>
                  <dd className="text-zinc-300">{selected.provider}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-zinc-500">Cache</dt>
                  <dd className={selected.cacheHit ? 'text-emerald-400' : 'text-zinc-500'}>{selected.cacheHit ? 'Hit' : 'Miss'}</dd>
                </div>
              </dl>

              <div className="border-t border-zinc-800 pt-3">
                <p className="text-xs font-medium text-zinc-500 mb-2">Timing</p>
                <dl className="space-y-1.5 text-xs">
                  <div className="flex justify-between">
                    <dt className="text-zinc-500">search_ms</dt>
                    <dd className="font-mono text-zinc-300">{selected.searchMs}ms</dd>
                  </div>
                  <div className="flex justify-between">
                    <dt className="text-zinc-500">rank_ms</dt>
                    <dd className="font-mono text-zinc-300">{selected.rankMs}ms</dd>
                  </div>
                  <div className="flex justify-between">
                    <dt className="text-zinc-500">output_ms</dt>
                    <dd className="font-mono text-zinc-300">{selected.outputMs}ms</dd>
                  </div>
                </dl>
              </div>

              <div className="border-t border-zinc-800 pt-3">
                <p className="text-xs font-medium text-zinc-500 mb-2">Token Budget</p>
                <div className="flex justify-between text-xs mb-1.5">
                  <span className="text-zinc-500">Used</span>
                  <span className="text-zinc-300">{selected.tokens} / {selected.tokenLimit}</span>
                </div>
                <div className="w-full bg-zinc-800 rounded-full h-1.5">
                  <div
                    className="bg-emerald-500 h-1.5 rounded-full"
                    style={{ width: `${Math.min(100, (selected.tokens / selected.tokenLimit) * 100)}%` }}
                  />
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </>
  )
}
