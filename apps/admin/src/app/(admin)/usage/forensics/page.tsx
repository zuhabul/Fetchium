'use client'

import { useState, FormEvent } from 'react'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { ArrowLeft, Copy, Check, Search } from 'lucide-react'

interface ForensicsResult {
  request_id: string
  timestamp: string
  org_id: string
  org_name: string
  key_prefix: string
  endpoint: string
  status_code: number
  duration_ms: number
}

function statusColor(code: number): string {
  if (code < 300) return 'bg-emerald-500/20 text-emerald-400'
  if (code < 500) return 'bg-amber-500/20 text-amber-400'
  return 'bg-red-500/20 text-red-400'
}

export default function ForensicsPage() {
  const [requestId, setRequestId] = useState('')
  const [result, setResult] = useState<ForensicsResult | null>(null)
  const [notFound, setNotFound] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    if (!requestId.trim()) return
    setLoading(true)
    setResult(null)
    setNotFound(false)
    setError(null)

    try {
      const res = await fetch(`/api/admin/usage/forensics/${encodeURIComponent(requestId.trim())}`)
      if (res.status === 404) {
        setNotFound(true)
      } else if (res.ok) {
        setResult(await res.json())
      } else {
        setError(`API error: ${res.status}`)
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Request failed')
    } finally {
      setLoading(false)
    }
  }

  function handleCopy() {
    if (!result) return
    navigator.clipboard.writeText(result.request_id).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <>
      <TopBar title="Request Forensics" subtitle="Look up individual request details by ID" />
      <div className="p-6 space-y-5">
        <Link href="/usage" className="inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors">
          <ArrowLeft className="w-3 h-3" /> Back to Usage
        </Link>

        {/* Search form */}
        <form onSubmit={handleSubmit} className="flex items-center gap-3">
          <input
            type="text"
            value={requestId}
            onChange={e => setRequestId(e.target.value)}
            placeholder="Enter request ID..."
            className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 w-96"
          />
          <button
            type="submit"
            disabled={loading || !requestId.trim()}
            className="flex items-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors disabled:opacity-50"
          >
            <Search className="w-3.5 h-3.5" />
            {loading ? 'Searching...' : 'Look up'}
          </button>
        </form>

        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {notFound && (
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-5 py-8 text-center">
            <p className="text-sm text-zinc-400">No request found with that ID.</p>
            <p className="text-xs text-zinc-600 mt-1">Check the ID and try again.</p>
          </div>
        )}

        {result && (
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-5 space-y-4">
            {/* Header row */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className="font-mono text-xs text-zinc-400">{result.request_id}</span>
                <button
                  onClick={handleCopy}
                  className="p-1 rounded hover:bg-zinc-700 text-zinc-500 hover:text-zinc-300 transition-colors"
                  title="Copy request ID"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                </button>
              </div>
              <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${statusColor(result.status_code)}`}>
                {result.status_code}
              </span>
            </div>

            <hr className="border-zinc-800" />

            {/* Detail grid */}
            <dl className="grid grid-cols-2 gap-4">
              {[
                ['Timestamp', result.timestamp ? new Date(result.timestamp).toLocaleString() : '—'],
                ['Endpoint', result.endpoint ?? '—'],
                ['Organization', result.org_name ?? result.org_id ?? '—'],
                ['Key Prefix', result.key_prefix ?? '—'],
                ['Status Code', String(result.status_code ?? '—')],
                ['Duration', result.duration_ms != null ? `${result.duration_ms}ms` : '—'],
              ].map(([label, val]) => (
                <div key={label}>
                  <dt className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">{label}</dt>
                  <dd className="text-sm text-zinc-200">
                    {label === 'Organization' && result.org_id ? (
                      <Link href={`/orgs/${result.org_id}`} className="text-blue-400 hover:underline">
                        {val}
                      </Link>
                    ) : (
                      <span className={label === 'Endpoint' ? 'font-mono' : ''}>{val}</span>
                    )}
                  </dd>
                </div>
              ))}
            </dl>
          </div>
        )}
      </div>
    </>
  )
}
