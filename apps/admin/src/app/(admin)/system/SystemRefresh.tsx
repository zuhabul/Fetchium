'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'

/** Auto-refreshes the server component data every 10 seconds via router.refresh(). */
export default function SystemRefresh() {
  const router = useRouter()
  useEffect(() => {
    const id = setInterval(() => router.refresh(), 10_000)
    return () => clearInterval(id)
  }, [router])
  return null
}
