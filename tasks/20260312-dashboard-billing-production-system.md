# Dashboard Billing Production System

> **Date:** 2026-03-12
> **Priority:** P1
> **Status:** `DONE — pre-Stripe phase complete, Stripe integration deferred`
> **Area:** `app.fetchium.com` billing
> **Primary surfaces:** dashboard billing page, customer billing API, Stripe sync, invoice history

## Objective

Replace the current billing handoff page with a real customer billing system backed by subscription truth, invoice records, and self-serve payment flows.

This task is complete only when:
- the Billing page is backed by billing/subscription data rather than inferred usage
- customers can self-serve allowed plan changes and payment management
- invoices and billing status are visible in the dashboard
- Stripe and internal subscription state remain synchronized through webhooks and tested reconciliation flows

## Current State

### Dashboard UI

Current billing page:
- [apps/dashboard/src/app/(dashboard)/dashboard/billing/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/billing/page.tsx)

Observed behavior:
- fetches `/api/usage`
- infers current plan from usage response
- shows current usage against monthly request limit
- links users to:
  - `https://fetchium.com/pricing`
  - `mailto:founders@fetchium.com`
- explicitly states that hosted billing changes happen outside the dashboard

This is a usage-informed placeholder, not a billing system.

### Dashboard Customer API

Current customer-facing dashboard proxy surface:
- [apps/dashboard/src/app/api/usage/route.ts](/home/echo/projects/Fetchium/apps/dashboard/src/app/api/usage/route.ts)

Observed behavior:
- only proxies usage summary
- no customer billing read model exists
- no checkout, portal, invoice, or subscription APIs exist in the dashboard app

### Backend Billing Data

Existing admin DB schema:
- [crates/fetchium-api/src/admin/db.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/admin/db.rs)

Existing billing-related tables:
- `organizations`
- `subscriptions`
- `invoices`
- `credits_ledger`
- `payment_events`

Existing relevant fields:
- `subscriptions`: `id`, `org_id`, `plan`, `status`, `current_period_start`, `current_period_end`
- `invoices`: `id`, `org_id`, `amount`, `currency`, `status`, `due_date`, `paid_at`, `created_at`
- `credits_ledger`: `id`, `org_id`, `amount`, `reason`, `granted_by`, `created_at`
- `payment_events`: `id`, `org_id`, `event_type`, `payload`, `created_at`

### Existing Billing Handlers

Admin-only billing handlers:
- [crates/fetchium-api/src/admin/billing.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/admin/billing.rs)

Current capabilities:
- list subscriptions
- fetch billing state for an org
- list invoices for an org
- add credits
- refund audit action
- view webhook log

These are internal staff tools, not customer-safe APIs.

### Product Plan

Planned billing work is already documented in:
- [LAUNCH_TASKS.md](/home/echo/projects/Fetchium/LAUNCH_TASKS.md)
- [PRODUCT_PLAN.md](/home/echo/projects/Fetchium/PRODUCT_PLAN.md)

Current intended product direction:
- Stripe-backed subscriptions
- plans:
  - Free
  - Starter
  - Pro
  - Enterprise
- metered overages
- checkout flow
- customer portal
- invoice history
- webhook-driven plan updates

## Findings

### 1. The current Billing page is intentionally narrow

The dashboard does not fake functionality that does not exist. This is correct.

### 2. Billing truth already exists in an internal domain

The admin DB and handlers prove the product already has billing-oriented entities, but they are not exposed through a customer-safe API surface.

### 3. Usage and billing are currently conflated

The dashboard derives plan state from `/api/usage`, but usage is not a billing truth source.

This is acceptable as a temporary heuristic, but not acceptable in production billing.

### 4. Organization mapping is the missing customer boundary

Billing tables are keyed by `org_id`.

A production customer billing page therefore needs a clear mapping between:
- dashboard session
- customer user or workspace
- organization
- subscription/customer in Stripe

That mapping is not yet defined in the dashboard customer auth model.

## Problem Statement

The Billing page currently behaves like a help center entry point rather than a production billing surface.

Current user limitations:
- cannot see billing status
- cannot see renewal date
- cannot inspect invoices
- cannot update payment method
- cannot self-serve plan upgrades/downgrades
- cannot understand whether plan data comes from billing truth or usage heuristics

Operational risks:
- support becomes the only mutation path
- plan mismatch between usage state and subscription state can go undetected
- product cannot support a clean paid conversion funnel inside the dashboard

## Decision

Build a customer billing domain backed by Stripe and internal organization-scoped billing records.

The billing page should be driven by a customer-safe billing read model.

Do not reuse admin billing endpoints directly in the customer dashboard.

## Scope

### In scope

- customer billing read model
- Stripe checkout session creation
- Stripe customer portal session creation
- invoice history in dashboard
- subscription state sync
- billing page UI replacement
- enterprise/manual fallback path

### Out of scope

- internal admin billing console redesign
- full finance/ERP reporting
- exposing refund/credit operations directly to customers
- replacing Stripe with another billing provider in this task

## Production Requirements

### Functional requirements

1. The Billing page must show billing truth, not inferred plan heuristics.
2. Customers must be able to:
   - view current plan
   - view subscription status
   - view renewal period
   - view invoice history
   - open a customer billing portal
   - begin allowed self-serve upgrades
3. Enterprise/manual billing flows must remain explicit where product policy requires them.
4. Usage context must be shown, but clearly separated from subscription truth.
5. Stripe webhook events must update internal state safely and idempotently.

### Data requirements

The customer billing read model must include:
- `org_id`
- `org_name`
- `plan`
- `subscription_status`
- `billing_interval`
- `current_period_start`
- `current_period_end`
- `stripe_customer_id` or internal equivalent, server-side only
- customer-safe payment method summary
- invoice list
- optional credit/overage summary
- usage summary for context

### Non-functional requirements

- webhook handlers must be signature-verified
- webhook processing must be idempotent
- customer dashboard must never expose provider secrets
- billing UI must degrade safely if Stripe or sync data is temporarily stale
- free-tier users without billing records must still see a coherent state

## Target Product Model

### Billing owner model

Billing must be organization-scoped, not API-key-scoped.

Reason:
- existing tables are keyed by `org_id`
- invoices and subscriptions belong to a customer organization
- API keys can change without changing the billing account

### Plan model

Planned plans from product docs:
- `free`
- `starter`
- `pro`
- `enterprise`

Recommended policy:
- `free`, `starter`, `pro` can be self-serve
- `enterprise` remains contact-sales/contact-billing or invite-based unless explicitly implemented otherwise

### Usage model

Usage remains a separate domain.

Billing page should consume usage only as supporting context:
- current month usage
- limit
- possible overage or upgrade recommendation

It must not use usage as the source of plan identity.

## Source-of-Truth Model

### External truth

Stripe owns:
- customer
- subscription lifecycle
- payment method
- invoice objects
- hosted portal state

### Internal truth

Fetchium owns:
- organization mapping
- plan normalization
- internal subscription snapshot
- invoice summary cache or mirror
- credit ledger
- payment event audit log

### Customer dashboard truth

The dashboard billing page should read from a normalized customer billing view in Fetchium, not directly from Stripe from the browser.

## Proposed Architecture

### Customer-facing endpoints

Add customer-safe endpoints such as:
- `GET /v1/dashboard/billing`
- `POST /v1/dashboard/billing/checkout`
- `POST /v1/dashboard/billing/portal`

Optional later:
- `GET /v1/dashboard/billing/invoices`
- `GET /v1/dashboard/billing/usage-impact`

### Suggested `GET /v1/dashboard/billing` response

```json
{
  "meta": {
    "request_id": "req_123",
    "status": "ok",
    "endpoint": "/v1/dashboard/billing",
    "duration_ms": 12
  },
  "organization": {
    "id": "org_123",
    "name": "Acme Research"
  },
  "subscription": {
    "plan": "pro",
    "status": "active",
    "billing_interval": "monthly",
    "current_period_start": "2026-03-01T00:00:00Z",
    "current_period_end": "2026-04-01T00:00:00Z",
    "self_serve_manageable": true
  },
  "payment_method": {
    "brand": "visa",
    "last4": "4242",
    "exp_month": 12,
    "exp_year": 2027
  },
  "usage": {
    "requests_this_month": 1033,
    "monthly_limit": 250000,
    "quota_remaining": 248967
  },
  "credits": {
    "balance_cents": 0
  },
  "invoices": [
    {
      "id": "inv_123",
      "amount_cents": 7900,
      "currency": "usd",
      "status": "paid",
      "invoice_date": "2026-03-01T00:00:00Z",
      "paid_at": "2026-03-01T00:01:02Z",
      "hosted_invoice_url": "https://billing.stripe.com/..."
    }
  ],
  "actions": {
    "can_upgrade": true,
    "can_downgrade": true,
    "can_open_portal": true,
    "requires_sales_contact": false
  }
}
```

### Suggested `POST /v1/dashboard/billing/checkout`

Request:

```json
{
  "target_plan": "pro"
}
```

Response:

```json
{
  "checkout_url": "https://checkout.stripe.com/..."
}
```

### Suggested `POST /v1/dashboard/billing/portal`

Response:

```json
{
  "portal_url": "https://billing.stripe.com/..."
}
```

## Required Data Model Additions

The existing admin DB schema is close but not enough for a production customer billing system.

### Existing tables can be retained

- `subscriptions`
- `invoices`
- `credits_ledger`
- `payment_events`

### Recommended additions

Add fields or companion tables for:
- external billing provider IDs
  - `stripe_customer_id`
  - `stripe_subscription_id`
  - `stripe_invoice_id`
  - `stripe_price_id`
- billing interval
- cancellation status and timestamps
- default payment method summary cache
- invoice hosted URL or provider link
- idempotency keys / processed webhook event tracking if `payment_events` is not sufficient

### Minimal normalized internal model

`subscriptions` should evolve to include:
- internal `id`
- `org_id`
- `provider`
- `provider_customer_id`
- `provider_subscription_id`
- `plan`
- `status`
- `billing_interval`
- `current_period_start`
- `current_period_end`
- `cancel_at_period_end`
- `created_at`
- `updated_at`

`invoices` should evolve to include:
- internal `id`
- `org_id`
- `provider_invoice_id`
- `amount`
- `currency`
- `status`
- `hosted_invoice_url`
- `due_date`
- `paid_at`
- `created_at`

## Customer Identity Mapping

This is the key prerequisite.

The dashboard must know which `org_id` the authenticated customer belongs to.

Required design decision:
- extend dashboard auth/session model to resolve an organization context for the customer session

Without this, billing cannot be scoped safely.

Minimum needed in session or derived customer context:
- `customer_user_id`
- `org_id`
- `org_role`

## Stripe Integration Requirements

### Provider configuration

Expected environment variables, aligned with project planning:
- `STRIPE_SECRET_KEY`
- `STRIPE_WEBHOOK_SECRET`
- `STRIPE_PRICE_STARTER`
- `STRIPE_PRICE_PRO`
- `STRIPE_PRICE_ENTERPRISE`

### Checkout flow

1. Customer selects upgrade target.
2. Backend validates org/session eligibility.
3. Backend creates Stripe Checkout session.
4. Customer is redirected to Stripe.
5. Stripe webhook confirms subscription state.
6. Internal subscription snapshot is updated.
7. Dashboard reflects updated plan.

### Portal flow

1. Customer clicks manage billing.
2. Backend creates Stripe customer portal session for the org's customer.
3. Customer is redirected to Stripe portal.

### Webhook flow

Required events, aligned with planning:
- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`
- `invoice.payment_failed`
- `invoice.payment_succeeded`
- optionally `checkout.session.completed`
- optionally `invoice.finalized`

Required behavior:
- signature verification
- idempotent processing
- durable event log to `payment_events`
- internal subscription/invoice state update

## Required Backend Changes

### 1. Customer billing read model

Add a customer-safe billing service layer that:
- resolves org from authenticated customer context
- reads normalized subscription state
- reads invoice summary
- merges usage summary for context

### 2. Checkout endpoint

Add a backend endpoint to create Stripe Checkout sessions for eligible plans.

Validation rules:
- free -> starter/pro allowed
- starter -> pro allowed
- enterprise may require manual path depending on policy
- invalid downgrade/upgrade transitions rejected cleanly

### 3. Portal endpoint

Add a backend endpoint to create a Stripe customer portal session for the org's Stripe customer.

### 4. Webhook endpoint

Add:
- `POST /webhooks/stripe`

Responsibilities:
- verify signature
- persist payment event
- update subscription snapshot
- update invoice records
- trigger entitlement/plan updates if applicable

### 5. Reconciliation path

Add a manual or scheduled reconciliation routine to repair drift between Stripe and internal state.

At minimum:
- detect missing internal subscription snapshot
- detect invoice mismatch
- reconcile org plan state

## Required Dashboard Changes

### 1. Replace placeholder page

Update:
- [apps/dashboard/src/app/(dashboard)/dashboard/billing/page.tsx](/home/echo/projects/Fetchium/apps/dashboard/src/app/(dashboard)/dashboard/billing/page.tsx)

Required behavior:
- fetch customer billing read model
- render subscription summary
- render invoice history
- render payment method summary
- render usage context separately
- offer self-serve upgrade/manage actions where allowed

### 2. Add dashboard billing proxy routes

Create:
- `apps/dashboard/src/app/api/dashboard/billing/route.ts`
- `apps/dashboard/src/app/api/dashboard/billing/checkout/route.ts`
- `apps/dashboard/src/app/api/dashboard/billing/portal/route.ts`

### 3. Preserve enterprise/manual path

The page must still support:
- `Contact billing`
- `Contact sales`

for enterprise plans or unsupported transitions.

### 4. Improve state handling

Required UI states:
- free user with no billing record
- active paid subscription
- payment failed / action required
- canceled but active until period end
- portal unavailable
- invoice list empty
- billing backend unavailable

## UX Requirements

The page should clearly separate:
- Subscription
- Payment method
- Invoices
- Usage context
- Plan actions

The page must not imply that:
- usage equals billing
- enterprise pricing is self-serve if it is not

## Rollout Plan

### Phase 1: Billing foundation

1. Finalize organization-to-customer mapping.
2. Add provider IDs and missing billing fields.
3. Add Stripe configuration and secret management.
4. Add webhook endpoint with signature verification.

### Phase 2: Internal synchronization

1. Implement subscription snapshot updates.
2. Implement invoice synchronization.
3. Implement payment event logging and idempotency.
4. Add reconciliation tooling.

### Phase 3: Customer billing API

1. Add `GET /v1/dashboard/billing`.
2. Add checkout endpoint.
3. Add portal endpoint.
4. Add entitlement update hooks if plan changes affect product access.

### Phase 4: Dashboard UI

1. Replace the placeholder Billing page.
2. Add invoice history UI.
3. Add manage billing / upgrade actions.
4. Add free-tier and enterprise-specific states.

### Phase 5: Hardening

1. Handle failed payment and grace states.
2. Add alerting for webhook failures.
3. Add admin/support visibility into customer billing state.

## Test Plan

### Backend tests

- subscription snapshot creation/update
- invoice persistence
- webhook signature rejection
- webhook idempotency
- checkout eligibility rules
- org scoping rules

### Dashboard tests

- free-tier empty state
- active paid subscription state
- invoice rendering
- portal button behavior
- payment-failed state rendering
- enterprise manual-contact path

### Integration tests

- checkout session created for eligible org
- webhook updates internal plan state
- billing page reflects updated plan after webhook processing
- invoice appears in dashboard after successful payment

## Acceptance Criteria

- The Billing page is backed by billing/subscription truth, not `/api/usage` alone.
- Customers can view current plan, status, renewal period, and invoices.
- Customers can open a customer billing portal.
- Self-serve plan changes work for allowed plans.
- Enterprise/manual billing paths remain explicit where required.
- Stripe webhook processing is verified, idempotent, and auditable.
- Dashboard never exposes billing provider secrets or admin-only operations.

## Risks

- without clean org mapping, customer billing access control will be unsafe
- webhook delivery failures can create stale dashboard state without reconciliation
- mixing free, self-serve paid, and enterprise manual plans can produce ambiguous UX if policy is not explicit

## Open Decisions

- exact self-serve plan transition matrix
- whether overages are auto-billed or credit-based first
- whether invoices are mirrored fully or partially from Stripe
- whether free-tier users get billing profile records before first paid conversion

## Recommended Implementation Order

1. org/customer identity mapping
2. Stripe and subscription schema foundation
3. webhook and sync pipeline
4. customer billing read model
5. checkout and portal endpoints
6. dashboard billing page replacement
