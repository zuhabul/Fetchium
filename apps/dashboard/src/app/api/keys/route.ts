import { NextRequest, NextResponse } from "next/server";
import { admin_secret, resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

export async function GET(req: NextRequest) {
  try {
    const apiBase = resolve_api_base();
    const res = await fetch(`${apiBase}/v1/keys`, {
      headers: {
        "X-Admin-Secret": admin_secret(),
      },
      cache: "no-store",
    });
    const body = await res.text();
    return new NextResponse(body, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return NextResponse.json(
      { error: "keys_fetch_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}

export async function POST(req: NextRequest) {
  try {
    const payload = (await req.json()) as { name?: string; plan?: string };
    const apiBase = resolve_api_base();
    const res = await fetch(`${apiBase}/v1/keys`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Admin-Secret": admin_secret(),
      },
      body: JSON.stringify({
        name: payload.name || "Dashboard key",
        plan: payload.plan || "free",
      }),
      cache: "no-store",
    });
    const body = await res.text();
    return new NextResponse(body, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return NextResponse.json(
      { error: "key_create_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}
