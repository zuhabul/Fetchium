'use client'

import { useState } from 'react'
interface TicketStub {
  id: string
  status: string
  priority: string
  assignee?: string
}

const STATUSES = ['open', 'pending', 'resolved', 'closed']
const PRIORITIES = ['low', 'normal', 'high', 'urgent']

export default function TicketActions({
  ticketId,
  ticket,
}: {
  ticketId: string
  ticket: TicketStub
}) {
  const [noteBody, setNoteBody] = useState('')
  const [isInternal, setIsInternal] = useState(false)
  const [assignee, setAssignee] = useState(ticket.assignee ?? '')
  const [status, setStatus] = useState(ticket.status)
  const [priority, setPriority] = useState(ticket.priority)
  const [loading, setLoading] = useState(false)
  const [toast, setToast] = useState<string | null>(null)

  function showToast(msg: string) {
    setToast(msg)
    setTimeout(() => setToast(null), 3000)
  }

  async function submitNote() {
    if (!noteBody.trim()) return
    setLoading(true)
    try {
      await fetch(`/api/admin/support/tickets/${ticketId}/notes`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ body: noteBody, internal: isInternal }),
      })
      setNoteBody('')
      showToast('Note added')
    } catch {
      showToast('Failed to add note')
    } finally {
      setLoading(false)
    }
  }

  async function updateStatus(value: string) {
    try {
      const res = await fetch(`/api/admin/support/tickets/${ticketId}/status`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status: value }),
      })
      if (res.ok) showToast('Status updated')
      else showToast('Failed to update status')
    } catch {
      showToast('Failed to update status')
    }
  }

  async function updateAssignee(value: string) {
    try {
      const res = await fetch(`/api/admin/support/tickets/${ticketId}/assign`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ assignee_id: value }),
      })
      if (res.ok) showToast('Assignee updated')
      else showToast('Failed to assign ticket')
    } catch {
      showToast('Failed to assign ticket')
    }
  }

  return (
    <>
      {/* Add Note Form */}
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
        <h2 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Add Reply</h2>
        <textarea
          value={noteBody}
          onChange={(e) => setNoteBody(e.target.value)}
          placeholder="Write a reply or internal note..."
          rows={4}
          className="w-full bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 resize-none"
        />
        <div className="flex items-center justify-between">
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={isInternal}
              onChange={(e) => setIsInternal(e.target.checked)}
              className="w-3.5 h-3.5 rounded border-zinc-600 bg-zinc-800 accent-zinc-400"
            />
            <span className="text-xs text-zinc-400">Internal note</span>
          </label>
          <button
            onClick={submitNote}
            disabled={loading || !noteBody.trim()}
            className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? 'Submitting…' : 'Submit'}
          </button>
        </div>
      </div>

      {/* Sidebar Actions */}
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
        <h2 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Actions</h2>

        <div className="space-y-1">
          <label className="text-xs text-zinc-500 uppercase tracking-wider font-medium block">Assign To</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={assignee}
              onChange={(e) => setAssignee(e.target.value)}
              placeholder="Name or email"
              className="flex-1 bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500"
            />
            <button
              onClick={() => updateAssignee(assignee)}
              className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
            >
              Assign
            </button>
          </div>
        </div>

        <div className="space-y-1">
          <label className="text-xs text-zinc-500 uppercase tracking-wider font-medium block">Status</label>
          <select
            value={status}
            onChange={(e) => { setStatus(e.target.value); updateStatus(e.target.value) }}
            className="w-full bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 focus:outline-none focus:border-zinc-500"
          >
            {STATUSES.map((s) => <option key={s} value={s}>{s}</option>)}
          </select>
        </div>

        <div className="space-y-1">
          <label className="text-xs text-zinc-500 uppercase tracking-wider font-medium block">Priority</label>
          <select
            value={priority}
            onChange={(e) => { setPriority(e.target.value) }}
            className="w-full bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 focus:outline-none focus:border-zinc-500"
          >
            {PRIORITIES.map((p) => <option key={p} value={p}>{p}</option>)}
          </select>
        </div>
      </div>

      {toast && (
        <div className="fixed bottom-4 right-4 z-50 bg-zinc-800 border border-zinc-700 rounded-xl px-4 py-3 text-sm text-zinc-200 shadow-xl">
          {toast}
        </div>
      )}
    </>
  )
}
