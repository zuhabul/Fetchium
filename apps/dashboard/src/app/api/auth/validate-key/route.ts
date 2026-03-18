import { NextRequest, NextResponse } from "next/server";
import { validate_api_key } from "@/lib/api-key-auth";

export const runtime = "nodejs";

export async function POST(req: NextRequest) {
  try {
    const body = (await req.json()) as { apiKey?: string };
    const result = await validate_api_key({ apiKey: body.apiKey || "" });

    if (!result.ok) {
      return NextResponse.json(
        {
          ok: false,
          status: result.status,
          message: result.message,
        },
        { status: result.status === 400 ? 400 : 401 },
      );
    }

    return NextResponse.json({
      ok: true,
      keyId: result.keyId,
      plan: result.plan,
    });
  } catch (error) {
    return NextResponse.json(
      {
        ok: false,
        status: 500,
        message: error instanceof Error ? error.message : "unknown error",
      },
      { status: 500 },
    );
  }
}
