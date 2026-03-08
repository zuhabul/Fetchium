import { NextRequest, NextResponse } from 'next/server'
import { jwtVerify } from 'jose'

const SECRET = new TextEncoder().encode(
  process.env.***REMOVED*** || 'dev-secret-change-in-production-32chars'
)
const PUBLIC = ['/login', '/api/auth', '/robots.txt', '/_next', '/favicon.ico']

export async function middleware(req: NextRequest) {
  const { pathname } = req.nextUrl

  if (PUBLIC.some((p) => pathname.startsWith(p))) return NextResponse.next()

  const cookie = req.cookies.get('fetchium_admin_session')
  if (!cookie) return NextResponse.redirect(new URL('/login', req.url))

  try {
    await jwtVerify(cookie.value, SECRET)
    return NextResponse.next()
  } catch {
    const res = NextResponse.redirect(new URL('/login', req.url))
    res.cookies.delete('fetchium_admin_session')
    return res
  }
}

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico).*)'],
}
