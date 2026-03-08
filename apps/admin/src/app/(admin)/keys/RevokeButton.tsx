'use client'

import { useState } from 'react'

export default function RevokeButton({ keyId }: { keyId: string }) {
  const [confirming, setConfirming] = useState(false)
  const [revoked, setRevoked] = useState(false)
  const [loading, setLoading] = useState(false)

  async function handleRevoke() {
    setLoading(true)
    try {
      const res = await fetch(`/api/admin/keys/${keyId}`, { method: 'DELETE' })
      if (res.ok) {
        setRevoked(true)
        setConfirming(false)
      }
    } catch {
      // silently fail — parent page will show stale state
    } finally {
      setLoading(false)
    }
  }

  if (revoked) {
    return (
      <span className="text-xs text-zinc-500">Revoked</span>
    )
  }

  if (confirming) {
    return (
      <div className="flex items-center gap-1">
        <button
          onClick={handleRevoke}
          disabled={loading}
          className="bg-red-500/20 hover:bg-red-500/30 border border-red-500/30 text-red-400 text-xs px-2.5 py-1.5 rounded-md transition-colors disabled:opacity-50"
        >
          {loading ? '...' : 'Confirm'}
        </button>
        <button
          onClick={() => setConfirming(false)}
          className="bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-400 text-xs px-2.5 py-1.5 rounded-md transition-colors"
        >
          Cancel
        </button>
      </div>
    )
  }

  return (
    <button
      onClick={() => setConfirming(true)}
      className="bg-red-500/20 hover:bg-red-500/30 border border-red-500/30 text-red-400 text-xs px-2.5 py-1.5 rounded-md transition-colors"
    >
      Revoke
    </button>
  )
}
