import { NextRequest, NextResponse } from "next/server";
import { admin_secret, assert_admin_keys_enabled, resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

export async function GET(req: NextRequest) {
  try {
    assert_admin_keys_enabled();
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
    if (err instanceof Error && err.message === "admin_key_management_disabled") {
      return NextResponse.json(
        {
          error: "admin_key_management_disabled",
          message: "API key management is disabled on the hosted dashboard.",
        },
        { status: 403 },
      );
    }
    return NextResponse.json(
      { error: "keys_fetch_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}

export async function POST(req: NextRequest) {
  try {
    assert_admin_keys_enabled();
    const payload = (await req.json()) as { name?: string; plan?: string };
    const plan = (payload.plan || "").trim();
    if (!["free", "starter", "pro", "enterprise"].includes(plan)) {
      return NextResponse.json(
        {
          error: "invalid_plan",
          message: "Plan must be one of free, starter, pro, enterprise.",
        },
        { status: 400 },
      );
    }
    const apiBase = resolve_api_base();
    const res = await fetch(`${apiBase}/v1/keys`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Admin-Secret": admin_secret(),
      },
      body: JSON.stringify({
        name: (payload.name || "Dashboard key").trim() || "Dashboard key",
        plan,
      }),
      cache: "no-store",
    });
    const body = await res.text();
    return new NextResponse(body, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    if (err instanceof Error && err.message === "admin_key_management_disabled") {
      return NextResponse.json(
        {
          error: "admin_key_management_disabled",
          message: "API key management is disabled on the hosted dashboard.",
        },
        { status: 403 },
      );
    }
    return NextResponse.json(
      { error: "key_create_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}
