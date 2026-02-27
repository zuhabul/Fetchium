# Revenue Model & Unit Economics

## Revenue Streams

| Stream | Type | Margin Profile | Priority |
|--------|------|---------------|----------|
| Pro subscriptions ($12/mo) | Recurring | High (~85%) | Primary |
| Teams subscriptions ($49/seat) | Recurring | High (~80%) | Secondary |
| API usage (pay-as-you-go) | Variable | Medium (~40%) | Secondary |
| Enterprise contracts | Recurring | Very high (~90%) | Long-term |
| Professional services | One-time | Low (~30%) | Opportunistic |

---

## Cost Structure Per Fetch

Understanding the cost per unit is critical for pricing decisions and margin management.

### Cost Components

| Component | Cost per Fetch | Notes |
|-----------|---------------|-------|
| SearXNG search | ~$0.0002 | Self-hosted; amortized compute |
| Content extraction (CEP) | ~$0.0003 | CPU time for HTML parsing |
| AI enrichment (Gemini/GPT) | ~$0.002 | Variable by model; skipped in fast mode |
| Bandwidth | ~$0.0001 | ~500KB average page, CDN egress |
| Storage (result cache) | ~$0.00005 | Redis TTL 1h, ~2KB per result |
| **Total (with AI)** | **~$0.003** | Per enriched fetch |
| **Total (no AI / fast mode)** | **~$0.0006** | Per fast fetch |

### Margin by Tier

**Free tier:**
- Revenue: $0
- Cost: ~$0.003 × 10 fetches/day × 30 days = $0.90/user/month
- Net: -$0.90/user — capped by design; acceptable for acquisition

**Pro tier ($12/month):**
- Revenue: $12
- Estimated cost: ~$0.003 × 200 fetches/day × 30 days = $18/month (worst case)
- Realistic usage: average Pro user fetches ~50/day = $4.50/month cost
- **Gross margin: ~62–85%** depending on usage patterns
- Safety valve: fair-use cap at 5K/day prevents runaway cost

**Teams tier ($49/seat/month):**
- Revenue per seat: $49
- Estimated cost per seat: ~$5/month (shared infra amortized)
- **Gross margin: ~90%** — shared infra, fixed SSO cost, shared API pool
- Break-even: 1 seat covers 10 free-tier users in infrastructure cost

**API tier ($0.003–$0.005/request):**
- Revenue at $0.004/req: $4 per 1K requests
- Cost at $0.003/req: $3 per 1K requests
- **Gross margin: ~25–40%** — tight but justified by volume
- At 1M requests/month: $3K revenue, $2K cost = $1K profit
- Margin improves significantly at scale (compute costs fixed; revenue scales)

---

## LTV / CAC Analysis

### Customer Acquisition Cost (CAC)

| Channel | CAC Estimate | Basis |
|---------|-------------|-------|
| Organic (HN, GitHub, blog) | $0–$5 | Time cost only; attributed at founder hourly rate |
| Discord / community | $10–$20 | Time + minor moderation cost |
| Newsletter sponsorship | $30–$80 | $500 sponsorship / 15–20 signups expected |
| Content SEO | $20–$50 | Time amortized over traffic |
| **Blended CAC (Year 1)** | **~$15** | Mostly organic, no paid ads |

### Lifetime Value (LTV)

| Cohort | Monthly ARPU | Monthly Churn | LTV |
|--------|-------------|--------------|-----|
| Pro | $12 | 5% | $240 |
| Teams (3 seats) | $147 | 3% | $4,900 |
| API | $40 avg | 8% | $500 |
| Enterprise | $2,000 avg | 2% | $100,000 |

**LTV/CAC ratios:**
- Pro: $240 / $15 = **16x** (healthy; target > 3x)
- Teams: $4,900 / $50 = **98x** (excellent)
- API: $500 / $15 = **33x** (strong)

### Payback Period

- Pro: $12 revenue, $15 CAC → payback in **1.5 months**
- CAC is so low that even modest retention makes this extremely capital-efficient

---

## 3-Year Revenue Projection

### Assumptions
- Organic-first growth; no paid acquisition in Year 1
- Free-to-Pro conversion: 3% (conservative; Cursor saw 36% — our market is different)
- Monthly churn: 5% Pro, 3% Teams
- Teams average deal: 5 seats
- API users: 20% of Pro signups also use API

### Year 1 — Foundation ($50K ARR target)

| Month | New Signups | Pro Users | Teams Seats | MRR |
|-------|------------|-----------|------------|-----|
| 1 | 200 | 6 | 0 | $72 |
| 2 | 400 | 18 | 0 | $216 |
| 3 | 600 | 36 | 5 | $627 |
| 4 | 800 | 60 | 10 | $1,210 |
| 5 | 1,000 | 90 | 20 | $1,960 |
| 6 | 1,200 | 126 | 35 | $2,907 |
| 9 | 2,000 | 240 | 80 | $6,080 |
| 12 | 3,500 | 400 | 150 | $10,150 |

**Year 1 ARR: ~$50K** (annualizing Month 12 MRR × 12)
**Year 1 Cumulative Revenue: ~$25K** (MRR ramps through the year)

### Year 2 — Growth ($500K ARR target)

Catalyst: Public launch, LangChain integration, benchmark blog post virality

| Quarter | Pro Users | Teams Seats | Enterprise | MRR |
|---------|-----------|------------|-----------|-----|
| Q1 | 600 | 250 | 0 | $15,850 |
| Q2 | 900 | 400 | 1 ($2K) | $27,600 |
| Q3 | 1,400 | 600 | 2 ($4K) | $41,800 |
| Q4 | 2,000 | 900 | 4 ($8K) | $59,600 |

**Year 2 ARR: ~$500K** (Q4 MRR × 12 ≈ $715K; accounting for ramp = ~$500K recognized)

### Year 3 — Scale ($5M ARR target)

Catalyst: Enterprise motion, channel partnerships, possible seed round

| Quarter | Pro Users | Teams Seats | Enterprise | MRR |
|---------|-----------|------------|-----------|-----|
| Q1 | 3,000 | 1,500 | 8 ($16K) | $127,500 |
| Q2 | 4,500 | 2,500 | 15 ($30K) | $205,500 |
| Q3 | 6,000 | 4,000 | 25 ($50K) | $318,000 |
| Q4 | 8,000 | 6,000 | 40 ($80K) | $490,000 |

**Year 3 ARR: ~$5M** (Q4 MRR × 12 ≈ $5.88M; conservative recognition = $5M)

---

## Path to Profitability

### Bootstrapped Scenario (No External Funding)

| Milestone | When | Monthly Revenue | Monthly Cost | Net |
|-----------|------|----------------|-------------|-----|
| First $1 | Month 2 | $12 | $200 (infra) | -$188 |
| Break-even on infra | Month 6 | $3,000 | $500 | +$2,500 |
| Founder salary ($5K/mo) | Month 10 | $8,000 | $5,500 | +$2,500 |
| Hire #1 ($8K/mo) | Month 18 | $20,000 | $14,000 | +$6,000 |
| Hire #2 ($8K/mo) | Month 24 | $42,000 | $24,000 | +$18,000 |

**Break-even timeline: Month 6** (infra covered), **Month 10** (founder paid)
This is achievable bootstrapped. The economics work without external capital.

### Key Levers for Profitability

1. **Reduce AI API cost:** Switch from OpenAI to Gemini Flash or local Llama for cheaper inference
2. **Improve conversion rate:** Even 1% more Free → Pro = significant MRR at scale
3. **Reduce churn:** Every percentage point of monthly churn saved = 12% more LTV
4. **Enterprise deals:** One $20K/year deal = equivalent of 138 Pro subscriptions

---

## Gross Margin Targets

| Year | Target Gross Margin | Notes |
|------|-------------------|-------|
| Year 1 | 60% | AI costs high; small scale |
| Year 2 | 70% | Better AI cost negotiation; more Pro/Teams mix |
| Year 3 | 75% | Volume discounts on AI APIs; enterprise margins high |

SaaS businesses typically target 70–80% gross margin. Fetchium is in range by Year 2.

---

## Revenue Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| AI API cost spikes | Margin compression | Fallback to local models (Llama, Gemini CLI) |
| High-usage free tier abuse | Increased cost | Rate limits, CAPTCHA for suspicious patterns |
| Pro churn > 8% | MRR growth stalls | Exit surveys, activation improvements |
| Competitor price war | Pricing pressure | Differentiate on quality + self-hostability |
| Enterprise sales cycle too long | Cash flow lag | Focus on product-led enterprise motion |
