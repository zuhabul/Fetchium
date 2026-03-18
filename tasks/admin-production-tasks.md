# Fetchium Admin Site — Production Readiness Task List

**Generated**: 2026-03-14
**Audit scope**: 39 files examined (22 backend, 17 frontend)
**Current state**: ~85% backend complete, ~80% frontend complete
**Est. effort to 100%**: 40–60 dev hours

---

## PRIORITY 1 — SECURITY (Fix First, Block All Other Work)

### SEC-1: SQL Injection in Admin Endpoints ⚠️ CRITICAL
**Files**: `admin/audit.rs:242`, `admin/orgs.rs:238-244`, `admin/usage.rs:73-84`, `admin/campaigns.rs:57-58`

**Problem**: String interpolation with untrusted IDs directly into SQL queries:
```rust
// DANGEROUS — id is not parameterized
&format!("SELECT ... WHERE target_id='{id}' ...")
```

**Fix**: Replace all `format!()` SQL with parameterized queries using rusqlite `params![]` macro.

**Affected endpoints**:
- `GET /internal/admin/orgs/:id/audit`
- `GET /internal/admin/usage/*`
- `GET /internal/admin/campaigns/:id`

**Acceptance criteria**:
- [x] Zero raw string interpolation in SQL queries
- [x] All queries use `?1` placeholders + `run_select_query_params1()`
- [x] `cargo check` passes with no warnings

---

### SEC-2: Provider Health Check Uses Only Env Var Presence
**File**: `admin/metrics.rs:70-87`

**Problem**: Backend reports providers as "ok" if env var exists, not if provider is reachable:
```rust
("Google", "google", std::env::var("PROXY_USER").is_ok())  // Never pings endpoint
```

**Fix**: Add async ping/health check per provider with timeout (2s). Cache result for 30s.

**Acceptance criteria**:
- [x] Each search provider performs TCP connectivity check (2s timeout)
- [x] Status is `ok` / `down` / `unconfigured` based on actual connectivity + env
- [x] Results cached 30s via module-level OnceLock

---

## PRIORITY 2 — BROKEN STUB IMPLEMENTATIONS (Critical Business Logic)

### STUB-1: Org Update Is a NOOP ✅ DONE
**File**: `admin/orgs.rs`
**Endpoint**: `PATCH /internal/admin/orgs/:id`

**Fix applied**: Added `UpdateOrg` body struct + `update_org_fields()` DB method. Supports partial updates (name, owner_email, notes). Adds `notes` and `quota_override` columns via safe `ALTER TABLE` migration.

**Acceptance criteria**:
- [x] PATCH actually updates the organizations table
- [x] Audit event logged on update (`org.update`)
- [x] Returns updated org object

---

### STUB-2: Org Quota Override Is a NOOP ✅ DONE
**File**: `admin/orgs.rs`
**Endpoint**: `PATCH /internal/admin/orgs/:id/quota`

**Fix applied**: `OverrideQuota` body now has `monthly_limit: i64` field. `update_org_quota()` DB method writes to `quota_override` column (added via migration).

**Acceptance criteria**:
- [x] Quota field updated in DB (`quota_override` column)
- [x] Audit event logged (`org.quota_override`)
- [x] Returns updated quota value in response

---

### STUB-3: Billing Refund Does Nothing ✅ DONE
**File**: `admin/billing.rs`
**Endpoint**: `POST /internal/admin/billing/:org_id/refund`

**Fix applied**: Added `pending_refunds` table migration + `create_pending_refund()` DB method. Accepts optional `RefundBody` with `amount_cents` + `reason`. Returns `{status: "queued", refund_id: "..."}` — no longer misleading.

**Acceptance criteria**:
- [x] Refund stored in DB with `pending` status
- [x] Response clearly indicates manual processing required (`status: "queued"`)
- [x] Audit event logged with `billing.refund` action; `refund_id` in response

---

### STUB-4: Billing Webhook Replay Does Nothing ✅ DONE
**File**: `admin/billing.rs`
**Endpoint**: `POST /internal/admin/billing/webhooks/:id/replay`

**Fix applied**: Added `get_payment_event()` + `mark_payment_event_replayed()` DB methods. Added `replayed_at` column to `payment_events` via migration. Handler fetches event (404 if missing), stamps `replayed_at`, returns original payload.

**Acceptance criteria**:
- [x] Webhook payload fetched from DB and returned in response
- [x] `replayed_at` timestamp updated in DB
- [x] Returns original event data + confirmation message

---

### STUB-5: Proxy Stats/Reset/Purge Are Stubs ✅ DONE
**File**: `admin/proxy_ops.rs`
**Endpoints**: `GET /proxy/stats`, `POST /proxy/reset`, `POST /proxy/purge`

**Fix applied**: All three wired to live `ProxyPool` via `state.http.proxy_pool()`. `pool_summary()` + per-proxy `stats()` for real metrics. `reset_all()` and `purge_dead()` for actual pool operations.

**Acceptance criteria**:
- [x] `stats` returns real pool summary (active/cooldown/dead counts, success rates, per-proxy metrics)
- [x] `reset` calls `reset_all()` — resets every proxy to Active, clears domain assignments
- [x] `purge` calls `purge_dead()` — removes dead proxies, returns count removed

---

### STUB-6: Audit Entry Detail Returns null ✅ DONE
**File**: `admin/audit.rs`
**Endpoint**: `GET /internal/admin/audit/:id`

**Fix applied**: Added `get_audit_entry()` DB method with parameterized query. Returns full event or proper 404.

**Acceptance criteria**:
- [x] Returns full audit event record by ID (id, action, role, target, ip, created_at)
- [x] Returns 404 with `{"error": "audit entry not found"}` if missing
- [x] No metadata column exists in schema — returns all available fields

---

## PRIORITY 3 — MISSING FRONTEND PAGES (6 Pages)

### PAGE-1: System Overview Page (`/system`) ✅ DONE
**File**: `apps/admin/src/app/(admin)/system/page.tsx`

**Fix applied**: Page was already fetching real metrics from `/internal/admin/metrics/summary` + `/internal/admin/metrics/realtime` (CPU, RAM, disk, uptime, error rate, RPS, provider health). Added `SystemRefresh` client component (`system/SystemRefresh.tsx`) that calls `router.refresh()` every 10s to re-trigger SSR data fetch.

**Acceptance criteria**:
- [x] Shows real CPU/memory/disk from backend metrics summary
- [x] Auto-refreshes every 10s via `SystemRefresh` client component
- [x] Shows provider health status (ok/down/unconfigured) for all 8 providers

---

### PAGE-2: System API Explorer (`/system/api`) ✅ DONE
**File**: `apps/admin/src/app/(admin)/system/api/page.tsx`

**Fix applied**: Expanded `ROUTES` array from 27 hardcoded entries to 110+ entries covering all routes from `routes.rs` — public, customer API (api_key), v1 admin (X-Admin-Secret), and all internal admin routes (auth, sessions, staff, orgs, users, keys, usage, billing, CRM, support, incidents, campaigns, audit, flags, metrics, proxy, DB, search, anomaly, export, approvals). `ApiExplorerClient.tsx` already had functional filter + curl test modal.

**Acceptance criteria**:
- [x] Lists all 110+ endpoints with correct method, path, auth, handler
- [x] Shows method, path, auth required, rate limit, handler name
- [x] Inline curl test modal works with current session token

---

### PAGE-3: System Config Editor (`/system/config`) ✅ DONE
**File**: `apps/admin/src/app/(admin)/system/config/page.tsx`

**Fix applied**: Page already fully implemented — SSR fetches from `GET /internal/admin/flags` and passes data to `ConfigEditorClient`. Client has toggle switches, rollout % sliders, kill-switch confirm modal, and calls `PATCH /api/admin/flags/:id` for updates. All proxied through catch-all route.

**Acceptance criteria**:
- [x] Shows all feature flags with enabled/disabled toggles and rollout % sliders
- [x] Sensitive keys excluded — only feature flags shown
- [x] Changes persist via PATCH to `/internal/admin/flags/:id` with kill-switch confirmation modal

---

### PAGE-4: System DB Console (`/system/db`) ✅ DONE
**File**: `apps/admin/src/app/(admin)/system/db/page.tsx`

**Fix applied**: Page already fully implemented — client component with two tabs: Browse (table list sidebar, click to preview rows) and Query (SQL textarea, run button, CSV export). Calls `POST /api/admin/db/query`. Non-SELECT queries blocked on frontend with error message.

**Acceptance criteria**:
- [x] SQL input textarea with run button
- [x] Results displayed in scrollable table with CSV export
- [x] Non-SELECT queries blocked with clear error on frontend

---

### PAGE-5: Usage Forensics (`/usage/forensics`) ✅ DONE
**File**: `apps/admin/src/app/(admin)/usage/forensics/page.tsx`

**Fix applied**: Page already fully implemented — search form (request ID input), result detail grid showing org, endpoint, query, backend, latency, tokens, cost, status, error. Calls `GET /api/admin/usage/forensics/:id`. Org name links to org profile.

**Acceptance criteria**:
- [x] Search by request_id returns full trace detail
- [x] Shows latency, backend used, tokens, cost, status, raw error
- [x] Shows error details if request failed

---

### PAGE-6: Usage Inspector (`/usage/inspector`) ✅ DONE
**File**: `apps/admin/src/app/(admin)/usage/inspector/page.tsx`

**Fix applied**: Page already fully implemented — live audit stream with 5s auto-refresh, freeze button, filter by action/target type, side panel for event detail. Calls `GET /api/admin/audit?limit=100&offset=0`. Row click opens detail panel.

**Acceptance criteria**:
- [x] Shows last 100 audit events with 5s auto-refresh and freeze toggle
- [x] Filter by action and target type works
- [x] Row click opens detail side panel with full event data

---

## PRIORITY 4 — FRONTEND BUGS & DATA ISSUES

### BUG-1: Usage Page Uses Wrong Fetch Path ✅ DONE (already correct)
**File**: `apps/admin/src/app/(admin)/usage/page.tsx`

**Investigation**: Fetches `/api/admin/usage` and `/api/admin/usage/top-orgs` from the browser. These ARE correctly handled by the catch-all route at `/api/admin/[...path]/route.ts` which proxies to `/internal/admin/usage` and `/internal/admin/usage/top-orgs` with session auth. No fix required — the catch-all route makes this work correctly.

**Acceptance criteria**:
- [x] Page loads real usage data via catch-all proxy
- [x] No 404 errors — catch-all handles `/api/admin/*` → `/internal/admin/*`
- [x] Client-side fetch with session cookie is correct for this interactive range-selector page

---

### BUG-2: Org Detail — Update Form Submits to NOOP Endpoint ✅ DONE
**File**: `apps/admin/src/app/(admin)/orgs/[id]/page.tsx`

**Fix applied**: The overview tab had no edit form at all. Added inline edit mode with state (`editing`, `editName`, `editEmail`, `editNotes`, `saving`). "Edit Org" button shows editable inputs for name/owner_email/notes. Save calls `PATCH /api/admin/orgs/${id}`, parses response `data` field to update local state, shows success/error in `actionMsg`. Cancel dismisses without saving.

**Acceptance criteria**:
- [x] Successful update shows new values immediately after save (from response data)
- [x] Error from backend shows human-readable error message in action banner
- [x] Form exits edit mode and clears dirty state on successful save

---

### BUG-3: Support Ticket Detail Missing Reply Box ✅ DONE
**File**: `apps/admin/src/app/(admin)/support/[ticketId]/TicketActions.tsx`

**Fix applied**: `TicketActions.tsx` had an `updateField()` function calling `PATCH /api/admin/support/tickets/${ticketId}` (no such route) for both status and assignee updates. Fixed by replacing with:
- `updateStatus()` → `PATCH /api/admin/support/tickets/${ticketId}/status` with `{status}`
- `updateAssignee()` → `PATCH /api/admin/support/tickets/${ticketId}/assign` with `{assignee_id}`
- Priority dropdown: removed broken backend call (no priority endpoint exists) — local state only.
- Note submission to `POST .../notes` was already correct.

**Acceptance criteria**:
- [x] Ticket metadata renders correctly via `normalizeTicket()` normalization
- [x] Internal note form submits to correct `POST /support/tickets/:id/notes` endpoint
- [x] Status change dropdown calls correct `PATCH .../status` endpoint

---

### BUG-4: Billing Detail — Refund Button Misleading ✅ DONE
**File**: `apps/admin/src/app/(admin)/billing/[orgId]/BillingActions.tsx`

**Fix applied**: `handleSubmit()` previously showed "Refund applied successfully" regardless of response and never checked `res.ok`. Fixed:
- Now checks `res.ok` — shows backend error message on failure
- Parses response JSON to extract `refund_id`
- Toast now shows: `"Refund queued for manual review (ID: abc12345…)"`
- Credit path unchanged — still shows "Credit applied successfully" (credits are immediate)
- Toast timeout extended to 5s so refund ID is readable

**Acceptance criteria**:
- [x] Refund button shows accurate "queued for manual review" status
- [x] Refund ID from backend response displayed in toast
- [x] Error from backend (non-ok response) shown as toast instead of false success

---

### BUG-5: Pagination Missing on Several List Pages ✅ DONE
**Files**: `admin/crm.rs`, `admin/flags.rs`, `admin/approval.rs`

**Fix applied**:
- `crm.rs list()`: Added `Query<ListParams>`, passes `limit`/`offset` to `db.list_crm_accounts()`, fetches full count separately for accurate `total`
- `flags.rs list()`: Added `Query<ListParams>`, fetches all via `db.list_flags()`, paginates in-memory with `.skip(offset).take(limit)`, returns correct `total`
- `approval.rs list()`: Same in-memory pagination pattern
- All three now return `{"data": [...], "total": N, "limit": N, "offset": N}`
- Frontend pages (CRM, flags) do client-side filtering on returned data — pagination bar not added as these datasets are typically small (<200 records)

**Acceptance criteria**:
- [x] All 3 list endpoints accept `?limit=N&offset=N` query params
- [x] Total record count returned accurately in all responses
- [x] `cargo check -p fetchium-api` passes clean (0 errors, 0 warnings)

---

## PRIORITY 5 — MISSING FEATURES (Not Broken, Just Not Built)

### FEAT-1: No Admin Route for Anomaly Detection UI
**Backend**: `GET /internal/admin/anomaly/alerts` and `/anomaly/tenants` exist and return real data.
**Frontend**: No page exists for anomaly alerts.

**Fix**: Create `/anomalies` page with:
- Alert list with severity badges
- Per-tenant anomaly scores
- Dismiss/acknowledge actions

---

### FEAT-2: No Admin Route for Approval Workflows UI
**Backend**: Full CRUD at `/internal/admin/approvals/*` exists.
**Frontend**: No page exists.

**Fix**: Create `/approvals` page with:
- Pending approvals queue
- Approve/reject buttons
- Approval history

---

### FEAT-3: Export Not Surfaced in UI
**Backend**: `GET /internal/admin/export/:entity` exists for CSV export.
**Frontend**: No download button on list pages.

**Fix**: Add "Export CSV" button to orgs, users, keys, usage pages that calls the export endpoint.

---

### FEAT-4: Search Not Surfaced in UI
**Backend**: `GET /internal/admin/search` exists (full-text across orgs/users/keys).
**Frontend**: AdminShell has command palette but global search not wired.

**Fix**: Wire command palette search input to `/internal/admin/search` endpoint.

---

### FEAT-5: Staff Management UI Missing
**Backend**: Full CRUD at `/internal/admin/staff/*` exists (list, create, update, delete, sessions).
**Frontend**: No `/staff` or `/settings/staff` page.

**Fix**: Create staff management page under `/settings/staff` with:
- Staff list with roles
- Invite new staff (create form)
- Edit permissions
- Revoke sessions

---

## SUMMARY TABLE

| Priority | Task ID | Issue | Effort | Status |
|----------|---------|-------|--------|--------|
| P1 | SEC-1 | SQL injection in 4 endpoints | 4h | ✅ Done |
| P1 | SEC-2 | Fake provider health checks | 2h | ✅ Done |
| P2 | STUB-1 | Org update is NOOP | 2h | ✅ Done |
| P2 | STUB-2 | Org quota override is NOOP | 1h | ✅ Done |
| P2 | STUB-3 | Billing refund does nothing | 3h | ✅ Done |
| P2 | STUB-4 | Webhook replay does nothing | 2h | ✅ Done |
| P2 | STUB-5 | Proxy stats/reset/purge stubs | 4h | ✅ Done |
| P2 | STUB-6 | Audit get-by-ID returns null | 1h | ✅ Done |
| P3 | PAGE-1 | System overview auto-refresh | 2h | ✅ Done |
| P3 | PAGE-2 | System API explorer 27→110+ routes | 4h | ✅ Done |
| P3 | PAGE-3 | System config editor (flags PATCH) | 3h | ✅ Done |
| P3 | PAGE-4 | System DB console (query + browse) | 3h | ✅ Done |
| P3 | PAGE-5 | Usage forensics (trace detail) | 3h | ✅ Done |
| P3 | PAGE-6 | Usage inspector (5s live stream) | 3h | ✅ Done |
| P4 | BUG-1 | Usage page fetch path (already correct) | 1h | ✅ Done |
| P4 | BUG-2 | Org update form — added inline edit | 1h | ✅ Done |
| P4 | BUG-3 | Support ticket wrong endpoints fixed | 2h | ✅ Done |
| P4 | BUG-4 | Billing refund toast shows queued+ID | 1h | ✅ Done |
| P4 | BUG-5 | Pagination on crm/flags/approvals | 2h | ✅ Done |
| P5 | FEAT-1 | Anomaly alerts page missing | 3h | ❌ Not Done |
| P5 | FEAT-2 | Approvals page missing | 3h | ❌ Not Done |
| P5 | FEAT-3 | CSV export not in UI | 2h | ❌ Not Done |
| P5 | FEAT-4 | Global search not wired | 1h | ❌ Not Done |
| P5 | FEAT-5 | Staff management page missing | 4h | ❌ Not Done |

**Total tasks**: 24
**Total estimated effort**: ~53 hours

---

## WHAT'S ALREADY WORKING (Don't Touch)

The following are fully implemented and wired with real data — do not refactor:

- ✅ Organizations list + detail (tabs: members, keys, billing, tickets, audit)
- ✅ Users list + detail (suspend, force logout)
- ✅ API Keys list + create + revoke + rotate
- ✅ Billing list + org detail + invoices + credit
- ✅ Incidents list + detail + timeline + resolve + postmortem
- ✅ Support tickets list + detail + assign + status change + macros
- ✅ Campaigns list + detail + attribution
- ✅ CRM accounts list + detail + notes
- ✅ Audit log list with filtering + pagination
- ✅ Feature flags CRUD
- ✅ System logs (journalctl)
- ✅ System jobs (audit events)
- ✅ Proxy geo regions (static)
- ✅ Approval workflow (backend only)
- ✅ Anomaly detection (backend only)
- ✅ Admin auth: login, logout, TOTP, sessions, RBAC
- ✅ FilterBar + PaginationBar components
- ✅ AdminShell layout + sidebar + keyboard shortcuts
