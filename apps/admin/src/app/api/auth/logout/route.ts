import { NextResponse } from 'next/server'
import { getSession, clearSession, adminFetch } from '@/lib/session'

export async function POST() {
  const session = await getSession()
  if (session) {
    // Tell Rust to revoke the session
    try {
      await adminFetch('/internal/admin/auth/logout', { method: 'POST' })
    } catch { /* ignore — still clear cookie */ }
  }
  await clearSession()
  return NextResponse.json({ ok: true })
}
