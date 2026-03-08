'use client'

import { useEffect, useRef, useState, useCallback } from 'react'
import { useRouter } from 'next/navigation'
import {
  Search, LayoutDashboard, Building2, Users, Key, BarChart3,
  CreditCard, HeartHandshake, Ticket, Flame, Shield, Flag,
  Settings, Server, X
} from 'lucide-react'

interface Command {
  id: string
  label: string
  href: string
  icon: React.ElementType
  group: string
  keywords?: string[]
}

const COMMANDS: Command[] = [
  { id: 'overview',   label: 'Overview',   href: '/overview',   icon: LayoutDashboard, group: 'Navigate' },
  { id: 'orgs',       label: 'Orgs',       href: '/orgs',       icon: Building2,       group: 'Navigate' },
  { id: 'users',      label: 'Users',      href: '/users',      icon: Users,           group: 'Navigate' },
  { id: 'keys',       label: 'API Keys',   href: '/keys',       icon: Key,             group: 'Navigate', keywords: ['api', 'keys'] },
  { id: 'usage',      label: 'Usage',      href: '/usage',      icon: BarChart3,       group: 'Navigate' },
  { id: 'billing',    label: 'Billing',    href: '/billing',    icon: CreditCard,      group: 'Navigate' },
  { id: 'crm',        label: 'CRM',        href: '/crm',        icon: HeartHandshake,  group: 'Navigate' },
  { id: 'support',    label: 'Support',    href: '/support',    icon: Ticket,          group: 'Navigate' },
  { id: 'incidents',  label: 'Incidents',  href: '/incidents',  icon: Flame,           group: 'Navigate' },
  { id: 'audit',      label: 'Audit',      href: '/audit',      icon: Shield,          group: 'Navigate' },
  { id: 'flags',      label: 'Flags',      href: '/flags',      icon: Flag,            group: 'Navigate' },
  { id: 'proxy',      label: 'Proxy Ops',  href: '/proxy',      icon: Server,          group: 'Navigate' },
  { id: 'settings',   label: 'Settings',   href: '/settings',   icon: Settings,        group: 'Navigate' },
  { id: 'system',     label: 'System',     href: '/system',     icon: Server,          group: 'Navigate' },
]

interface CommandPaletteProps {
  isOpen: boolean
  onClose: () => void
}

export default function CommandPalette({ isOpen, onClose }: CommandPaletteProps) {
  const router = useRouter()
  const [query, setQuery] = useState('')
  const [highlighted, setHighlighted] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  const filtered = query.trim() === ''
    ? COMMANDS
    : COMMANDS.filter(cmd => {
        const q = query.toLowerCase()
        return (
          cmd.label.toLowerCase().includes(q) ||
          cmd.group.toLowerCase().includes(q) ||
          (cmd.keywords || []).some(k => k.includes(q))
        )
      })

  useEffect(() => {
    if (isOpen) {
      setQuery('')
      setHighlighted(0)
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [isOpen])

  useEffect(() => {
    setHighlighted(0)
  }, [query])

  const navigate = useCallback((cmd: Command) => {
    router.push(cmd.href)
    onClose()
  }, [router, onClose])

  useEffect(() => {
    if (!isOpen) return
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') { onClose(); return }
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setHighlighted(h => Math.min(h + 1, filtered.length - 1))
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setHighlighted(h => Math.max(h - 1, 0))
      } else if (e.key === 'Enter' && filtered[highlighted]) {
        navigate(filtered[highlighted])
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [isOpen, filtered, highlighted, navigate, onClose])

  if (!isOpen) return null

  const groups = [...new Set(filtered.map(c => c.group))]

  return (
    <div
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-start justify-center pt-[15vh]"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        {/* Search header */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-zinc-800">
          <Search className="w-4 h-4 text-zinc-500 flex-shrink-0" />
          <input
            ref={inputRef}
            type="text"
            placeholder="Search commands..."
            value={query}
            onChange={e => setQuery(e.target.value)}
            className="flex-1 bg-transparent text-sm text-zinc-100 placeholder-zinc-600 outline-none"
          />
          <button
            onClick={onClose}
            className="text-zinc-600 hover:text-zinc-400 transition-colors"
            aria-label="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Results */}
        <div className="max-h-80 overflow-y-auto py-2">
          {filtered.length === 0 ? (
            <p className="text-sm text-zinc-600 text-center py-8">No results for &ldquo;{query}&rdquo;</p>
          ) : (
            groups.map(group => {
              const items = filtered.filter(c => c.group === group)
              return (
                <div key={group}>
                  <p className="px-4 pt-2 pb-1 text-[10px] font-semibold text-zinc-600 uppercase tracking-wider">
                    {group}
                  </p>
                  {items.map(cmd => {
                    const Icon = cmd.icon
                    const idx = filtered.indexOf(cmd)
                    return (
                      <button
                        key={cmd.id}
                        onClick={() => navigate(cmd)}
                        onMouseEnter={() => setHighlighted(idx)}
                        className={`w-full flex items-center gap-3 px-4 py-2 text-sm transition-colors ${
                          highlighted === idx
                            ? 'bg-zinc-800 text-zinc-100'
                            : 'text-zinc-400 hover:text-zinc-200'
                        }`}
                      >
                        <Icon className="w-4 h-4 flex-shrink-0 text-zinc-500" />
                        <span className="flex-1 text-left">{cmd.label}</span>
                        {highlighted === idx && (
                          <kbd className="text-[10px] bg-zinc-700 text-zinc-400 px-1.5 py-0.5 rounded">↵</kbd>
                        )}
                      </button>
                    )
                  })}
                </div>
              )
            })
          )}
        </div>

        {/* Footer hint */}
        <div className="px-4 py-2 border-t border-zinc-800 flex items-center gap-3 text-[10px] text-zinc-600">
          <span><kbd className="bg-zinc-800 px-1 rounded">↑↓</kbd> navigate</span>
          <span><kbd className="bg-zinc-800 px-1 rounded">↵</kbd> select</span>
          <span><kbd className="bg-zinc-800 px-1 rounded">Esc</kbd> close</span>
        </div>
      </div>
    </div>
  )
}
