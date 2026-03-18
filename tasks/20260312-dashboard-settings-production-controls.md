# Dashboard Settings Production Controls Task

> **Date:** 2026-03-12
> **Priority:** P1
> **Area:** `app.fetchium.com` settings page
> **Status:** `DONE`

## Summary

The current Settings page is not a full settings surface. It is a hosted-session diagnostics page with sign-out and connection verification.

Current state:
- shows production API base
- shows session-derived plan, key ID, and masked key preview
- verifies API health plus authenticated usage access
- allows sign-out

This is useful and honest, but it is not a production settings page in the broader product sense. It does not allow users to manage account preferences, workspace configuration, API defaults, security settings, or notification settings.

## Research Findings

### Frontend

Current Settings page:
- [apps/dashboard/src/app/(dashboard)/dashboard/settings/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/settings/page.tsx)

Observed behavior:
- fetches `/api/auth/session`
- displays `plan`, `keyId`, and `apiKeyPreview`
- shows `DEFAULT_API_BASE`
- runs a connection test by combining:
  - `/api/health`
  - `/api/usage`
- allows sign-out

Important current behavior:
- the page explicitly states it is read-only for hosted production settings
- the API base is intentionally locked in hosted production

### Dashboard session model

Relevant auth/session code:
- [apps/dashboard/src/auth.ts](/home/echo/projects/Fetchium/apps/dashboard/src/auth.ts)

Observed facts:
- session includes:
  - `apiBase`
  - `plan`
  - `keyId`
  - `apiKeyPreview`
- the full API key remains server-side in the session token for proxy use
- the page correctly avoids re-exposing the full key

### Connection verification path

Relevant routes:
- [apps/dashboard/src/app/api/health/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/health/route.ts)
- [apps/dashboard/src/app/api/usage/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/usage/route.ts)
- [apps/dashboard/src/lib/server-api.ts](/home/echo/projects/Fetchium/apps/dashboard/src/lib/server-api.ts)

Observed facts:
- hosted production dashboard is locked to `***REMOVED***`
- `/api/health` verifies API liveness
- `/api/usage` verifies authenticated key/session access
- together they provide a reasonable end-to-end connectivity check

This means the current page is best understood as:
- session inspection
- connectivity diagnostics
- sign-out

### Product gap

There is no broader customer settings domain implemented for:
- profile/account settings
- workspace metadata
- notification preferences
- security/session management
- API defaults/preferences

So the current Settings page is not incomplete by accident. It reflects the absence of those product domains.

## Problem Statement

The page is labeled `Settings`, which implies a broader configuration surface than what actually exists.

Current user expectation risk:
- users may expect editable settings
- users may expect security controls like session/device management
- users may expect account preferences or workspace controls

What the page actually provides:
- hosted session diagnostics
- a health check
- sign-out

That is valuable, but narrower than the label suggests.

## What The Page Does Correctly Today

The current page gets several important things right:

- it does not allow unsafe runtime mutation of the hosted API base
- it does not re-expose the full API key
- it verifies connection using both health and usage validation
- it provides a simple session status view

Those properties should be preserved.

## Production Requirements

The product needs to make a clear decision about this page:

1. Keep it as a diagnostics/settings-lite page, or
2. Evolve it into a full customer settings surface

### Minimum production requirements if kept as diagnostics-first

- clearly position it as `Settings & Diagnostics` or similar
- keep session info accurate
- keep connection verification robust
- show session expiry or validity context if available
- show meaningful failure states

### Must-have capabilities for a true production settings page

1. Account/workspace identity
- workspace name or account label
- owner/admin context if applicable

2. Security/session controls
- current session details
- session/device list if supported
- revoke other sessions if supported

3. Notification/preferences
- email preferences
- release/update notifications
- alert preferences if product supports them

4. API/workflow defaults
- default region/base if product ever supports it
- default request preferences if relevant

5. Support/debug info
- connection test
- current environment
- request/debug identifiers where useful

### Nice-to-have capabilities

- theme preference if not handled globally elsewhere
- webhook preferences
- personal access or secondary token controls if product evolves there

## Source-of-Truth Requirements

Settings should not be fabricated from the session alone if they are meant to persist as customer-managed configuration.

Production settings truth should be:
- session state for auth/session diagnostics
- a customer settings/profile/workspace store for persistent preferences

The current page only has the first half.

## Proposed Architecture

### Option A: Formalize this page as diagnostics-first

If the near-term product does not need broader settings, make that explicit.

Recommended scope:
- session information
- production API target
- connection verification
- sign-out

Recommended naming/copy:
- `Settings & Diagnostics`
- or `Session & Connection`

This is the most honest short-term product move.

### Option B: Build a real settings domain

If the dashboard is meant to mature into a full customer control panel, add customer-facing endpoints such as:
- `GET /v1/dashboard/settings`
- `PATCH /v1/dashboard/settings`
- `GET /v1/dashboard/sessions`
- `DELETE /v1/dashboard/sessions/:id`

Suggested `GET /v1/dashboard/settings` response:

```json
{
  "meta": {
    "request_id": "req_123",
    "status": "ok",
    "endpoint": "/v1/dashboard/settings",
    "duration_ms": 8
  },
  "workspace": {
    "name": "Acme Research",
    "plan": "pro"
  },
  "session": {
    "key_id": "key_123",
    "api_key_preview": "fetchium_abcd...1234",
    "api_base": "***REMOVED***"
  },
  "preferences": {
    "email_updates": true,
    "incident_alerts": false
  }
}
```

## Proposed Backend Work

### 1. Decide whether customer settings exist beyond session state

If yes:
- define the persisted customer settings model
- define workspace/account ownership model

If no:
- avoid pretending this page is a full settings surface

### 2. Add customer settings endpoints if broader settings are required

Potential endpoints:
- `GET /v1/dashboard/settings`
- `PATCH /v1/dashboard/settings`
- `GET /v1/dashboard/sessions`
- `POST /v1/dashboard/verify-connection` if connection diagnostics are formalized

### 3. Keep security boundaries strict

Do not expose:
- full API key
- admin session controls
- mutable hosted API base in production

## Proposed Dashboard Work

### 1. Clarify page role immediately

Short-term improvement:
- rename or reframe the page as settings/diagnostics rather than implying broad configuration

### 2. Keep diagnostics as a first-class section

Even if a broader settings system is built later, retain:
- production API base visibility
- connection verification
- authenticated session summary

These are genuinely useful operational controls.

### 3. Add real settings sections only when backed by product state

Potential future sections:
- Workspace
- Security
- Notifications
- Preferences

Do not add fake controls that do not persist.

## Implementation Plan

### Phase 1: Product decision

1. Decide whether Settings remains diagnostics-first or becomes a true settings surface.
2. Update product copy and IA accordingly.

### Phase 2A: Diagnostics-first hardening

1. Improve connection verification messaging.
2. Add clearer session/loading/error states.
3. Optionally show session expiry or last validation details.

### Phase 2B: Full settings domain

1. Add backend customer settings model.
2. Add settings/session endpoints.
3. Add dashboard proxy routes.
4. Add editable settings UI backed by persisted data.

## Acceptance Criteria

- The Settings page accurately reflects its true product role.
- Session diagnostics remain correct and secure.
- Hosted production API base remains non-editable in production.
- The page never exposes the full API key.
- If persistent settings are introduced, they are backed by real server-side state and not local-only placeholders.

## Risks and Decisions

### Open decisions

- whether settings are per user, per workspace, or both
- whether session management should be exposed to customers
- whether the page should be renamed if broader settings are deferred

### Risks

- calling a diagnostics page “Settings” can create ongoing product confusion
- adding fake or local-only settings would be worse than keeping the page narrow
- if customer/workspace identity is not clearly modeled, settings ownership will become ambiguous

## Non-Goals

- exposing admin settings or staff session management
- making the hosted production API base editable in-browser
- re-exposing raw API credentials

## Recommended Next Task

Choose one of these paths:

1. short-term: formalize the page as a diagnostics-first settings surface and improve messaging/states
2. long-term: build a real customer settings domain with persisted preferences, workspace identity, and session management
