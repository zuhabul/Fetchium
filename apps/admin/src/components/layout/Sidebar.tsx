'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import {
  LayoutDashboard, Building2, Users, Key, BarChart3, CreditCard,
  HeartHandshake, Ticket, Megaphone, Flame, Server, Shield, Flag, Settings,
  LogOut, ChevronRight, X
} from 'lucide-react'
import { can } from '@/lib/permissions'
import { ROLE_COLORS, ROLE_LABELS } from '@/types/admin'
import type { AdminRole, Permission } from '@/lib/permissions'

interface NavItem {
  href: string
  label: string
  icon: React.ElementType
  perm?: Permission
}

const NAV: NavItem[] = [
  { href: '/overview',   label: 'Overview',    icon: LayoutDashboard },
  { href: '/orgs',       label: 'Orgs',        icon: Building2,  perm: 'orgs.read' },
  { href: '/users',      label: 'Users',       icon: Users,      perm: 'users.read' },
  { href: '/keys',       label: 'API Keys',    icon: Key,        perm: 'keys.read' },
  { href: '/usage',      label: 'Usage',       icon: BarChart3,  perm: 'keys.read' },
  { href: '/billing',    label: 'Billing',     icon: CreditCard, perm: 'billing.read' },
  { href: '/crm',        label: 'CRM',         icon: HeartHandshake, perm: 'crm.read' },
  { href: '/support',    label: 'Support',     icon: Ticket,     perm: 'support.read' },
  { href: '/campaigns',  label: 'Campaigns',   icon: Megaphone,  perm: 'campaigns.read' },
  { href: '/incidents',  label: 'Incidents',   icon: Flame,      perm: 'incidents.read' },
  { href: '/proxy',      label: 'Proxy Ops',   icon: Server,     perm: 'proxy.read' },
  { href: '/audit',      label: 'Audit',       icon: Shield,     perm: 'audit.read' },
  { href: '/flags',      label: 'Flags',       icon: Flag,       perm: 'flags.read' },
  { href: '/system',     label: 'System',      icon: Server,     perm: 'proxy.read' as Permission },
  { href: '/settings',   label: 'Settings',    icon: Settings },
]

interface SidebarProps {
  user: { name: string; email: string; role: AdminRole }
  mobileOpen?: boolean
  onClose?: () => void
}

export default function Sidebar({
  user,
  mobileOpen = false,
  onClose,
}: SidebarProps) {
  const pathname = usePathname()

  async function handleLogout() {
    await fetch('/api/auth/logout', { method: 'POST' })
    window.location.href = '/login'
  }

  const visibleNav = NAV.filter(item => !item.perm || can(user.role, item.perm as Permission))

  return (
    <aside
      id="admin-sidebar"
      className={`fixed inset-y-0 left-0 z-40 flex h-screen w-[min(18rem,calc(100vw-1rem))] flex-shrink-0 flex-col border-r border-zinc-800 bg-zinc-900 shadow-2xl transition-transform duration-200 lg:static lg:w-56 lg:translate-x-0 lg:shadow-none ${
        mobileOpen ? 'translate-x-0' : '-translate-x-full'
      }`}
      style={{
        paddingTop: 'env(safe-area-inset-top, 0px)',
        paddingBottom: 'env(safe-area-inset-bottom, 0px)',
      }}
    >
      {/* Logo */}
      <div className="px-4 py-4 border-b border-zinc-800">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 bg-zinc-800 border border-zinc-700 rounded flex items-center justify-center">
            <Shield className="w-3.5 h-3.5 text-zinc-300" />
          </div>
          <span className="font-semibold text-sm text-zinc-100">Fetchium</span>
          <span className="text-[10px] font-bold bg-red-500/20 text-red-400 border border-red-500/30 px-1 py-0.5 rounded leading-none ml-auto">
            ADMIN
          </span>
          <button
            type="button"
            onClick={onClose}
            className="ml-1 flex h-10 w-10 items-center justify-center rounded-md text-zinc-500 hover:bg-zinc-800 hover:text-zinc-200 lg:hidden"
            aria-label="Close navigation"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 space-y-1 overflow-y-auto px-2 py-3">
        {visibleNav.map(item => {
          const Icon = item.icon
          const active = pathname === item.href || pathname.startsWith(item.href + '/')
          return (
            <Link
              key={item.href}
              href={item.href}
              onClick={onClose}
              className={`group flex min-h-11 items-center gap-2.5 rounded-md px-2.5 py-2 text-sm transition-colors ${
                active
                  ? 'bg-zinc-800 text-zinc-100'
                  : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/60'
              }`}
            >
              <Icon className={`w-4 h-4 flex-shrink-0 ${active ? 'text-zinc-300' : 'text-zinc-600 group-hover:text-zinc-400'}`} />
              {item.label}
              {active && <ChevronRight className="w-3 h-3 ml-auto text-zinc-600" />}
            </Link>
          )
        })}
      </nav>

      {/* User */}
      <div className="px-3 py-3 border-t border-zinc-800">
        <div className="flex items-center gap-2.5 mb-2">
          <div className="w-7 h-7 rounded-full bg-zinc-700 flex items-center justify-center text-xs font-medium text-zinc-300 flex-shrink-0">
            {user.name.charAt(0).toUpperCase()}
          </div>
          <div className="min-w-0">
            <p className="text-xs font-medium text-zinc-200 truncate">{user.name}</p>
            <p className="text-[10px] text-zinc-600 truncate">{user.email}</p>
          </div>
        </div>
        <div className="flex items-center justify-between gap-3">
          <span className={`text-[10px] font-medium px-1.5 py-0.5 rounded border ${ROLE_COLORS[user.role]}`}>
            {ROLE_LABELS[user.role]}
          </span>
          <button
            onClick={handleLogout}
            className="flex min-h-11 items-center gap-1 text-[11px] text-zinc-600 transition-colors hover:text-red-400"
          >
            <LogOut className="w-3 h-3" />
            Sign out
          </button>
        </div>
      </div>
    </aside>
  )
}
