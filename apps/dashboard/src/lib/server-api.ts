export const DEFAULT_API_BASE = "http://localhost:3050";

export function resolve_api_base(input?: string): string {
  const base = (input || process.env.FETCHIUM_API_BASE_URL || DEFAULT_API_BASE).trim();
  return base.replace(/\/+$/, "");
}

export function admin_secret(): string {
  const secret = process.env.FETCHIUM_ADMIN_SECRET;
  if (!secret) {
    throw new Error("FETCHIUM_ADMIN_SECRET is not set");
  }
  return secret;
}

