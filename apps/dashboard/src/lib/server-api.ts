export const DEFAULT_API_BASE = "***REMOVED***";

export function resolve_api_base(input?: string): string {
  const base = (input || process.env.***REMOVED*** || DEFAULT_API_BASE).trim();
  return base.replace(/\/+$/, "");
}

export function admin_secret(): string {
  const secret = process.env.***REMOVED***;
  if (!secret) {
    throw new Error("***REMOVED*** is not set");
  }
  return secret;
}
