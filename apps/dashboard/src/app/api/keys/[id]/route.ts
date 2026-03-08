import { NextResponse } from "next/server";
import { admin_secret, assert_admin_keys_enabled, resolve_api_base } from "@/lib/server-api";

export const runtime = "nodejs";

export async function DELETE(
  _req: Request,
  context: { params: Promise<{ id: string }> },
) {
  try {
    assert_admin_keys_enabled();
    const { id } = await context.params;
    const apiBase = resolve_api_base();
    const res = await fetch(`${apiBase}/v1/keys/${encodeURIComponent(id)}`, {
      method: "DELETE",
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
      { error: "key_revoke_failed", message: err instanceof Error ? err.message : "unknown error" },
      { status: 500 },
    );
  }
}
