# Pricing Strategy

## Pricing Philosophy

**Principle:** Make the free tier genuinely useful (not crippled), make Pro obviously
worth $12, and make Teams obviously cheaper than the alternative (building it yourself).
Developers are price-sensitive but will pay without friction when the value is clear.

**Anti-patterns to avoid:**
- Credit systems that require mental math ("how many credits does a deep-research fetch cost?")
- Paywalling features that competitors give away free
- Surprise overages — hard limits with clear messages beat silent billing

---

## Tier Structure

### Free — $0/month

**Target:** Individual developers evaluating Fetchium, students, OSS contributors

| Limit | Value |
|-------|-------|
| Fetches | 10/day |
| AI queries | 3/day |
| Fetch modes | Basic (fast, extract, clean) |
| API access | None |
| Support | Community Discord |
| Data retention | None (stateless) |

**What is intentionally unlimited:**
- CLI usage (local SearXNG, no account needed)
- Self-hosted deployments
- Open-source integrations

**Upgrade trigger:** Daily limit hit, with a clear in-terminal and dashboard message.
The limit is low enough that any regular user will hit it within a week of real use.

---

### Pro — $12/month (or $99/year — 31% discount)

**Target:** Individual developers, researchers, indie hackers, freelancers

| Feature | Value |
|---------|-------|
| Fetches | Unlimited (fair use: 5K/day soft cap) |
| AI queries | Unlimited (fair use: 500/day soft cap) |
| Fetch modes | All 7 (fast, extract, clean, deep, pdf, headless, research) |
| API access | 1,000 requests/day |
| API rate limit | 60 req/min |
| Result history | 30-day search history in dashboard |
| Priority support | 24h response via email |
| Webhook support | Up to 3 webhooks |

**Why $12:** Matches the psychological "Netflix tier" — below $15/month is a quick yes
for a developer who finds the tool valuable. Annual plan at $99 improves cash flow and
reduces churn.

**Fair use policy:** Soft caps at 5K fetches/day and 500 AI queries/day. Hitting these
triggers an email, not a hard stop. Consistent over-use prompts a Teams conversation.

---

### Teams — $49/seat/month (min 3 seats; or $399/seat/year)

**Target:** Engineering teams, research groups, AI product teams

| Feature | Value |
|---------|-------|
| Everything in Pro | Included |
| Shared knowledge base | Team-scoped fetch history + saved results |
| Admin dashboard | Usage analytics, per-member stats, billing |
| SSO | SAML 2.0 / OIDC (Okta, Google Workspace, Azure AD) |
| API access | 10,000 requests/day (pooled across team) |
| API rate limit | 200 req/min |
| Result history | 90-day history |
| Webhooks | Unlimited |
| Priority support | 8h response via dedicated Slack channel |
| Audit logs | 90-day activity log |

**Minimum 3 seats** prevents a solo user from gaming Teams pricing. 3 × $49 = $147/month
vs. $12/month solo — the $135 premium buys shared infra and SSO, which teams need.

**Seat-based vs. usage-based for Teams:** Seat-based is predictable for budget owners
at companies. They can approve $X/month for N people. Usage overages create budget anxiety.

---

### API Plan — Pay-as-You-Go

**Target:** Builders with bursty usage, pipelines, automated workflows

**Base rate:** $0.005 per fetch request

| Volume | Price per request | Monthly cost |
|--------|------------------|-------------|
| 1–10K | $0.005 | Up to $50 |
| 10K–100K | $0.004 | $50–$400 |
| 100K–1M | $0.003 | $400–$3,000 |
| 1M+ | $0.002 | $2,000+ (contact sales) |

**AI enrichment add-on:** +$0.005 per request (billed separately)
**No monthly minimum.** Prepay credits in $50 increments; credits never expire.

**Why tiered and not flat:** Encourages high-volume customers to grow. A pipeline builder
testing at 1K/month becomes a 1M/month customer within 6 months if the product works.

---

### Enterprise — Custom

**Target:** Companies with >50 employees using Fetchium in production

| Feature | Value |
|---------|-------|
| Everything in Teams | Included |
| On-premises deployment | Docker or Kubernetes charts |
| Unlimited API | Custom rate limits |
| Custom data retention | Up to 5 years |
| SLA | 99.9% uptime guarantee with financial penalties |
| Dedicated support | Named CSM, dedicated Slack, < 2h response |
| Custom integrations | Paid professional services |
| Security review | SOC 2 report, pen test results, GDPR DPA |
| Invoice billing | Net-30 / Net-60 available |

**Typical deal size:** $20K–$100K/year
**Sales motion:** Async-first (Loom demo, written proposal) — no SDR, no cold calling.
Enterprise deals come from inbound (Pro users whose company notices the spend).

---

## Competitor Pricing Comparison

| Product | Free | Pro/Individual | Team | API |
|---------|------|---------------|------|-----|
| **Fetchium** | 10 fetches/day | $12/mo | $49/seat | $0.003–$0.005/req |
| Perplexity API | — | $20/mo (Pro) | — | $5/1K searches |
| Tavily | — | $25/mo | — | $0.015/search |
| Exa | Limited | $50/mo | $149/seat | $0.01/search |
| SerpAPI | — | $50/mo | — | $0.005/search |
| ScrapingBee | — | $49/mo | — | $0.001/request (no AI) |
| Firecrawl | — | $16/mo | — | $0.002/page (no AI) |

**Fetchium price advantage:**
- 2.5x cheaper than Tavily API per request
- 3x cheaper than Exa per request
- Full AI enrichment included (others charge separately)
- Self-hostable (zero marginal cost for high-volume technical users)

---

## Pricing Psychology & Anchoring

1. **Show annual savings prominently:** "Save $45/year" badge on annual Pro toggle
2. **Teams page default to 5 seats:** $245/month looks reasonable; 3-seat minimum anchors it
3. **API calculator on pricing page:** Let users type their expected volume and see cost
4. **Enterprise card has no price:** "Contact us" — lets sales have the conversation
5. **Free tier is honest:** Don't hide that it's only 10/day. Developers respect honesty.

---

## Discounts & Special Programs

| Program | Discount | Eligibility |
|---------|----------|-------------|
| Annual billing | 31% off Pro, 32% off Teams | Any paying customer |
| OSS maintainers | Free Pro | Verified OSS project maintainer |
| Students | Free Pro | .edu email or GitHub Student Pack |
| Startups (pre-Series A) | 50% off Teams for 12 months | Verified by Crunchbase or LinkedIn |
| HN / PH launch special | 3 months Pro free | Limited to launch week |

---

## Billing Implementation

- **Payment processor:** Stripe (Billing + Payment Links + Customer Portal)
- **Metering:** Stripe Meters for API usage (exact billing, no estimation)
- **Invoicing:** Stripe Invoice for Teams and Enterprise
- **Self-serve portal:** Customers manage their own subscription via Stripe Customer Portal
- **Dunning:** Stripe handles failed payment retries (3 attempts over 7 days)
- **No hidden fees:** All prices shown include payment processing. No "plus tax" surprises.

---

## Pricing Review Cadence

- **Month 3:** Review activation and conversion data — adjust free tier limits if activation > 60%
- **Month 6:** Review Pro churn rate — if > 5%/month, survey churned users
- **Month 12:** Consider adding a $29/mo "Pro+" tier if power users are consistently hitting fair-use caps
- **Year 2:** Revisit per-seat Teams pricing based on enterprise deal data
