'use client'

import { useState, useEffect } from 'react'
import CommandPalette from './CommandPalette'
import Sidebar from './Sidebar'
import { useKeyboardShortcuts } from '@/lib/shortcuts'
import type { AdminRole } from '@/types/admin'

interface AdminShellProps {
  children: React.ReactNode
  user: { name: string; email: string; role: AdminRole }
}

export default function AdminShell({ children, user }: AdminShellProps) {
  const [paletteOpen, setPaletteOpen] = useState(false)
  const [sidebarOpen, setSidebarOpen] = useState(false)

  // Activate global two-key shortcuts (g+o, g+k, etc.)
  useKeyboardShortcuts()

  // Cmd+K / Ctrl+K to open palette
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setPaletteOpen(v => !v)
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [])

  // Allow TopBar Search button to open palette via custom event
  useEffect(() => {
    function handleOpen() { setPaletteOpen(true) }
    window.addEventListener('fetchium:open-palette', handleOpen)
    return () => window.removeEventListener('fetchium:open-palette', handleOpen)
  }, [])

  useEffect(() => {
    function handleSidebarOpen() {
      setSidebarOpen(true)
    }

    function handleSidebarClose() {
      setSidebarOpen(false)
    }

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setSidebarOpen(false)
      }
    }

    window.addEventListener('fetchium:open-sidebar', handleSidebarOpen)
    window.addEventListener('fetchium:close-sidebar', handleSidebarClose)
    window.addEventListener('keydown', handleKeydown)

    return () => {
      window.removeEventListener('fetchium:open-sidebar', handleSidebarOpen)
      window.removeEventListener('fetchium:close-sidebar', handleSidebarClose)
      window.removeEventListener('keydown', handleKeydown)
    }
  }, [])

  useEffect(() => {
    const previousOverflow = document.body.style.overflow
    document.body.style.overflow = sidebarOpen ? 'hidden' : previousOverflow

    return () => {
      document.body.style.overflow = previousOverflow
    }
  }, [sidebarOpen])

  return (
    <div className="flex h-screen bg-zinc-950 overflow-hidden">
      <CommandPalette isOpen={paletteOpen} onClose={() => setPaletteOpen(false)} />

      <div
        aria-hidden={!sidebarOpen}
        className={`fixed inset-0 z-30 bg-black/50 backdrop-blur-sm transition-opacity lg:hidden ${
          sidebarOpen ? 'opacity-100' : 'pointer-events-none opacity-0'
        }`}
        onClick={() => setSidebarOpen(false)}
      />

      <Sidebar
        user={user}
        mobileOpen={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
      />

      <main className="flex-1 flex min-w-0 flex-col overflow-auto">
        {children}
      </main>
    </div>
  )
}
