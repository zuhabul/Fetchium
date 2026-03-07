# Sprint 06: Monetization

**Duration:** 2 weeks
**Theme:** First paying customer, billing infrastructure, 10 Pro subscribers
**Goal:** Fetchium earns its first dollar of revenue with self-serve billing end-to-end
**Dependency:** Sprint 05 complete (1K installs, 200+ active users from public launch)

---

## Context

Revenue is not just a number — it is validation that Fetchium solves a real problem
worth paying for. The first $1 changes the psychological frame for the entire project.

This sprint builds the entire billing infrastructure from scratch: Stripe integration,
Pro tier enforcement, API key dashboard, billing docs, and the upgrade flow that turns
a free-tier user into a paying customer.

**Target metrics:**
- First dollar of revenue
- 10 Pro subscribers at $12/month = $120 MRR
- API key dashboard live
- Billing docs published

---

## Pre-Sprint: Stripe Account Setup

Do this before the sprint starts — Stripe account approval takes 1–3 business days.

**Task 0.1 — Create Stripe account**
- Go to stripe.com, create an account with the business email (`billing@fetchium.com`)
- Complete business verification (entity type, EIN, banking info)
- Important: verify with your real business entity from Sprint 01 — Stripe will eventually ask
- Enable "Test mode" — all development happens here until go-live

**Task 0.2 — Create products in Stripe**

In Stripe Dashboard → Products:

**Product 1: Fetchium Pro**
- Name: "Fetchium Pro"
- Description: "Unlimited fetches, all 7 modes, API access (1K req/day), priority support"
- Price 1: $12.00/month (recurring, USD) — ID: `price_pro_monthly`
- Price 2: $99.00/year (recurring, USD) — ID: `price_pro_yearly`

**Product 2: Fetchium Teams**
- Name: "Fetchium Teams"
- Description: "Per-seat plan: shared knowledge base, SSO, admin dashboard, 10K API req/day"
- Price: $49.00/month per seat — ID: `price_teams_monthly_per_seat`
- Minimum quantity: 3 seats

**Product 3: API Credits (PAYG)**
- Name: "Fetchium API Credits"
- Pricing model: "Usage-based" via Stripe Meter
- Unit: "api_request"
- Per-unit price: $0.005

---

## Week 1: Core Billing Infrastructure

### Day 1–2: Stripe Integration

**Task 6.1 — Add Stripe SDK to fetchium-api**

```toml
# In Cargo.toml workspace dependencies
stripe = "0.26"  # stripe-rust crate
```

**Task 6.2 — Stripe webhook handler**

All billing state changes come through Stripe webhooks. Build this first.

```rust
// In fetchium-api/src/billing/webhook.rs
pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode> {
    let sig = headers.get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(FetchiumError::Unauthorized)?;

    let event = stripe::Webhook::construct_event(
        &body,
        sig,
        &state.config.stripe.webhook_secret,
    )?;

    match event.type_ {
        EventType::CustomerSubscriptionCreated => on_subscription_created(&state, event).await,
        EventType::CustomerSubscriptionDeleted => on_subscription_deleted(&state, event).await,
        EventType::CustomerSubscriptionUpdated => on_subscription_updated(&state, event).await,
        EventType::InvoicePaymentSucceeded => on_payment_succeeded(&state, event).await,
        EventType::InvoicePaymentFailed => on_payment_failed(&state, event).await,
        _ => Ok(StatusCode::OK),
    }
}
```

**Task 6.3 — Subscription state in database**

Add subscription tracking to the auth database:

```sql
CREATE TABLE IF NOT EXISTS subscriptions (
    user_id         TEXT NOT NULL REFERENCES users(id),
    stripe_customer_id   TEXT UNIQUE,
    stripe_sub_id       TEXT UNIQUE,
    plan            TEXT NOT NULL DEFAULT 'free',  -- 'free', 'pro', 'teams', 'enterprise'
    status          TEXT NOT NULL DEFAULT 'active', -- 'active', 'past_due', 'canceled'
    current_period_end  INTEGER,  -- Unix timestamp
    seat_count      INTEGER DEFAULT 1,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);
```

**Task 6.4 — Plan enforcement middleware**

Every API request must check the user's plan against what they are trying to do:

```rust
pub async fn enforce_plan_limits(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    request: Request,
    next: Next,
) -> Response {
    let sub = state.auth_db.get_subscription(&auth_user.user_id).await?;
    let endpoint = request.uri().path();

    match (sub.plan.as_str(), endpoint) {
        ("free", "/api/v1/fetch") => {
            // Check daily fetch count
            let count = state.auth_db.daily_fetch_count(&auth_user.user_id).await?;
            if count >= 10 {
                return upgrade_required_response("You've used your 10 free fetches today");
            }
        }
        ("free", "/api/v1/search") => {
            let count = state.auth_db.daily_ai_count(&auth_user.user_id).await?;
            if count >= 3 {
                return upgrade_required_response("You've used your 3 free AI queries today");
            }
        }
        ("free", "/api/v1/keys") => {
            return upgrade_required_response("API key access requires Pro ($12/month)");
        }
        _ => {} // Pro and above: no limits (soft caps enforced separately)
    }

    next.run(request).await
}

fn upgrade_required_response(message: &str) -> Response {
    let body = serde_json::json!({
        "error": "plan_limit_reached",
        "message": message,
        "upgrade_url": "https://fetchium.com/pricing",
        "upgrade_text": "Upgrade to Pro for unlimited access"
    });
    (StatusCode::PAYMENT_REQUIRED, Json(body)).into_response()
}
```

### Day 3–4: Checkout Flow

**Task 6.5 — Stripe Checkout session endpoint**

```rust
// POST /api/v1/billing/checkout
pub async fn create_checkout_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>> {
    let price_id = match req.plan.as_str() {
        "pro_monthly" => &state.config.stripe.price_pro_monthly,
        "pro_yearly"  => &state.config.stripe.price_pro_yearly,
        "teams"       => &state.config.stripe.price_teams_monthly,
        _ => return Err(FetchiumError::BadRequest("Invalid plan".into())),
    };

    let session = stripe::CheckoutSession::create(
        &state.stripe_client,
        CreateCheckoutSession {
            mode: Some(CheckoutSessionMode::Subscription),
            line_items: Some(vec![CreateCheckoutSessionLineItems {
                price: Some(price_id.clone()),
                quantity: Some(req.seats.unwrap_or(1) as u64),
                ..Default::default()
            }]),
            success_url: Some("https://app.fetchium.com/billing/success?session_id={CHECKOUT_SESSION_ID}"),
            cancel_url:  Some("https://fetchium.com/pricing"),
            customer_email: Some(auth_user.email.as_str()),
            metadata: Some([("user_id".into(), auth_user.user_id.clone())].into()),
            ..Default::default()
        },
    ).await?;

    Ok(Json(CheckoutResponse { checkout_url: session.url.unwrap() }))
}
```

**Task 6.6 — Customer Portal endpoint**

For existing subscribers to manage their subscription:

```rust
// POST /api/v1/billing/portal
pub async fn create_portal_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<PortalResponse>> {
    let sub = state.auth_db.get_subscription(&auth_user.user_id).await?;
    let customer_id = sub.stripe_customer_id
        .ok_or(FetchiumError::NotFound("No subscription found".into()))?;

    let session = stripe::BillingPortalSession::create(
        &state.stripe_client,
        CreateBillingPortalSession {
            customer: customer_id,
            return_url: Some("https://app.fetchium.com/settings/billing"),
            ..Default::default()
        },
    ).await?;

    Ok(Json(PortalResponse { portal_url: session.url }))
}
```

### Day 5: Upgrade CTA in Dashboard

**Task 6.7 — Usage meter in dashboard**

In the `fetchium-dashboard` web app, add a usage meter to the sidebar:

```tsx
// components/UsageMeter.tsx
export function UsageMeter({ usage }: { usage: UsageSummary }) {
  const fetchPercent = (usage.dailyFetches / usage.fetchLimit) * 100;

  return (
    <div className="p-4 bg-slate-800 rounded-lg">
      <div className="flex justify-between text-sm text-slate-400 mb-1">
        <span>Daily fetches</span>
        <span>{usage.dailyFetches} / {usage.fetchLimit}</span>
      </div>
      <div className="w-full bg-slate-700 rounded-full h-2">
        <div
          className={`h-2 rounded-full ${fetchPercent > 80 ? 'bg-amber-500' : 'bg-emerald-500'}`}
          style={{ width: `${Math.min(fetchPercent, 100)}%` }}
        />
      </div>
      {fetchPercent >= 100 && (
        <div className="mt-3">
          <p className="text-sm text-amber-400 mb-2">Daily limit reached</p>
          <a
            href="/billing/upgrade"
            className="block w-full text-center bg-indigo-600 hover:bg-indigo-500 text-white text-sm py-2 rounded-md"
          >
            Upgrade to Pro — $12/month
          </a>
        </div>
      )}
    </div>
  );
}
```

**Task 6.8 — Billing settings page**

`/settings/billing` page in the dashboard:
- Current plan (with badge: Free / Pro / Teams)
- Usage this month (fetches, AI queries, API calls)
- Upgrade button (links to Stripe Checkout)
- Manage subscription button (links to Stripe Portal)
- Invoice history (pulled from Stripe API)

---

## Week 2: API Key Dashboard & Billing Docs

### Day 6–7: API Key Dashboard

**Task 6.9 — API key management UI**

Pro and higher users can create, view, and revoke API keys.

**Database schema:**
```sql
CREATE TABLE IF NOT EXISTS api_keys (
    id          TEXT PRIMARY KEY,  -- 'fxm_' + 64 hex chars
    user_id     TEXT NOT NULL REFERENCES users(id),
    name        TEXT NOT NULL,     -- user-defined label: "My LangChain Agent"
    last_used   INTEGER,           -- Unix timestamp
    created_at  INTEGER NOT NULL,
    revoked_at  INTEGER            -- NULL = active
);
```

**UI features:**
- List all API keys with name, creation date, last used
- Create new key (show full key once on creation — never again)
- Copy key to clipboard button
- Revoke key button (with confirmation modal)
- Usage stats per key (last 7 days fetch count)

**Task 6.10 — API key usage analytics**

Simple stats page showing:
- Total requests this month (by API key)
- Requests by endpoint (fetch / search / ai)
- P95 latency for your requests
- Error rate

This answers "Is my integration working?" without needing to check server logs.

### Day 8–9: Billing Documentation

**Task 6.11 — Write billing docs (5 pages)**

**Page 1: Pricing overview** (`docs.fetchium.com/billing`)
- Free tier limits
- Pro plan features and limits
- Teams plan features and limits
- API PAYG pricing with calculator
- FAQ: "What happens when I hit the limit?" "Can I downgrade?" "Do you offer refunds?"

**Page 2: Upgrading to Pro** (`docs.fetchium.com/billing/upgrade`)
- Step-by-step: Log in → Settings → Billing → Upgrade
- What happens immediately after upgrade
- Annual vs. monthly comparison

**Page 3: API key setup** (`docs.fetchium.com/billing/api-keys`)
- How to create an API key
- How to use it in requests (`Authorization: Bearer fxm_...`)
- Security best practices (never commit to git, use env vars)
- Code examples in Python, TypeScript, curl

**Page 4: Managing your subscription** (`docs.fetchium.com/billing/manage`)
- How to access the Stripe portal
- How to update payment method
- How to cancel
- What happens to data after cancellation

**Page 5: Enterprise billing** (`docs.fetchium.com/billing/enterprise`)
- What enterprise includes
- How to contact sales (`sales@fetchium.com`)
- Invoice billing process
- SLA and DPA availability

### Day 10: Revenue Push

**Task 6.12 — Targeted upgrade emails**

Identify users who have hit their free tier limit at least once:
```sql
SELECT u.email, u.name, COUNT(*) as limit_hits
FROM rate_limit_events rle
JOIN users u ON u.id = rle.user_id
WHERE rle.event_type = 'daily_limit_reached'
  AND rle.created_at > strftime('%s', 'now', '-7 days')
GROUP BY u.id
HAVING limit_hits >= 2
ORDER BY limit_hits DESC;
```

Email these users (target: 30–50 from launch traffic):

> Subject: You've been hitting Fetchium's limits — here's how to remove them
>
> Hey [name],
>
> We noticed you've hit Fetchium's free tier limit a couple of times this week.
> That means the tool is actually useful to you — which is great.
>
> Pro removes all daily limits, adds API access, and prioritizes your requests.
> It's $12/month (or $99/year, saving $45).
>
> Upgrade: fetchium.com/billing/upgrade
>
> Questions? Reply to this email — I read everything.
> — [Founder name]

Expected conversion: 5–15% of limit-hit users who receive this email.

**Task 6.13 — Post in communities about self-serve billing**

"We just added self-serve billing to Fetchium — you can now upgrade to Pro directly
from the dashboard. Here's what you get for $12/month: [list]"

Post to: Discord `#announcements`, Twitter/X, HN (as a comment in any relevant thread)

### Day 11–12: Stripe Go-Live

**Task 6.14 — Switch from test mode to live mode**

Stripe test mode uses fake cards. Before going live:
- [ ] Test the full checkout flow with a real Visa (charge $1, immediately refund)
- [ ] Test webhook delivery in production (Stripe Dashboard → Webhooks → Send test event)
- [ ] Test the customer portal flow
- [ ] Test subscription cancellation and verify plan downgrades to free
- [ ] Test failed payment flow (Stripe test card: 4000000000000341)

Then: toggle off test mode in the Stripe Dashboard. Update the environment variable:
```bash
# ***REMOVED***
STRIPE_SECRET_KEY=sk_live_...   # was sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_... # regenerate for live mode
```

**Task 6.15 — Go-live checklist**

- [ ] Stripe live mode enabled
- [ ] Webhook endpoint registered in Stripe live mode
- [ ] Upgrade CTA in dashboard links to real checkout
- [ ] Customer portal link works for existing subscribers
- [ ] Billing docs published
- [ ] API key dashboard works for Pro users
- [ ] Invoice email configured in Stripe (send receipts automatically)

### Day 13–14: First Revenue Celebration + Analysis

**Task 6.16 — Monitor first week of live billing**

Watch daily:
- New subscribers in Stripe Dashboard
- Webhook delivery success rate (target: 100%)
- Any failed payments
- Support emails about billing

**Task 6.17 — Revenue retrospective**

At end of sprint:
- How many paying customers?
- What plan do they use? (monthly vs. annual)
- What was the conversion rate from limit-hit email?
- What was the conversion rate from dashboard CTA?
- What friction did customers report during checkout?

---

## Definition of Done

Sprint 06 is complete when:
- [ ] Stripe integration is live (production mode)
- [ ] At least 1 paying customer has been processed successfully
- [ ] 10 Pro subscribers ($120 MRR)
- [ ] Checkout → webhook → plan upgrade flow works end-to-end
- [ ] API key dashboard is live for Pro users
- [ ] Billing docs are published (all 5 pages)
- [ ] Customer portal allows self-serve plan management
- [ ] Zero billing-related P0 incidents

---

## Revenue Velocity Tracking

After Sprint 06, track weekly:

| Week | New Subscribers | Churned | MRR | Note |
|------|----------------|---------|-----|------|
| Launch +0 | — | — | $0 | |
| Launch +1 | ? | 0 | ? | Sprint 06 start |
| Launch +2 | ? | ? | ? | Billing live |
| Launch +4 | ? | ? | ? | Target: $120 MRR |
| Launch +8 | ? | ? | ? | Target: $500 MRR |

**If MRR growth is < $50/week after billing goes live:**
- Run a user survey: "Why haven't you upgraded?" — single most valuable data point
- Experiment with pricing: try a 14-day free trial for Pro (no credit card)
- Check the upgrade flow for bugs: can a real person complete checkout in < 2 minutes?

**If MRR growth is > $200/week:**
- Start building Teams tier immediately (the next set of customers are waiting)
- Contact the top 10 users about an early-adopter enterprise conversation
