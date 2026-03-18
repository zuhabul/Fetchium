import { NextRequest, NextResponse } from 'next/server'
import { createSession } from '@/lib/session'
import type { AdminRole } from '@/types/admin'

const API = process.env.***REMOVED*** || '***REMOVED***'

interface LoginUpstreamResponse {
  id?: string
  email?: string
  name?: string
  role?: AdminRole
  sessionToken?: string
  error?: string
}

export async function POST(req: NextRequest) {
  try {
    const { email, password, totp_code } = await req.json()

    const body: Record<string, string> = { email, password }
    if (totp_code) body.totp_code = totp_code

    let res: Response
    try {
      res = await fetch(`${API}/internal/admin/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
        cache: 'no-store',
      })
    } catch (error) {
      console.error('admin login upstream unreachable', {
        api: API,
        error: error instanceof Error ? error.message : String(error),
      })
      return NextResponse.json({ error: 'Auth service unavailable' }, { status: 502 })
    }

    const raw = await res.text()
    let data: LoginUpstreamResponse = {}
    try {
      data = raw ? (JSON.parse(raw) as LoginUpstreamResponse) : {}
    } catch {
      console.error('admin login upstream returned non-json', {
        api: API,
        status: res.status,
        body: raw.slice(0, 300),
      })
      return NextResponse.json({ error: 'Auth service error' }, { status: 502 })
    }

    if (!res.ok) {
      // Signal TOTP requirement to client
      if (data.error === 'TOTP_REQUIRED') {
        return NextResponse.json({ totp_required: true }, { status: 202 })
      }
      if (res.status >= 500) {
        console.error('admin login upstream failed', {
          api: API,
          status: res.status,
          body: raw.slice(0, 300),
        })
        return NextResponse.json({ error: 'Auth service error' }, { status: 502 })
      }
      return NextResponse.json({ error: 'Invalid credentials' }, { status: 401 })
    }

    if (!data.sessionToken || !data.id || !data.email || !data.name || !data.role) {
      console.error('admin login upstream returned incomplete payload', {
        api: API,
        status: res.status,
        body: raw.slice(0, 300),
      })
      return NextResponse.json({ error: 'Auth service error' }, { status: 502 })
    }

    await createSession({
      sessionToken: data.sessionToken,
      id: data.id,
      email: data.email,
      name: data.name,
      role: data.role,
    })

    return NextResponse.json({ ok: true, role: data.role, name: data.name })
  } catch (error) {
    console.error('admin login route failed', {
      error: error instanceof Error ? error.message : String(error),
    })
    return NextResponse.json({ error: 'Internal error' }, { status: 500 })
  }
}
