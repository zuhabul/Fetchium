'use client'

import { useEffect, useState } from 'react'
import TopBar from '@/components/layout/TopBar'

interface Flag {
  id: string
  key: string
  enabled: boolean
  description: string | null
  created_at: string
  updated_at: string
}

export default function FlagsPage() {
  const [flags, setFlags] = useState<Flag[]>([])
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [showCreate, setShowCreate] = useState(false)
  const [newKey, setNewKey] = useState('')
  const [newDesc, setNewDesc] = useState('')
  const [creating, setCreating] = useState(false)

  async function load() {
    setLoading(true)
    try {
      const res = await fetch('/api/admin/flags')
      if (res.ok) {
        const body = await res.json()
        setFlags(body.data ?? [])
      } else {
        setError('Failed to load flags')
      }
    } catch {
      setError('Network error')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  async function toggle(flag: Flag) {
    setSaving(flag.id)
    try {
      const res = await fetch(`/api/admin/flags/${flag.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: !flag.enabled }),
      })
      if (res.ok) {
        setFlags(prev => prev.map(f => f.id === flag.id ? { ...f, enabled: !f.enabled } : f))
      }
    } finally {
      setSaving(null)
    }
  }

  async function create() {
    if (!newKey.trim()) return
    setCreating(true)
    try {
      const res = await fetch('/api/admin/flags', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key: newKey.trim(), description: newDesc.trim() }),
      })
      if (res.ok) {
        setNewKey('')
        setNewDesc('')
        setShowCreate(false)
        load()
      }
    } finally {
      setCreating(false)
    }
  }

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="Feature Flags" />
      <div className="p-6 space-y-4">
        <div className="flex justify-between items-center">
          <p className="text-sm text-zinc-500">{flags.length} flags</p>
          <button
            onClick={() => setShowCreate(true)}
            className="text-xs px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-lg border border-zinc-700"
          >
            + New Flag
          </button>
        </div>

        {showCreate && (
          <div className="bg-zinc-900 border border-zinc-700 rounded-xl p-4 space-y-3">
            <h3 className="text-sm font-medium text-zinc-200">Create Flag</h3>
            <input
              value={newKey}
              onChange={e => setNewKey(e.target.value)}
              placeholder="flag.key (e.g. feature.new_dashboard)"
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm text-zinc-200 font-mono placeholder-zinc-600 focus:outline-none focus:border-zinc-500"
            />
            <input
              value={newDesc}
              onChange={e => setNewDesc(e.target.value)}
              placeholder="Description (optional)"
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-zinc-500"
            />
            <div className="flex gap-2">
              <button
                onClick={create}
                disabled={creating || !newKey.trim()}
                className="text-xs px-3 py-1.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white rounded-lg"
              >
                {creating ? 'Creating…' : 'Create'}
              </button>
              <button
                onClick={() => setShowCreate(false)}
                className="text-xs px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 rounded-lg border border-zinc-700"
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-3 text-red-400 text-sm">{error}</div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          {loading ? (
            <div className="p-8 text-center text-zinc-500 text-sm">Loading…</div>
          ) : flags.length === 0 ? (
            <div className="p-8 text-center text-zinc-500 text-sm">
              No feature flags yet. Create one to control feature rollouts.
            </div>
          ) : (
            <table className="w-full">
              <thead>
                <tr className="border-b border-zinc-800">
                  {['Flag Key', 'Description', 'Status', 'Updated', ''].map(h => (
                    <th key={h} className="text-left text-xs font-medium text-zinc-500 uppercase tracking-wider px-4 py-2.5">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {flags.map(flag => (
                  <tr key={flag.id} className="border-b border-zinc-800/60 hover:bg-zinc-800/30">
                    <td className="px-4 py-3 font-mono text-sm text-zinc-300">{flag.key}</td>
                    <td className="px-4 py-3 text-sm text-zinc-500">{flag.description ?? '—'}</td>
                    <td className="px-4 py-3">
                      <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${flag.enabled ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30' : 'bg-zinc-800 text-zinc-500 border-zinc-700'}`}>
                        {flag.enabled ? 'Enabled' : 'Disabled'}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-xs text-zinc-600">
                      {new Date(flag.updated_at).toLocaleDateString()}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <button
                        onClick={() => toggle(flag)}
                        disabled={saving === flag.id}
                        className={`text-xs px-3 py-1 rounded-lg border transition-colors ${
                          flag.enabled
                            ? 'bg-red-500/10 border-red-500/30 text-red-400 hover:bg-red-500/20'
                            : 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/20'
                        } disabled:opacity-50`}
                      >
                        {saving === flag.id ? '…' : flag.enabled ? 'Disable' : 'Enable'}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  )
}
