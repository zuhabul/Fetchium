# Dashboard Overview Production Data

> **Date:** 2026-03-12
> **Priority:** P1
> **Status:** `DONE`
> **Area:** `app.fetchium.com` overview
> **Primary surfaces:** dashboard overview KPIs, recent requests, endpoint snapshot

## Objective

Replace the Overview page's mixed real/local telemetry with a fully backend-backed customer overview so the page is trustworthy in production.

This task is complete only when:
- Overview telemetry is sourced from backend truth for the authenticated key
- browser-local request logs are no longer used for Overview KPIs or recent requests
- the page can explain quota, request health, and recent activity using server data
- loading, empty, and failure states are explicit and correct

## Current State

### Dashboard UI

Current page:
- [apps/dashboard/src/app/(dashboard)/dashboard/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/page.tsx)

Observed behavior:
- loads `/api/usage`
- loads `loadRequestLogs()` from browser local storage
- computes:
  - `Avg Latency` from local logs
  - `Success Rate` from local logs
  - `Recent requests` from local logs
- shows:
  - static onboarding cards
  - static suggested endpoints

Current Overview page therefore mixes:
- real backend quota data
- browser-local playground history
- static product guidance

### Dashboard Proxy

Current proxy route:
- [apps/dashboard/src/app/api/usage/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/usage/route.ts)

Observed behavior:
- validates dashboard session
- extracts API key from session token
- proxies to `GET /v1/usage`
- exposes only coarse usage totals

There is no current dashboard API route for overview-specific telemetry.

### Backend Usage Data

Current auth DB and usage model:
- [crates/fetchium-api/src/auth.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/auth.rs)
- [crates/fetchium-api/src/handlers_auth.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/handlers_auth.rs)

Existing facts:
- `usage_logs` already stores:
  - `key_id`
  - `endpoint`
  - `status`
  - `tokens_used`
  - `duration_ms`
  - `created_at`
- `GET /v1/usage` returns:
  - `requests_today`
  - `requests_this_month`
  - `tokens_this_month`
  - `plan`
  - `monthly_limit`
  - `quota_remaining`

The raw data exists to power a true Overview page.

### Local Browser Data

Current local log helper:
- [apps/dashboard/src/lib/client-config.ts](/home/echo/projects/Fetchium/apps/dashboard/src/lib/client-config.ts)

Observed limitation:
- local logs only reflect requests sent through that browser
- requests from SDKs, curl, CI, or another device never appear
- clearing browser storage erases Overview telemetry

### Admin Analytics

Admin analytics exist under:
- [crates/fetchium-api/src/admin/usage.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/admin/usage.rs)

These are not valid sources for customer Overview because they are:
- admin-only
- derived from `audit_events`
- cross-org/internal in intent

## Findings

### 1. The page is partially authoritative

Quota-related values are real.

Health and recent activity values are not authoritative.

### 2. The misleading part is not obvious to users

The page visually presents all metrics as if they are equally trustworthy.

### 3. The backend already has sufficient raw telemetry

This is not blocked by missing raw events. It is blocked by the absence of a customer overview read model.

### 4. Static guidance should remain, but separate from telemetry

The onboarding cards are useful product guidance. They just should not be mixed conceptually with live telemetry.

## Problem Statement

The Overview page currently looks like a production dashboard but is only partially backed by production data.

User risk:
- latency and success metrics can be wrong
- recent requests can appear missing when traffic happened outside the browser
- debugging and support become confusing because page state does not match backend truth

Product risk:
- trust in the dashboard degrades
- new users may misread empty local logs as “no API traffic”
- cross-device consistency is broken

## Decision

Build a dedicated customer overview read model over `usage_logs` and use it as the sole source of Overview telemetry.

Keep static onboarding content in the page, but treat it as guidance, not telemetry.

Do not use browser-local request logs as a source of truth for Overview.

## Scope

### In scope

- Overview KPI cards
- recent requests list
- endpoint usage snapshot
- usage trend data
- customer-safe overview API response
- dashboard proxy route
- Overview loading/error/empty states

### Out of scope

- replacing static onboarding cards with CMS or personalization
- admin analytics reuse
- multi-key or org-wide rollups unless explicitly added later
- full analytics product with charts for every dimension

## Production Requirements

### Functional requirements

1. All Overview telemetry must come from backend data for the authenticated key.
2. The page must show:
   - requests today
   - requests this month
   - monthly limit
   - remaining quota
   - latency summary for a defined window
   - success rate for a defined window
   - recent backend requests
   - top endpoints or endpoint usage snapshot
3. The page must clearly label time windows for non-quota metrics.
4. The page must distinguish telemetry from static onboarding guidance.
5. The page must support no-traffic, loading, unauthorized, and backend-failure states.

### Data requirements

The Overview read model must include:
- `plan`
- `requests_today`
- `requests_this_month`
- `monthly_limit`
- `quota_remaining`
- `tokens_this_month`
- `avg_latency_ms_<window>`
- `success_rate_<window>`
- `recent_requests`
- `top_endpoints`
- `timeseries`

### Non-functional requirements

- no raw API key exposure
- no cross-key leakage
- query cost bounded by explicit time windows and limits
- overview payload fast enough for dashboard load

## Target Product Model

### Scope of truth

Overview should be per authenticated API key for now.

Reason:
- existing auth and usage model is per key
- `usage_logs` are keyed by `key_id`
- current dashboard session is key-centered

If org-level billing/usage is added later, Overview semantics can expand, but not in this task.

### Time window policy

Recommended defaults:
- quota cards: current day/current month
- latency and success: trailing 7 days
- recent requests: latest 20 requests
- trend: trailing 14 days
- top endpoints: trailing 30 days or current month

These windows must be explicit in labels or supporting text.

## Source-of-Truth Model

### Backend owns

- request activity truth
- latency truth
- success/failure truth
- request counts and usage totals

### Dashboard owns

- rendering and grouping
- loading/empty state copy
- static onboarding cards

### Browser local storage

May remain for Playground convenience only.

It must not drive Overview telemetry.

## Proposed Architecture

### Customer-facing endpoint

Add:
- `GET /v1/dashboard/overview`

This endpoint should:
- authenticate using the same customer key boundary as `GET /v1/usage`
- read `usage_logs` for the authenticated key
- return a compact overview response optimized for dashboard load

### Suggested response

```json
{
  "meta": {
    "request_id": "req_123",
    "status": "ok",
    "endpoint": "/v1/dashboard/overview",
    "duration_ms": 12
  },
  "summary": {
    "key_id": "key_123",
    "plan": "pro",
    "requests_today": 42,
    "requests_this_month": 1033,
    "tokens_this_month": 843221,
    "monthly_limit": 250000,
    "quota_remaining": 248967,
    "avg_latency_ms_7d": 182,
    "success_rate_7d": 99.2
  },
  "timeseries": [
    { "date": "2026-03-01", "requests": 120 },
    { "date": "2026-03-02", "requests": 98 }
  ],
  "top_endpoints": [
    {
      "endpoint": "/v1/search",
      "requests": 611,
      "last_seen_at": "2026-03-12T10:31:00Z"
    }
  ],
  "recent_requests": [
    {
      "endpoint": "/v1/search",
      "status": 200,
      "duration_ms": 143,
      "tokens_used": 1200,
      "created_at": "2026-03-12T10:31:00Z"
    }
  ]
}
```

## Required Backend Changes

### 1. Add overview response types

Create typed serializable structs for:
- overview summary
- recent request item
- endpoint summary item
- trend point

Suggested location:
- `crates/fetchium-api/src/types.rs`
- or a dedicated `dashboard_types.rs`

### 2. Add `AuthDb` query helpers

Add read helpers over `usage_logs`:
- `get_recent_requests(key_id, limit)`
- `get_overview_timeseries(key_id, days)`
- `get_top_endpoints(key_id, days, limit)`
- `get_latency_success_summary(key_id, days)`

Recommended query behavior:
- explicit day windows
- explicit row limits
- ordered descending for recent requests
- no full-table unbounded scans

### 3. Add overview handler

Add:
- `GET /v1/dashboard/overview`

The handler should:
- use the authenticated key extractor
- call the `AuthDb` helpers
- reuse `get_usage_stats()` for quota summary where possible
- return a single compact dashboard response

### 4. Add indexing review

Current index:
- `idx_usage_key_date ON usage_logs(key_id, created_at)`

Review whether additional indexes are needed once overview queries are added, especially for:
- `key_id, endpoint, created_at`
- `key_id, status, created_at`

If current index is sufficient for near-term volumes, document that explicitly and defer extra indexes.

## Required Dashboard Changes

### 1. Add overview proxy route

Create:
- `apps/dashboard/src/app/api/dashboard/overview/route.ts`

Behavior:
- authenticate dashboard session
- proxy the session key to `GET /v1/dashboard/overview`
- return JSON directly to the client

### 2. Refactor Overview page

Update:
- [apps/dashboard/src/app/(dashboard)/dashboard/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/page.tsx)

Required behavior:
- stop calling `loadRequestLogs()` for Overview metrics
- fetch `/api/dashboard/overview`
- render backend recent requests
- render backend latency and success metrics
- preserve onboarding cards as static guidance

### 3. Separate telemetry from guidance visually

Recommended structure:
- section 1: live telemetry
- section 2: quota
- section 3: recent requests
- section 4: product guidance / next actions

## UX Requirements

The page should clearly distinguish:
- telemetry cards
- recent activity
- quota context
- next-step guidance

Required UI states:
- loading
- no traffic yet
- telemetry unavailable
- unauthorized session

Recommended copy change:
- remove phrasing that implies local playground logs are the source

## Rollout Plan

### Phase 1: Backend overview model

1. Add response types.
2. Add `AuthDb` query helpers.
3. Add `GET /v1/dashboard/overview`.
4. Add backend tests.

### Phase 2: Dashboard integration

1. Add `/api/dashboard/overview`.
2. Refactor Overview page to consume it.
3. Remove local Overview telemetry dependency.

### Phase 3: Hardening

1. Add explicit loading/error/empty states.
2. Verify mobile layout with real payload sizes.
3. Validate cross-device consistency.

## Test Plan

### Backend tests

- empty key with no usage
- mixed success and error requests
- multiple endpoints
- timeseries generation for a fixed window
- recent request ordering

### Dashboard tests

- overview page renders backend KPIs
- recent requests come from backend response
- local storage clearing does not affect Overview data
- loading and error states render correctly

### Regression tests

- `/api/usage` remains unchanged for pages that still use it
- Playground local logs remain usable for Playground-only UX

## Acceptance Criteria

- `Avg Latency` and `Success Rate` are no longer computed from browser local storage.
- `Recent requests` is sourced from backend request history.
- Overview is consistent across browsers and devices for the same authenticated key.
- Static onboarding cards remain visible but are clearly not live telemetry.
- The page remains functional when browser local storage is empty.
- No admin/internal analytics data is exposed.

## Risks

- new overview queries may require additional indexing at higher usage volumes
- if the product later shifts from key-based to org-based dashboards, overview semantics will need redesign
- overloading the Overview payload with too much analytics will slow page load

## Open Decisions

- whether to include p95 latency now or defer to Usage analytics
- whether trend should be 14 days or 30 days by default
- whether tokens should appear on Overview or stay secondary

## Recommended Implementation Order

1. backend overview read model
2. dashboard overview proxy
3. page refactor away from local logs
4. test coverage and query/index review
