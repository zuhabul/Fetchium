# Metrics & KPIs

## North Star Metric

**Weekly Active Fetches (WAF)** — the number of unique fetch/AI-query requests made
by active users in a rolling 7-day window.

**Why WAF:**
- Directly measures whether the product is being used to do real work
- Grows with both user acquisition (more users) and engagement (more fetches/user)
- Predictive of revenue: high WAF users convert at 4x the rate of low-WAF users
- Easy to explain to any stakeholder: "X developers fetched Y pieces of content this week"

**WAF Targets:**

| Period | WAF Target |
|--------|-----------|
| Month 1 | 500 |
| Month 3 | 5,000 |
| Month 6 | 25,000 |
| Month 12 | 100,000 |
| Month 24 | 500,000 |

---

## Metric Framework

Metrics are organized in three layers: **Acquisition → Engagement → Revenue**.
Review at different cadences: daily (health checks), weekly (trends), monthly (strategy).

---

## Layer 1: Acquisition Metrics

### Primary
| Metric | Definition | Target (M1) | Target (M6) | Target (M12) |
|--------|-----------|------------|------------|-------------|
| New signups | Accounts created per week | 50 | 500 | 2,000 |
| Organic traffic | Unique visitors to docs/landing | 1K | 10K | 50K |
| GitHub stars | Cumulative repo stars | 200 | 3,000 | 10,000 |
| CLI installs | `cargo install` + `pip install` combined | 300 | 5,000 | 20,000 |

### Channel Attribution
| Channel | Expected % of Signups | Track via |
|---------|----------------------|----------|
| GitHub / OSS | 40% | UTM + referrer |
| Hacker News | 20% | UTM on HN links |
| Blog / SEO | 15% | UTM + search console |
| Discord / community | 15% | Referral codes |
| Direct / word of mouth | 10% | Default attribution |

### Top-of-Funnel Health Signals
- **Bounce rate on landing page:** Target < 60%
- **Docs → GitHub conversion:** Target > 20% (visitors who click the GitHub link)
- **README → install rate:** Target > 15% (GitHub visitors who proceed to install)

---

## Layer 2: Activation Metrics

**Activation event:** User completes their first successful fetch (any mode) within 24 hours of signup.

### Activation Funnel
```
Signup
  ↓ (target: 70%) Install CLI or create API key
  ↓ (target: 60%) Run first command / make first API call
  ↓ (target: 55%) Get a successful result (non-error response)
  ↓ (target: 40%) Make a second fetch within 24 hours  ← Activation event
  ↓ (target: 25%) Return within 7 days (retained user)
```

| Activation Metric | Definition | Target (M1) | Target (M6) |
|-------------------|-----------|------------|------------|
| Activation rate | % of signups who activate in 24h | 35% | 55% |
| Time-to-first-fetch | Median minutes from signup to first fetch | < 10 min | < 5 min |
| Setup completion rate | % who complete `fetchium quickstart` | 50% | 65% |
| First-fetch success rate | % of first fetches that return a result | 80% | 92% |

### Activation Failure Modes to Track
- Authentication errors on first use (fix: better default config)
- Timeout on first fetch (fix: faster default backend selection)
- Confusing CLI output (fix: better formatting, `--help` improvements)
- SearXNG not running (fix: auto-detect and suggest `fetchium setup`)

---

## Layer 3: Engagement Metrics

### Daily Engagement
| Metric | Definition | Target (M3) | Target (M12) |
|--------|-----------|------------|-------------|
| DAU | Unique users with >= 1 fetch | 100 | 2,000 |
| WAU | Unique users with >= 1 fetch in 7 days | 500 | 8,000 |
| MAU | Unique users with >= 1 fetch in 30 days | 1,500 | 20,000 |
| DAU/MAU ratio | Stickiness | 6% | 10% |
| Fetches per DAU | Avg fetches per active user per day | 8 | 20 |

### Feature Adoption
| Feature | Definition | Target (M6) |
|---------|-----------|------------|
| AI mode usage | % of fetches using AI enrichment | 40% |
| Deep mode usage | % of fetches using deep/research mode | 15% |
| API vs CLI ratio | % of fetches via API (vs CLI) | 30% |
| Multi-mode usage | Users who have tried >= 3 modes | 25% |

### Retention Cohorts (Monthly)
| Cohort Age | Target Retention |
|-----------|----------------|
| Month 1 (D30 retention) | 30% |
| Month 3 (D90 retention) | 20% |
| Month 6 (D180 retention) | 15% |
| Month 12 (D365 retention) | 10% |

These benchmarks are calibrated to developer tools — not consumer apps. Developer
tools typically see lower absolute retention but higher LTV per retained user.

---

## Layer 4: Revenue Metrics

### Monthly Revenue Dashboard
| Metric | Definition | Target (M6) | Target (M12) |
|--------|-----------|------------|-------------|
| MRR | Monthly recurring revenue | $5K | $30K |
| ARR | MRR × 12 | $60K | $360K |
| ARPU (Pro) | Avg revenue per Pro user | $12 | $14 (expansion) |
| ARPU (all paying) | Avg revenue across all tiers | $15 | $20 |
| MRR growth rate | Month-over-month MRR change | 20% | 15% |

### Conversion Funnel
| Step | Metric | Target (M6) |
|------|--------|------------|
| Signup → Activated | Activation rate | 55% |
| Activated → Regular (D7 return) | D7 retention | 35% |
| Regular → Free limit hit | Free limit hit rate | 60% of regular users |
| Free limit hit → Pro upgrade | Upgrade conversion | 12% |
| Overall Free → Pro conversion | All signups | ~3% |

### Churn & Expansion
| Metric | Definition | Target |
|--------|-----------|--------|
| Monthly Pro churn | Pro users who cancel / total Pro users | < 5% |
| Monthly Teams churn | Teams accounts who cancel | < 3% |
| Net Revenue Retention (NRR) | (Ending MRR − Churned + Expansion) / Beginning MRR | > 105% |
| Expansion MRR | Revenue added from existing customers (seat additions, tier upgrades) | 15% of new MRR |

**NRR > 100% means the existing customer base grows revenue even with no new customers.**
This is the most important long-term SaaS health metric.

---

## Layer 5: Product Quality Metrics

### Performance
| Metric | Definition | Target |
|--------|-----------|--------|
| P50 latency (fast mode) | Median fetch time | < 1.5s |
| P95 latency (fast mode) | 95th percentile fetch time | < 4s |
| P50 latency (AI mode) | Median AI-enriched fetch time | < 8s |
| P95 latency (AI mode) | 95th percentile AI mode | < 20s |
| Error rate | % of API requests returning 5xx | < 0.5% |
| Uptime | Monthly availability | > 99.9% |

### Quality
| Metric | Definition | Target |
|--------|-----------|--------|
| Content extraction success | % of fetches with non-empty content | > 95% |
| AI answer relevance (self-reported) | Thumbs up % on AI results | > 75% |
| Fetch mode fallback rate | % of fetches that fall back to a secondary mode | < 10% |

### NPS
- Collect NPS via in-product prompt after 30 days of use (Typeform or Delighted)
- **Target NPS: > 40** (good for developer tools; great is > 50)
- Survey every 90 days for existing customers; monthly for churned users
- Act on every Detractor score (< 7) with a personal email within 48 hours

---

## Instrumentation Plan

### What to Track (Events)
```
fetch_started      { mode, query_length, user_tier, timestamp }
fetch_completed    { mode, latency_ms, content_tokens, ai_used, success }
fetch_failed       { mode, error_type, error_code }
ai_query_started   { model, query_length, context_tokens }
ai_query_completed { model, latency_ms, answer_tokens, success }
user_signed_up     { source, utm_medium, utm_campaign }
user_activated     { time_to_activate_minutes }
plan_upgraded      { from_tier, to_tier, trigger }
plan_downgraded    { from_tier, to_tier, reason }
api_key_created    { }
rate_limit_hit     { tier, endpoint }
```

### Tooling (in order of deployment)
1. **Month 1:** Structured logs (stdout JSON) → Loki + Grafana (already on server)
2. **Month 2:** PostHog (self-hosted) for product analytics — free, open-source
3. **Month 4:** Stripe Dashboard for revenue metrics
4. **Month 6:** Custom metrics dashboard in hsx-dashboard app

### Weekly Metrics Review (Solo Founder Process)
Every Monday, 30 minutes:
1. Check WAF vs. prior week — is it growing?
2. Review top 5 errors from logs — any new failure patterns?
3. Check MRR in Stripe — any new subscribers or cancellations?
4. Review GitHub issues opened — any blocking bugs?
5. Check NPS scores received — any detractors to follow up with?

---

## Metric Anti-Patterns to Avoid

- **Vanity metrics:** Total signups without activation rate tells you nothing useful
- **Averaging latency:** Always use percentiles (P95, P99) — averages hide tail latency
- **MRR without churn:** High MRR with high churn is a leaky bucket
- **DAU without stickiness:** DAU/MAU ratio reveals real engagement vs. tourists
- **Feature usage without activation correlation:** Track which features predict retention
