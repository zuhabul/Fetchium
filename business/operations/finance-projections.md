# Finance Projections & Fundraising Strategy

## Revenue Streams Summary

| Stream | Year 1 | Year 2 | Year 3 |
|--------|--------|--------|--------|
| Pro subscriptions ($12/mo) | $20K | $150K | $800K |
| Teams subscriptions ($49/seat) | $8K | $150K | $1.2M |
| API usage (PAYG) | $5K | $80K | $600K |
| Enterprise contracts | $0 | $100K | $2.4M |
| Professional services | $0 | $20K | $100K |
| **Total Revenue** | **$33K** | **$500K** | **$5.1M** |

Note: Year 1 revenue is the cumulative recognized revenue across 12 months of ramp,
not ARR at month 12. ARR at end of Year 1 is ~$120K; cumulative receipts ~$33K.

---

## Cost Structure

### Infrastructure Costs

| Phase | Monthly Infra | Annual |
|-------|--------------|--------|
| Year 1 (solo server + CDN) | $100–200 | $1,500 |
| Year 2 (multi-region, Redis, Postgres) | $300–500 | $4,500 |
| Year 3 (Kubernetes, 4 regions) | $1,000–2,000 | $18,000 |

### AI API Costs (Variable with Revenue)

| Year | Est. Fetches | AI API Cost | % of Revenue |
|------|-------------|------------|-------------|
| Year 1 | 500K total | $1,500 | ~5% |
| Year 2 | 5M total | $10,000 | ~2% |
| Year 3 | 50M total | $75,000 | ~1.5% |

AI API costs decline as % of revenue due to:
1. Volume discounts from providers
2. Shift to local inference (Llama, Gemini CLI) for free tier and batch jobs
3. Better caching — repeat queries served from Redis, not re-processed

### Salary Costs

| Phase | Team Size | Monthly Salaries | Annual |
|-------|-----------|-----------------|--------|
| Year 1 (Month 1–10) | 1 (founder unpaid) | $0 | $0 |
| Year 1 (Month 11–12) | 1 (founder $5K/mo) | $5,000 | $10,000 |
| Year 2 | 3 people | $20,000 avg | $240,000 |
| Year 3 | 8 people | $60,000 avg | $720,000 |

Founder salary starts when monthly profit consistently exceeds personal runway needs.
At $10K MRR with $3K infra costs, there is $7K/month available for founder salary.

### Tools & Software

| Tool | Cost |
|------|------|
| GitHub Pro | $4/month |
| Cloudflare Pro | $20/month |
| PostHog (self-hosted) | $0 |
| Stripe (2.9% + $0.30) | Variable ~2.9% of revenue |
| Grafana Cloud | $0–$50/month |
| Notion / Linear | $10–$20/month |
| Domain registrations | $50/year |
| **Year 1 tools total** | ~$2,400/year |

---

## 3-Year P&L Model

### Year 1: Foundation

| Item | Amount |
|------|--------|
| **Revenue** | $33,000 |
| — Infrastructure | ($1,500) |
| — AI API costs | ($1,500) |
| — Stripe processing (~3%) | ($1,000) |
| — Tools & software | ($2,400) |
| — Legal setup | ($3,000) |
| — Founder salary (M11–M12) | ($10,000) |
| — Misc (travel, conferences) | ($2,000) |
| **Total Expenses** | ($21,400) |
| **Net Income (Loss)** | **$11,600** |
| **Cash on hand (end of Y1)** | **~$12,000** |

Year 1 is mildly profitable if founder defers salary until Month 11. The business
funds itself from Month 6 onward on an operating cost basis.

### Year 2: Growth

| Item | Amount |
|------|--------|
| **Revenue** | $500,000 |
| — Infrastructure | ($4,500) |
| — AI API costs | ($10,000) |
| — Stripe processing | ($14,500) |
| — Salaries (founder + 2 hires) | ($240,000) |
| — Tools & software | ($10,000) |
| — Legal (DPAs, contracts) | ($8,000) |
| — Marketing (newsletters, conferences) | ($15,000) |
| — Recruiting | ($10,000) |
| — Misc | ($5,000) |
| **Total Expenses** | ($317,000) |
| **Net Income** | **$183,000** |
| **Margin** | **37%** |

Year 2 becomes meaningfully profitable. This funds the Year 3 hiring plan without external capital.

### Year 3: Scale

| Item | Amount |
|------|--------|
| **Revenue** | $5,100,000 |
| — Infrastructure | ($18,000) |
| — AI API costs | ($75,000) |
| — Stripe processing | ($148,000) |
| — Salaries (8 FTE + benefits) | ($900,000) |
| — Tools & software | ($50,000) |
| — Legal (SOC 2, enterprise) | ($60,000) |
| — Marketing | ($80,000) |
| — Recruiting | ($50,000) |
| — Office / co-working | ($30,000) |
| — Misc | ($20,000) |
| **Total Expenses** | ($1,431,000) |
| **Net Income** | **$3,669,000** |
| **Margin** | **72%** |

Year 3 margin of 72% reflects high-margin SaaS economics at scale with enterprise mix.

---

## Burn Rate Scenarios

### Scenario A: Bootstrapped (No External Funding)

**Starting capital:** $0 (pure revenue-funded)

| Period | Monthly Revenue | Monthly Expenses | Monthly Net | Cumulative Cash |
|--------|----------------|-----------------|------------|----------------|
| M1–3 | $200 avg | $400 | -$200 | -$600 |
| M4–6 | $1,500 avg | $500 | +$1,000 | +$1,800 |
| M7–9 | $4,000 avg | $600 | +$3,400 | +$12,000 |
| M10–12 | $8,500 avg | $5,600 | +$2,900 | +$20,700 |
| M13–18 | $20,000 avg | $18,000 | +$2,000 | +$32,700 |
| M19–24 | $42,000 avg | $28,000 | +$14,000 | +$116,700 |

**Risk:** Months 1–3 require personal savings of ~$600 minimum. In practice, the founder
needs 6 months of personal runway ($10K–$30K depending on location) before the
business becomes self-sustaining.

**Verdict:** Bootstrapping works. The economics are favorable. No dilution, full control.

---

### Scenario B: Pre-Seed / Angel Round ($500K)

**Raise:** $500K at $3M post-money valuation (17% dilution)
**When:** Month 3, after 200 GitHub stars and first paying customers as proof points

| Use of Funds | Amount | Purpose |
|-------------|--------|---------|
| Hire #1 (Backend Engineer, 12 months) | $110,000 | Ship product 3x faster |
| Hire #2 (DevRel, 12 months) | $85,000 | Build community to 10K stars |
| Marketing (content, conferences) | $50,000 | Accelerate organic acquisition |
| Infrastructure (scale earlier) | $30,000 | Handle 10x traffic capacity |
| Legal (entity, DPAs, SOC2 prep) | $25,000 | Enterprise readiness |
| Runway buffer | $200,000 | 18 months of founder salary + buffer |
| **Total** | **$500,000** | 18–24 months of runway |

**Monthly burn with $500K raised:**
- Month 1–6: $15,000–$20,000/month (2 hires + infra + marketing)
- Month 7–12: $25,000–$30,000/month (fully ramped)
- Runway at $25K burn: 20 months → reach $50K MRR before runway ends

**Target milestones before Series A ask:**
- $50K MRR sustained for 3+ months
- 10K GitHub stars
- 3+ enterprise customer conversations

---

### Scenario C: Seed Round ($3M)

**Raise:** $3M at $15M post-money (20% dilution)
**When:** Month 12–18, after proving $30K MRR and clear enterprise demand

| Use of Funds | Amount | Purpose |
|-------------|--------|---------|
| Engineering team (4 hires, 24 months) | $1,000,000 | Product velocity |
| Sales (1 hire, 24 months) | $250,000 | Enterprise deals |
| DevRel + Marketing (2 hires) | $350,000 | Community + content |
| Infrastructure (multi-region) | $150,000 | Enterprise SLA capability |
| Legal + Compliance (SOC 2) | $100,000 | Enterprise trust |
| Marketing & conferences | $200,000 | Brand building |
| Runway buffer | $950,000 | 18 months at full burn |
| **Total** | **$3,000,000** | 24 months of runway |

**Monthly burn at Seed:** $100,000–$125,000/month (8-person team)
**Runway:** 24–30 months
**Target before Series A:** $5M ARR, 20+ enterprise customers, >100% NRR

---

## Break-Even Analysis

### Operating Break-Even (No Salary)

The business covers its own operating costs (infra, tools, AI APIs) at:
- Monthly operating costs: ~$500 (Year 1 baseline)
- Revenue needed: ~$500 MRR = 42 Pro subscribers
- **Expected timeline: Month 4–5**

### Founder-Salary Break-Even ($5,000/month)

Revenue needed: ~$5,500 MRR = 460 Pro subscribers OR 112 Teams seats
- **Expected timeline: Month 9–11** (bootstrapped scenario)

### Team Break-Even (3 people at $20K/month total salaries)

Revenue needed: ~$25,000 MRR = 2,100 Pro subscribers OR comparable mix
- **Expected timeline: Month 18–24** (bootstrapped) or Month 12 (with pre-seed funding)

---

## Fundraising Strategy

### Should You Raise?

| Factor | Bootstrapped | Pre-Seed | Seed |
|--------|-------------|---------|------|
| Speed to market | Slow | Medium | Fast |
| Founder ownership | 100% | ~83% | ~63% |
| Risk of failure | Low (slow death) | Medium | High (burn) |
| Upside potential | Medium | High | Very High |
| Control | Full | High | Medium |
| Pressure | Low | Medium | High |

**Recommendation:** Bootstrap to $10K MRR, then raise $500K from angels if there is
clear enterprise demand and a need to hire. This gives 2+ years of data, a proven
product, and significantly better valuation than raising on a pitch deck alone.

### Investor Types for Fetchium

**Angels (ideal for pre-seed):**
- Former developer tool founders (Sourcegraph, Retool, Linear alumni)
- AI/search engineers with networks
- Target: 5–10 angels at $50K–$100K each

**Funds (ideal for seed):**
- Tier 1: a16z, Sequoia, General Catalyst (AI focus)
- Developer-tool specialists: Heavybit, Boldstart, Crane Venture Partners
- AI-specific: Conviction, Elad Gil, SV Angel

### Fundraising Process (Pre-Seed)

1. **Build the story:** 10-slide deck (problem, solution, traction, team, market, ask)
2. **Traction first:** Never fundraise without 3 months of consistent MRR growth
3. **Warm intros only:** Use your network; AngelList after exhausting network
4. **Timeline:** 4–8 weeks for pre-seed from first meeting to wire
5. **Legal:** Use SAFE notes (YC standard) for pre-seed — no price, no cap table complexity

### Key Metrics for Fundraising Conversations

| Metric | Pre-Seed Target | Seed Target |
|--------|----------------|------------|
| MRR | $5K | $30K |
| MRR growth (M/M) | > 15% | > 20% |
| GitHub stars | 1K | 10K |
| Paying customers | 50 | 500 |
| NPS | > 40 | > 50 |
| NRR | N/A | > 100% |
| Gross margin | N/A | > 70% |

---

## Financial Controls (Solo Founder)

### Monthly Financial Review (30 minutes)

1. Export Stripe revenue report — reconcile with bank statement
2. Check AWS/Hetzner/Cloudflare invoices — any unexpected spikes?
3. Update burn rate spreadsheet — how many months of runway remain?
4. Review AI API usage costs — are free-tier users costing too much?
5. Check accounts receivable — any enterprise invoices overdue?

### Tools

| Tool | Purpose | Cost |
|------|---------|------|
| Mercury Bank | Business checking, no fees | Free |
| Stripe | Payment processing + subscriptions | 2.9% + $0.30 |
| Wave Accounting | Bookkeeping (free alternative to QuickBooks) | Free |
| Notion / Google Sheets | Cap table, burn tracker | Free |
| Pilot.com | Bookkeeping service (when revenue > $20K MRR) | $200/month |

### Tax Obligations

- **US C-Corp:** File federal + state corporate taxes annually (April 15 or extension)
- **Quarterly estimated taxes:** If profitable, pay quarterly to avoid penalties
- **R&D Tax Credit:** Fetch-related AI research may qualify for federal R&D credit
  — consult a CPA when revenue exceeds $100K
- **Sales tax (US):** SaaS is subject to sales tax in 30+ US states — use TaxJar ($19/month) to automate
- **VAT (EU):** Required when EU revenue exceeds the registration threshold (~€10K) — use Stripe Tax
