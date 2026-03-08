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
const SECRET = new TextEncoder().encode(
  process.env.***REMOVED*** || 'dev-secret-change-in-production-32chars'
)

export async function createSession(session: AdminSession): Promise<void> {
  const token = await new SignJWT({ ...session })
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime('8h')
    .sign(SECRET)

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
    const { payload } = await jwtVerify(cookie.value, SECRET)
    return payload as unknown as AdminSession
  } catch {
    return null
  }
}

export async function clearSession(): Promise<void> {
  const cookieStore = await cookies()
  cookieStore.delete(COOKIE)
}

export const INTERNAL_API = process.env.***REMOVED*** || '***REMOVED***'

/** Server-side fetch to Rust API with admin session token */
export async function adminFetch(
  path: string,
  session: AdminSession,
  options: RequestInit = {}
): Promise<Response> {
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
