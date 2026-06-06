# Dashboard Quickstart Production Flow Task

> **Date:** 2026-03-12
> **Priority:** P1
> **Area:** `app.fetchium.com` dashboard quickstart
> **Status:** `DONE`

## Summary

The Quickstart page is currently a polished onboarding page, but it is mostly static product guidance rather than a production-backed first-success workflow.

Current state:
- session context is loaded from the authenticated dashboard session
- endpoint suggestions come from a static local catalog
- curl snippets are static template strings
- recommended steps are static copy

This is useful for orientation, but it does not yet verify that a user has actually completed a successful first request, which endpoint they should use next, or whether their current environment is ready beyond basic session presence.

## Research Findings

### Frontend

Current Quickstart implementation:
- [apps/dashboard/src/app/(dashboard)/dashboard/quickstart/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/quickstart/page.tsx)

Observed behavior:
- fetches `/api/auth/session` on mount
- displays `plan`, `keyId`, and `apiKeyPreview`
- renders static “Recommended path” links
- renders static step cards
- renders static sample curl snippets from the local endpoint catalog
- does not show whether the user has completed a first successful request
- does not check API health, quota viability, or backend connectivity on this page

Current endpoint catalog:
- [apps/dashboard/src/lib/dashboard-catalog.ts](/home/echo/projects/Fetchium/apps/dashboard/src/lib/dashboard-catalog.ts)

Observed behavior:
- stores hardcoded endpoint metadata
- stores hardcoded docs links
- stores hardcoded sample request bodies
- stores hardcoded curl snippets with `fetchium_YOUR_KEY`

This is acceptable for docs-style guidance, but not sufficient as a production onboarding state machine.

### Session/auth model

Dashboard auth configuration:
- [apps/dashboard/src/auth.ts](/home/echo/projects/Fetchium/apps/dashboard/src/auth.ts)

Key findings:
- sign-in is API-key based through Auth.js credentials
- session data includes `apiBase`, `plan`, `keyId`, and `apiKeyPreview`
- the full API key is retained in the JWT token server-side for dashboard proxy actions
- Quickstart is currently reading the default Auth.js session endpoint, not a custom quickstart-specific backend route

This means the page knows who the authenticated key is, but it does not yet fetch any backend onboarding state.

### Current backend capabilities

Existing validation path:
- [apps/dashboard/src/lib/api-key-auth.ts](/home/echo/projects/Fetchium/apps/dashboard/src/lib/api-key-auth.ts)

Existing dashboard routes:
- [apps/dashboard/src/app/api/usage/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/usage/route.ts)
- [apps/dashboard/src/app/api/playground/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/playground/route.ts)
- [apps/dashboard/src/app/api/health/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/health/route.ts)

Existing Fetchium API facts:
- `GET /v1/usage` validates key/session and exposes quota state
- dashboard playground can execute approved POST endpoints through the authenticated session
- backend `usage_logs` tracks real request activity by key

There is enough existing infrastructure to make Quickstart production-aware without inventing a separate onboarding backend from scratch.

## Problem Statement

The Quickstart page currently tells users what to do, but it does not reliably tell them:

- whether their hosted session is healthy end-to-end
- whether they have already made a successful first API request
- which first action is blocked right now
- whether the shown snippets and next steps match their real account state

In production, a quickstart surface should reduce time-to-first-success and remove ambiguity. Right now it is closer to a static guide than an operational onboarding page.

## Production Requirements

The Quickstart page should become a first-success workflow for the authenticated dashboard session.

### Must-have behavior

1. Session and connectivity validation
- show authenticated session state
- verify dashboard -> API connectivity
- verify current key is valid
- surface failures clearly

2. First-request completion state
- detect whether the authenticated key has made at least one successful request
- distinguish:
  - no traffic yet
  - first request succeeded
  - requests exist but failing

3. Actionable next step logic
- if no requests yet: prioritize “Send first request”
- if first request succeeded: prioritize deeper endpoints
- if quota/connectivity issue exists: prioritize fixing that state

4. Production-safe snippets
- keep default snippets copyable
- explain that dashboard proxy actions use the authenticated session
- avoid exposing the full key back into the UI

5. Accurate empty/error/loading states
- loading session
- session missing/expired
- API unreachable
- key valid but no usage yet

### Nice-to-have behavior

- mark first request completion with timestamp and endpoint
- show “last successful request” summary
- personalize recommended next endpoints based on actual usage
- add one-click “Run starter request” through the dashboard proxy

## Proposed Backend Work

### Option A: Lightweight quickstart status endpoint

Add an authenticated endpoint such as:
- `GET /v1/dashboard/quickstart`

Suggested payload:

```json
{
  "meta": {
    "request_id": "req_123",
    "status": "ok",
    "endpoint": "/v1/dashboard/quickstart",
    "duration_ms": 9
  },
  "session": {
    "plan": "pro",
    "key_id": "key_123"
  },
  "connectivity": {
    "api_reachable": true,
    "usage_check_ok": true
  },
  "first_success": {
    "has_successful_request": true,
    "first_success_at": "2026-03-12T10:31:00Z",
    "first_success_endpoint": "/v1/search"
  },
  "recent_activity": {
    "request_count_7d": 42,
    "last_request_at": "2026-03-12T10:31:00Z",
    "last_request_status": 200
  },
  "recommended_next_steps": [
    "playground_search",
    "usage_check",
    "try_fetch"
  ]
}
```

This can be derived from:
- current key/session
- `GET /v1/usage`
- `usage_logs` for the authenticated key

### Option B: Reuse a future overview endpoint

If `GET /v1/dashboard/overview` is implemented first, Quickstart can consume a narrower derived route from that same backend data.

Recommended approach:
- keep Quickstart-specific UI logic in the dashboard
- keep raw telemetry query logic in the API backend

## Proposed Dashboard Work

### 1. Add a dashboard quickstart proxy route

Add:
- `apps/dashboard/src/app/api/dashboard/quickstart/route.ts`

This route should:
- resolve the authenticated session
- proxy to the backend quickstart-status endpoint
- return minimal JSON tailored to the Quickstart UI

### 2. Replace static success assumptions

Update [quickstart/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/quickstart/page.tsx):
- do not always imply “Session active” is sufficient
- load real quickstart status
- show blocked/success/in-progress states
- compute recommended path from live state, not only static copy

### 3. Keep static catalog content, but treat it as secondary

The catalog in [dashboard-catalog.ts](/home/echo/projects/Fetchium/apps/dashboard/src/lib/dashboard-catalog.ts) should remain the source for:
- endpoint labels
- docs links
- sample request bodies

But it should no longer be the only source of onboarding state.

### 4. Consider a “Run starter request” action

The most effective Quickstart improvement would be a guided starter action:
- preselect `/v1/search`
- send a safe example payload via `/api/playground`
- show response success inline
- then unlock the next recommendations

This would make Quickstart materially different from the Playground while still using the same secure proxy path.

## Implementation Plan

### Phase 1: Backend/session read model

1. Add backend quickstart status query using existing session key and `usage_logs`.
2. Return:
   - current plan/key metadata
   - connectivity status
   - first successful request summary
   - recent activity summary

### Phase 2: Dashboard integration

1. Add `/api/dashboard/quickstart` proxy route.
2. Update Quickstart page to fetch it.
3. Replace static success assumptions with live status states.
4. Keep snippets/catalog as documentation support.

### Phase 3: Guided activation

1. Add a one-click starter request using the existing playground proxy.
2. Persist result in UI state.
3. Refresh quickstart status after successful run.
4. Promote the correct next step based on real usage.

## Acceptance Criteria

- Quickstart no longer relies only on static copy to imply readiness.
- The page clearly shows whether the authenticated key has completed a successful first request.
- The page distinguishes:
  - session present
  - backend reachable
  - no traffic yet
  - first success achieved
  - recent failures/blockers
- Recommended next actions are based on live state.
- Copyable snippets remain available without re-exposing the full key.
- Quickstart remains useful on first login and after repeated production use.

## Risks and Decisions

### Open decisions

- whether “quickstart complete” means any successful request or specifically `/v1/search`
- whether to add a dedicated quickstart endpoint or derive from a shared overview endpoint
- whether to include a live inline request runner on this page

### Risks

- if Quickstart and Overview both query overlapping telemetry independently, duplication can creep in
- showing too much operational state can make the page feel like a second Overview
- one-click starter requests need guardrails to avoid surprising write-like behavior if future endpoints are added

## Non-Goals

- turning Quickstart into a full API explorer
- replacing the endpoint catalog or docs site
- exposing raw credentials in the browser UI

## Recommended Next Task

Implement Quickstart as a production-backed first-success workflow in this order:

1. backend: add quickstart status read model from session + `usage_logs`
2. dashboard: add `/api/dashboard/quickstart` proxy
3. UI: replace static status assumptions with live onboarding states
4. optional UX win: add one-click starter request through the existing playground proxy
