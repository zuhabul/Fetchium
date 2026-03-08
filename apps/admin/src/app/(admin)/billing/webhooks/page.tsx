'use client'

import { useState, useEffect } from 'react'
import TopBar from '@/components/layout/TopBar'

interface WebhookEvent {
  id: string
  event_type: string
  org_id?: string
  org_name?: string
  amount_cents?: number
  status: 'processed' | 'failed'
  created_at: string
  payload: Record<string, unknown>
}

function StatusBadge({ status }: { status: string }) {
  const cls =
    status === 'processed'
      ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'
      : 'bg-red-500/20 text-red-400 border-red-500/30'
  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${cls}`}>
      {status}
    </span>
  )
}

function PayloadViewer({ payload }: { payload: Record<string, unknown> }) {
  const [open, setOpen] = useState(false)
  return (
    <div>
      <button
        onClick={() => setOpen((o) => !o)}
        className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
      >
        {open ? 'Hide payload' : 'View payload'}
      </button>
      {open && (
        <pre className="mt-2 text-xs text-zinc-400 bg-zinc-950 border border-zinc-800 rounded-lg p-3 overflow-x-auto max-h-48">
          {JSON.stringify(payload, null, 2)}
        </pre>
      )}
    </div>
  )
}

export default function WebhooksPage() {
  const [events, setEvents] = useState<WebhookEvent[]>([])
  const [error, setError] = useState(false)
  const [replayingId, setReplayingId] = useState<string | null>(null)
  const [toast, setToast] = useState<string | null>(null)

  useEffect(() => {
    fetch('/api/admin/billing/webhooks', { cache: 'no-store' })
      .then((r) => (r.ok ? r.json() : Promise.reject()))
      .then(setEvents)
      .catch(() => setError(true))
  }, [])

  async function handleReplay(id: string) {
    setReplayingId(id)
    try {
      await fetch(`/api/admin/billing/webhooks/${id}/replay`, { method: 'POST' })
      setToast('Webhook replayed successfully')
    } catch {
      setToast('Failed to replay webhook')
    } finally {
      setReplayingId(null)
      setTimeout(() => setToast(null), 3000)
    }
  }

  return (
    <div className="flex flex-col min-h-full">
      <TopBar title="Webhook Events" />
      <div className="p-6">
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-red-400 text-sm mb-6">
            Failed to load data
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-xl">
          <div className="px-4 py-3 border-b border-zinc-800">
            <h2 className="text-sm font-semibold text-zinc-100">Webhook Event Log</h2>
          </div>
          <table className="w-full">
            <thead>
              <tr className="border-b border-zinc-800">
                {['Event Type', 'Org', 'Amount', 'Status', 'Created At', 'Actions'].map((h) => (
                  <th
                    key={h}
                    className="text-xs font-medium text-zinc-500 uppercase tracking-wider px-3 py-2 text-left"
                  >
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {events.length === 0 && !error ? (
                <tr>
                  <td colSpan={6} className="px-3 py-8 text-center text-zinc-500 text-sm">
                    No webhook events
                  </td>
                </tr>
              ) : (
                events.map((ev) => (
                  <>
                    <tr key={ev.id} className="hover:bg-zinc-800/40 border-b border-zinc-800/60">
                      <td className="px-3 py-2.5 text-sm font-mono text-zinc-300">{ev.event_type}</td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">{ev.org_name ?? ev.org_id ?? '—'}</td>
                      <td className="px-3 py-2.5 text-sm text-zinc-300">
                        {ev.amount_cents != null ? `$${(ev.amount_cents / 100).toFixed(2)}` : '—'}
                      </td>
                      <td className="px-3 py-2.5">
                        <StatusBadge status={ev.status} />
                      </td>
                      <td className="px-3 py-2.5 text-sm text-zinc-400">
                        {new Date(ev.created_at).toLocaleDateString('en-US', {
                          month: 'short',
                          day: 'numeric',
                          year: 'numeric',
                        })}
                      </td>
                      <td className="px-3 py-2.5">
                        {ev.status === 'failed' && (
                          <button
                            onClick={() => handleReplay(ev.id)}
                            disabled={replayingId === ev.id}
                            className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors disabled:opacity-50"
                          >
                            {replayingId === ev.id ? 'Replaying…' : 'Replay'}
                          </button>
                        )}
                      </td>
                    </tr>
                    <tr key={`${ev.id}-payload`} className="border-b border-zinc-800/60">
                      <td colSpan={6} className="px-3 pb-2">
                        <PayloadViewer payload={ev.payload} />
                      </td>
                    </tr>
                  </>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {toast && (
        <div className="fixed bottom-4 right-4 z-50 bg-zinc-800 border border-zinc-700 rounded-xl px-4 py-3 text-sm text-zinc-200 shadow-xl">
          {toast}
        </div>
      )}
    </div>
  )
}
