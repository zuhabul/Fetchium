import { NextRequest, NextResponse } from "next/server";
import { resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

export async function GET(req: NextRequest) {
  try {
    const apiBase = resolve_api_base(req.nextUrl.searchParams.get("apiBase") || undefined);
    const res = await fetch(`${apiBase}/health`, { cache: "no-store" });
    const text = await res.text();
    return new NextResponse(text, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return NextResponse.json(
      { error: "health_check_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}

