'use client'

import { useState, useEffect } from 'react'
import CommandPalette from './CommandPalette'
import { useKeyboardShortcuts } from '@/lib/shortcuts'

interface AdminShellProps {
  children: React.ReactNode
}

export default function AdminShell({ children }: AdminShellProps) {
  const [paletteOpen, setPaletteOpen] = useState(false)

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

  return (
    <>
      <CommandPalette isOpen={paletteOpen} onClose={() => setPaletteOpen(false)} />
      {children}
    </>
  )
}
