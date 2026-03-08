'use client'

import { useState, useEffect, useRef } from 'react'
import { Bell, Flame, Ticket, CheckCircle, X } from 'lucide-react'

interface Notification {
  id: string
  type: 'incident' | 'support' | 'system'
  message: string
  time: string
  href: string
  read: boolean
}

const INITIAL_NOTIFICATIONS: Notification[] = [
  {
    id: '1',
    type: 'incident',
    message: 'High error rate auto-incident opened',
    time: '2m ago',
    href: '/incidents',
    read: false,
  },
  {
    id: '2',
    type: 'support',
    message: 'Urgent ticket #42 SLA breaching in 30min',
    time: '5m ago',
    href: '/support',
    read: false,
  },
  {
    id: '3',
    type: 'system',
    message: 'System operational — all providers healthy',
    time: '10m ago',
    href: '/overview',
    read: false,
  },
]

function NotifIcon({ type }: { type: Notification['type'] }) {
  if (type === 'incident') return <Flame className="w-4 h-4 text-red-400" />
  if (type === 'support')  return <Ticket className="w-4 h-4 text-amber-400" />
  return <CheckCircle className="w-4 h-4 text-emerald-400" />
}

export default function NotificationCenter() {
  const [isOpen, setIsOpen] = useState(false)
  const [notifications, setNotifications] = useState<Notification[]>(INITIAL_NOTIFICATIONS)
  const panelRef = useRef<HTMLDivElement>(null)

  const unreadCount = notifications.filter(n => !n.read).length

  // Poll system status every 30s (demo: just keeps existing notifications)
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        await fetch('/api/admin/metrics/summary', {
        })
        // In production, derive new notifications from summary response
      } catch {
        // Silently ignore polling errors
      }
    }, 30_000)
    return () => clearInterval(interval)
  }, [])

  // Close on outside click
  useEffect(() => {
    if (!isOpen) return
    function handleClick(e: MouseEvent) {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClick)
    return () => document.removeEventListener('mousedown', handleClick)
  }, [isOpen])

  function markAllRead() {
    setNotifications(prev => prev.map(n => ({ ...n, read: true })))
  }

  function dismiss(id: string) {
    setNotifications(prev => prev.filter(n => n.id !== id))
  }

  return (
    <div className="relative" ref={panelRef}>
      <button
        onClick={() => setIsOpen(v => !v)}
        className="w-8 h-8 flex items-center justify-center rounded-md hover:bg-zinc-800 text-zinc-500 hover:text-zinc-300 transition-colors relative"
        aria-label="Notifications"
      >
        <Bell className="w-4 h-4" />
        {unreadCount > 0 && (
          <span className="absolute top-1 right-1 w-2 h-2 bg-red-500 rounded-full" />
        )}
      </button>

      {isOpen && (
        <div className="absolute right-0 top-10 w-80 bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl overflow-hidden z-50">
          {/* Header */}
          <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800">
            <span className="text-sm font-semibold text-zinc-100">Notifications</span>
            {unreadCount > 0 && (
              <button
                onClick={markAllRead}
                className="text-[11px] text-zinc-500 hover:text-zinc-300 transition-colors"
              >
                Mark all read
              </button>
            )}
          </div>

          {/* List */}
          <div className="divide-y divide-zinc-800 max-h-72 overflow-y-auto">
            {notifications.length === 0 ? (
              <p className="text-sm text-zinc-600 text-center py-8">No notifications</p>
            ) : (
              notifications.map(n => (
                <div
                  key={n.id}
                  className={`flex items-start gap-3 px-4 py-3 hover:bg-zinc-800/50 transition-colors ${
                    !n.read ? 'bg-zinc-800/20' : ''
                  }`}
                >
                  <div className="mt-0.5 flex-shrink-0">
                    <NotifIcon type={n.type} />
                  </div>
                  <a href={n.href} className="flex-1 min-w-0" onClick={() => setIsOpen(false)}>
                    <p className={`text-xs leading-snug ${n.read ? 'text-zinc-400' : 'text-zinc-200'}`}>
                      {n.message}
                    </p>
                    <p className="text-[10px] text-zinc-600 mt-0.5">{n.time}</p>
                  </a>
                  <button
                    onClick={() => dismiss(n.id)}
                    className="text-zinc-700 hover:text-zinc-400 transition-colors flex-shrink-0"
                    aria-label="Dismiss"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  )
}
