"use client";

export type DashboardConfig = {
  apiBaseUrl: string;
  apiKey: string;
};

export type RequestLog = {
  endpoint: string;
  status: number;
  latencyMs: number;
  timeIso: string;
};

const CFG_KEY = "fetchium_dashboard_config_v1";
const LOG_KEY = "fetchium_dashboard_logs_v1";

export const DEFAULT_API_BASE = "***REMOVED***";

export function normalize_api_base(input: string): string {
  return input.trim().replace(/\/+$/, "");
}

export function normalize_api_key(input: string): string {
  return input.trim();
}

function is_local_host(hostname: string): boolean {
  return (
    hostname === "localhost" ||
    hostname === "127.0.0.1" ||
    hostname === "[::1]"
  );
}

export function validate_api_base(input: string): string | null {
  const value = normalize_api_base(input);
  if (!value) return "API base URL is required.";

  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return "Enter a valid API base URL.";
  }

  if (!["http:", "https:"].includes(url.protocol)) {
    return "API base URL must use http or https.";
  }

  if (typeof window !== "undefined" && !is_local_host(window.location.hostname)) {
    if (url.protocol !== "https:") {
      return "Production settings must use an https API base URL.";
    }
    if (is_local_host(url.hostname)) {
      return "Localhost API bases are only allowed in local development.";
    }
  }

  return null;
}

export function validate_api_key(input: string): string | null {
  const value = normalize_api_key(input);
  if (!value) return "API key is required.";
  if (!value.startsWith("fetchium_")) {
    return "API key must start with fetchium_.";
  }
  if (value.length < 16) {
    return "API key looks too short.";
  }
  return null;
}

export function loadDashboardConfig(): DashboardConfig {
  if (typeof window === "undefined") {
    return { apiBaseUrl: DEFAULT_API_BASE, apiKey: "" };
  }
  try {
    const raw = localStorage.getItem(CFG_KEY);
    if (!raw) return { apiBaseUrl: DEFAULT_API_BASE, apiKey: "" };
    const parsed = JSON.parse(raw) as Partial<DashboardConfig>;
    return {
      apiBaseUrl: normalize_api_base(parsed.apiBaseUrl || DEFAULT_API_BASE),
      apiKey: normalize_api_key(parsed.apiKey || ""),
    };
  } catch {
    return { apiBaseUrl: DEFAULT_API_BASE, apiKey: "" };
  }
}

export function saveDashboardConfig(cfg: DashboardConfig): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(
    CFG_KEY,
    JSON.stringify({
      apiBaseUrl: normalize_api_base(cfg.apiBaseUrl) || DEFAULT_API_BASE,
      apiKey: normalize_api_key(cfg.apiKey),
    }),
  );
}

export function loadRequestLogs(): RequestLog[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(LOG_KEY);
    if (!raw) return [];
    const logs = JSON.parse(raw) as RequestLog[];
    return Array.isArray(logs) ? logs : [];
  } catch {
    return [];
  }
}

export function appendRequestLog(log: RequestLog): void {
  if (typeof window === "undefined") return;
  const logs = loadRequestLogs();
  logs.unshift(log);
  localStorage.setItem(LOG_KEY, JSON.stringify(logs.slice(0, 100)));
}
