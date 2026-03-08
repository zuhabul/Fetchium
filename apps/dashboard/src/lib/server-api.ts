export const DEFAULT_API_BASE = "***REMOVED***";
const ADMIN_KEYS_ENABLED = process.env.***REMOVED*** === "true";

export function resolve_api_base(input?: string): string {
  const base = (input || process.env.***REMOVED*** || DEFAULT_API_BASE)
    .trim()
    .replace(/\/+$/, "");

  if (process.env.NODE_ENV === "production" && base !== DEFAULT_API_BASE) {
    throw new Error(`Hosted dashboard only supports ${DEFAULT_API_BASE}`);
  }

  return base;
}

export function admin_secret(): string {
  const secret = process.env.***REMOVED***;
  if (!secret) {
    throw new Error("***REMOVED*** is not set");
  }
  return secret;
}

export function assert_admin_keys_enabled(): void {
  if (process.env.NODE_ENV === "production" && !ADMIN_KEYS_ENABLED) {
    throw new Error("admin_key_management_disabled");
  }
}
