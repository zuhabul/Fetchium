'use client'

import { useEffect } from 'react'
import { X } from 'lucide-react'

interface EntityDrawerProps {
  isOpen: boolean
  onClose: () => void
  title: string
  children: React.ReactNode
}

export default function EntityDrawer({ isOpen, onClose, title, children }: EntityDrawerProps) {
  // Close on Escape
  useEffect(() => {
    if (!isOpen) return
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [isOpen, onClose])

  return (
    <>
      {/* Slide-in panel */}
      <div
        className={`fixed inset-y-0 right-0 z-40 w-96 bg-zinc-900 border-l border-zinc-800 shadow-2xl
          transform transition-transform duration-200 ease-in-out flex flex-col
          ${isOpen ? 'translate-x-0' : 'translate-x-full'}`}
        aria-hidden={!isOpen}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-zinc-800 flex-shrink-0">
          <h2 className="text-sm font-semibold text-zinc-100 truncate">{title}</h2>
          <button
            onClick={onClose}
            className="w-7 h-7 flex items-center justify-center rounded-md text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800 transition-colors"
            aria-label="Close drawer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-5 py-4">
          {children}
        </div>
      </div>
    </>
  )
}
