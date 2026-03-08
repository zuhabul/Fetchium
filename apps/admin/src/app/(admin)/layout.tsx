import { redirect } from 'next/navigation'
import { getSession } from '@/lib/session'
import Sidebar from '@/components/layout/Sidebar'
import AdminShell from '@/components/layout/AdminShell'

export default async function AdminLayout({ children }: { children: React.ReactNode }) {
  const session = await getSession()
  if (!session) redirect('/login')

  return (
    <div className="flex h-screen bg-zinc-950 overflow-hidden">
      <Sidebar user={{ name: session.name, email: session.email, role: session.role }} />
      <main className="flex-1 flex flex-col min-w-0 overflow-auto">
        <AdminShell>
          {children}
        </AdminShell>
      </main>
    </div>
  )
}
