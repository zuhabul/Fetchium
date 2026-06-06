'use client'

import { useEffect, useState } from 'react'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { useParams, useRouter } from 'next/navigation'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import {
  ArrowLeft,
  User,
  ShieldOff,
  ShieldCheck,
  LogOut,
  Building2,
  CheckCircle2,
  XCircle,
  Clock,
  Mail,
  BadgeCheck,
} from 'lucide-react'

interface UserDetail {
  id: string
  email: string
  name: string | null
  org_id: string | null
  org_name: string | null
  role: string
  created_at: string
  last_login_at: string | null
  is_active: boolean
  totp_enabled: boolean
}

function fmt(date: string | null) {
  if (!date) return '—'
  return new Date(date).toLocaleDateString('en-US', {
    month: 'short', day: 'numeric', year: 'numeric',
  })
}

function DetailRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="py-3 border-b border-zinc-800/60 last:border-0 grid grid-cols-[120px_1fr] gap-3 items-start sm:grid-cols-[140px_1fr]">
      <dt className="text-xs font-medium text-zinc-500 uppercase tracking-wider pt-0.5 shrink-0">{label}</dt>
      <dd className="text-sm text-zinc-200 min-w-0 break-words">{children}</dd>
    </div>
  )
}

export default function UserProfilePage() {
  const { id } = useParams<{ id: string }>()
  const router = useRouter()
  const [user, setUser] = useState<UserDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [actionMsg, setActionMsg] = useState<{ text: string; ok: boolean } | null>(null)
  const [suspending, setSuspending] = useState(false)
  const [loggingOut, setLoggingOut] = useState(false)

  useEffect(() => {
    fetch(`/api/admin/users/${id}`)
      .then(r => r.ok ? r.json() : Promise.reject(`API ${r.status}`))
      .then(json => setUser(json.data ?? json))
      .catch(e => setError(String(e)))
      .finally(() => setLoading(false))
  }, [id])

  function flash(text: string, ok = true) {
    setActionMsg({ text, ok })
    setTimeout(() => setActionMsg(null), 3000)
  }

  async function handleSuspend() {
    if (!user || suspending) return
    const action = user.is_active ? 'suspend' : 'unsuspend'
    setSuspending(true)
    try {
      const res = await fetch(`/api/admin/users/${id}/${action}`, { method: 'POST' })
      if (res.ok) {
        setUser(prev => prev ? { ...prev, is_active: !prev.is_active } : prev)
        flash(`User ${action}ed successfully`)
      } else {
        flash(`Failed to ${action} user`, false)
      }
    } catch {
      flash('Request failed', false)
    } finally {
      setSuspending(false)
    }
  }

  async function handleForceLogout() {
    if (!user || loggingOut) return
    setLoggingOut(true)
    try {
      const res = await fetch(`/api/admin/users/${id}/force-logout`, { method: 'POST' })
      flash(res.ok ? 'All sessions terminated' : 'Failed to force logout', res.ok)
    } catch {
      flash('Request failed', false)
    } finally {
      setLoggingOut(false)
    }
  }

  if (loading) {
    return (
      <>
        <TopBar title="User" />
        <div className={`${ADMIN_PAGE_PADDING} flex h-48 items-center justify-center`}>
          <div className="text-zinc-500 text-sm animate-pulse">Loading…</div>
        </div>
      </>
    )
  }

  if (error || !user) {
    return (
      <>
        <TopBar title="User" />
        <div className={`${ADMIN_PAGE_PADDING} space-y-4`}>
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error ?? 'User not found'}
          </div>
          <button
            onClick={() => router.push('/users')}
            className="inline-flex items-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-lg transition-colors"
          >
            <ArrowLeft className="w-3.5 h-3.5" /> Back to Users
          </button>
        </div>
      </>
    )
  }

  return (
    <>
      <TopBar title={user.email} subtitle={`User · ${user.role}`} />
      <div className={`${ADMIN_PAGE_PADDING} space-y-5`}>

        <Link
          href="/users"
          className="inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
        >
          <ArrowLeft className="w-3 h-3" /> All Users
        </Link>

        {/* Profile header card */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 sm:p-5">
          <div className="flex items-start gap-3 sm:gap-4 min-w-0">
            <div className="w-10 h-10 sm:w-12 sm:h-12 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0">
              <User className="w-5 h-5 sm:w-6 sm:h-6 text-zinc-400" />
            </div>
            <div className="min-w-0 flex-1">
              <h2 className="text-base sm:text-lg font-bold text-zinc-100 break-all leading-snug">
                {user.email}
              </h2>
              <div className="flex flex-wrap items-center gap-x-3 gap-y-1 mt-1.5">
                <span className="text-xs text-zinc-500 capitalize">{user.role}</span>
                <span className={`inline-flex items-center gap-1 text-xs font-medium ${!user.is_active ? 'text-red-400' : 'text-emerald-400'}`}>
                  {!user.is_active
                    ? <><XCircle className="w-3 h-3" /> Suspended</>
                    : <><CheckCircle2 className="w-3 h-3" /> Active</>}
                </span>
                {user.totp_enabled && (
                  <span className="inline-flex items-center gap-1 text-xs text-blue-400">
                    <BadgeCheck className="w-3 h-3" /> 2FA On
                  </span>
                )}
              </div>
            </div>
          </div>
        </div>

        {actionMsg && (
          <div className={`rounded-xl border px-4 py-2.5 text-sm ${
            actionMsg.ok
              ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
              : 'bg-red-500/10 border-red-500/30 text-red-400'
          }`}>
            {actionMsg.text}
          </div>
        )}

        {/* Main content + sidebar */}
        <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:gap-6">

          {/* Details — full width on mobile, 65% on desktop */}
          <div className="min-w-0 flex-1 space-y-4">
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 sm:p-5">
              <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">
                User Details
              </h3>
              <dl className="mt-1">
                <DetailRow label="Email">
                  <span className="flex items-center gap-1.5 break-all">
                    <Mail className="w-3.5 h-3.5 text-zinc-500 shrink-0" />
                    {user.email}
                  </span>
                </DetailRow>
                <DetailRow label="Role">
                  <span className="capitalize">{user.role}</span>
                </DetailRow>
                <DetailRow label="Organization">
                  {user.org_id ? (
                    <Link
                      href={`/orgs/${user.org_id}`}
                      className="inline-flex items-center gap-1.5 text-blue-400 hover:text-blue-300 transition-colors"
                    >
                      <Building2 className="w-3.5 h-3.5 shrink-0" />
                      {user.org_name ?? user.org_id}
                    </Link>
                  ) : (
                    <span className="text-zinc-500">No organization</span>
                  )}
                </DetailRow>
                <DetailRow label="2FA">
                  {user.totp_enabled
                    ? <span className="text-emerald-400 flex items-center gap-1"><CheckCircle2 className="w-3.5 h-3.5" /> Enabled</span>
                    : <span className="text-zinc-500 flex items-center gap-1"><XCircle className="w-3.5 h-3.5" /> Disabled</span>}
                </DetailRow>
                <DetailRow label="Status">
                  <span className={!user.is_active ? 'text-red-400' : 'text-emerald-400'}>
                    {!user.is_active ? 'Suspended' : 'Active'}
                  </span>
                </DetailRow>
                <DetailRow label="Created">
                  <span className="flex items-center gap-1.5 text-zinc-300">
                    <Clock className="w-3.5 h-3.5 text-zinc-500 shrink-0" />
                    {fmt(user.created_at)}
                  </span>
                </DetailRow>
                <DetailRow label="Last Login">
                  {user.last_login_at
                    ? <span className="flex items-center gap-1.5 text-zinc-300">
                        <Clock className="w-3.5 h-3.5 text-zinc-500 shrink-0" />
                        {fmt(user.last_login_at)}
                      </span>
                    : <span className="text-zinc-500">Never</span>}
                </DetailRow>
              </dl>
            </div>
          </div>

          {/* Sidebar — full width on mobile, fixed 280px on desktop */}
          <div className="w-full lg:w-72 xl:w-80 shrink-0 space-y-4">

            {/* Actions */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-2.5">
              <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">Actions</h3>

              <button
                onClick={handleSuspend}
                disabled={suspending}
                className={`w-full inline-flex items-center justify-center gap-2 text-sm px-4 py-2.5 rounded-lg border font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
                  !user.is_active
                    ? 'bg-emerald-500/15 hover:bg-emerald-500/25 border-emerald-500/30 text-emerald-400'
                    : 'bg-red-500/15 hover:bg-red-500/25 border-red-500/30 text-red-400'
                }`}
              >
                {!user.is_active
                  ? <><ShieldCheck className="w-4 h-4" />{suspending ? 'Unsuspending…' : 'Unsuspend User'}</>
                  : <><ShieldOff className="w-4 h-4" />{suspending ? 'Suspending…' : 'Suspend User'}</>}
              </button>

              <button
                onClick={handleForceLogout}
                disabled={loggingOut}
                className="w-full inline-flex items-center justify-center gap-2 text-sm px-4 py-2.5 rounded-lg border font-medium bg-amber-500/10 hover:bg-amber-500/20 border-amber-500/30 text-amber-400 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <LogOut className="w-4 h-4" />
                {loggingOut ? 'Terminating…' : 'Force Logout'}
              </button>

              {user.org_id && (
                <Link
                  href={`/orgs/${user.org_id}`}
                  className="w-full inline-flex items-center justify-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-4 py-2.5 rounded-lg font-medium transition-colors"
                >
                  <Building2 className="w-4 h-4" />
                  View Organization
                </Link>
              )}
            </div>

            {/* Quick info */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-2.5">
              <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">Quick Info</h3>
              <div className="space-y-2 text-xs">
                <div className="flex justify-between gap-2">
                  <span className="text-zinc-500">User ID</span>
                  <span className="font-mono text-zinc-400 truncate max-w-[140px]" title={user.id}>
                    {user.id.slice(0, 8)}…
                  </span>
                </div>
                {user.org_id && (
                  <div className="flex justify-between gap-2">
                    <span className="text-zinc-500">Org ID</span>
                    <span className="font-mono text-zinc-400 truncate max-w-[140px]" title={user.org_id}>
                      {user.org_id.slice(0, 8)}…
                    </span>
                  </div>
                )}
                <div className="flex justify-between gap-2">
                  <span className="text-zinc-500">Joined</span>
                  <span className="text-zinc-400">{fmt(user.created_at)}</span>
                </div>
              </div>
            </div>

          </div>
        </div>

      </div>
    </>
  )
}
