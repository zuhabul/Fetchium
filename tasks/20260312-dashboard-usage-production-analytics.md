# Dashboard Usage Production Analytics Task

> **Date:** 2026-03-12
> **Priority:** P1
> **Area:** `app.fetchium.com` usage page
> **Status:** `DONE`

## Summary

The Usage page is already backed by real backend data, but it is not yet a true production analytics surface.

Current state:
- the page fetches `/api/usage`
- `/api/usage` proxies `GET /v1/usage`
- `GET /v1/usage` returns per-key monthly/today usage totals plus quota information

That means the page is authoritative for quota summary, but limited in scope. It does not yet support production-grade usage analytics, operational debugging, or billing-grade usage visibility beyond coarse counters.

## Research Findings

### Frontend

Current Usage page:
- [apps/dashboard/src/app/(dashboard)/dashboard/usage/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/usage/page.tsx)

Observed behavior:
- loads live usage on mount
- supports manual refresh
- renders current plan, requests today, requests this month, tokens this month, monthly limit, and quota remaining
- computes quota-used percentage client-side from backend totals
- does not show:
  - trend over time
  - per-endpoint usage
  - latency
  - success/error breakdown
  - rate-limit status
  - last request activity

Important note:
- the page title says `Usage Analytics`
- the actual data is closer to a quota summary than analytics

### Dashboard proxy

Current proxy route:
- [apps/dashboard/src/app/api/usage/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/usage/route.ts)

Observed behavior:
- reads the authenticated dashboard session
- extracts the stored API key from the session token
- proxies directly to `GET /v1/usage`
- does not enrich or reshape data

### Backend usage model

Current auth/usage implementation:
- [crates/fetchium-api/src/auth.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/auth.rs)
- [crates/fetchium-api/src/handlers_auth.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/handlers_auth.rs)

Existing backend facts:
- `usage_logs` already stores:
  - `key_id`
  - `endpoint`
  - `status`
  - `tokens_used`
  - `duration_ms`
  - `created_at`
- `get_usage_stats()` currently returns only:
  - `requests_this_month`
  - `requests_today`
  - `tokens_this_month`
  - `monthly_limit`
  - `quota_remaining`
  - `plan`
  - `key_id`

This means the backend already records enough raw data to power richer usage analytics, but the current API only exposes a narrow aggregate summary.

### Rate limiting context

Relevant backend code:
- [crates/fetchium-api/src/middleware.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/middleware.rs)

Observed fact:
- per-minute rate limiting exists in middleware using `PlanLimits`
- the current Usage page does not expose plan rate-limit capacity or current rate-limit posture

## Problem Statement

The current Usage page is reliable for quota summary, but it is not sufficient for production customers who need to understand:

- where their traffic is going
- how usage is changing over time
- whether failures are increasing
- which endpoints consume the most tokens
- whether they are near plan thresholds beyond the monthly quota

As a result, the page is useful for checking consumption, but not for diagnosing usage patterns or making billing/plan decisions confidently.

## Production Requirements

The Usage page should become the production usage and quota surface for the authenticated key.

### Must-have data

1. Quota summary
- current plan
- requests today
- requests this month
- tokens this month
- monthly limit
- quota remaining

2. Usage trend
- daily request counts for last 7d, 14d, or 30d
- optionally daily token usage

3. Endpoint breakdown
- request count by endpoint
- token usage by endpoint
- optionally last seen timestamp

4. Request health summary
- success rate for a defined window
- error count for a defined window
- optional top failing endpoints

5. Rate limit visibility
- plan per-minute limit
- remaining headroom language or current policy description

### Nice-to-have data

- p50/p95 latency
- hourly usage for current day
- exportable CSV
- comparison to previous period
- projected month-end usage

## What The Page Can Trust Today

Today the page can trust:
- `GET /v1/usage` for monthly and daily counts
- `tokens_this_month`
- plan-derived monthly quota limits

Today it cannot trust or display because there is no endpoint:
- trends
- endpoint distribution
- rate-limit analytics
- recent usage history
- failure breakdown

## Proposed Backend Work

### 1. Keep `GET /v1/usage` as the lightweight summary endpoint

This route is already useful and should remain the fast summary source.

It should continue to answer:
- who is the current key
- what plan is active
- how much quota has been consumed

### 2. Add a richer analytics endpoint

Add an authenticated endpoint such as:
- `GET /v1/dashboard/usage`

Suggested response shape:

```json
{
  "meta": {
    "request_id": "req_123",
    "status": "ok",
    "endpoint": "/v1/dashboard/usage",
    "duration_ms": 11
  },
  "summary": {
    "key_id": "key_123",
    "plan": "pro",
    "requests_today": 42,
    "requests_this_month": 1033,
    "tokens_this_month": 843221,
    "monthly_limit": 250000,
    "quota_remaining": 248967,
    "requests_per_minute_limit": 500
  },
  "timeseries": [
    { "date": "2026-03-10", "requests": 120, "tokens": 22000 },
    { "date": "2026-03-11", "requests": 98, "tokens": 18500 }
  ],
  "endpoint_breakdown": [
    {
      "endpoint": "/v1/search",
      "requests": 611,
      "tokens_used": 421000,
      "error_count": 3
    }
  ],
  "health": {
    "success_rate_30d": 99.2,
    "error_count_30d": 8
  }
}
```

### 3. Add query helpers in `AuthDb`

Add focused read methods over `usage_logs`:
- `get_usage_timeseries(key_id, days)`
- `get_endpoint_usage_breakdown(key_id, days, limit)`
- `get_usage_health_summary(key_id, days)`
- `get_recent_usage_activity(key_id, limit)` if needed

These should use the existing `usage_logs` table and align with the authenticated key boundary.

### 4. Expose plan rate-limit metadata explicitly

`PlanLimits` already contains `requests_per_min`.

The dashboard should not have to infer this indirectly. Include it in the usage response so the UI can explain:
- monthly quota
- per-minute rate limit

## Proposed Dashboard Work

### 1. Split summary from analytics

The current page should either:
- remain a summary page until richer data exists, or
- be upgraded to consume a richer analytics endpoint

Recommended path:
- keep the current summary cards
- add trend and breakdown sections once backend support exists

### 2. Add a dashboard usage proxy route

Add:
- `apps/dashboard/src/app/api/dashboard/usage/route.ts`

This route should proxy the richer backend usage analytics endpoint and keep secrets off the client.

### 3. Improve labels

Until richer analytics exist, the page should avoid overstating itself as analytics if it only shows quota totals.

Recommended near-term copy:
- `Usage & Quota`

Recommended production copy once backend work lands:
- `Usage Analytics`

### 4. Add clear state handling

The page should distinguish:
- loading summary
- no traffic this month
- usage available but no breakdown data for selected window
- backend/API failure
- unauthorized session

## Implementation Plan

### Phase 1: Backend summary-plus analytics model

1. Keep `GET /v1/usage` unchanged for lightweight polling.
2. Add `GET /v1/dashboard/usage`.
3. Add `AuthDb` query helpers over `usage_logs`.
4. Include per-minute rate-limit data from `PlanLimits`.

### Phase 2: Dashboard integration

1. Add `/api/dashboard/usage` proxy route.
2. Update the Usage page to fetch richer analytics data.
3. Add trend and endpoint breakdown UI.
4. Keep the existing quota summary visible at the top.

### Phase 3: Hardening

1. Add handler tests for empty usage, mixed endpoints, and error-heavy periods.
2. Add UI states for no-traffic and partial-data cases.
3. Validate mobile layout with denser analytics content.

## Acceptance Criteria

- The Usage page continues to show correct live quota data for the authenticated key.
- The page clearly separates quota summary from richer analytics.
- Production users can see:
  - total requests and tokens
  - trend over time
  - endpoint usage distribution
  - request health summary
  - plan rate-limit information
- No usage data from other keys or admin-only systems is exposed.
- The page remains accurate after browser refresh and across devices because data comes from the backend.

## Risks and Decisions

### Open decisions

- whether this page is strictly per-key or eventually aggregated per organization
- whether to expose rate-limit remaining in real time or only the plan ceiling
- whether to keep `GET /v1/usage` minimal or expand it instead of adding a second endpoint

### Risks

- larger analytics queries may become expensive without additional indexes if usage grows significantly
- if organization-level billing is introduced later, per-key usage semantics may need expansion
- mixing quota, billing, and analytics concerns on one page can create UI bloat

## Non-Goals

- building a full billing console on this page
- exposing admin analytics or cross-tenant usage
- replacing the existing lightweight `/v1/usage` summary endpoint

## Recommended Next Task

Implement the production Usage page in this order:

1. backend: add `GET /v1/dashboard/usage` over `usage_logs`
2. dashboard: add `/api/dashboard/usage` proxy
3. UI: extend the page from quota summary to real usage analytics
4. tests: validate summary totals, trend windows, endpoint breakdowns, and empty/error states
