import { NextRequest, NextResponse } from 'next/server'
import { createSession } from '@/lib/session'

const API = process.env.FETCHIUM_INTERNAL_API_URL || 'http://127.0.0.1:3050'

export async function POST(req: NextRequest) {
  try {
    const { email, password, totp_code } = await req.json()

    const body: Record<string, string> = { email, password }
    if (totp_code) body.totp_code = totp_code

    const res = await fetch(`${API}/internal/admin/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      cache: 'no-store',
    })

    const data = await res.json()

    if (!res.ok) {
      // Signal TOTP requirement to client
      if (data.error === 'TOTP_REQUIRED') {
        return NextResponse.json({ totp_required: true }, { status: 202 })
      }
      return NextResponse.json({ error: 'Invalid credentials' }, { status: 401 })
    }

    await createSession({
      sessionToken: data.sessionToken,
      id: data.id,
      email: data.email,
      name: data.name,
      role: data.role,
    })

    return NextResponse.json({ ok: true, role: data.role, name: data.name })
  } catch {
    return NextResponse.json({ error: 'Internal error' }, { status: 500 })
  }
}
