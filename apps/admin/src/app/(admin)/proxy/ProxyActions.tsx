'use client'

import { useState } from 'react'
import { INTERNAL_API } from '@/lib/session'

export default function ProxyActions({ sessionToken }: { sessionToken: string }) {
  const [status, setStatus] = useState<string | null>(null)
  const [loading, setLoading] = useState<string | null>(null)

  async function call(endpoint: string, label: string) {
    setLoading(label)
    setStatus(null)
    try {
      const res = await fetch(`/api/admin/proxy/${endpoint}`, { method: 'POST' })
      const body = await res.json()
      setStatus(body.message ?? (res.ok ? 'Done' : 'Failed'))
    } catch {
      setStatus('Network error')
    } finally {
      setLoading(null)
    }
  }

  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
      <h2 className="text-sm font-semibold text-zinc-100">Actions</h2>
      {status && (
        <p className="text-xs text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 rounded-lg px-3 py-2">{status}</p>
      )}
      <div className="flex flex-wrap gap-2">
        <button
          onClick={() => call('reset', 'reset')}
          disabled={loading === 'reset'}
          className="text-xs px-4 py-2 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30 text-amber-400 rounded-lg disabled:opacity-50"
        >
          {loading === 'reset' ? 'Resetting…' : 'Reset Proxy Pool'}
        </button>
        <button
          onClick={() => call('purge', 'purge')}
          disabled={loading === 'purge'}
          className="text-xs px-4 py-2 bg-red-500/10 hover:bg-red-500/20 border border-red-500/30 text-red-400 rounded-lg disabled:opacity-50"
        >
          {loading === 'purge' ? 'Purging…' : 'Purge Cache'}
        </button>
      </div>
    </div>
  )
}
