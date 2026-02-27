# Fetchium — 90-Day Launch Playbook

**Scope:** Month 3, Weeks 9–12 (post 100-user beta) through Month 6 end
**Owner:** Founder
**Goal:** 500 registered users, 50 paying customers, $5K MRR by end of Day 90

---

## Pre-Launch Checklist (Complete Before Any Public Post)

### Product Readiness
- [ ] All Phase 1 endpoints respond correctly (100% of manual test cases)
- [ ] p95 search latency < 3s (Prometheus histogram, 7-day average)
- [ ] p95 AI latency < 10s (7-day average)
- [ ] Uptime > 99% for 30 days (UptimeRobot)
- [ ] Rate limiting enforced per key (confirmed via load test)
- [ ] Stripe billing live with working upgrade flow (tested with real card)
- [ ] Free tier properly capped (100 searches/day enforced)
- [ ] Error messages are informative, never expose internals
- [ ] `/health` endpoint returns correct SearXNG status

### Content Readiness
- [ ] README: animated GIF demo, benchmark table, 60-second quick-start
- [ ] Docs site live at docs.fetchium.dev (all Phase 1 endpoints documented)
- [ ] Landing page live at fetchium.dev with pricing + CTA
- [ ] Blog post: "Introducing Fetchium" (750 words, drafted + reviewed)
- [ ] HN Show HN post: drafted, reviewed by 2 beta users for feedback
- [ ] Twitter thread: 8-tweet launch announcement thread, drafted
- [ ] Discord server live with 5 channels, 10 beta users seeded

### Operations Readiness
- [ ] Support email: support@fetchium.dev forwards to founder's email
- [ ] Status page live at status.fetchium.dev
- [ ] On-call alert: UptimeRobot → email + SMS for downtime
- [ ] Backup: daily SQLite dump to separate storage
- [ ] Logging: errors written to file, reviewed daily

---

## Phase 1: Soft Launch (Weeks 1–2)

**Goal:** Get 50 hand-picked users using the product daily and collecting feedback.

### Who Gets Invited First

**Tier 1 — Active contributors and engaged beta users (20 people)**
- Developers who reported bugs or requested features during beta
- GitHub contributors
- People who emailed or DM'd asking about the project

**Tier 2 — Targeted cold invites (30 people)**
- Authors of popular LangChain/LlamaIndex tutorials (Twitter/GitHub bio)
- Active contributors to r/LocalLLaMA who mention needing search APIs
- Developers who starred Tavily or Exa repos on GitHub (find via GitHub API)

**Invitation message (DM or email):**
```
Hey [name],

I've been building Fetchium — a search API specifically for AI agents and
research pipelines. It combines web search, AI synthesis, video extraction,
and social aggregation in one API call.

Saw your work on [relevant project/post] and thought it might be useful.

Would you be interested in early access? Free Pro account for the first month.
Happy to get on a 15-minute call if you have questions.

Repo: github.com/fetchium/fetchium
Demo: fetchium.dev

— [Name]
```

### Week 1–2 Activities

**Daily (30 min/day):**
- Reply to every Discord message within 4 hours
- Review error logs for new patterns
- Follow up personally with users who sign up but don't make their first API call

**Weekly:**
- 1-hour "office hours" voice call on Discord (open to all early users)
- NPS survey sent to all active users (> 5 API calls that week)

**Metrics to watch:**
- Daily active users (should be 30+ of the 50 by end of week 2)
- API calls/day (should be 500+ by end of week 2)
- Error rate (keep < 1%)
- Support requests (categorize: billing, latency, docs, bugs)

---

## Phase 2: Public Launch (Weeks 3–4)

### Week 3: Hacker News Show HN

**Target:** Front page (> 100 points). Realistically achievable if the product is good and the post is well-written.

**Timing:** Tuesday or Wednesday, 9-10am Eastern. Avoid Mondays (crowded) and Fridays (fewer readers).

**HN Post title options (A/B test in beta user feedback):**
- "Show HN: Fetchium – Web search + AI synthesis + video extraction in one API call"
- "Show HN: I built an open search API because Bing died and Perplexity got expensive"
- "Show HN: Fetchium – Add web search to your AI agent for $49/month flat"

**Post body (500-800 words):**
1. The problem (3 paragraphs): Bing retired, Perplexity token pricing, Tavily acquired
2. The solution: what Fetchium does in one code snippet
3. A real benchmark: side-by-side timing comparison
4. What's different: CLI, flat pricing, open-source core, video + social
5. Where to find it: GitHub, docs, Discord

**On HN launch day:**
- Respond to every comment within 30 minutes for the first 6 hours
- Be genuinely helpful, even to critical comments
- Don't argue — acknowledge valid criticism, explain tradeoffs
- Have 5 beta users comment from their real accounts (natural, not scripted)

**Expected results:** 200–2,000 signups depending on ranking. Even page 2 = 100–500 signups.

### Week 4: Product Hunt Launch

**Timing:** 12:01am PST on a Tuesday (avoid Mondays — too competitive).

**Product Hunt listing:**
- Tagline: "Fetch anything. Verified. Fast."
- Description: 260 characters max — lead with the benchmark
- Gallery: 5 screenshots (CLI demo, API response, benchmark, docs, pricing)
- First comment: founder note (longer story, acknowledges limitations)
- Video: 60-second demo (screen record of CLI + API in action)

**Hunter:** Ask the most prominent beta user with a Product Hunt account to hunt it. Alternatively, founder hunts it.

**Pre-launch seating:** 48 hours before launch, message all 50+ beta users:
> "Fetchium launches on Product Hunt Tuesday at midnight PST. If you've found it useful, an upvote would mean a lot. Here's the link: [PH link]"

**Expected results:** Top 5 on launch day = 500–1,500 additional signups.

**Simultaneous posts on HN launch day:**
- Dev.to post: "Introducing Fetchium: search API built for AI agents"
- Twitter/X thread: 8-tweet launch thread with GIF demo
- Discord announcement
- GitHub release: v1.0.0 with full changelog

---

## Phase 3: Momentum (Weeks 5–8)

### Week 5–6: Developer Blog Series

**Post 1 (Week 5):** "How Fetchium's HyperFusion ranking works"
- Explain 8 signals, how they're combined
- Include code: the actual Rust implementation (or pseudocode)
- End with: "Try it yourself: fetchium search 'your query'"

**Post 2 (Week 6):** "Adding web search to a LangChain agent in 5 minutes"
- Step-by-step tutorial
- Working Python code (copy/paste ready)
- Comparison: same agent with Tavily vs with Fetchium

**Post 3 (Week 7):** "Benchmark: AI search APIs compared (Fetchium, Perplexity, Tavily, Exa)"
- Methodology: 100 queries across 5 categories
- Metrics: latency, relevance, cost
- Published methodology so anyone can reproduce

**Post 4 (Week 8):** "How we built a search API in Rust that handles 50K requests/day"
- System architecture deep dive
- SearXNG integration, HyperFusion, CEP pipeline
- What we'd do differently

**Distribution for each post:**
- fetchium.dev/blog (primary)
- Dev.to (same day)
- Hashnode (same day)
- HN "Ask HN" or plain URL submission (on high-quality posts)
- Twitter/X thread summary

### Week 5–8: First Outreach Wave

**Reddit:**
- r/LocalLLaMA: Tutorial post ("I built a search layer for my local LLM setup")
- r/MachineLearning: Share benchmark post (no promotion, just data)
- r/Python: Tutorial post ("Web search in your Python AI pipeline — 10 lines of code")

**Twitter/X:**
- Reach out to 5 AI developer influencers (> 10K followers) with personalized demo
- Quote-tweet developers who mention needing a search API

**YouTube:**
- First video: "Fetchium in 5 minutes" (screen record + voiceover)
- Second video: "Build an AI research agent with Fetchium + LangChain"

---

## Phase 4: Partnerships & Conferences (Weeks 9–12)

### Partnership Announcements

**Target partnerships by Week 12:**
1. **LangChain** — Reach out to @hwchase17 with completed integration + demo
2. **LlamaIndex** — Reach out to Jerry Liu's team (active on Twitter)
3. **n8n** — Community node submitted to n8n's marketplace (self-serve)
4. **Zapier** — Application submitted to Zapier Developer Program

**Partnership announcement template:**
> "Fetchium is now available as a native [Partner] integration. Add web search, video extraction, and social aggregation to your [Partner] workflows in 2 minutes."

### Conference Strategy

**Target events (Month 5-6 window):**
- AI Engineer World's Fair (recurring, SF) — submit talk proposal
- NeurIPS Workshops — poster or demo session
- Local developer meetups (speak for free, gain credibility)

**Talk title options:**
- "Building a search API for the post-Bing era"
- "HyperFusion: combining 8 ranking signals in a multi-source search system"
- "Why your AI agent needs a better search layer"

**Podcast outreach (Week 10):**
Target 5 developer-focused podcasts:
- Changelog (changelog.com)
- Software Unscripted
- The AI Engineer (Latent Space)
- Developer Tea
- PodRocket

**Pitch email:**
> "I built Fetchium — a search API for AI agents that combines web, video, and social search. The Bing API retired Aug 2025 and I think there's an interesting story about what comes next. Happy to do a 30-min recording if you're interested."

### Week 12: Milestone Review

**Metrics review against Phase 1 targets:**

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Registered users | 500 | ? | — |
| Paying customers | 50 | ? | — |
| MRR | $5,000 | ? | — |
| GitHub stars | 2,000 | ? | — |
| Discord members | 300 | ? | — |
| NPS | > 45 | ? | — |
| p95 latency | < 3s | ? | — |
| Uptime | > 99% | ? | — |

**Decision gate:** If MRR < $2,500, delay Phase 2 and focus on conversion/retention. If MRR > $5K, proceed to Phase 2 as planned.

---

## Contingency Plans

| Scenario | Response |
|----------|---------|
| HN post flops (< 20 points) | Repost in 60 days with different angle; prioritize Reddit and dev.to instead |
| Server overwhelmed on launch day | Pre-scale to 2 VPS instances; Cloudflare rate limiting as buffer |
| Negative HN comment goes viral | Respond factually, acknowledge valid criticism, improve the product publicly |
| First paying customer cancels | Personal call within 24 hours; understand why; offer refund + fix |
| Competitor posts similar tool on HN same day | Comment honestly in their thread; differentiation sells itself |
