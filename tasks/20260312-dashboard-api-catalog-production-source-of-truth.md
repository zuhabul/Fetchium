# Dashboard API Catalog Production Source of Truth

> **Date:** 2026-03-12
> **Priority:** P1
> **Status:** `DONE`
> **Area:** `app.fetchium.com` dashboard API Catalog
> **Primary surfaces:** dashboard catalog, playground endpoint list, docs links

## Objective

Replace the dashboard's hardcoded API Catalog with a production-safe, backend-aligned route registry so the dashboard always renders an accurate customer-facing API surface.

This task is complete only when:
- the dashboard catalog is no longer the primary route registry
- playground capabilities and catalog capabilities are derived from the same source
- customer-facing docs links are aligned with real backend routes
- route drift is caught automatically in CI

## Current State

### Dashboard UI

Current catalog page:
- [apps/dashboard/src/app/(dashboard)/dashboard/api/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/api/page.tsx)

Current local registry:
- [apps/dashboard/src/lib/dashboard-catalog.ts](/home/echo/projects/Fetchium/apps/dashboard/src/lib/dashboard-catalog.ts)

Observed behavior:
- the page renders entirely from `dashboardEndpoints`
- endpoint metadata, docs links, example payloads, and curl snippets are hardcoded
- the page performs no backend fetch
- the page cannot know whether a listed route is currently valid, supported in the playground, or stale

### Backend Router

Customer API routes:
- [crates/fetchium-api/src/routes.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/routes.rs)

Current authenticated customer routes under `/v1`:
- `POST /v1/search`
- `POST /v1/scrape`
- `POST /v1/fetch`
- `POST /v1/research`
- `POST /v1/youtube/search`
- `POST /v1/youtube/analyze`
- `POST /v1/social/research`
- `POST /v1/social/research/jobs`
- `POST /v1/social/reddit`
- `POST /v1/social/reddit/jobs`
- `POST /v1/social/hackernews`
- `POST /v1/social/hackernews/jobs`
- `POST /v1/estimate`
- `POST /v1/research/jobs`
- `POST /v1/youtube/search/jobs`
- `POST /v1/youtube/analyze/jobs`
- `GET /v1/jobs/:id`
- `GET /v1/usage`

### Docs Surface

Docs pages currently exist under:
- [apps/web/src/app/docs/api/search/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/search/page.tsx)
- [apps/web/src/app/docs/api/scrape/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/scrape/page.tsx)
- [apps/web/src/app/docs/api/research/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/research/page.tsx)
- [apps/web/src/app/docs/api/async-jobs/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/async-jobs/page.tsx)
- [apps/web/src/app/docs/api/estimate/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/estimate/page.tsx)
- [apps/web/src/app/docs/api/youtube/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/youtube/page.tsx)
- [apps/web/src/app/docs/api/social/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/social/page.tsx)
- [apps/web/src/app/docs/api/usage/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/usage/page.tsx)
- [apps/web/src/app/docs/api/health/page.tsx](/home/echo/projects/Fetchium/apps/web/src/app/docs/api/health/page.tsx)

### Playground Scope

Current playground proxy:
- [apps/dashboard/src/app/api/playground/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/playground/route.ts)

Current allowlist includes routes not selectable in the current dashboard catalog:
- `/v1/scrape`
- `/v1/social/hackernews`
- `/v1/social/research`

## Findings

### 1. The dashboard catalog is incomplete

The current `dashboardEndpoints` list omits valid customer routes already in the backend router and already allowed in the dashboard playground.

### 2. There are multiple route truth sources

Current route truth is split across:
- backend router in `fetchium-api`
- dashboard local catalog
- playground hardcoded allowlist
- docs pages in `apps/web`
- the API root summary response in [routes.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/routes.rs)

This guarantees drift over time.

### 3. The dashboard lacks capability metadata

The catalog cannot currently answer:
- is this route customer-facing?
- is it documented?
- is it runnable in the dashboard playground?
- does it have an async job variant?
- does it require polling?

### 4. The current local catalog mixes two different concerns

`dashboard-catalog.ts` currently acts as both:
- route registry
- UX metadata store

Those should be split.

## Problem Statement

The API Catalog page currently looks authoritative but is backed by a manual array. This makes it unsafe as a production reference surface.

Risks:
- stale or missing endpoints in the dashboard
- docs links drifting from actual API routes
- playground exposing routes the catalog does not show
- backend changes silently breaking product truth

## Decision

Adopt a backend-owned customer route registry as the primary source of truth.

Use the dashboard local catalog only for UX supplements:
- example payloads
- example curl templates
- presentational labels if needed

Do not continue using `dashboard-catalog.ts` as the authoritative route registry.

## Scope

### In scope

- customer-facing authenticated `/v1` routes
- route metadata needed by dashboard catalog and playground
- docs link mapping for customer routes
- capability badges for dashboard and playground
- CI drift detection

### Out of scope

- admin routes under `/v1` requiring `X-Admin-Secret`
- internal staff routes under `/internal/admin/*`
- full OpenAPI generation in this task
- replacing the public docs site

## Production Requirements

### Functional requirements

1. The catalog must render from a backend-owned or generated route registry.
2. The playground endpoint selector must derive from the same registry.
3. The registry must distinguish:
   - customer-visible route
   - dashboard-visible route
   - playground-supported route
   - async route
   - polling route
4. The dashboard must still support UX-specific sample payloads and curl snippets.
5. Drift between backend routes and dashboard catalog/playground must fail tests.

### Data requirements

Each customer route record must include:
- `path`
- `method`
- `category`
- `label`
- `description`
- `docs_href`
- `auth_mode`
- `dashboard_visible`
- `playground_supported`
- `async_variant`
- `polling_route`
- `sample_key`

### Non-functional requirements

- no customer dashboard exposure of admin/internal routes
- no browser-side trust in a local route registry as source of truth
- safe fallback behavior if route metadata fetch fails
- metadata fetch cheap enough for dashboard page load

## Proposed Design

### Preferred design

Add a lightweight backend metadata endpoint:
- `GET /v1/meta/routes`

This returns only customer-safe route metadata and no secrets.

The dashboard consumes this via:
- `apps/dashboard/src/app/api/dashboard/routes/route.ts`

The proxy route may merge in local UX metadata from a reduced `dashboard-catalog.ts`.

### Route metadata schema

```json
{
  "routes": [
    {
      "path": "/v1/search",
      "method": "POST",
      "category": "core",
      "label": "Search",
      "description": "Federated search across retrieval backends with ranking and citations.",
      "docs_href": "https://docs.fetchium.com/api/search",
      "auth_mode": "bearer_key",
      "dashboard_visible": true,
      "playground_supported": true,
      "async_variant": null,
      "polling_route": null,
      "sample_key": "search"
    },
    {
      "path": "/v1/research",
      "method": "POST",
      "category": "research",
      "label": "Research",
      "description": "Multi-step research synthesis with source-backed output.",
      "docs_href": "https://docs.fetchium.com/api/research",
      "auth_mode": "bearer_key",
      "dashboard_visible": true,
      "playground_supported": true,
      "async_variant": "/v1/research/jobs",
      "polling_route": "/v1/jobs/:id",
      "sample_key": "research"
    }
  ]
}
```

### Category contract

Allowed customer categories:
- `core`
- `research`
- `media`
- `social`
- `jobs`
- `utility`

### Auth contract

Allowed values:
- `public`
- `bearer_key`
- `admin_secret`
- `internal_session`

The dashboard catalog must only render:
- `public`
- `bearer_key`

And by default should only show customer-facing routes marked `dashboard_visible: true`.

## Concrete Route Inventory For Initial Registry

### Must be present in the customer registry

- `POST /v1/search`
- `POST /v1/scrape`
- `POST /v1/fetch`
- `POST /v1/research`
- `POST /v1/research/jobs`
- `POST /v1/youtube/search`
- `POST /v1/youtube/search/jobs`
- `POST /v1/youtube/analyze`
- `POST /v1/youtube/analyze/jobs`
- `POST /v1/social/research`
- `POST /v1/social/research/jobs`
- `POST /v1/social/reddit`
- `POST /v1/social/reddit/jobs`
- `POST /v1/social/hackernews`
- `POST /v1/social/hackernews/jobs`
- `POST /v1/estimate`
- `GET /v1/jobs/:id`
- `GET /v1/usage`
- `GET /v1/health`

### Must not be present in the customer registry

- `/v1/keys`
- `/v1/proxy/*`
- `/internal/admin/*`

## Ownership Boundaries

### Backend owns

- route existence
- method
- auth mode
- customer visibility
- playground support eligibility
- async relationships
- polling relationships

### Dashboard owns

- example request bodies
- example curl templates
- optional display copy refinements
- grouping/layout behavior in the UI

### Docs app owns

- deep reference content for each route
- extended examples
- prose documentation

## Required Backend Changes

### 1. Add a typed route metadata model

Create a typed serializable model in `fetchium-api` for customer route metadata.

Suggested files:
- `crates/fetchium-api/src/types.rs`
- or a new `crates/fetchium-api/src/route_registry.rs`

### 2. Add registry builder

Build a single internal route registry that declares customer-facing route metadata once.

The router and metadata endpoint should both derive from that registry where feasible.

Minimum acceptable fallback:
- router remains explicit
- metadata registry is separately declared
- tests enforce alignment

### 3. Add metadata endpoint

Add:
- `GET /v1/meta/routes`

Response requirements:
- JSON
- no auth secrets
- customer-safe only
- stable field names

### 4. Add route alignment tests

Add tests that fail if:
- a customer route in the router is missing from the metadata registry
- a metadata route references an invalid docs page
- a playground-supported route is absent from the registry

## Required Dashboard Changes

### 1. Add dashboard proxy route

Create:
- `apps/dashboard/src/app/api/dashboard/routes/route.ts`

Behavior:
- fetch backend route metadata
- merge dashboard UX metadata by `sample_key` or `path`
- return normalized payload to the client

### 2. Refactor `dashboard-catalog.ts`

Convert it from route registry to UX supplement store.

It should no longer define authoritative route presence.

Recommended structure:
- `samplePayloadsByKey`
- `sampleCurlTemplateByKey`
- optional `displayOverridesByKey`

### 3. Refactor API Catalog page

Update:
- [apps/dashboard/src/app/(dashboard)/dashboard/api/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/api/page.tsx)

Required behavior:
- fetch route metadata
- render capability badges
- render docs links from normalized metadata
- surface async relationships
- show example payload/snippet only when available

### 4. Refactor Playground selector

Update:
- [apps/dashboard/src/app/(dashboard)/dashboard/playground/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/playground/page.tsx)
- [apps/dashboard/src/app/api/playground/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/playground/route.ts)

Required behavior:
- derive selectable routes from the same registry
- stop maintaining a separate hardcoded allowlist unless generated from the same data

## UX Requirements

The API Catalog page should visibly distinguish:
- `Dashboard`
- `Playground`
- `Async`
- `Polling`
- `Docs`

Recommended sections:
- Interactive in dashboard
- Async and job-based routes
- Reference-only customer routes

Recommended empty/error states:
- metadata loading
- metadata unavailable
- docs link missing

## Rollout Plan

### Phase 1: Registry foundation

1. Add backend metadata type and registry.
2. Add `GET /v1/meta/routes`.
3. Add backend tests for registry correctness.

### Phase 2: Dashboard adoption

1. Add `/api/dashboard/routes` proxy.
2. Refactor API Catalog to consume backend metadata.
3. Refactor local catalog into UX supplement data.

### Phase 3: Playground alignment

1. Derive playground selector from the same route metadata.
2. Remove drift between selector and proxy allowlist.
3. Add dashboard tests for capability alignment.

### Phase 4: Hardening

1. Add CI drift checks.
2. Validate docs link coverage.
3. Validate no admin/internal route leakage.

## Test Plan

### Backend tests

- registry includes every intended customer route
- registry excludes admin and internal routes
- async variants point to valid route records
- docs links are non-empty and structurally valid

### Dashboard tests

- `/dashboard/api` renders backend-provided routes
- example payloads merge correctly
- missing UX supplement does not break rendering
- playground selector only shows routes marked `playground_supported`

### Drift tests

- route registry vs router alignment
- route registry vs playground support alignment
- route registry vs docs mapping alignment

## Acceptance Criteria

- The API Catalog page no longer renders from a hardcoded authoritative route list.
- A backend-owned customer route registry exists and is consumed by the dashboard.
- `/v1/scrape`, `/v1/social/hackernews`, and `/v1/social/research` are accurately represented.
- Async routes and polling routes are explicitly represented.
- Admin/internal routes are excluded from the customer catalog.
- Playground endpoint selection is aligned with the same capability model.
- CI fails when backend routes and dashboard route metadata drift.

## Risks

- duplicating route declarations without alignment tests will reintroduce drift
- trying to jump straight to full OpenAPI may slow delivery unnecessarily
- mixing docs content ownership with route ownership will blur responsibilities

## Open Decisions

- whether to build the registry directly from route declarations or maintain a typed parallel registry with tests
- whether `GET /v1/health` should appear in the dashboard catalog by default
- whether async child routes should appear inline or grouped under parent endpoints

## Recommended Implementation Order

1. backend typed route registry
2. backend metadata endpoint
3. dashboard proxy route
4. API Catalog page refactor
5. Playground alignment
6. CI drift protection
