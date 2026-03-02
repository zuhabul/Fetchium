import { NextResponse } from "next/server";
import { resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

const ALLOWED_ENDPOINTS = new Set([
  "/v1/search",
  "/v1/scrape",
  "/v1/fetch",
  "/v1/research",
  "/v1/youtube/search",
  "/v1/youtube/analyze",
  "/v1/social/reddit",
  "/v1/social/hackernews",
  "/v1/social/research",
  "/v1/estimate",
]);

type PlaygroundRequest = {
  apiKey?: string;
  apiBase?: string;
  endpoint?: string;
  payload?: unknown;
};

export async function POST(req: Request) {
  const started = Date.now();
  try {
    const body = (await req.json()) as PlaygroundRequest;
    const apiKey = (body.apiKey || "").trim();
    const endpoint = (body.endpoint || "").trim();
    if (!apiKey.startsWith("fetchium_")) {
      return NextResponse.json(
        { error: "invalid_api_key", message: "A valid fetchium_ API key is required." },
        { status: 400 },
      );
    }
    if (!ALLOWED_ENDPOINTS.has(endpoint)) {
      return NextResponse.json(
        { error: "invalid_endpoint", message: "Endpoint is not allowed in dashboard playground." },
        { status: 400 },
      );
    }
    const apiBase = resolve_api_base(body.apiBase);
    const res = await fetch(`${apiBase}${endpoint}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify(body.payload ?? {}),
      cache: "no-store",
    });
    const responseText = await res.text();
    let parsed: unknown = responseText;
    try {
      parsed = JSON.parse(responseText);
    } catch {
      // Keep raw string when backend returns plain text.
    }
    return NextResponse.json({
      status: res.status,
      duration_ms: Date.now() - started,
      rate_limit: {
        limit: res.headers.get("x-ratelimit-limit"),
        remaining: res.headers.get("x-ratelimit-remaining"),
        reset: res.headers.get("x-ratelimit-reset"),
      },
      data: parsed,
    });
  } catch (err) {
    return NextResponse.json(
      {
        error: "playground_request_failed",
        message: err instanceof Error ? err.message : "unknown error",
      },
      { status: 500 },
    );
  }
}

