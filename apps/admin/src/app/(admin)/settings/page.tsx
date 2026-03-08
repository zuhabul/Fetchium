import { redirect } from 'next/navigation'
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

export default async function SettingsPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let sessions: SessionRow[] = []
  try {
    const res = await adminFetch('/internal/admin/sessions')
    if (res.ok) sessions = await res.json()
  } catch {
    // Non-fatal — show empty state
  }

  return (
    <>
      <TopBar title="Settings" subtitle="Account preferences and security" />
      <div className="p-6 max-w-2xl space-y-6">

        {/* Profile */}
        <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-zinc-300 mb-4">Profile</h2>
          <div className="flex items-center gap-4 mb-4">
            <div className="w-12 h-12 rounded-full bg-zinc-700 flex items-center justify-center text-lg font-semibold text-zinc-300">
              {session.name.charAt(0).toUpperCase()}
            </div>
            <div>
              <p className="text-sm font-medium text-zinc-100">{session.name}</p>
              <p className="text-xs text-zinc-500">{session.email}</p>
            </div>
            <span className={`ml-auto text-[10px] font-medium px-1.5 py-0.5 rounded border ${ROLE_COLORS[session.role]}`}>
              {ROLE_LABELS[session.role]}
            </span>
          </div>
          <p className="text-xs text-zinc-600">Name and email are managed by the system administrator.</p>
        </section>

        {/* Two-Factor Auth */}
        <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-zinc-300 mb-4">Two-Factor Authentication</h2>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-zinc-300">TOTP Authenticator</p>
              <p className="text-xs text-zinc-500 mt-0.5">Use an authenticator app to generate one-time codes.</p>
            </div>
            <a
              href="/settings/totp-setup"
              className="px-3 py-1.5 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors"
            >
              Setup TOTP
            </a>
          </div>
        </section>

        {/* Active Sessions */}
        <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-zinc-300 mb-4">Active Sessions</h2>
          {sessions.length === 0 ? (
            <p className="text-xs text-zinc-600">No session data available.</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-xs">
                <thead>
                  <tr className="text-left text-zinc-600 border-b border-zinc-800">
                    <th className="pb-2 font-medium">IP</th>
                    <th className="pb-2 font-medium">User Agent</th>
                    <th className="pb-2 font-medium">Last Active</th>
                    <th className="pb-2 font-medium"></th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800">
                  {sessions.map(s => (
                    <tr key={s.id} className="text-zinc-400">
                      <td className="py-2 pr-3 font-mono">{s.ip}</td>
                      <td className="py-2 pr-3 max-w-[180px] truncate">{s.user_agent}</td>
                      <td className="py-2 pr-3 whitespace-nowrap">{s.last_active}</td>
                      <td className="py-2">
                        <form action={`/api/admin/sessions/${s.id}/revoke`} method="POST">
                          <button className="text-red-500 hover:text-red-400 transition-colors">Revoke</button>
                        </form>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>

        {/* Display Preferences (client) */}
        <SettingsClient />

        {/* Security */}
        <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-zinc-300 mb-4">Security</h2>
          <dl className="space-y-2 text-xs">
            <div className="flex justify-between">
              <dt className="text-zinc-500">Account ID</dt>
              <dd className="text-zinc-400 font-mono">{session.id}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-zinc-500">Role</dt>
              <dd className="text-zinc-400 capitalize">{session.role}</dd>
            </div>
          </dl>
        </section>

      </div>
    </>
  )
}
