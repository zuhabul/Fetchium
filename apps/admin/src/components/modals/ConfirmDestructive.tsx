'use client'

import { useState, useEffect } from 'react'
import { AlertTriangle, X } from 'lucide-react'

interface ConfirmDestructiveProps {
  isOpen: boolean
  onClose: () => void
  onConfirm: (totp?: string) => void
  title: string
  description: string
  confirmLabel?: string
  requireTotp?: boolean
}

export default function ConfirmDestructive({
  isOpen,
  onClose,
  onConfirm,
  title,
  description,
  confirmLabel = 'Confirm',
  requireTotp = false,
}: ConfirmDestructiveProps) {
  const [totp, setTotp] = useState('')

  // Reset TOTP on open
  useEffect(() => {
    if (isOpen) setTotp('')
  }, [isOpen])

  // Close on Escape
  useEffect(() => {
    if (!isOpen) return
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [isOpen, onClose])

  if (!isOpen) return null

  const canConfirm = !requireTotp || totp.length === 6

  return (
    <div
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-start justify-between px-5 pt-5 pb-4">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center flex-shrink-0">
              <AlertTriangle className="w-5 h-5 text-amber-400" />
            </div>
            <h2 className="text-sm font-semibold text-amber-400 leading-snug">{title}</h2>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-600 hover:text-zinc-400 transition-colors"
            aria-label="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="px-5 pb-4">
          <p className="text-sm text-zinc-400 leading-relaxed">{description}</p>

          {requireTotp && (
            <div className="mt-4">
              <label className="block text-xs font-medium text-zinc-400 mb-1.5">
                TOTP Code
              </label>
              <input
                type="text"
                inputMode="numeric"
                pattern="[0-9]*"
                maxLength={6}
                placeholder="000000"
                value={totp}
                onChange={e => setTotp(e.target.value.replace(/\D/g, '').slice(0, 6))}
                className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-2 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 w-full tracking-widest"
              />
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="px-5 pb-5 flex items-center justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={() => onConfirm(requireTotp ? totp : undefined)}
            disabled={!canConfirm}
            className="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-500 disabled:opacity-40 disabled:cursor-not-allowed rounded-md transition-colors"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  )
}
