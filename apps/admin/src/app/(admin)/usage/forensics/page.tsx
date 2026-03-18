'use client'

import { useState, FormEvent } from 'react'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { ArrowLeft, Copy, Check, Search, Fingerprint } from 'lucide-react'

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
  if (code < 300) return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'
  if (code < 500) return 'bg-amber-500/20 text-amber-400 border-amber-500/30'
  return 'bg-red-500/20 text-red-400 border-red-500/30'
}

function durationColor(ms: number): string {
  if (ms < 300) return 'text-emerald-400'
  if (ms < 1000) return 'text-amber-400'
  return 'text-red-400'
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
      if (res.status === 404) setNotFound(true)
      else if (res.ok) setResult(await res.json())
      else setError(`API error: ${res.status}`)
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
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>

        <Link
          href="/usage"
          className="inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
        >
          <ArrowLeft className="w-3 h-3" /> Back to Usage
        </Link>

        {/* Search form — full width, wraps on mobile */}
        <form onSubmit={handleSubmit} className="flex flex-col gap-2 sm:flex-row sm:items-center">
          <div className="relative flex-1 min-w-0">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-500 pointer-events-none" />
            <input
              type="text"
              value={requestId}
              onChange={e => setRequestId(e.target.value)}
              placeholder="Enter request ID (e.g. req_abc123…)"
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg pl-9 pr-3 py-2.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 transition-colors"
            />
          </div>
          <button
            type="submit"
            disabled={loading || !requestId.trim()}
            className="inline-flex items-center justify-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-4 py-2.5 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed shrink-0 min-h-[42px]"
          >
            <Search className="w-3.5 h-3.5" />
            {loading ? 'Searching…' : 'Look up'}
          </button>
        </form>

        {/* Error */}
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* Not found */}
        {notFound && (
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-5 py-10 text-center space-y-1">
            <Fingerprint className="w-8 h-8 text-zinc-700 mx-auto mb-3" />
            <p className="text-sm font-medium text-zinc-400">No request found with that ID</p>
            <p className="text-xs text-zinc-600">Double-check the ID format and try again</p>
          </div>
        )}

        {/* Result card */}
        {result && (
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">

            {/* Card header */}
            <div className="flex flex-col gap-3 border-b border-zinc-800 p-4 sm:flex-row sm:items-center sm:justify-between">
              <div className="flex items-center gap-2 min-w-0">
                <Fingerprint className="w-4 h-4 text-zinc-500 shrink-0" />
                <span className="font-mono text-xs text-zinc-300 truncate min-w-0">
                  {result.request_id}
                </span>
                <button
                  onClick={handleCopy}
                  className="p-1.5 rounded-md hover:bg-zinc-700 text-zinc-500 hover:text-zinc-300 transition-colors shrink-0"
                  title="Copy request ID"
                >
                  {copied
                    ? <Check className="w-3.5 h-3.5 text-emerald-400" />
                    : <Copy className="w-3.5 h-3.5" />}
                </button>
              </div>
              <div className="flex items-center gap-2 shrink-0">
                <span className={`inline-flex items-center px-2.5 py-1 rounded-lg border text-xs font-semibold ${statusColor(result.status_code)}`}>
                  {result.status_code}
                </span>
                {result.duration_ms != null && (
                  <span className={`text-xs font-medium tabular-nums ${durationColor(result.duration_ms)}`}>
                    {result.duration_ms}ms
                  </span>
                )}
              </div>
            </div>

            {/* Detail grid — 1 col on mobile, 2 on sm, 3 on lg */}
            <dl className="grid grid-cols-1 gap-0 divide-y divide-zinc-800/60 sm:grid-cols-2 sm:divide-y-0 lg:grid-cols-3">
              {[
                {
                  label: 'Timestamp',
                  value: result.timestamp ? new Date(result.timestamp).toLocaleString() : '—',
                },
                {
                  label: 'Endpoint',
                  value: result.endpoint ?? '—',
                  mono: true,
                },
                {
                  label: 'Organization',
                  value: result.org_name ?? result.org_id ?? '—',
                  link: result.org_id ? `/orgs/${result.org_id}` : undefined,
                },
                {
                  label: 'Key Prefix',
                  value: result.key_prefix ?? '—',
                  mono: true,
                },
                {
                  label: 'Status Code',
                  value: String(result.status_code ?? '—'),
                },
                {
                  label: 'Duration',
                  value: result.duration_ms != null ? `${result.duration_ms}ms` : '—',
                },
              ].map(({ label, value, mono, link }) => (
                <div key={label} className="px-4 py-3.5 space-y-1">
                  <dt className="text-[11px] font-medium text-zinc-500 uppercase tracking-wider">
                    {label}
                  </dt>
                  <dd className="text-sm text-zinc-200 break-all">
                    {link ? (
                      <Link href={link} className="text-blue-400 hover:text-blue-300 transition-colors">
                        {value}
                      </Link>
                    ) : (
                      <span className={mono ? 'font-mono text-xs text-zinc-300' : ''}>
                        {value}
                      </span>
                    )}
                  </dd>
                </div>
              ))}
            </dl>
          </div>
        )}

        {/* Empty state — no search yet */}
        {!result && !notFound && !error && !loading && (
          <div className="rounded-xl border border-zinc-800 bg-zinc-900 px-5 py-12 text-center space-y-2">
            <Search className="w-8 h-8 text-zinc-700 mx-auto mb-3" />
            <p className="text-sm font-medium text-zinc-400">Enter a request ID above to look it up</p>
            <p className="text-xs text-zinc-600">
              Request IDs are returned in the <span className="font-mono text-zinc-500">X-Request-Id</span> response header
            </p>
          </div>
        )}

      </div>
    </>
  )
}
