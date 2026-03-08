'use client'

import { useState } from 'react'
import TopBar from '@/components/layout/TopBar'

const TABLES = [
  'admin_users', 'admin_sessions', 'organizations', 'subscriptions',
  'support_tickets', 'incidents', 'audit_events', 'feature_flags',
  'approval_requests', 'campaigns',
]

interface QueryResult {
  columns: string[]
  rows: unknown[][]
  row_count?: number
  error?: string
}

function downloadCsv(result: QueryResult) {
  const lines = [
    result.columns.join(','),
    ...result.rows.map(row => row.map(v => JSON.stringify(v ?? '')).join(',')),
  ]
  const blob = new Blob([lines.join('\n')], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'query_result.csv'
  a.click()
  URL.revokeObjectURL(url)
}

export default function DbInspectorPage() {
  const [tab, setTab] = useState<'browse' | 'query'>('browse')
  const [selectedTable, setSelectedTable] = useState<string | null>(null)
  const [browseResult, setBrowseResult] = useState<QueryResult | null>(null)
  const [browseLoading, setBrowseLoading] = useState(false)
  const [sql, setSql] = useState('')
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null)
  const [queryLoading, setQueryLoading] = useState(false)

  async function loadTable(table: string) {
    setSelectedTable(table)
    setBrowseLoading(true)
    setBrowseResult(null)
    try {
      const res = await fetch('/api/admin/db/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sql: `SELECT * FROM ${table} LIMIT 50` }),
      })
      const data = await res.json()
      setBrowseResult(data)
    } catch (e) {
      setBrowseResult({ columns: [], rows: [], error: String(e) })
    } finally {
      setBrowseLoading(false)
    }
  }

  async function runQuery() {
    if (!sql.trim()) return
    setQueryLoading(true)
    setQueryResult(null)
    try {
      const res = await fetch('/api/admin/db/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sql }),
      })
      const data = await res.json()
      setQueryResult(data)
    } catch (e) {
      setQueryResult({ columns: [], rows: [], error: String(e) })
    } finally {
      setQueryLoading(false)
    }
  }

  const activeResult = tab === 'browse' ? browseResult : queryResult

  return (
    <>
      <TopBar title="DB Inspector" subtitle="Browse tables and run read-only queries" />
      <div className="p-6 space-y-4 max-w-6xl">

        {/* Caution banner */}
        <div className="bg-amber-500/10 border border-amber-500/20 rounded-lg px-4 py-2.5 text-xs text-amber-400">
          Read-only — INSERT / UPDATE / DELETE are rejected by the server.
        </div>

        {/* Tabs */}
        <div className="flex gap-1 bg-zinc-900 border border-zinc-800 rounded-lg p-1 w-fit">
          {(['browse', 'query'] as const).map(t => (
            <button key={t} onClick={() => setTab(t)}
              className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors capitalize ${tab === t ? 'bg-zinc-700 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}>
              {t === 'browse' ? 'Browse Tables' : 'Run Query'}
            </button>
          ))}
        </div>

        <div className="flex gap-4">
          {/* Browse tab: table list */}
          {tab === 'browse' && (
            <div className="w-48 flex-shrink-0 bg-zinc-900 border border-zinc-800 rounded-xl p-2 h-fit">
              {TABLES.map(t => (
                <button key={t} onClick={() => loadTable(t)}
                  className={`w-full text-left px-3 py-2 rounded-md text-xs transition-colors ${selectedTable === t ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/60'}`}>
                  {t}
                </button>
              ))}
            </div>
          )}

          {/* Main content */}
          <div className="flex-1 min-w-0 space-y-3">
            {tab === 'query' && (
              <div className="space-y-2">
                <textarea
                  value={sql}
                  onChange={e => setSql(e.target.value)}
                  placeholder="SELECT * FROM organizations LIMIT 10"
                  rows={4}
                  className="w-full bg-zinc-900 border border-zinc-800 rounded-xl p-3 text-xs font-mono text-zinc-300 placeholder-zinc-600 resize-none focus:outline-none focus:border-zinc-600"
                />
                <div className="flex items-center gap-2">
                  <button onClick={runQuery} disabled={queryLoading}
                    className="px-4 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors disabled:opacity-50">
                    {queryLoading ? 'Running…' : 'Run'}
                  </button>
                  <span className="text-[10px] text-zinc-600">SELECT only — write statements are rejected</span>
                </div>
              </div>
            )}

            {browseLoading && (
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-8 text-center text-xs text-zinc-500">Loading…</div>
            )}

            {activeResult && !activeResult.error && activeResult.columns.length > 0 && (
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-zinc-800">
                  <span className="text-xs text-zinc-500">{activeResult.rows.length} rows</span>
                  <button onClick={() => downloadCsv(activeResult)}
                    className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors">
                    Export CSV
                  </button>
                </div>
                <div className="overflow-x-auto">
                  <table className="w-full text-xs">
                    <thead>
                      <tr className="border-b border-zinc-800">
                        {activeResult.columns.map(col => (
                          <th key={col} className="px-3 py-2 text-left font-medium text-zinc-500 whitespace-nowrap">{col}</th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {activeResult.rows.map((row, i) => (
                        <tr key={i} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                          {row.map((cell, j) => (
                            <td key={j} className="px-3 py-2 text-zinc-400 font-mono whitespace-nowrap max-w-[200px] truncate">
                              {cell === null ? <span className="text-zinc-600">null</span> : String(cell)}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {activeResult?.error && (
              <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-4 text-xs text-red-400 font-mono">
                {activeResult.error}
              </div>
            )}

            {activeResult && !activeResult.error && activeResult.columns.length === 0 && (
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-8 text-center text-xs text-zinc-500">
                No results returned.
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
