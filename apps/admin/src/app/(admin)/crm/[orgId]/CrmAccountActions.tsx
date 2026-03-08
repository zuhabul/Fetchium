'use client'

import { useState } from 'react'
import type { AdminSession } from '@/lib/session'

interface AccountStub {
  org_id: string
  lifecycle_stage: string
  csm?: string
}

const STAGES = ['prospect', 'trial', 'customer', 'expansion', 'churned']

export default function CrmAccountActions({
  account,
  session,
}: {
  account: AccountStub
  session: AdminSession
}) {
  const [stage, setStage] = useState(account.lifecycle_stage)
  const [csm, setCsm] = useState(account.csm ?? '')
  const [note, setNote] = useState('')
  const [loading, setLoading] = useState(false)
  const [toast, setToast] = useState<string | null>(null)

  function showToast(msg: string) {
    setToast(msg)
    setTimeout(() => setToast(null), 3000)
  }

  async function updateStage(newStage: string) {
    setStage(newStage)
    try {
      await fetch(`/internal/admin/crm/accounts/${account.org_id}/stage`, {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${session.sessionToken}`,
        },
        body: JSON.stringify({ stage: newStage }),
      })
      showToast('Stage updated')
    } catch {
      showToast('Failed to update stage')
    }
  }

  async function assignCsm() {
    if (!csm.trim()) return
    try {
      await fetch(`/internal/admin/crm/accounts/${account.org_id}/csm`, {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${session.sessionToken}`,
        },
        body: JSON.stringify({ csm }),
      })
      showToast('CSM assigned')
    } catch {
      showToast('Failed to assign CSM')
    }
  }

  async function addNote() {
    if (!note.trim()) return
    setLoading(true)
    try {
      await fetch(`/internal/admin/crm/accounts/${account.org_id}/notes`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${session.sessionToken}`,
        },
        body: JSON.stringify({ body: note }),
      })
      setNote('')
      showToast('Note added')
    } catch {
      showToast('Failed to add note')
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        {/* Stage Selector */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-2">
          <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider block">
            Lifecycle Stage
          </label>
          <select
            value={stage}
            onChange={(e) => updateStage(e.target.value)}
            className="w-full bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 focus:outline-none focus:border-zinc-500"
          >
            {STAGES.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
        </div>

        {/* Assign CSM */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-2">
          <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider block">
            Assign CSM
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={csm}
              onChange={(e) => setCsm(e.target.value)}
              placeholder="CSM name or email"
              className="flex-1 bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500"
            />
            <button
              onClick={assignCsm}
              className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
            >
              Assign
            </button>
          </div>
        </div>
      </div>

      {/* Add Note */}
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
        <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider block">
          Add Note
        </label>
        <textarea
          value={note}
          onChange={(e) => setNote(e.target.value)}
          placeholder="Write a note about this account..."
          rows={3}
          className="w-full bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 resize-none"
        />
        <div className="flex justify-end">
          <button
            onClick={addNote}
            disabled={loading || !note.trim()}
            className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? 'Saving…' : 'Add Note'}
          </button>
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
