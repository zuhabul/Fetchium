'use client'

import { useState } from 'react'
import { Bot } from 'lucide-react'

export default function PostmortemClient({ incidentId }: { incidentId: string }) {
  const [loading, setLoading] = useState(false)
  const [text, setText] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  async function generate() {
    setLoading(true)
    setError(null)
    try {
      const res = await fetch(`/api/admin/incidents/${incidentId}/postmortem`, { method: 'POST' })
      if (res.ok) {
        const data = await res.json()
        setText(data.postmortem ?? data.text ?? JSON.stringify(data))
      } else {
        setError(`Failed to generate (${res.status})`)
      }
    } catch {
      setError('Network error')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-3">
      <button
        onClick={generate}
        disabled={loading}
        className="flex items-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors disabled:opacity-50"
      >
        <Bot className="w-3.5 h-3.5" />
        {loading ? 'Generating…' : 'Generate Postmortem'}
      </button>
      {error && <p className="text-xs text-red-400">{error}</p>}
      {text && (
        <div className="bg-zinc-800/50 border border-zinc-700/50 rounded-lg p-4 text-xs text-zinc-300 whitespace-pre-wrap font-mono leading-relaxed">
          {text}
        </div>
      )}
    </div>
  )
}
