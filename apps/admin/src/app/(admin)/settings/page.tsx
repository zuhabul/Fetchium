import { redirect } from 'next/navigation'
import Link from 'next/link'
import { CheckCircle2, Shield, Smartphone, Users } from 'lucide-react'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import { ROLE_COLORS, ROLE_LABELS } from '@/types/admin'
import SettingsClient from './SettingsClient'

interface SessionRow {
  id: string
  ip: string
  user_agent: string
  last_active: string
  created_at: string
}

function normalize_sessions(payload: unknown): SessionRow[] {
  if (Array.isArray(payload)) return payload as SessionRow[]

  if (typeof payload === 'object' && payload !== null) {
    const body = payload as Record<string, unknown>
    if (Array.isArray(body.sessions)) return body.sessions as SessionRow[]
    if (Array.isArray(body.data)) return body.data as SessionRow[]
  }

  return []
}

function format_date(value?: string) {
  if (!value) return 'Unknown'
  return new Date(value).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
}

function StatCard({
  label,
  value,
  icon: Icon,
  color,
}: {
  label: string
  value: string
  icon: React.ElementType
  color: string
}) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
      <div className="mb-3 flex items-center justify-between gap-3">
        <span className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">{label}</span>
        <div className={`flex h-8 w-8 items-center justify-center rounded-lg ${color}`}>
          <Icon className="h-4 w-4" />
        </div>
      </div>
      <p className="text-[1.75rem] font-bold leading-none text-zinc-100 lg:text-2xl">{value}</p>
    </div>
  )
}

export default async function SettingsPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let sessions: SessionRow[] = []
  try {
    const res = await adminFetch('/internal/admin/sessions')
    if (res.ok) sessions = normalize_sessions(await res.json())
  } catch {
    // Non-fatal — show empty state
  }

  const firstName = session.name?.charAt(0).toUpperCase() || '?'
  const activeSessions = sessions.length.toLocaleString()

  return (
    <>
      <TopBar title="Settings" subtitle="Account preferences and security" />
      <div className={`${ADMIN_PAGE_PADDING} max-w-full space-y-6`}>
        <div className="grid grid-cols-1 gap-3 min-[420px]:grid-cols-2 xl:grid-cols-4">
          <StatCard
            label="Role"
            value={ROLE_LABELS[session.role]}
            icon={Shield}
            color="bg-blue-500/20 text-blue-400"
          />
          <StatCard
            label="Sessions"
            value={activeSessions}
            icon={Users}
            color="bg-emerald-500/20 text-emerald-400"
          />
          <StatCard
            label="Two-Factor"
            value="Available"
            icon={Smartphone}
            color="bg-amber-500/20 text-amber-400"
          />
          <StatCard
            label="Status"
            value="Secure"
            icon={CheckCircle2}
            color="bg-purple-500/20 text-purple-400"
          />
        </div>

        <div className="grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1.9fr)_minmax(22rem,1fr)] 2xl:grid-cols-[minmax(0,2.2fr)_minmax(24rem,1fr)]">
          <div className="space-y-6">
            <section className="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
              <div className="mb-4 flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
                <h2 className="text-sm font-semibold text-zinc-300">Active Sessions</h2>
                <p className="text-xs text-zinc-500">
                  {activeSessions} active session{sessions.length === 1 ? '' : 's'}
                </p>
              </div>
              {sessions.length === 0 ? (
                <p className="text-xs text-zinc-600">No session data available.</p>
              ) : (
                <>
                  <div className="divide-y divide-zinc-800/60 lg:hidden">
                    {sessions.map(s => (
                      <div key={s.id} className="space-y-4 py-4">
                        <div className="space-y-1">
                          <p className="font-mono text-sm text-zinc-200">{s.ip || 'unknown'}</p>
                          <p className="break-all text-xs text-zinc-500">
                            {s.user_agent || 'unknown device'}
                          </p>
                        </div>

                        <dl className="grid grid-cols-2 gap-3 text-sm">
                          <div>
                            <dt className="text-[11px] uppercase tracking-wider text-zinc-600">
                              Last Active
                            </dt>
                            <dd className="mt-1 text-zinc-400">{format_date(s.last_active)}</dd>
                          </div>
                          <div>
                            <dt className="text-[11px] uppercase tracking-wider text-zinc-600">
                              Created
                            </dt>
                            <dd className="mt-1 text-zinc-400">{format_date(s.created_at)}</dd>
                          </div>
                        </dl>

                        <form action={`/api/admin/sessions/${s.id}/revoke`} method="POST">
                          <button className="inline-flex min-h-11 w-full items-center justify-center rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-400 transition-colors hover:bg-red-500/20">
                            Revoke Session
                          </button>
                        </form>
                      </div>
                    ))}
                  </div>

                  <div className="hidden lg:block">
                    <div className="rounded-xl border border-zinc-800/80 bg-zinc-950/30">
                      <table className="w-full text-sm">
                        <thead>
                          <tr className="border-b border-zinc-800 text-left text-zinc-600">
                            <th className="px-4 py-3 text-xs font-medium uppercase tracking-wider">IP</th>
                            <th className="px-4 py-3 text-xs font-medium uppercase tracking-wider">User Agent</th>
                            <th className="px-4 py-3 text-xs font-medium uppercase tracking-wider">Created</th>
                            <th className="px-4 py-3 text-xs font-medium uppercase tracking-wider">Last Active</th>
                            <th className="px-4 py-3 text-xs font-medium uppercase tracking-wider"></th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-zinc-800/70">
                          {sessions.map(s => (
                            <tr key={s.id} className="transition-colors hover:bg-zinc-800/25">
                              <td className="px-4 py-3 align-top font-mono text-xs text-zinc-300">
                                {s.ip || 'unknown'}
                              </td>
                              <td className="px-4 py-3 align-top">
                                <p className="max-w-[28rem] truncate text-sm text-zinc-300">
                                  {s.user_agent || 'unknown device'}
                                </p>
                                <p className="mt-1 font-mono text-[11px] text-zinc-600">{s.id}</p>
                              </td>
                              <td className="px-4 py-3 align-top text-xs text-zinc-400">
                                {format_date(s.created_at)}
                              </td>
                              <td className="px-4 py-3 align-top text-xs text-zinc-400">
                                {format_date(s.last_active)}
                              </td>
                              <td className="px-4 py-3 text-right align-top">
                                <form action={`/api/admin/sessions/${s.id}/revoke`} method="POST">
                                  <button className="inline-flex items-center rounded-md border border-red-500/30 bg-red-500/10 px-3 py-1.5 text-xs text-red-400 transition-colors hover:bg-red-500/20">
                                    Revoke
                                  </button>
                                </form>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                </>
              )}
            </section>

            <SettingsClient />
          </div>

          <div className="space-y-6">
            <section className="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
              <div className="flex flex-col gap-5">
                <div className="flex items-start gap-4">
                  <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-zinc-800 text-xl font-semibold text-zinc-200">
                    {firstName}
                  </div>
                  <div className="min-w-0">
                    <p className="truncate text-lg font-semibold text-zinc-100">{session.name}</p>
                    <p className="mt-1 break-all text-sm text-zinc-500">{session.email}</p>
                    <span
                      className={`mt-3 inline-flex rounded border px-2 py-1 text-[10px] font-medium ${ROLE_COLORS[session.role]}`}
                    >
                      {ROLE_LABELS[session.role]}
                    </span>
                  </div>
                </div>

                <div className="rounded-xl border border-zinc-800/80 bg-zinc-950/40 px-4 py-3">
                  <p className="text-[11px] uppercase tracking-wider text-zinc-600">Profile</p>
                  <p className="mt-2 text-sm text-zinc-400">
                    Name and email are managed by the system administrator.
                  </p>
                </div>
              </div>
            </section>

            <section className="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
              <div className="flex flex-col gap-4">
                <div>
                  <h2 className="text-sm font-semibold text-zinc-300">Two-Factor Authentication</h2>
                  <p className="mt-2 text-sm text-zinc-300">TOTP Authenticator</p>
                  <p className="mt-1 text-xs text-zinc-500">
                    Use an authenticator app to generate one-time codes for secure admin access.
                  </p>
                </div>
                <Link
                  href="/settings/totp-setup"
                  className="inline-flex min-h-11 items-center justify-center rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 sm:min-h-9 sm:self-start sm:py-1.5"
                >
                  Setup TOTP
                </Link>
              </div>
            </section>

            <section className="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
              <h2 className="mb-4 text-sm font-semibold text-zinc-300">Security</h2>
              <dl className="space-y-3 text-xs">
                <div className="flex flex-col gap-1">
                  <dt className="text-zinc-500">Account ID</dt>
                  <dd className="break-all font-mono text-zinc-400">{session.id}</dd>
                </div>
                <div className="flex flex-col gap-1">
                  <dt className="text-zinc-500">Role</dt>
                  <dd className="text-zinc-400 capitalize">{session.role}</dd>
                </div>
              </dl>
            </section>
          </div>
        </div>
      </div>
    </>
  )
}
