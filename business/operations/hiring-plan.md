# Hiring Plan

## Philosophy

Hire slow. Every early hire shapes the culture and has outsized influence on the product.
For a developer tool company, the first 5 hires should all be technical or technical-adjacent.
A bad cultural fit at 5 people creates dysfunction at 15 people.

**AI-first staffing:** Use AI tools aggressively before hiring. One AI-augmented founder
can do the work of 3 people in 2024–2026 for engineering tasks. Hire humans for things
AI cannot do: customer relationships, community presence, judgment calls.

**Remote-first from day one.** There is no office. Async by default, synchronous when needed.

---

## Phase 1: Solo (Month 1–12, $0–50K ARR)

**Team:** 1 person (founder)

The founder is the engineer, the marketer, the support agent, the designer, and the CEO.
AI tools make this sustainable. The goal is not to stay solo forever — it is to reach
$10K MRR before making the first hire, ensuring the business can fund growth.

### Tools that Replace Early Hires

| Role | Tool |
|------|------|
| Junior engineer | Claude Code, Copilot for boilerplate and tests |
| Copywriter | Claude/GPT for docs, blog posts, landing copy |
| Designer | Figma + AI plugins, Tailwind components |
| Data analyst | PostHog dashboards, Stripe analytics |
| Support | Discord (community-supported), GitHub Issues, canned responses |
| Marketing | Scheduled tweets, newsletter templates |

### Founder's Weekly Time Allocation (Solo Phase)

| Activity | Hours/Week |
|----------|-----------|
| Product engineering | 25 |
| Customer conversations | 5 |
| Community / content | 5 |
| Operations / finance | 3 |
| Strategy / planning | 2 |
| **Total** | **40** |

### Solo Phase Milestones (Before Hiring)
- [ ] $10K MRR sustained for 2+ months
- [ ] 500+ GitHub stars
- [ ] 100+ active users, NPS > 40
- [ ] First 5 paying customers on record

---

## Phase 2: Small Team (Month 12–18, $50K–200K ARR)

**Team:** 3 people — Founder + Backend Engineer + DevRel

### Hire #1: Backend/Infrastructure Engineer

**When:** $10K MRR OR product quality issues blocking growth
**Compensation:** $90K–$120K salary + 0.5–1.0% equity (4-year vest, 1-year cliff)
**What they own:** API reliability, SearXNG infrastructure, performance, scaling

**Must-haves:**
- 3+ years Rust or strong Go/Python with willingness to learn Rust
- Experience with async systems, API design, database optimization
- Self-directed — can pick up GitHub issues and ship without daily oversight
- Genuine interest in AI agents / search (not just a job)

**Red flags:**
- Needs micromanagement or daily standups to stay productive
- No public code (OSS, GitHub projects, blog) — developers who care have a trail
- Asks about vacation policy before asking about the technical challenges

**Interview process (async-first):**
1. Take-home: "Add a new fetch mode to this stripped-down Fetchium module" (4-hour task)
2. Code review call: 45 minutes discussing their implementation
3. Reference check: one technical reference who has shipped with them

### Hire #2: Developer Relations / Community

**When:** $15K MRR OR GitHub issues unanswered for > 48 hours regularly
**Compensation:** $70K–$90K salary + 0.25–0.5% equity
**What they own:** Discord, GitHub Issues, tutorials, developer advocates, HN/Reddit presence

**Must-haves:**
- Technical enough to understand the product deeply (ideally former developer)
- Strong written communication — can write a clear, helpful GitHub issue response
- Track record of community building (Discord mod, OSS contributor, blogger)
- Genuine passion for developer tools

**What they do:**
- Respond to every GitHub issue within 24 hours
- Post weekly content: changelogs, tutorials, demos
- Run the Discord community — welcome new members, answer questions
- Write integration guides (LangChain, CrewAI, etc.)
- Represent Fetchium at 2-3 developer conferences per year

---

## Phase 3: Growing Team (Month 18–30, $200K–1M ARR)

**Team:** 8 people

### Hire #3 & #4: Software Engineers

**When:** Hire #1 is overwhelmed, product roadmap is backlogged 3+ months
**Compensation:** $100K–$130K salary + 0.25–0.5% equity each
**Profiles:** One generalist (frontend/backend), one infrastructure/devops specialist

**Generalist engineer owns:** Web dashboard, CLI UX, SDK quality
**Infra engineer owns:** Kubernetes migration, multi-region, monitoring

### Hire #5: Product Designer

**When:** User research reveals UX confusion is the top barrier to conversion
**Compensation:** $80K–$100K salary + 0.2–0.4% equity
**What they own:** Dashboard UI, docs site, CLI output aesthetics, brand consistency

**Note:** A designer at this stage is not a luxury — a confusing dashboard costs more
in conversion rate than the designer's salary within 6 months.

### Hire #6: Marketing / Growth

**When:** Organic growth plateau — GitHub star growth slows, new signups flatten
**Compensation:** $80K–$100K salary + 0.2–0.4% equity
**What they own:** SEO, content calendar, newsletter, conference sponsorships, paid experiments

**Must-haves:**
- Developer marketing experience specifically (B2D — business to developer)
- Data-driven: can run A/B tests, analyze conversion funnels, interpret PostHog data
- Technical empathy: has used developer tools extensively, can write accurate copy

### Hire #7: Customer Success / Support

**When:** Support volume exceeds 2 hours/day of founder or DevRel time
**Compensation:** $60K–$80K salary + 0.1–0.2% equity
**What they own:** Onboarding new Teams and Enterprise customers, support tickets,
expansion conversations, QBRs for enterprise accounts

---

## Phase 4: Scaled Team (Month 30–48, $1M–5M ARR)

**Team:** 15 people

### Key Additions

**Engineering Manager** ($140K–$160K, 0.25–0.5%)
- Manages the 4+ engineers; owns delivery, process, hiring pipelines
- Hire when engineering team hits 5+ people and founder is spending > 20% time on eng coordination

**Sales Lead** ($90K base + commission, 0.2–0.4%)
- Owns enterprise deals > $20K
- Inbound only — responds to enterprise inquiries, runs demos, closes contracts
- No cold outbound at this stage; let the product-led motion generate leads

**Finance / Operations** ($80K–$100K, 0.1–0.2%)
- Bookkeeping, payroll, legal coordination, vendor contracts
- Previously a contractor; hire FTE when complexity exceeds 5 hours/week of founder time

**3 Additional Engineers** (various profiles)
- 1 AI/ML engineer (fine-tuning, model evaluation, inference optimization)
- 1 frontend engineer (dashboard, SDK DX)
- 1 platform/reliability engineer (SLA, incident response, on-call)

---

## Org Chart at 15 People

```
CEO (Founder)
├── Engineering Manager
│   ├── Backend Engineer #1 (hire 1)
│   ├── Backend Engineer #2 (hire 3)
│   ├── Infra Engineer (hire 4)
│   ├── AI/ML Engineer
│   ├── Frontend Engineer
│   └── Platform Engineer
├── Developer Relations (hire 2)
│   └── Customer Success (hire 7)
├── Product Designer (hire 5)
├── Marketing / Growth (hire 6)
├── Sales Lead
└── Finance / Operations
```

---

## Compensation Philosophy

**Base salary:** Market rate for the role, adjusted for remote and cost of living.
Reference: levels.fyi, Glassdoor, Payscale for developer tool companies of similar stage.

**Equity:**
- All equity is subject to standard 4-year vesting with 1-year cliff
- Use a standard 409A valuation before granting options
- Target: employees at 15 people total should hold 15–20% of the company collectively
- Never grant equity without a signed option agreement

**Benefits (remote-first):**
- Health insurance stipend ($500/month US, equivalent cash internationally)
- Home office stipend ($1,500 one-time at hiring, $500/year thereafter)
- Learning budget ($1,000/year for courses, books, conferences)
- Async-first culture — no 9-5, no mandatory daily standups, results > hours

**What we don't offer (at this stage):**
- On-site office perks (no office)
- Unlimited PTO (minimum 15 days encouraged, clearly stated)
- Pension matching (add at Series A)

---

## Remote Culture Principles

1. **Written by default:** Decisions are documented in Notion/GitHub, not made in ephemeral calls
2. **Overlap hours:** 3-4 hours of overlap expected per day for async collaboration; no full timezone lockout
3. **Async video:** Use Loom for walkthroughs, not calendar invites for everything
4. **Meetings have agendas:** No standing meetings without a written agenda distributed 24h beforehand
5. **Transparent by default:** Salary bands, revenue, roadmap — all visible to the team
6. **Ship fast:** Prefer a shipped experiment over a perfect plan; postmortems are blameless

---

## Hiring Anti-Patterns (What We Avoid)

- Hiring for "potential" without evidence of execution at this stage — we need output now
- Hiring friends/family without rigorous process — applies even to advisors
- Hiring to solve a process problem — fix the process first, then hire if it persists
- Hiring a "VP of X" before having a team for them to manage
- Equity promises without board approval and legal documentation
