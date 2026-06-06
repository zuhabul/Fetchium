'use client'

import { useEffect, useState } from 'react'
import { Activity, CheckCircle2, Plus, ToggleLeft } from 'lucide-react'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import TopBar from '@/components/layout/TopBar'

interface Flag {
  id: string
  key: string
  enabled: boolean
  description: string | null
  created_at: string
  updated_at: string
}

function format_date(value: string) {
  return new Date(value).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
}

function StatCard({
  label,
  value,
  icon: Icon,
  color,
}: {
  label: string
  value: number
  icon: React.ElementType
  color: string
}) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
      <div className="mb-3 flex items-center justify-between gap-3">
        <span className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">{label}</span>
        <div className={`flex h-8 w-8 items-center justify-center rounded-lg ${color}`}>
          <Icon className="h-4 w-4" />
        </div>
      </div>
      <p className="text-[1.75rem] font-bold leading-none text-zinc-100 lg:text-2xl">
        {value.toLocaleString()}
      </p>
    </div>
  )
}

function StatusBadge({ enabled }: { enabled: boolean }) {
  return (
    <span
      className={`rounded-full border px-2 py-0.5 text-xs font-medium ${
        enabled
          ? 'border-emerald-500/30 bg-emerald-500/20 text-emerald-400'
          : 'border-zinc-700 bg-zinc-800 text-zinc-500'
      }`}
    >
      {enabled ? 'Enabled' : 'Disabled'}
    </span>
  )
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
    setError(null)
    try {
      const res = await fetch('/api/admin/flags')
      if (res.ok) {
        const body = await res.json()
        setFlags(Array.isArray(body?.data) ? body.data : [])
      } else {
        setError('Failed to load flags')
      }
    } catch {
      setError('Network error')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    load()
  }, [])

  async function toggle(flag: Flag) {
    setSaving(flag.id)
    try {
      const res = await fetch(`/api/admin/flags/${flag.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: !flag.enabled }),
      })
      if (res.ok) {
        setFlags(prev =>
          prev.map(f => (f.id === flag.id ? { ...f, enabled: !f.enabled } : f))
        )
      } else {
        setError('Failed to update flag')
      }
    } catch {
      setError('Network error')
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
      } else {
        setError('Failed to create flag')
      }
    } catch {
      setError('Network error')
    } finally {
      setCreating(false)
    }
  }

  const enabledCount = flags.filter(flag => flag.enabled).length
  const disabledCount = flags.length - enabledCount

  return (
    <div className="flex min-h-full flex-col">
      <TopBar title="Feature Flags" subtitle="Manage rollout switches and guarded features" />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>
        <div className="grid grid-cols-1 gap-3 min-[420px]:grid-cols-2 xl:grid-cols-4">
          <StatCard
            label="Total"
            value={flags.length}
            icon={Activity}
            color="bg-blue-500/20 text-blue-400"
          />
          <StatCard
            label="Enabled"
            value={enabledCount}
            icon={CheckCircle2}
            color="bg-emerald-500/20 text-emerald-400"
          />
          <StatCard
            label="Disabled"
            value={disabledCount}
            icon={ToggleLeft}
            color="bg-zinc-700 text-zinc-300"
          />
          <StatCard
            label="Loaded"
            value={loading ? 0 : flags.length}
            icon={Activity}
            color="bg-amber-500/20 text-amber-400"
          />
        </div>

        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <h2 className="text-sm font-semibold text-zinc-100">Flag Registry</h2>
              <p className="mt-1 text-xs text-zinc-500">
                {flags.length.toLocaleString()} feature flag{flags.length === 1 ? '' : 's'} tracked
              </p>
            </div>

            <button
              onClick={() => setShowCreate(prev => !prev)}
              className="inline-flex min-h-11 items-center justify-center gap-1.5 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 sm:min-h-9 sm:self-start sm:py-1.5 lg:self-auto"
            >
              <Plus className="h-3.5 w-3.5" />
              {showCreate ? 'Close' : 'New Flag'}
            </button>
          </div>
        </div>

        {showCreate && (
          <div className="rounded-xl border border-zinc-700 bg-zinc-900 p-4">
            <div className="flex flex-col gap-4">
              <div>
                <h3 className="text-sm font-medium text-zinc-200">Create Flag</h3>
                <p className="mt-1 text-xs text-zinc-500">
                  Use stable dot-separated keys for long-term rollout control.
                </p>
              </div>

              <div className="grid gap-3 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)]">
                <input
                  value={newKey}
                  onChange={e => setNewKey(e.target.value)}
                  placeholder="flag.key (e.g. feature.new_dashboard)"
                  className="min-h-11 w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 font-mono text-sm text-zinc-200 placeholder-zinc-600 focus:border-zinc-500 focus:outline-none sm:min-h-10"
                />
                <input
                  value={newDesc}
                  onChange={e => setNewDesc(e.target.value)}
                  placeholder="Description (optional)"
                  className="min-h-11 w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-200 placeholder-zinc-600 focus:border-zinc-500 focus:outline-none sm:min-h-10"
                />
              </div>

              <div className="flex flex-col gap-2 sm:flex-row">
                <button
                  onClick={create}
                  disabled={creating || !newKey.trim()}
                  className="inline-flex min-h-11 items-center justify-center rounded-md bg-blue-600 px-3 py-2 text-sm text-white transition-colors hover:bg-blue-500 disabled:opacity-50 sm:min-h-9 sm:px-4 sm:py-1.5"
                >
                  {creating ? 'Creating…' : 'Create Flag'}
                </button>
                <button
                  onClick={() => setShowCreate(false)}
                  className="inline-flex min-h-11 items-center justify-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-400 transition-colors hover:bg-zinc-700 sm:min-h-9 sm:px-4 sm:py-1.5"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        )}

        {error && (
          <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900">
          {loading ? (
            <div className="p-8 text-center text-sm text-zinc-500">Loading…</div>
          ) : flags.length === 0 ? (
            <div className="p-8 text-center text-sm text-zinc-500">
              No feature flags yet. Create one to control feature rollouts.
            </div>
          ) : (
            <>
              <div className="divide-y divide-zinc-800/60 lg:hidden">
                {flags.map(flag => (
                  <div key={flag.id} className="space-y-4 px-4 py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 space-y-1">
                        <p className="break-all font-mono text-sm text-zinc-200">{flag.key}</p>
                        <p className="text-xs text-zinc-500">
                          {flag.description ?? 'No description provided'}
                        </p>
                      </div>
                      <StatusBadge enabled={flag.enabled} />
                    </div>

                    <dl className="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Created</dt>
                        <dd className="mt-1 text-zinc-400">{format_date(flag.created_at)}</dd>
                      </div>
                      <div>
                        <dt className="text-[11px] uppercase tracking-wider text-zinc-600">Updated</dt>
                        <dd className="mt-1 text-zinc-400">{format_date(flag.updated_at)}</dd>
                      </div>
                    </dl>

                    <button
                      onClick={() => toggle(flag)}
                      disabled={saving === flag.id}
                      className={`inline-flex min-h-11 w-full items-center justify-center rounded-md border px-3 py-2 text-sm transition-colors disabled:opacity-50 ${
                        flag.enabled
                          ? 'border-red-500/30 bg-red-500/10 text-red-400 hover:bg-red-500/20'
                          : 'border-emerald-500/30 bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20'
                      }`}
                    >
                      {saving === flag.id ? 'Updating…' : flag.enabled ? 'Disable Flag' : 'Enable Flag'}
                    </button>
                  </div>
                ))}
              </div>

              <table className="hidden w-full text-sm lg:table">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['Flag Key', 'Description', 'Status', 'Created', 'Updated', ''].map(h => (
                      <th
                        key={h}
                        className="px-4 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-zinc-500"
                      >
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {flags.map(flag => (
                    <tr key={flag.id} className="border-b border-zinc-800/60 hover:bg-zinc-800/30">
                      <td className="px-4 py-3 font-mono text-sm text-zinc-300">{flag.key}</td>
                      <td className="px-4 py-3 text-sm text-zinc-500">
                        {flag.description ?? '—'}
                      </td>
                      <td className="px-4 py-3">
                        <StatusBadge enabled={flag.enabled} />
                      </td>
                      <td className="px-4 py-3 text-xs text-zinc-600">
                        {format_date(flag.created_at)}
                      </td>
                      <td className="px-4 py-3 text-xs text-zinc-600">
                        {format_date(flag.updated_at)}
                      </td>
                      <td className="px-4 py-3 text-right">
                        <button
                          onClick={() => toggle(flag)}
                          disabled={saving === flag.id}
                          className={`rounded-lg border px-3 py-1.5 text-xs transition-colors disabled:opacity-50 ${
                            flag.enabled
                              ? 'border-red-500/30 bg-red-500/10 text-red-400 hover:bg-red-500/20'
                              : 'border-emerald-500/30 bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20'
                          }`}
                        >
                          {saving === flag.id ? '…' : flag.enabled ? 'Disable' : 'Enable'}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </div>
      </div>
    </div>
  )
}
