'use client'

import { useState } from 'react'

interface ModalProps {
  title: string
  onClose: () => void
  onSubmit: (amount: string, reason: string) => void
  loading: boolean
}

function ActionModal({ title, onClose, onSubmit, loading }: ModalProps) {
  const [amount, setAmount] = useState('')
  const [reason, setReason] = useState('')

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6 w-full max-w-sm space-y-4">
        <h3 className="text-sm font-semibold text-zinc-100">{title}</h3>
        <div className="space-y-3">
          <div>
            <label className="text-xs text-zinc-500 uppercase tracking-wider font-medium block mb-1">
              Amount ($)
            </label>
            <input
              type="number"
              min="0"
              step="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="w-full bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500"
            />
          </div>
          <div>
            <label className="text-xs text-zinc-500 uppercase tracking-wider font-medium block mb-1">
              Reason
            </label>
            <textarea
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Describe the reason..."
              rows={3}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 resize-none"
            />
          </div>
        </div>
        <div className="flex gap-2 justify-end">
          <button
            onClick={onClose}
            className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={() => onSubmit(amount, reason)}
            disabled={loading || !amount || !reason}
            className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm px-3 py-1.5 rounded-md transition-colors"
          >
            {loading ? 'Submitting…' : 'Submit'}
          </button>
        </div>
      </div>
    </div>
  )
}

export default function BillingActions({
  orgId,
}: {
  orgId: string
}) {
  const [modal, setModal] = useState<'refund' | 'credit' | null>(null)
  const [loading, setLoading] = useState(false)
  const [toast, setToast] = useState<string | null>(null)

  async function handleSubmit(type: 'refund' | 'credit', amount: string, reason: string) {
    setLoading(true)
    try {
      const endpoint =
        type === 'refund'
          ? `/api/admin/billing/${orgId}/refund`
          : `/api/admin/billing/${orgId}/credit`
      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ amount_cents: Math.round(parseFloat(amount) * 100), reason }),
      })
      if (res.ok) {
        if (type === 'refund') {
          const data = await res.json().catch(() => ({}))
          const refundId = data?.refund_id ? ` (ID: ${String(data.refund_id).slice(0, 8)}…)` : ''
          setToast(`Refund queued for manual review${refundId}`)
        } else {
          setToast('Credit applied successfully')
        }
        setModal(null)
      } else {
        const err = await res.json().catch(() => ({}))
        setToast(err?.error ?? 'Failed to submit — please try again')
      }
    } catch {
      setToast('Failed to submit — please try again')
    } finally {
      setLoading(false)
      setTimeout(() => setToast(null), 5000)
    }
  }

  return (
    <>
      <div className="flex gap-2">
        <button
          onClick={() => setModal('refund')}
          className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
        >
          Issue Refund
        </button>
        <button
          onClick={() => setModal('credit')}
          className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
        >
          Apply Credit
        </button>
      </div>

      {toast && (
        <div className="fixed bottom-4 right-4 z-50 bg-zinc-800 border border-zinc-700 rounded-xl px-4 py-3 text-sm text-zinc-200 shadow-xl">
          {toast}
        </div>
      )}

      {modal === 'refund' && (
        <ActionModal
          title="Issue Refund"
          onClose={() => setModal(null)}
          onSubmit={(a, r) => handleSubmit('refund', a, r)}
          loading={loading}
        />
      )}
      {modal === 'credit' && (
        <ActionModal
          title="Apply Credit"
          onClose={() => setModal(null)}
          onSubmit={(a, r) => handleSubmit('credit', a, r)}
          loading={loading}
        />
      )}
    </>
  )
}
