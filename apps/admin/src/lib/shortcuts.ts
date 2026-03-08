'use client'

import { useEffect, useRef } from 'react'
import { useRouter } from 'next/navigation'

const SEQUENCES: Record<string, string> = {
  'g+o': '/overview',
  'g+r': '/orgs',
  'g+k': '/keys',
  'g+u': '/usage',
  'g+b': '/billing',
  'g+s': '/support',
  'g+i': '/incidents',
  'g+a': '/audit',
}

const PREFIXES = new Set(
  Object.keys(SEQUENCES).map(seq => seq.split('+')[0])
)

/**
 * Global keyboard shortcut hook.
 * - g+o = overview, g+r = orgs, g+k = keys, g+u = usage
 * - g+b = billing, g+s = support, g+i = incidents, g+a = audit
 * - ? = show shortcuts (dispatches custom event)
 * Two-key: listens for prefix key, then waits 500ms for second key.
 */
export function useKeyboardShortcuts() {
  const router = useRouter()
  const pendingRef = useRef<string | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    function handler(e: KeyboardEvent) {
      // Skip when typing in inputs/textareas
      const target = e.target as HTMLElement
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) return

      // Skip modifier combos (Cmd+K handled by CommandPalette separately)
      if (e.metaKey || e.ctrlKey || e.altKey) return

      const key = e.key.toLowerCase()

      if (key === '?') {
        window.dispatchEvent(new CustomEvent('fetchium:show-shortcuts'))
        return
      }

      if (pendingRef.current !== null) {
        // We have a pending first key — check sequence
        const seq = `${pendingRef.current}+${key}`
        if (timerRef.current) clearTimeout(timerRef.current)
        pendingRef.current = null

        const route = SEQUENCES[seq]
        if (route) {
          e.preventDefault()
          router.push(route)
        }
        return
      }

      // Check if this key is a sequence prefix
      if (PREFIXES.has(key)) {
        e.preventDefault()
        pendingRef.current = key
        timerRef.current = setTimeout(() => {
          pendingRef.current = null
        }, 500)
      }
    }

    window.addEventListener('keydown', handler)
    return () => {
      window.removeEventListener('keydown', handler)
      if (timerRef.current) clearTimeout(timerRef.current)
    }
  }, [router])
}
