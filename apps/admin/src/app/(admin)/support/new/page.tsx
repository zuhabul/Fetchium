'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import TopBar from '@/components/layout/TopBar'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { MessageSquare, AlertCircle, Clock, ArrowLeft } from 'lucide-react'

const PRIORITIES = ['low', 'normal', 'high', 'urgent'] as const

const PRIORITY_CONFIG: Record<string, { active: string; dot: string; label: string }> = {
  low:    { active: 'border-zinc-600 bg-zinc-800 text-zinc-300',           dot: 'bg-zinc-500',   label: 'Response within 5 days' },
  normal: { active: 'border-blue-500/50 bg-blue-500/10 text-blue-300',    dot: 'bg-blue-400',   label: 'Response within 24 hours' },
  high:   { active: 'border-amber-500/50 bg-amber-500/10 text-amber-300', dot: 'bg-amber-400',  label: 'Response within 4 hours' },
  urgent: { active: 'border-red-500/50 bg-red-500/10 text-red-300',       dot: 'bg-red-400',    label: 'Response within 1 hour' },
}

const PRIORITY_INACTIVE = 'border-zinc-800 bg-zinc-900 text-zinc-500 hover:border-zinc-700 hover:text-zinc-400'

export default function NewTicketPage() {
  const router = useRouter()
  const [subject, setSubject] = useState('')
  const [body, setBody] = useState('')
  const [priority, setPriority] = useState<string>('normal')
  const [orgId, setOrgId] = useState('')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!subject.trim()) { setError('Subject is required'); return }
    setSaving(true)
    setError('')
    try {
      const res = await fetch('/api/admin/support/tickets', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          subject: subject.trim(),
          body: body.trim() || undefined,
          priority,
          org_id: orgId.trim() || undefined,
        }),
      })
      const data = await res.json()
      if (!res.ok) { setError(data.error ?? 'Failed to create ticket'); return }
      router.push(`/support/${data.id}`)
      router.refresh()
    } catch {
      setError('Network error — please try again')
    } finally {
      setSaving(false)
    }
  }

  const cfg = PRIORITY_CONFIG[priority]

  return (
    <>
      <TopBar title="New Ticket" subtitle="Open a support ticket on behalf of an organization" />
      <div className={`${ADMIN_PAGE_PADDING}`}>
        <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:gap-6">

          {/* ── Main Form ───────────────────────────────────────── */}
          <div className="min-w-0 flex-1 space-y-4">

            {/* Back link */}
            <button
              onClick={() => router.back()}
              className="inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
            >
              <ArrowLeft className="h-3.5 w-3.5" />
              Back to tickets
            </button>

            {/* Subject */}
            <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-5 space-y-4">
              <div className="flex items-center gap-2 border-b border-zinc-800 pb-4">
                <MessageSquare className="h-4 w-4 text-blue-400" />
                <h2 className="text-sm font-semibold text-zinc-200">Ticket Details</h2>
              </div>

              <div>
                <label className="block text-xs font-medium uppercase tracking-wider text-zinc-500 mb-2">
                  Subject <span className="text-red-400 normal-case tracking-normal">*</span>
                </label>
                <input
                  type="text"
                  value={subject}
                  onChange={e => setSubject(e.target.value)}
                  placeholder="Brief description of the issue"
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-100 placeholder:text-zinc-600 focus:border-zinc-500 focus:outline-none transition-colors"
                  autoFocus
                />
              </div>

              <div>
                <label className="block text-xs font-medium uppercase tracking-wider text-zinc-500 mb-2">
                  Organization ID
                  <span className="ml-1.5 normal-case tracking-normal text-zinc-600 font-normal">optional</span>
                </label>
                <input
                  type="text"
                  value={orgId}
                  onChange={e => setOrgId(e.target.value)}
                  placeholder="Paste org UUID to link this ticket"
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-100 placeholder:text-zinc-600 font-mono focus:border-zinc-500 focus:outline-none transition-colors"
                />
              </div>

              <div>
                <label className="block text-xs font-medium uppercase tracking-wider text-zinc-500 mb-2">
                  Description
                  <span className="ml-1.5 normal-case tracking-normal text-zinc-600 font-normal">optional</span>
                </label>
                <textarea
                  value={body}
                  onChange={e => setBody(e.target.value)}
                  rows={7}
                  placeholder="Steps to reproduce, error messages, affected users, current behavior vs expected..."
                  className="w-full resize-y rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-100 placeholder:text-zinc-600 focus:border-zinc-500 focus:outline-none transition-colors min-h-[140px]"
                />
              </div>
            </div>

            {/* Priority */}
            <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-5 space-y-3">
              <h3 className="text-xs font-medium uppercase tracking-wider text-zinc-500">Priority</h3>
              <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                {PRIORITIES.map(p => (
                  <button
                    key={p}
                    type="button"
                    onClick={() => setPriority(p)}
                    className={`flex items-center gap-2 rounded-lg border px-3 py-2.5 text-xs font-medium capitalize transition-colors ${
                      priority === p ? PRIORITY_CONFIG[p].active : PRIORITY_INACTIVE
                    }`}
                  >
                    <span className={`h-1.5 w-1.5 rounded-full shrink-0 ${priority === p ? PRIORITY_CONFIG[p].dot : 'bg-zinc-700'}`} />
                    {p}
                  </button>
                ))}
              </div>
              <p className="text-xs text-zinc-600">{cfg.label}</p>
            </div>

            {error && (
              <div className="flex items-start gap-2 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3">
                <AlertCircle className="h-4 w-4 text-red-400 shrink-0 mt-0.5" />
                <p className="text-sm text-red-400">{error}</p>
              </div>
            )}

            <div className="flex gap-3">
              <button
                onClick={handleSubmit as unknown as React.MouseEventHandler}
                disabled={saving || !subject.trim()}
                className="flex-1 rounded-lg border border-blue-500/30 bg-blue-500/15 px-4 py-2.5 text-sm font-medium text-blue-300 transition-colors hover:bg-blue-500/25 disabled:opacity-40 disabled:cursor-not-allowed"
              >
                {saving ? 'Creating ticket…' : 'Create Ticket'}
              </button>
              <button
                type="button"
                onClick={() => router.back()}
                className="rounded-lg border border-zinc-700 bg-zinc-800 px-5 py-2.5 text-sm text-zinc-400 transition-colors hover:bg-zinc-700 hover:text-zinc-200"
              >
                Cancel
              </button>
            </div>
          </div>

          {/* ── Sidebar ─────────────────────────────────────────── */}
          <div className="w-full lg:w-72 xl:w-80 shrink-0 space-y-4">

            {/* Live preview */}
            <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4 space-y-3">
              <h3 className="text-xs font-medium uppercase tracking-wider text-zinc-500">Preview</h3>
              <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3 space-y-2">
                <p className="text-sm font-medium text-zinc-200 leading-snug line-clamp-2">
                  {subject.trim() || <span className="text-zinc-600 font-normal italic">No subject yet</span>}
                </p>
                <div className="flex items-center gap-2 flex-wrap">
                  <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium capitalize ${cfg.active}`}>
                    <span className={`h-1.5 w-1.5 rounded-full ${cfg.dot}`} />
                    {priority}
                  </span>
                  <span className="text-xs text-zinc-600">status: open</span>
                </div>
                {orgId.trim() && (
                  <p className="text-xs font-mono text-zinc-600 truncate">org: {orgId.trim()}</p>
                )}
                {body.trim() && (
                  <p className="text-xs text-zinc-500 line-clamp-3 leading-relaxed border-t border-zinc-800 pt-2">
                    {body.trim()}
                  </p>
                )}
              </div>
            </div>

            {/* SLA reference */}
            <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4 space-y-3">
              <h3 className="text-xs font-medium uppercase tracking-wider text-zinc-500">SLA Reference</h3>
              <ul className="space-y-2.5">
                {PRIORITIES.map(p => (
                  <li key={p} className="flex items-center justify-between gap-2">
                    <span className="flex items-center gap-1.5 text-xs capitalize text-zinc-400">
                      <span className={`h-1.5 w-1.5 rounded-full ${PRIORITY_CONFIG[p].dot}`} />
                      {p}
                    </span>
                    <span className="text-xs text-zinc-600">{PRIORITY_CONFIG[p].label.replace('Response ', '')}</span>
                  </li>
                ))}
              </ul>
            </div>

            {/* Tips */}
            <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4 space-y-3">
              <div className="flex items-center gap-1.5">
                <Clock className="h-3.5 w-3.5 text-zinc-500" />
                <h3 className="text-xs font-medium uppercase tracking-wider text-zinc-500">Tips</h3>
              </div>
              <ul className="space-y-2 text-xs text-zinc-500 leading-relaxed">
                <li>• Link an org ID to auto-associate billing and usage context.</li>
                <li>• Include error messages or request IDs in the description to speed up triage.</li>
                <li>• Use <span className="text-red-400">urgent</span> only for active outages affecting production.</li>
              </ul>
            </div>
          </div>

        </div>
      </div>
    </>
  )
}
