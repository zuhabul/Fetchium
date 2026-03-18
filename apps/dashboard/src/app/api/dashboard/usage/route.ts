import { NextRequest, NextResponse } from "next/server";
import { getToken } from "next-auth/jwt";
import { auth_secret } from "@/auth";
import { resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

export async function GET(req: NextRequest) {
  try {
    const token = await getToken({
      req,
      secret: auth_secret() || undefined,
      secureCookie: true,
      cookieName: "__Secure-authjs.session-token",
    });
    const rawApiKey = token?.apiKey;
    if (!rawApiKey?.startsWith("fetchium_")) {
      return NextResponse.json(
        { error: "unauthorized", message: "An authenticated dashboard session is required." },
        { status: 401 },
      );
    }
    const apiBase = resolve_api_base(token && typeof token.apiBase === "string" ? token.apiBase : undefined);
    const res = await fetch(`${apiBase}/v1/dashboard/usage`, {
      headers: { Authorization: `Bearer ${rawApiKey}` },
      cache: "no-store",
    });
    const text = await res.text();
    return new NextResponse(text, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return NextResponse.json(
      { error: "usage_analytics_fetch_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}
