"use client";

import Link from "next/link";
import { Copy, ExternalLink, Loader2, AlertTriangle, Play, Zap, Clock as ClockIcon } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import {
  DASHBOARD_CARD_PADDED,
  DASHBOARD_PAGE_HEADER,
  DASHBOARD_PAGE_LEAD,
  DASHBOARD_PAGE_STACK,
} from "@/lib/dashboard-layout";
import { dashboardEndpoints } from "@/lib/dashboard-catalog";
import type { DashboardEndpoint } from "@/lib/dashboard-catalog";

/* ─── Types ──────────────────────────────────────────────────────────── */

type RouteMetadata = {
  path: string;
  method: string;
  category: string;
  label: string;
  description: string;
  docs_href: string;
  auth_mode: string;
  dashboard_visible: boolean;
  playground_supported: boolean;
  async_variant: string | null;
  polling_route: string | null;
  sample_key: string;
};

type CatalogEntry = RouteMetadata & {
  sampleBody?: string;
  sampleCurl?: string;
};

const CATEGORY_ORDER: Record<string, number> = {
  core: 0,
  research: 1,
  media: 2,
  social: 3,
  jobs: 4,
  utility: 5,
};

const CATEGORY_LABELS: Record<string, string> = {
  core: "Core API",
  research: "Research",
  media: "Media",
  social: "Social",
  jobs: "Async Jobs",
  utility: "Utility",
};

/* ─── Page ───────────────────────────────────────────────────────────── */

export default function ApiCatalogPage() {
  const [entries, setEntries] = useState<CatalogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [filterCategory, setFilterCategory] = useState<string>("all");

  const loadRoutes = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/dashboard/routes", { cache: "no-store" });
      if (!res.ok) {
        // Fallback to local catalog
        setEntries(localFallback());
        setError("Using local catalog — backend route metadata unavailable.");
        return;
      }
      const body = await res.json();
      const routes: RouteMetadata[] = body.routes || [];

      // Merge with local UX data (sample payloads, curls)
      const merged = routes
        .filter((r) => r.dashboard_visible)
        .map((route): CatalogEntry => {
          const local = dashboardEndpoints.find((e) => e.path === route.path);
          return {
            ...route,
            sampleBody: local?.sampleBody,
            sampleCurl: local?.sampleCurl,
          };
        });
      setEntries(merged);
    } catch {
      setEntries(localFallback());
      setError("Using local catalog — backend route metadata unavailable.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadRoutes();
  }, [loadRoutes]);

  async function copy(text: string, key: string) {
    await navigator.clipboard.writeText(text);
    setCopied(key);
    window.setTimeout(() => setCopied(null), 1500);
  }

  const categories = Array.from(new Set(entries.map((e) => e.category))).sort(
    (a, b) => (CATEGORY_ORDER[a] ?? 99) - (CATEGORY_ORDER[b] ?? 99),
  );

  const filtered = filterCategory === "all" ? entries : entries.filter((e) => e.category === filterCategory);

  // Group by category
  const grouped = new Map<string, CatalogEntry[]>();
  for (const entry of filtered) {
    const list = grouped.get(entry.category) || [];
    list.push(entry);
    grouped.set(entry.category, list);
  }

  return (
    <div className={DASHBOARD_PAGE_STACK}>
      {/* Header */}
      <div className={DASHBOARD_PAGE_HEADER}>
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">API Catalog</h1>
          <p className={DASHBOARD_PAGE_LEAD}>
            Production route registry with capabilities, sample payloads, and docs links.
          </p>
        </div>

        <div className={`${DASHBOARD_CARD_PADDED} space-y-3 lg:max-w-md`}>
          <p className="text-sm leading-6 text-[var(--text-muted)]">
            Route metadata is sourced from the backend. Use this to choose the right endpoint before opening Playground.
          </p>
          <Link href="/dashboard/playground" className="inline-flex text-sm text-brand-400 hover:underline">
            Open Playground →
          </Link>
        </div>
      </div>

      {error && (
        <div className="flex items-center gap-2 rounded-xl border border-amber-500/20 bg-amber-500/5 p-3 text-sm text-amber-300">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {/* Category filter */}
      <div className="flex flex-wrap gap-2">
        <FilterChip label="All" active={filterCategory === "all"} onClick={() => setFilterCategory("all")} count={entries.length} />
        {categories.map((cat) => (
          <FilterChip
            key={cat}
            label={CATEGORY_LABELS[cat] || cat}
            active={filterCategory === cat}
            onClick={() => setFilterCategory(cat)}
            count={entries.filter((e) => e.category === cat).length}
          />
        ))}
      </div>

      {loading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-brand-400" />
        </div>
      )}

      {/* Grouped entries */}
      {!loading &&
        Array.from(grouped.entries())
          .sort(([a], [b]) => (CATEGORY_ORDER[a] ?? 99) - (CATEGORY_ORDER[b] ?? 99))
          .map(([category, categoryEntries]) => (
            <div key={category} className="space-y-4">
              <h2 className="text-lg font-semibold text-[var(--text-primary)]">
                {CATEGORY_LABELS[category] || category}
              </h2>
              <div className="grid gap-4">
                {categoryEntries.map((entry) => (
                  <EndpointCard key={entry.path} entry={entry} copied={copied} onCopy={copy} />
                ))}
              </div>
            </div>
          ))}
    </div>
  );
}

/* ─── Fallback ───────────────────────────────────────────────────────── */

function localFallback(): CatalogEntry[] {
  return dashboardEndpoints.map((e): CatalogEntry => ({
    path: e.path,
    method: e.method,
    category: e.category === "Core API" ? "core" : e.category === "Async Jobs" ? "jobs" : e.category.toLowerCase(),
    label: e.label,
    description: e.description,
    docs_href: e.docsHref,
    auth_mode: "bearer_key",
    dashboard_visible: true,
    playground_supported: e.playgroundSupported ?? false,
    async_variant: e.asyncVariant ?? null,
    polling_route: e.pollingRoute ?? null,
    sample_key: e.path.replace(/^\/v1\//, "").replace(/\//g, "_"),
    sampleBody: e.sampleBody,
    sampleCurl: e.sampleCurl,
  }));
}

/* ─── Sub-components ─────────────────────────────────────────────────── */

function FilterChip({
  label,
  active,
  onClick,
  count,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
  count: number;
}) {
  return (
    <button
      onClick={onClick}
      className={`inline-flex items-center gap-1.5 rounded-full border px-3 py-1.5 text-xs font-medium transition-colors ${
        active
          ? "border-brand-500/30 bg-brand-500/10 text-brand-300"
          : "border-[var(--border-subtle)] text-[var(--text-muted)] hover:border-[var(--brand-border)] hover:text-[var(--text-primary)]"
      }`}
    >
      {label}
      <span className="rounded-full bg-[var(--surface-sunken)] px-1.5 py-0.5 text-[10px]">{count}</span>
    </button>
  );
}

function EndpointCard({
  entry,
  copied,
  onCopy,
}: {
  entry: CatalogEntry;
  copied: string | null;
  onCopy: (text: string, key: string) => Promise<void>;
}) {
  const [expanded, setExpanded] = useState(false);

  return (
    <section className="overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-base)]">
      {/* Header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-start gap-4 px-4 py-4 text-left transition-colors hover:bg-[var(--surface-hover)] sm:px-5"
      >
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className={`rounded-full border px-2.5 py-0.5 text-xs font-medium ${
              entry.method === "GET"
                ? "border-emerald-500/20 bg-emerald-500/10 text-emerald-300"
                : "border-brand-500/20 bg-brand-500/10 text-brand-300"
            }`}>
              {entry.method}
            </span>
            <span className="font-mono text-sm font-medium text-[var(--text-primary)]">{entry.path}</span>
          </div>
          <p className="mt-2 text-sm text-[var(--text-muted)]">{entry.description}</p>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {entry.playground_supported && <Badge label="Playground" color="bg-brand-500/10 text-brand-300" icon={Play} />}
            {entry.async_variant && <Badge label="Async" color="bg-purple-500/10 text-purple-300" icon={ClockIcon} />}
            {entry.polling_route && <Badge label="Pollable" color="bg-amber-500/10 text-amber-300" icon={Zap} />}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Link
            href={entry.docs_href}
            target="_blank"
            rel="noopener noreferrer"
            onClick={(e) => e.stopPropagation()}
            className="inline-flex items-center gap-1 rounded-lg border border-[var(--border-subtle)] px-3 py-2 text-xs text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
          >
            Docs <ExternalLink className="h-3 w-3" />
          </Link>
          <span className="text-xs text-[var(--text-faint)]">{expanded ? "▲" : "▼"}</span>
        </div>
      </button>

      {/* Expanded details */}
      {expanded && (
        <div className="border-t border-[var(--border-subtle)]">
          <div className="grid gap-4 p-4 sm:p-5 xl:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
            <div className="space-y-3">
              <div className="text-xs uppercase tracking-[0.18em] text-[var(--text-faint)]">Sample payload</div>
              {entry.sampleBody ? (
                <pre className="overflow-x-auto rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-xs leading-relaxed text-[var(--text-secondary)]">
                  {entry.sampleBody}
                </pre>
              ) : (
                <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-sm text-[var(--text-muted)]">
                  No request body required.
                </div>
              )}
              {entry.async_variant && (
                <div className="rounded-lg border border-purple-500/10 bg-purple-500/5 p-3">
                  <p className="text-xs text-purple-300">
                    Async variant: <span className="font-mono">{entry.async_variant}</span>
                  </p>
                  {entry.polling_route && (
                    <p className="mt-1 text-xs text-purple-300">
                      Poll with: <span className="font-mono">{entry.polling_route}</span>
                    </p>
                  )}
                </div>
              )}
            </div>
            <div className="space-y-3">
              {entry.sampleCurl && (
                <>
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-xs uppercase tracking-[0.18em] text-[var(--text-faint)]">Curl snippet</div>
                    <button
                      onClick={() => void onCopy(entry.sampleCurl!, entry.path)}
                      className="inline-flex items-center gap-1 rounded-lg border border-[var(--border-subtle)] px-3 py-1.5 text-xs text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]"
                    >
                      <Copy className="h-3.5 w-3.5" />
                      {copied === entry.path ? "Copied" : "Copy"}
                    </button>
                  </div>
                  <pre className="overflow-x-auto rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-raised)] p-4 text-xs leading-relaxed text-[var(--text-secondary)]">
                    {entry.sampleCurl}
                  </pre>
                </>
              )}
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

function Badge({ label, color, icon: Icon }: { label: string; color: string; icon: typeof Play }) {
  return (
    <span className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium ${color}`}>
      <Icon className="h-2.5 w-2.5" />
      {label}
    </span>
  );
}
