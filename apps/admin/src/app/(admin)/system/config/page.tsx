import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import ConfigEditorClient from './ConfigEditorClient'

interface Flag {
  id: string
  name: string
  description: string
  enabled: boolean
  is_dangerous: boolean
  rollout_percent?: number
}

export default async function ConfigPage() {
  const session = await getSession()
  if (!session) redirect('/login')
  if (session.role !== 'owner') {
    return (
      <>
        <TopBar title="Config Editor" subtitle="Feature flags & kill switches" />
        <div className="p-6">
          <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-8 text-center">
            <p className="text-sm font-semibold text-red-400">Access denied — owner only</p>
          </div>
        </div>
      </>
    )
  }

  let flags: Flag[] = []
  try {
    const res = await adminFetch('/internal/admin/flags')
    if (res.ok) {
      const data = await res.json()
      flags = Array.isArray(data) ? data : (data.flags ?? [])
    }
  } catch { /* non-fatal */ }

  // flags stays empty — ConfigEditorClient shows "No flags yet" empty state

  return (
    <>
      <TopBar title="Config Editor" subtitle="Feature flags & kill switches" />
      <div className="p-6 max-w-3xl">
        <ConfigEditorClient flags={flags} />
      </div>
    </>
  )
}
