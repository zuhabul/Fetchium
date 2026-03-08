'use client'

import { useEffect, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { ArrowLeft, Building2 } from 'lucide-react'

interface OrgDetail {
  id: string
  name: string
  slug: string
  plan: string
  status: string
  mrr_cents: number
  member_count: number
  key_count: number
  owner_email: string
  health_score: number
  created_at: string
}

const PLAN_BADGE: Record<string, string> = {
  free: 'bg-zinc-500/20 text-zinc-400',
  starter: 'bg-blue-500/20 text-blue-400',
  pro: 'bg-purple-500/20 text-purple-400',
  enterprise: 'bg-amber-500/20 text-amber-400',
}

const STATUS_BADGE: Record<string, string> = {
  active: 'bg-emerald-500/20 text-emerald-400',
  suspended: 'bg-red-500/20 text-red-400',
  trial: 'bg-amber-500/20 text-amber-400',
  churned: 'bg-zinc-500/20 text-zinc-400',
}

function Badge({ value, map }: { value: string; map: Record<string, string> }) {
  const cls = map[value] ?? 'bg-zinc-500/20 text-zinc-400'
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium capitalize ${cls}`}>
      {value}
    </span>
  )
}

function healthColor(score: number): string {
  if (score >= 80) return 'text-emerald-400'
  if (score >= 50) return 'text-amber-400'
  return 'text-red-400'
}

interface Member { id: string; email: string; name: string | null; status: string; created_at: string }
interface AuditRow { id: string; action: string; role: string | null; actor_email: string | null; ip: string | null; created_at: string }
interface ApiKey { id: string; name: string; key_prefix: string; active: boolean; created_at: string }

type Tab = 'overview' | 'members' | 'keys' | 'audit'

export default function OrgProfilePage() {
  const { id } = useParams<{ id: string }>()
  const router = useRouter()
  const [org, setOrg] = useState<OrgDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [tab, setTab] = useState<Tab>('overview')
  const [actionMsg, setActionMsg] = useState<string | null>(null)
  const [members, setMembers] = useState<Member[]>([])
  const [auditRows, setAuditRows] = useState<AuditRow[]>([])
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([])
  const [tabLoading, setTabLoading] = useState(false)

  useEffect(() => {
    fetch(`/api/admin/orgs/${id}`)
      .then(r => r.ok ? r.json() : Promise.reject(`API ${r.status}`))
      .then(setOrg)
      .catch(e => setError(String(e)))
      .finally(() => setLoading(false))
  }, [id])

  async function loadTab(t: Tab) {
    setTab(t)
    if (t === 'members' && members.length === 0) {
      setTabLoading(true)
      fetch(`/api/admin/orgs/${id}/members`).then(r => r.ok ? r.json() : null).then(d => { if (d) setMembers(d.data ?? []) }).finally(() => setTabLoading(false))
    }
    if (t === 'audit' && auditRows.length === 0) {
      setTabLoading(true)
      fetch(`/api/admin/orgs/${id}/audit`).then(r => r.ok ? r.json() : null).then(d => { if (d) setAuditRows(d.data ?? []) }).finally(() => setTabLoading(false))
    }
    if (t === 'keys' && apiKeys.length === 0) {
      setTabLoading(true)
      fetch(`/api/admin/orgs/${id}/keys`).then(r => r.ok ? r.json() : null).then(d => { if (d) setApiKeys(d.data ?? []) }).finally(() => setTabLoading(false))
    }
  }

  async function handleSuspend() {
    if (!org) return
    const isSuspended = org.status === 'suspended'
    const action = isSuspended ? 'reactivate' : 'suspend'
    try {
      const res = await fetch(`/api/admin/orgs/${id}/${action}`, { method: 'POST' })
      if (res.ok) {
        setOrg(prev => prev ? { ...prev, status: isSuspended ? 'active' : 'suspended' } : prev)
        setActionMsg(`Org ${action}d successfully`)
      } else {
        setActionMsg(`Failed to ${action}`)
      }
    } catch {
      setActionMsg('Request failed')
    }
    setTimeout(() => setActionMsg(null), 3000)
  }

  const TABS: { key: Tab; label: string }[] = [
    { key: 'overview', label: 'Overview' },
    { key: 'members', label: 'Members' },
    { key: 'keys', label: 'API Keys' },
    { key: 'audit', label: 'Audit' },
  ]

  if (loading) {
    return (
      <>
        <TopBar title="Organization" />
        <div className="p-6 flex items-center justify-center h-48">
          <div className="text-zinc-500 text-sm animate-pulse">Loading...</div>
        </div>
      </>
    )
  }

  if (error || !org) {
    return (
      <>
        <TopBar title="Organization" />
        <div className="p-6 space-y-4">
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error ?? 'Org not found'}
          </div>
          <button
            onClick={() => router.push('/orgs')}
            className="flex items-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
          >
            <ArrowLeft className="w-3.5 h-3.5" /> Back to Orgs
          </button>
        </div>
      </>
    )
  }

  return (
    <>
      <TopBar title={org.name} subtitle={`/${org.slug}`} />
      <div className="p-6 space-y-5">
        {/* Back */}
        <Link href="/orgs" className="inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors">
          <ArrowLeft className="w-3 h-3" /> All Orgs
        </Link>

        {/* Header */}
        <div className="flex items-start gap-4">
          <div className="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center flex-shrink-0">
            <Building2 className="w-5 h-5 text-zinc-400" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className="text-xl font-bold text-zinc-100">{org.name}</h2>
              <Badge value={org.plan} map={PLAN_BADGE} />
              <Badge value={org.status} map={STATUS_BADGE} />
            </div>
            <div className="flex items-center gap-4 mt-1 text-sm text-zinc-500">
              <span>MRR: <span className="text-zinc-300">{org.mrr_cents ? `$${(org.mrr_cents / 100).toFixed(2)}` : '—'}</span></span>
              <span>Health: <span className={`font-semibold ${healthColor(org.health_score ?? 0)}`}>{org.health_score ?? '—'}</span></span>
            </div>
          </div>
        </div>

        {actionMsg && (
          <div className="bg-blue-500/10 border border-blue-500/30 rounded-xl px-4 py-2 text-sm text-blue-400">
            {actionMsg}
          </div>
        )}

        {/* Two-column layout */}
        <div className="flex gap-5 items-start">
          {/* Main content (70%) */}
          <div className="flex-[7] min-w-0 space-y-4">
            {/* Tabs */}
            <div className="flex items-center gap-1 border-b border-zinc-800 pb-0">
              {TABS.map(t => (
                <button
                  key={t.key}
                  onClick={() => loadTab(t.key)}
                  className={`px-3 py-2 text-sm font-medium transition-colors border-b-2 -mb-px ${
                    tab === t.key
                      ? 'border-zinc-300 text-zinc-100'
                      : 'border-transparent text-zinc-500 hover:text-zinc-300'
                  }`}
                >
                  {t.label}
                </button>
              ))}
            </div>

            {/* Tab content */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
              {tab === 'overview' && (
                <dl className="grid grid-cols-2 gap-4">
                  {[
                    ['Created', org.created_at ? new Date(org.created_at).toLocaleDateString() : '—'],
                    ['Owner Email', org.owner_email ?? '—'],
                    ['Slug', org.slug ?? '—'],
                    ['Plan', org.plan ?? '—'],
                    ['Status', org.status ?? '—'],
                    ['Members', String(org.member_count ?? '—')],
                    ['API Keys', String(org.key_count ?? '—')],
                    ['MRR', org.mrr_cents ? `$${(org.mrr_cents / 100).toFixed(2)}` : '—'],
                  ].map(([label, val]) => (
                    <div key={label}>
                      <dt className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">{label}</dt>
                      <dd className="text-sm text-zinc-200">{val}</dd>
                    </div>
                  ))}
                </dl>
              )}
              {tab === 'members' && (
                tabLoading ? <div className="text-sm text-zinc-500 animate-pulse">Loading...</div> :
                members.length === 0 ? <div className="text-sm text-zinc-500">No members found.</div> :
                <table className="w-full text-sm">
                  <thead><tr className="text-left text-xs text-zinc-500 uppercase tracking-wider border-b border-zinc-800">
                    <th className="pb-2 pr-4">Email</th><th className="pb-2 pr-4">Name</th><th className="pb-2 pr-4">Status</th><th className="pb-2">Joined</th>
                  </tr></thead>
                  <tbody className="divide-y divide-zinc-800">
                    {members.map(m => (
                      <tr key={m.id}>
                        <td className="py-2 pr-4 text-zinc-300">{m.email}</td>
                        <td className="py-2 pr-4 text-zinc-400">{m.name ?? '—'}</td>
                        <td className="py-2 pr-4"><span className={`text-xs px-1.5 py-0.5 rounded ${m.status === 'active' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-zinc-500/20 text-zinc-400'}`}>{m.status}</span></td>
                        <td className="py-2 text-zinc-500">{new Date(m.created_at).toLocaleDateString()}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
              {tab === 'keys' && (
                tabLoading ? <div className="text-sm text-zinc-500 animate-pulse">Loading...</div> :
                apiKeys.length === 0 ? <div className="text-sm text-zinc-500">No API keys found.</div> :
                <table className="w-full text-sm">
                  <thead><tr className="text-left text-xs text-zinc-500 uppercase tracking-wider border-b border-zinc-800">
                    <th className="pb-2 pr-4">Name</th><th className="pb-2 pr-4">Prefix</th><th className="pb-2 pr-4">Status</th><th className="pb-2">Created</th>
                  </tr></thead>
                  <tbody className="divide-y divide-zinc-800">
                    {apiKeys.map(k => (
                      <tr key={k.id}>
                        <td className="py-2 pr-4 text-zinc-300">{k.name}</td>
                        <td className="py-2 pr-4 font-mono text-zinc-400">{k.key_prefix}…</td>
                        <td className="py-2 pr-4"><span className={`text-xs px-1.5 py-0.5 rounded ${k.active ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'}`}>{k.active ? 'active' : 'revoked'}</span></td>
                        <td className="py-2 text-zinc-500">{new Date(k.created_at).toLocaleDateString()}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
              {tab === 'audit' && (
                tabLoading ? <div className="text-sm text-zinc-500 animate-pulse">Loading...</div> :
                auditRows.length === 0 ? <div className="text-sm text-zinc-500">No audit events found.</div> :
                <table className="w-full text-sm">
                  <thead><tr className="text-left text-xs text-zinc-500 uppercase tracking-wider border-b border-zinc-800">
                    <th className="pb-2 pr-4">Action</th><th className="pb-2 pr-4">Actor</th><th className="pb-2 pr-4">Role</th><th className="pb-2 pr-4">IP</th><th className="pb-2">Time</th>
                  </tr></thead>
                  <tbody className="divide-y divide-zinc-800">
                    {auditRows.map(a => (
                      <tr key={a.id}>
                        <td className="py-2 pr-4 font-mono text-xs text-zinc-300">{a.action}</td>
                        <td className="py-2 pr-4 text-zinc-400">{a.actor_email ?? '—'}</td>
                        <td className="py-2 pr-4 text-zinc-500">{a.role ?? '—'}</td>
                        <td className="py-2 pr-4 font-mono text-xs text-zinc-500">{a.ip ?? '—'}</td>
                        <td className="py-2 text-zinc-500">{new Date(a.created_at).toLocaleString()}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </div>

          {/* Sidebar (30%) */}
          <div className="flex-[3] min-w-0">
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-2">
              <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Actions</h3>
              <button
                onClick={handleSuspend}
                className={`w-full text-left text-sm px-3 py-1.5 rounded-md border transition-colors ${
                  org.status === 'suspended'
                    ? 'bg-emerald-500/20 hover:bg-emerald-500/30 border-emerald-500/30 text-emerald-400'
                    : 'bg-red-500/20 hover:bg-red-500/30 border-red-500/30 text-red-400'
                }`}
              >
                {org.status === 'suspended' ? 'Reactivate Org' : 'Suspend Org'}
              </button>
              <Link
                href={`/keys?org=${org.id}`}
                className="block w-full text-left bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
              >
                View API Keys
              </Link>
              <button
                onClick={() => setTab('audit')}
                className="w-full text-left bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
              >
                View Audit Log
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
