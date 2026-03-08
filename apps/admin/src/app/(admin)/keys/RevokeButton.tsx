'use client'

import { useState } from 'react'

export default function RevokeButton({ keyId }: { keyId: string }) {
  const [confirming, setConfirming] = useState(false)
  const [revoked, setRevoked] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleRevoke() {
    setLoading(true)
    setError(null)
    try {
      const res = await fetch(`/api/admin/keys/${keyId}`, { method: 'DELETE' })
      if (res.ok) {
        setRevoked(true)
        setConfirming(false)
      } else {
        setError(`Failed (${res.status})`)
        setConfirming(false)
      }
    } catch {
      setError('Network error')
      setConfirming(false)
    } finally {
      setLoading(false)
    }
  }

  if (error) {
    return <span className="text-xs text-red-400">{error}</span>
  }

  if (revoked) {
    return <span className="text-xs text-zinc-500">Revoked</span>
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
