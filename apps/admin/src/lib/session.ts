import { SignJWT, jwtVerify } from 'jose'
import { cookies } from 'next/headers'
import type { AdminRole } from '@/types/admin'

export interface AdminSession {
  sessionToken: string
  id: string
  email: string
  name: string
  role: AdminRole
}

const COOKIE = 'fetchium_admin_session'
const CURRENT_SECRET = process.env.ADMIN_SESSION_SECRET || 'dev-secret-change-in-production-32chars'
const PREVIOUS_SECRET = process.env.ADMIN_SESSION_SECRET_PREVIOUS || ''
const SIGNING_SECRET = new TextEncoder().encode(CURRENT_SECRET)
const VERIFY_SECRETS = [CURRENT_SECRET, PREVIOUS_SECRET]
  .filter(Boolean)
  .map((value) => new TextEncoder().encode(value))

export async function createSession(session: AdminSession): Promise<void> {
  const token = await new SignJWT({ ...session })
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime('8h')
    .sign(SIGNING_SECRET)

  const cookieStore = await cookies()
  cookieStore.set(COOKIE, token, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict',
    maxAge: 60 * 60 * 8,
    path: '/',
  })
}

export async function getSession(): Promise<AdminSession | null> {
  try {
    const cookieStore = await cookies()
    const cookie = cookieStore.get(COOKIE)
    if (!cookie) return null
    for (const secret of VERIFY_SECRETS) {
      try {
        const { payload } = await jwtVerify(cookie.value, secret)
        return payload as unknown as AdminSession
      } catch {
        continue
      }
    }
    return null
  } catch {
    return null
  }
}

export async function clearSession(): Promise<void> {
  const cookieStore = await cookies()
  cookieStore.delete(COOKIE)
}

export const INTERNAL_API = process.env.FETCHIUM_INTERNAL_API_URL || 'http://127.0.0.1:3050'

/** Server-side fetch to Rust API — auto-reads session from cookie */
export async function adminFetch(
  path: string,
  options: RequestInit = {}
): Promise<Response> {
  const session = await getSession()
  if (!session) throw new Error('Not authenticated')
  return fetch(`${INTERNAL_API}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${session.sessionToken}`,
      ...(options.headers as Record<string, string> || {}),
    },
    cache: 'no-store',
  })
}
