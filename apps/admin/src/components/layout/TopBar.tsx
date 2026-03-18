'use client'

import { Menu, Search } from 'lucide-react'
import NotificationCenter from './NotificationCenter'

interface TopBarProps {
  title: string
  subtitle?: string
}

export default function TopBar({ title, subtitle }: TopBarProps) {
  function openPalette() {
    window.dispatchEvent(new CustomEvent('fetchium:open-palette'))
  }

  function openSidebar() {
    window.dispatchEvent(new CustomEvent('fetchium:open-sidebar'))
  }

  return (
    <header className="sticky top-0 z-10 flex min-h-14 items-center gap-3 border-b border-zinc-800 bg-zinc-900/80 px-3 backdrop-blur sm:px-4 sm:gap-4">
      <button
        type="button"
        onClick={openSidebar}
        className="flex h-11 w-11 items-center justify-center rounded-md border border-zinc-800 bg-zinc-900 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 lg:hidden"
        aria-label="Open navigation"
        aria-controls="admin-sidebar"
      >
        <Menu className="h-4 w-4" />
      </button>
      <div className="flex-1 min-w-0">
        <h1 className="text-sm font-semibold text-zinc-100 truncate">{title}</h1>
        {subtitle && <p className="text-xs text-zinc-500 truncate hidden sm:block">{subtitle}</p>}
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={openPalette}
          className="flex min-h-11 items-center gap-2 rounded-md border border-zinc-700 bg-zinc-800 px-2.5 py-1.5 text-xs text-zinc-400 transition-colors hover:bg-zinc-700"
        >
          <Search className="w-3.5 h-3.5" />
          <span className="hidden sm:inline">Search</span>
          <kbd className="hidden md:inline text-[10px] bg-zinc-700 px-1 rounded">⌘K</kbd>
        </button>
        <NotificationCenter />
      </div>
    </header>
  )
}
