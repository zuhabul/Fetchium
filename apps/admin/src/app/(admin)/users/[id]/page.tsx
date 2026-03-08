'use client'

import { useEffect, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import TopBar from '@/components/layout/TopBar'
import Link from 'next/link'
import { ArrowLeft, User } from 'lucide-react'

interface UserDetail {
  id: string
  email: string
  org_id: string | null
  org_name: string | null
  role: string
  created_at: string
  last_active_at: string | null
  is_suspended: boolean
  email_verified: boolean
}

export default function UserProfilePage() {
  const { id } = useParams<{ id: string }>()
  const router = useRouter()
  const [user, setUser] = useState<UserDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [actionMsg, setActionMsg] = useState<string | null>(null)

  useEffect(() => {
    fetch(`/api/admin/users/${id}`)
      .then(r => r.ok ? r.json() : Promise.reject(`API ${r.status}`))
      .then(setUser)
      .catch(e => setError(String(e)))
      .finally(() => setLoading(false))
  }, [id])

  async function handleSuspend() {
    if (!user) return
    const action = user.is_suspended ? 'unsuspend' : 'suspend'
    try {
      const res = await fetch(`/api/admin/users/${id}/${action}`, { method: 'POST' })
      if (res.ok) {
        setUser(prev => prev ? { ...prev, is_suspended: !prev.is_suspended } : prev)
        setActionMsg(`User ${action}ed successfully`)
      } else {
        setActionMsg(`Failed to ${action} user`)
      }
    } catch {
      setActionMsg('Request failed')
    }
    setTimeout(() => setActionMsg(null), 3000)
  }

  if (loading) {
    return (
      <>
        <TopBar title="User" />
        <div className="p-6 flex items-center justify-center h-48">
          <div className="text-zinc-500 text-sm animate-pulse">Loading...</div>
        </div>
      </>
    )
  }

  if (error || !user) {
    return (
      <>
        <TopBar title="User" />
        <div className="p-6 space-y-4">
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl px-4 py-3 text-sm text-red-400">
            {error ?? 'User not found'}
          </div>
          <button
            onClick={() => router.push('/users')}
            className="flex items-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
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
      <div className="p-6 space-y-5">
        <Link href="/users" className="inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors">
          <ArrowLeft className="w-3 h-3" /> All Users
        </Link>

        {/* Header */}
        <div className="flex items-start gap-4">
          <div className="w-10 h-10 rounded-xl bg-zinc-800 border border-zinc-700 flex items-center justify-center flex-shrink-0">
            <User className="w-5 h-5 text-zinc-400" />
          </div>
          <div>
            <h2 className="text-xl font-bold text-zinc-100">{user.email}</h2>
            <div className="flex items-center gap-3 mt-1 text-sm text-zinc-500">
              <span className="capitalize">{user.role}</span>
              <span className={user.is_suspended ? 'text-red-400' : 'text-emerald-400'}>
                {user.is_suspended ? 'Suspended' : 'Active'}
              </span>
              {user.email_verified && (
                <span className="text-blue-400">Email verified</span>
              )}
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
          <div className="flex-[7] min-w-0">
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
              <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-4">User Details</h3>
              <dl className="grid grid-cols-2 gap-4">
                {[
                  ['Email', user.email],
                  ['Role', user.role ?? '—'],
                  ['Organization', user.org_name ?? user.org_id ?? '—'],
                  ['Email Verified', user.email_verified ? 'Yes' : 'No'],
                  ['Status', user.is_suspended ? 'Suspended' : 'Active'],
                  ['Created', user.created_at ? new Date(user.created_at).toLocaleDateString() : '—'],
                  ['Last Active', user.last_active_at ? new Date(user.last_active_at).toLocaleDateString() : '—'],
                ].map(([label, val]) => (
                  <div key={label}>
                    <dt className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">{label}</dt>
                    <dd className="text-sm text-zinc-200">
                      {label === 'Organization' && user.org_id ? (
                        <Link href={`/orgs/${user.org_id}`} className="text-blue-400 hover:underline">
                          {val}
                        </Link>
                      ) : val}
                    </dd>
                  </div>
                ))}
              </dl>
            </div>
          </div>

          {/* Sidebar (30%) */}
          <div className="flex-[3] min-w-0">
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-2">
              <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Actions</h3>
              <button
                onClick={handleSuspend}
                className={`w-full text-left text-sm px-3 py-1.5 rounded-md border transition-colors ${
                  user.is_suspended
                    ? 'bg-emerald-500/20 hover:bg-emerald-500/30 border-emerald-500/30 text-emerald-400'
                    : 'bg-red-500/20 hover:bg-red-500/30 border-red-500/30 text-red-400'
                }`}
              >
                {user.is_suspended ? 'Unsuspend User' : 'Suspend User'}
              </button>
              {user.org_id && (
                <Link
                  href={`/orgs/${user.org_id}`}
                  className="block w-full text-left bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md transition-colors"
                >
                  View Organization
                </Link>
              )}
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
