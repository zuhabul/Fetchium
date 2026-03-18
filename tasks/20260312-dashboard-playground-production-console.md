# Dashboard Playground Production Console Task

> **Date:** 2026-03-12
> **Priority:** P1
> **Area:** `app.fetchium.com` playground page
> **Status:** `DONE`

## Summary

The Playground page is already a real authenticated request console, but it is still a limited MVP rather than a production-grade API console.

Current state:
- sends live requests through the authenticated dashboard proxy
- supports a selected set of POST endpoints
- allows editing JSON payloads
- renders raw JSON response
- exposes direct curl snippets

This is useful and real, but it is incomplete for production use. It does not yet cover async job workflows, richer response metadata, structured error handling, request history from the backend, or a single source of truth for endpoint capability.

## Research Findings

### Frontend

Current Playground implementation:
- [apps/dashboard/src/app/(dashboard)/dashboard/playground/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/playground/page.tsx)

Observed behavior:
- endpoint list comes from `playgroundEndpoints` derived from a local catalog
- request body is edited as raw JSON text
- request is always sent to `/api/playground`
- response is rendered as raw pretty-printed JSON
- request history is written only to browser local storage using `appendRequestLog()`
- the page does not show:
  - request ID separately
  - response headers in a readable form
  - job polling support
  - endpoint-specific form inputs
  - saved snippets or history across devices
  - validation of request schemas before send

### Dashboard proxy

Current playground proxy:
- [apps/dashboard/src/app/api/playground/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/playground/route.ts)

Observed behavior:
- uses dashboard session token to retrieve the stored API key
- allows only a hardcoded set of endpoints
- always sends `POST`
- forwards JSON payload
- returns:
  - backend status
  - measured duration
  - rate-limit headers
  - parsed response body

Important current limitations:
- no support for `GET` requests such as `/v1/jobs/:id`
- no support for async workflow helpers
- no explicit propagation of backend request ID into a top-level dashboard field
- no capability metadata beyond the hardcoded allowlist

### Endpoint catalog

Current shared catalog:
- [apps/dashboard/src/lib/dashboard-catalog.ts](/home/echo/projects/Fetchium/apps/dashboard/src/lib/dashboard-catalog.ts)

Observed facts:
- `playgroundEndpoints` is derived from `dashboardEndpoints.filter(method === "POST")`
- this omits supported playground endpoints already allowlisted in the proxy but not present in the catalog
- examples:
  - `/v1/scrape`
  - `/v1/social/hackernews`
  - `/v1/social/research`

This creates drift between:
- what the UI lets the user select
- what the proxy actually supports

### Backend route and response capabilities

Relevant backend files:
- [crates/fetchium-api/src/routes.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/routes.rs)
- [crates/fetchium-api/src/types.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/types.rs)
- [crates/fetchium-api/src/handlers.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/handlers.rs)

Observed facts:
- the backend supports both synchronous and async job-based routes
- job submission routes exist for:
  - `/v1/research/jobs`
  - `/v1/youtube/search/jobs`
  - `/v1/youtube/analyze/jobs`
  - `/v1/social/research/jobs`
  - `/v1/social/reddit/jobs`
  - `/v1/social/hackernews/jobs`
- job polling exists at:
  - `GET /v1/jobs/:id`
- responses include `meta.request_id`, endpoint, and duration information
- request IDs are already part of the backend response model

The current dashboard Playground does not expose these backend capabilities well.

## Problem Statement

The Playground works as a basic authenticated proxy, but it is not yet a production-grade developer console.

Current user-facing gaps:
- incomplete endpoint coverage
- no async job support
- no authoritative endpoint capability model
- local-only history
- limited response introspection
- raw JSON editing for every endpoint with no schema guidance

This makes the page usable for simple POST testing, but weak for serious production debugging and API exploration.

## What The Playground Does Correctly Today

The current implementation gets several important things right:

- requests run through the authenticated dashboard session
- raw API keys are not re-exposed in the browser UI
- endpoint access is allowlisted rather than open-ended
- the page shows live responses, not mocked results

These constraints should be preserved.

## Production Requirements

The Playground should become a safe, developer-quality API console for the authenticated customer session.

### Must-have capabilities

1. Accurate endpoint coverage
- endpoint list aligned with actual playground-supported routes
- clear method and capability badges
- no drift between UI and proxy allowlist

2. Better request execution
- support both sync and async-capable endpoints
- support `GET /v1/jobs/:id` polling for async jobs
- preserve per-endpoint request metadata

3. Better response inspection
- show status code
- show request ID
- show duration
- show rate-limit headers
- show response payload cleanly

4. Better error handling
- distinguish invalid JSON, validation failure, unauthorized session, backend failure, and rate limiting
- surface backend error body clearly

5. Durable workflow support
- recent requests should come from backend history or a dashboard-side persisted model, not only browser local storage

### Nice-to-have capabilities

- endpoint-specific request templates/forms
- job polling helper UI
- copyable response/request IDs
- response tabs: body / meta / headers
- saved examples per endpoint
- shareable repro snippets

## Key Gaps Identified

### 1. UI/proxy drift

Current mismatch:
- proxy allowlist contains endpoints not present in the UI catalog
- users cannot select some routes the proxy already supports

### 2. POST-only mental model

Current proxy contract assumes:
- endpoint is selected
- payload is posted

This blocks first-class support for:
- polling async jobs
- future GET-based inspection routes

### 3. Local-only request history

Current logging path:
- browser local storage via `appendRequestLog()`

That means:
- history is device-specific
- history is not production-truth
- clearing storage removes it

### 4. Limited introspection

Backend already returns structured metadata, but Playground compresses most of the experience into one raw JSON block.

## Proposed Architecture

### 1. Introduce a capability-aware playground model

Add a backend or generated route metadata source that includes:
- path
- method
- category
- dashboard supported
- playground supported
- async job support
- polling route if applicable
- docs link

The Playground UI and proxy should derive from the same capability model.

### 2. Expand the playground proxy contract

Instead of a POST-only route shape, move toward a normalized request contract like:

```json
{
  "endpoint": "/v1/search",
  "method": "POST",
  "payload": {
    "query": "rust async programming"
  }
}
```

And for polling:

```json
{
  "endpoint": "/v1/jobs/abc123",
  "method": "GET"
}
```

### 3. Preserve strict allowlisting

Even with broader capability support:
- do not permit arbitrary path entry by default
- allow only known, customer-safe, dashboard-approved routes

## Proposed Backend Work

### 1. Add or reuse route metadata

The Playground needs a source of truth for:
- supported routes
- methods
- async relationship
- docs linkage

This should ideally align with the same production route metadata used for API Catalog.

### 2. Expose job workflows clearly

The backend already supports async jobs. The dashboard should be able to use:
- submit job endpoint
- `GET /v1/jobs/:id`

without inventing new backend semantics.

### 3. Consider a backend recent-request read model

If the product wants persistent Playground history, that should come from:
- `usage_logs`
- or a dedicated recent-request read model

not browser local storage.

## Proposed Dashboard Work

### 1. Refactor `/api/playground`

Update [playground/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/playground/route.ts):
- support method-aware requests
- align allowlist with shared route metadata
- surface backend `request_id` and other meta explicitly
- preserve rate-limit header visibility

### 2. Refactor Playground UI

Update [playground/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/playground/page.tsx):
- render endpoint capabilities from shared metadata
- show response meta separately from payload
- support async job polling flow
- improve error states and validation messaging

### 3. Decouple history from browser local storage

Local logs can remain as a temporary convenience, but production history should not depend on them.

Recommended path:
- local storage for immediate UX
- backend history for authoritative recent requests

### 4. Add structured response panels

Recommended UI sections:
- Request
- Response body
- Response metadata
- Rate limits

This is more usable than one undifferentiated JSON block.

## Implementation Plan

### Phase 1: Capability alignment

1. Introduce shared route/capability metadata.
2. Align Playground UI endpoint list with proxy allowlist.
3. Include missing currently supported routes in the selectable UI.

### Phase 2: Proxy expansion

1. Extend `/api/playground` to support method-aware requests.
2. Expose backend request ID and structured meta.
3. Support async job polling through approved GET routes.

### Phase 3: UI upgrade

1. Add structured response inspection.
2. Add async job workflow support.
3. Improve validation and error messaging.
4. Surface rate-limit information more clearly.

### Phase 4: History hardening

1. Replace or supplement local-only request logs with backend-backed recent history.
2. Add empty/loading/error states for history.
3. Validate multi-device consistency.

## Acceptance Criteria

- The Playground endpoint list matches the routes the proxy actually supports.
- The Playground can execute both synchronous requests and supported async job workflows.
- Response metadata includes request ID, status, duration, and rate-limit info in a clearly readable way.
- Errors are surfaced cleanly by type, not only as generic text.
- The page keeps the secure session-based proxy model and does not expose raw API keys.
- Playground history is no longer solely dependent on browser local storage for production usage.

## Risks and Decisions

### Open decisions

- whether to allow free-form endpoint entry for advanced users or stay curated-only
- whether async jobs belong directly in Playground or as a separate “Jobs” surface
- whether recent history should be keyed per API key or per workspace/org later

### Risks

- expanding route support without a proper capability model will increase drift
- free-form request support can weaken safety if not strictly bounded
- combining sync and async workflows on one page can make the UI heavier if not structured carefully

## Non-Goals

- exposing arbitrary internal routes
- turning Playground into a full Postman replacement
- bypassing the dashboard proxy with raw browser-side API calls

## Recommended Next Task

Implement the production Playground in this order:

1. shared route/capability metadata
2. proxy refactor for method-aware and async-safe requests
3. UI upgrade for structured response inspection and job workflows
4. backend-backed recent history instead of browser-only logs
