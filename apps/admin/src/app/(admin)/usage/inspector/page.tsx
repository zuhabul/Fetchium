'use client'

import { useState, useEffect, useRef } from 'react'
import TopBar from '@/components/layout/TopBar'

interface AuditRow {
  id: string
  user_email: string | null
  user_name: string | null
  role: string | null
  target_type: string
  target_id: string | null
  action: string
  ip: string | null
  created_at: string
}

const ACTION_COLOR: Record<string, string> = {
  create: 'text-emerald-400',
  update: 'text-blue-400',
  delete: 'text-red-400',
  suspend: 'text-amber-400',
  reset: 'text-orange-400',
}

function actionColor(action: string) {
  for (const [k, v] of Object.entries(ACTION_COLOR)) {
    if (action.includes(k)) return v
  }
  return 'text-zinc-400'
}

function fmt(date: string) {
  const d = new Date(date)
  return d.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

export default function RequestInspectorPage() {
  const [rows, setRows] = useState<AuditRow[]>([])
  const [frozen, setFrozen] = useState(false)
  const [selected, setSelected] = useState<AuditRow | null>(null)
  const [actionFilter, setActionFilter] = useState('')
  const [targetFilter, setTargetFilter] = useState('')
  const [error, setError] = useState<string | null>(null)
  const frozenRef = useRef(false)

  useEffect(() => { frozenRef.current = frozen }, [frozen])

  async function fetchActivity() {
    try {
      const res = await fetch('/api/admin/audit?limit=100&offset=0')
      if (!res.ok) { setError('Failed to load'); return }
      const body = await res.json()
      setError(null)
      if (!frozenRef.current) setRows(body.data ?? [])
    } catch { setError('Network error') }
  }

  useEffect(() => {
    fetchActivity()
    const interval = setInterval(() => { if (!frozenRef.current) fetchActivity() }, 5000)
    return () => clearInterval(interval)
  }, [])

  const visible = rows.filter(r => {
    if (actionFilter && !r.action.includes(actionFilter)) return false
    if (targetFilter && r.target_type !== targetFilter) return false
    return true
  })

  const targetTypes = [...new Set(rows.map(r => r.target_type))].sort()

  return (
    <>
      <TopBar title="Activity Inspector" subtitle="Live admin audit stream — refreshes every 5s" />
      <div className="p-6 space-y-4 max-w-full">

        <div className="flex flex-wrap items-center gap-2">
          <input type="text" placeholder="Filter action…" value={actionFilter}
            onChange={e => setActionFilter(e.target.value)}
            className="bg-zinc-900 border border-zinc-800 rounded-md px-3 py-1.5 text-xs text-zinc-300 focus:outline-none w-40 placeholder-zinc-600" />
          <select value={targetFilter} onChange={e => setTargetFilter(e.target.value)}
            className="bg-zinc-900 border border-zinc-800 rounded-md px-3 py-1.5 text-xs text-zinc-300 focus:outline-none">
            <option value="">All targets</option>
            {targetTypes.map(t => <option key={t} value={t}>{t}</option>)}
          </select>
          <div className="flex-1" />
          <button onClick={() => { setFrozen(f => !f); if (frozen) fetchActivity() }}
            className={'px-3 py-1.5 text-xs font-medium border rounded-md transition-colors ' + (frozen ? 'bg-amber-500/20 border-amber-500/30 text-amber-400' : 'bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700')}>
            {frozen ? 'Frozen — Resume' : 'Freeze'}
          </button>
        </div>

        {error && <div className="bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2 text-xs text-red-400">{error}</div>}

        <div className="flex gap-4">
          <div className="flex-1 min-w-0 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            <div className="overflow-x-auto max-h-[calc(100vh-280px)] overflow-y-auto">
              <table className="w-full text-xs">
                <thead className="sticky top-0 bg-zinc-900 border-b border-zinc-800">
                  <tr>
                    {['Time', 'Actor', 'Role', 'Action', 'Target', 'IP'].map(h => (
                      <th key={h} className="px-3 py-2.5 text-left font-medium text-zinc-500">{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {visible.length === 0 && !error && (
                    <tr><td colSpan={6} className="px-3 py-12 text-center text-zinc-600">No events yet…</td></tr>
                  )}
                  {visible.map(r => (
                    <tr key={r.id} onClick={() => setSelected(selected?.id === r.id ? null : r)}
                      className={'border-b border-zinc-800/40 hover:bg-zinc-800/30 cursor-pointer ' + (selected?.id === r.id ? 'bg-zinc-800/50' : '')}>
                      <td className="px-3 py-2 font-mono text-zinc-600">{fmt(r.created_at)}</td>
                      <td className="px-3 py-2 text-zinc-300">{r.user_name ?? 'system'}</td>
                      <td className="px-3 py-2 text-zinc-500">{r.role ?? '—'}</td>
                      <td className={'px-3 py-2 font-mono ' + actionColor(r.action)}>{r.action}</td>
                      <td className="px-3 py-2 text-zinc-500">{r.target_type}{r.target_id ? ` / ${r.target_id.slice(0,8)}…` : ''}</td>
                      <td className="px-3 py-2 font-mono text-zinc-700">{r.ip ?? '—'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {selected && (
            <div className="w-64 flex-shrink-0 bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3 h-fit">
              <div className="flex items-center justify-between">
                <p className="text-sm font-semibold text-zinc-300">Event Detail</p>
                <button onClick={() => setSelected(null)} className="text-zinc-600 hover:text-zinc-400 text-xs">✕</button>
              </div>
              <dl className="space-y-2 text-xs">
                {[
                  ['ID', selected.id.slice(0, 12) + '…'],
                  ['Actor', selected.user_name ?? 'system'],
                  ['Email', selected.user_email ?? '—'],
                  ['Role', selected.role ?? '—'],
                  ['Action', selected.action],
                  ['Target type', selected.target_type],
                  ['Target ID', selected.target_id?.slice(0, 16) ?? '—'],
                  ['IP', selected.ip ?? '—'],
                  ['Time', new Date(selected.created_at).toLocaleString()],
                ].map(([k, v]) => (
                  <div key={k} className="flex justify-between gap-2">
                    <dt className="text-zinc-500 flex-shrink-0">{k}</dt>
                    <dd className="font-mono text-zinc-300 text-right truncate">{v}</dd>
                  </div>
                ))}
              </dl>
            </div>
          )}
        </div>

        <p className="text-xs text-zinc-700">Auto-refreshes every 5s · {visible.length} events shown</p>
      </div>
    </>
  )
}
