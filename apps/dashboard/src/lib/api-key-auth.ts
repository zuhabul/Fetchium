import { DEFAULT_API_BASE, resolve_api_base } from "@/lib/server-api";

type UsageEnvelope = {
  meta?: { request_id?: string };
  usage?: {
    key_id?: string;
    plan?: string;
    requests_this_month?: number;
    requests_today?: number;
    monthly_limit?: number | null;
  };
};

export type ApiKeyValidationResult =
  | {
      ok: true;
      apiBase: string;
      keyId: string;
      plan: string;
      usage: NonNullable<UsageEnvelope["usage"]>;
    }
  | {
      ok: false;
      apiBase: string;
      status: number;
      message: string;
    };

function extract_api_key(credentials: Record<string, unknown>): string {
  const candidates = [
    credentials.apiKey,
    credentials.api_key,
    credentials.key,
    credentials.token,
    credentials.password,
  ];

  for (const value of candidates) {
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }

  return "";
}

async function read_error_message(res: Response): Promise<string> {
  const contentType = res.headers.get("content-type") || "";
  if (contentType.includes("application/json")) {
    const body = (await res.json()) as { title?: string; message?: string; error?: string };
    return body.title || body.message || body.error || `API request failed with ${res.status}.`;
  }

  const text = (await res.text()).trim();
  return text || `API request failed with ${res.status}.`;
}

export async function validate_api_key(
  credentials: Record<string, unknown>,
): Promise<ApiKeyValidationResult> {
  const apiKey = extract_api_key(credentials);
  const apiBase = resolve_api_base();

  if (!apiKey.startsWith("fetchium_") || apiKey.length < 16) {
    return {
      ok: false,
      apiBase,
      status: 400,
      message: "API key must start with fetchium_ and include the full token.",
    };
  }

  let res: Response;
  try {
    res = await fetch(`${apiBase}/v1/usage`, {
      headers: {
        Authorization: `Bearer ${apiKey}`,
      },
      cache: "no-store",
    });
  } catch (error) {
    return {
      ok: false,
      apiBase,
      status: 503,
      message: error instanceof Error ? error.message : "Unable to reach the Fetchium API.",
    };
  }

  if (!res.ok) {
    return {
      ok: false,
      apiBase,
      status: res.status,
      message: await read_error_message(res),
    };
  }

  const body = (await res.json()) as UsageEnvelope;
  const usage = body.usage || {};
  const keyId = usage.key_id || `key_${apiKey.slice(-8)}`;
  const plan = usage.plan || "unknown";

  return {
    ok: true,
    apiBase: apiBase || DEFAULT_API_BASE,
    keyId,
    plan,
    usage,
  };
}
