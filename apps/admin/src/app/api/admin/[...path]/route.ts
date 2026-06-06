/**
 * Catch-all proxy: /api/admin/** → http://127.0.0.1:3050/internal/admin/**
 * Reads JWT session cookie, forwards Bearer token to Rust backend.
 */
import { NextRequest, NextResponse } from 'next/server'
import { getSession } from '@/lib/session'

const RUST_BASE = process.env.FETCHIUM_INTERNAL_API_URL || 'http://127.0.0.1:3050'

async function proxy(req: NextRequest, params: { path: string[] }) {
  const session = await getSession()
  if (!session) {
    return NextResponse.json({ error: 'Not authenticated' }, { status: 401 })
  }

  const path = params.path.join('/')
  const search = req.nextUrl.search
  const upstreamUrl = `${RUST_BASE}/internal/admin/${path}${search}`

  const headers: Record<string, string> = {
    Authorization: `Bearer ${session.sessionToken}`,
    'Content-Type': 'application/json',
  }

  let body: string | undefined
  if (req.method !== 'GET' && req.method !== 'HEAD') {
    try { body = await req.text() } catch { /* empty body ok */ }
  }

  const upstreamRes = await fetch(upstreamUrl, {
    method: req.method,
    headers,
    body: body || undefined,
    cache: 'no-store',
  })

  const text = await upstreamRes.text()
  let data: unknown
  try { data = JSON.parse(text) } catch { data = { raw: text } }

  return NextResponse.json(data, { status: upstreamRes.status })
}

export async function GET(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, await params)
}
export async function POST(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, await params)
}
export async function PATCH(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, await params)
}
export async function PUT(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, await params)
}
export async function DELETE(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, await params)
}
