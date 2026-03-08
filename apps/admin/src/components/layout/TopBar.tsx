'use client'

import { Search } from 'lucide-react'
import NotificationCenter from './NotificationCenter'

interface TopBarProps {
  title: string
  subtitle?: string
}

export default function TopBar({ title, subtitle }: TopBarProps) {
  function openPalette() {
    window.dispatchEvent(new CustomEvent('fetchium:open-palette'))
  }

  return (
    <header className="h-12 bg-zinc-900/80 backdrop-blur border-b border-zinc-800 flex items-center px-4 gap-4 sticky top-0 z-10">
      <div className="flex-1 min-w-0">
        <h1 className="text-sm font-semibold text-zinc-100 truncate">{title}</h1>
        {subtitle && <p className="text-xs text-zinc-500 truncate hidden sm:block">{subtitle}</p>}
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={openPalette}
          className="flex items-center gap-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md px-2.5 py-1.5 text-xs text-zinc-400 transition-colors"
        >
          <Search className="w-3.5 h-3.5" />
          <span className="hidden md:inline">Search</span>
          <kbd className="hidden md:inline text-[10px] bg-zinc-700 px-1 rounded">⌘K</kbd>
        </button>
        <NotificationCenter />
      </div>
    </header>
  )
}
