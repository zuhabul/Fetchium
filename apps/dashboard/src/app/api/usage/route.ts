import { NextResponse } from "next/server";
import { resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

type UsageRequest = {
  apiKey?: string;
  apiBase?: string;
};

export async function POST(req: Request) {
  try {
    const body = (await req.json()) as UsageRequest;
    const apiKey = (body.apiKey || "").trim();
    if (!apiKey.startsWith("fetchium_")) {
      return NextResponse.json(
        { error: "invalid_api_key", message: "A valid fetchium_ API key is required." },
        { status: 400 },
      );
    }
    const apiBase = resolve_api_base(body.apiBase);
    const res = await fetch(`${apiBase}/v1/usage`, {
      headers: {
        Authorization: `Bearer ${apiKey}`,
      },
      cache: "no-store",
    });
    const text = await res.text();
    return new NextResponse(text, {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return NextResponse.json(
      { error: "usage_fetch_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}

