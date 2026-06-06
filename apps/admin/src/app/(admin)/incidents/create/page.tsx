'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import TopBar from '@/components/layout/TopBar'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { AlertTriangle } from 'lucide-react'

const SEVERITIES = ['low', 'medium', 'high', 'critical'] as const

export default function CreateIncidentPage() {
  const router = useRouter()
  const [title, setTitle] = useState('')
  const [severity, setSeverity] = useState<string>('low')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!title.trim()) { setError('Title is required'); return }
    setSaving(true)
    setError('')
    try {
      const res = await fetch('/api/admin/incidents', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: title.trim(), severity }),
      })
      const data = await res.json()
      if (!res.ok) { setError(data.error ?? 'Failed to create incident'); return }
      router.push(`/incidents/${data.id}`)
      router.refresh()
    } catch {
      setError('Network error — please try again')
    } finally {
      setSaving(false)
    }
  }

  return (
    <>
      <TopBar title="Create Incident" subtitle="Open a new incident and start the response timeline" />
      <div className={`${ADMIN_PAGE_PADDING} max-w-xl`}>
        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-6 space-y-5">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 text-red-400" />
            <h2 className="text-sm font-semibold text-zinc-200">New Incident</h2>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-zinc-400 mb-1.5">
                Title <span className="text-red-400">*</span>
              </label>
              <input
                type="text"
                value={title}
                onChange={e => setTitle(e.target.value)}
                placeholder="e.g. Search API elevated error rate"
                className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600 focus:border-zinc-500 focus:outline-none"
                autoFocus
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-zinc-400 mb-1.5">Severity</label>
              <div className="grid grid-cols-4 gap-2">
                {SEVERITIES.map(s => {
                  const styles: Record<string, string> = {
                    low: 'border-blue-500/40 bg-blue-500/10 text-blue-400',
                    medium: 'border-amber-500/40 bg-amber-500/10 text-amber-400',
                    high: 'border-orange-500/40 bg-orange-500/10 text-orange-400',
                    critical: 'border-red-500/40 bg-red-500/10 text-red-400',
                  }
                  const inactive = 'border-zinc-700 bg-zinc-800 text-zinc-500'
                  return (
                    <button
                      key={s}
                      type="button"
                      onClick={() => setSeverity(s)}
                      className={`rounded-md border px-3 py-2 text-xs font-medium capitalize transition-colors ${severity === s ? styles[s] : inactive}`}
                    >
                      {s}
                    </button>
                  )
                })}
              </div>
            </div>

            {error && (
              <p className="rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-400">
                {error}
              </p>
            )}

            <div className="flex gap-2 pt-1">
              <button
                type="submit"
                disabled={saving}
                className="flex-1 rounded-md bg-red-500/20 border border-red-500/30 px-4 py-2 text-sm font-medium text-red-300 transition-colors hover:bg-red-500/30 disabled:opacity-50"
              >
                {saving ? 'Creating…' : 'Create Incident'}
              </button>
              <button
                type="button"
                onClick={() => router.back()}
                className="rounded-md border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm text-zinc-400 transition-colors hover:bg-zinc-700"
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      </div>
    </>
  )
}
