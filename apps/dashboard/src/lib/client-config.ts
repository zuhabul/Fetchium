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

export const DEFAULT_API_BASE = "http://localhost:3050";

export function loadDashboardConfig(): DashboardConfig {
  if (typeof window === "undefined") {
    return { apiBaseUrl: DEFAULT_API_BASE, apiKey: "" };
  }
  try {
    const raw = localStorage.getItem(CFG_KEY);
    if (!raw) return { apiBaseUrl: DEFAULT_API_BASE, apiKey: "" };
    const parsed = JSON.parse(raw) as Partial<DashboardConfig>;
    return {
      apiBaseUrl: (parsed.apiBaseUrl || DEFAULT_API_BASE).trim(),
      apiKey: (parsed.apiKey || "").trim(),
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
      apiBaseUrl: cfg.apiBaseUrl.trim() || DEFAULT_API_BASE,
      apiKey: cfg.apiKey.trim(),
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

