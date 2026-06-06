import { NextRequest, NextResponse } from "next/server";
import { getToken } from "next-auth/jwt";
import { auth_secret } from "@/auth";
import { resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

async function resolveAuth(req: NextRequest) {
  const token = await getToken({
    req,
    secret: auth_secret() || undefined,
    secureCookie: true,
    cookieName: "__Secure-authjs.session-token",
  });
  const rawApiKey = token?.apiKey;
  if (!rawApiKey?.startsWith("fetchium_")) {
    return { error: true as const };
  }
  const apiBase = resolve_api_base(
    token && typeof token.apiBase === "string" ? token.apiBase : undefined,
  );
  return { error: false as const, rawApiKey, apiBase };
}

export async function GET(req: NextRequest) {
  try {
    const auth = await resolveAuth(req);
    if (auth.error) {
      return NextResponse.json(
        { error: "unauthorized", message: "An authenticated dashboard session is required." },
        { status: 401 },
      );
    }
    const res = await fetch(`${auth.apiBase}/v1/dashboard/settings`, {
      headers: { Authorization: `Bearer ${auth.rawApiKey}` },
      cache: "no-store",
    });
    const text = await res.text();
    return new NextResponse(text, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return NextResponse.json(
      { error: "settings_fetch_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}

export async function PATCH(req: NextRequest) {
  try {
    const auth = await resolveAuth(req);
    if (auth.error) {
      return NextResponse.json(
        { error: "unauthorized", message: "An authenticated dashboard session is required." },
        { status: 401 },
      );
    }
    const body = await req.text();
    const res = await fetch(`${auth.apiBase}/v1/dashboard/settings`, {
      method: "PATCH",
      headers: {
        Authorization: `Bearer ${auth.rawApiKey}`,
        "Content-Type": "application/json",
      },
      body,
      cache: "no-store",
    });
    const text = await res.text();
    return new NextResponse(text, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return NextResponse.json(
      { error: "settings_update_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}
