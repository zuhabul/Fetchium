export const DEFAULT_API_BASE = "https://api.fetchium.com";
const ADMIN_KEYS_ENABLED = process.env.FETCHIUM_DASHBOARD_ENABLE_ADMIN_KEYS === "true";

export function resolve_api_base(input?: string): string {
  const base = (input || process.env.FETCHIUM_API_BASE_URL || DEFAULT_API_BASE)
    .trim()
    .replace(/\/+$/, "");

  if (process.env.NODE_ENV === "production" && base !== DEFAULT_API_BASE) {
    throw new Error(`Hosted dashboard only supports ${DEFAULT_API_BASE}`);
  }

  return base;
}

export function admin_secret(): string {
  const secret = process.env.FETCHIUM_ADMIN_SECRET;
  if (!secret) {
    throw new Error("FETCHIUM_ADMIN_SECRET is not set");
  }
  return secret;
}

export function assert_admin_keys_enabled(): void {
  if (process.env.NODE_ENV === "production" && !ADMIN_KEYS_ENABLED) {
    throw new Error("admin_key_management_disabled");
  }
}
