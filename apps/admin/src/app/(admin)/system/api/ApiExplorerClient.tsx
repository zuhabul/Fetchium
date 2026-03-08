'use client'

import { useState } from 'react'
import type { Route } from './page'

const METHOD_BADGE: Record<Route['method'], string> = {
  GET:    'bg-blue-500/20 text-blue-400 border-blue-500/30',
  POST:   'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
  PATCH:  'bg-amber-500/20 text-amber-400 border-amber-500/30',
  DELETE: 'bg-red-500/20 text-red-400 border-red-500/30',
  PUT:    'bg-purple-500/20 text-purple-400 border-purple-500/30',
}

interface Props { routes: Route[] }

export default function ApiExplorerClient({ routes }: Props) {
  const [filter, setFilter] = useState('')
  const [testPath, setTestPath] = useState<string | null>(null)

  const filtered = routes.filter(r =>
    r.path.toLowerCase().includes(filter.toLowerCase()) ||
    r.handler.toLowerCase().includes(filter.toLowerCase())
  )

  return (
    <div className="space-y-4">
      {/* Test modal */}
      {testPath && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6 max-w-lg w-full mx-4 space-y-4">
            <p className="text-sm font-semibold text-zinc-300">Test with curl</p>
            <pre className="bg-zinc-950 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-zinc-400 overflow-x-auto whitespace-pre-wrap">
{`curl -X GET https://api.hypersearchx.zuhabul.com${testPath} \\
  -H "Authorization: Bearer YOUR_TOKEN" \\
  -H "Content-Type: application/json"`}
            </pre>
            <button onClick={() => setTestPath(null)}
              className="w-full py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
              Close
            </button>
          </div>
        </div>
      )}

      {/* Search */}
      <input
        type="text"
        value={filter}
        onChange={e => setFilter(e.target.value)}
        placeholder="Filter by path or handler…"
        className="w-full max-w-sm bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-300 placeholder-zinc-600 focus:outline-none focus:border-zinc-600"
      />

      <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
        <div className="px-4 py-3 border-b border-zinc-800 flex items-center justify-between">
          <p className="text-sm font-semibold text-zinc-300">Routes</p>
          <span className="text-xs text-zinc-500">{filtered.length} / {routes.length}</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-zinc-800">
                <th className="px-4 py-2.5 text-left font-medium text-zinc-500 w-20">Method</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Path</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Auth</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Rate Limit</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Handler</th>
                <th className="px-4 py-2.5 text-left font-medium text-zinc-500"></th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((r, i) => (
                <tr key={i} className="border-b border-zinc-800/50 hover:bg-zinc-800/20">
                  <td className="px-4 py-3">
                    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${METHOD_BADGE[r.method]}`}>
                      {r.method}
                    </span>
                  </td>
                  <td className="px-4 py-3 font-mono text-zinc-300">{r.path}</td>
                  <td className="px-4 py-3 text-zinc-500">{r.auth}</td>
                  <td className="px-4 py-3 text-zinc-500">{r.rateLimit}</td>
                  <td className="px-4 py-3 font-mono text-zinc-600">{r.handler}</td>
                  <td className="px-4 py-3">
                    <button onClick={() => setTestPath(r.path)}
                      className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors">
                      Test
                    </button>
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
