# Fetchium — Marketing Strategy

**Last updated:** 2026-02-27
**Owner:** Founder (solo → first marketing hire at $50K MRR)
**Philosophy:** Developer-first, product-led growth, community-driven. Zero paid ads until $1M ARR.

---

## Core Principle: Build in Public, Let the Product Sell Itself

Cursor hit $100M ARR in 12 months with zero marketing budget. The mechanism: a product so good that developers evangelize it organically. Every tweet, every GitHub star, every "how did you do that?" in a Slack channel is a marketing event.

Fetchium's marketing strategy starts from the same premise: **the best marketing is a product that makes developers feel superhuman.**

Secondary principle: **show, don't tell.** Every piece of content should contain a concrete result — a benchmark, a code snippet, a before/after comparison. No vague claims.

---

## Target Audience

### Primary: AI/Automation Developers
- Building AI agents, pipelines, and research tools
- Use Python, TypeScript, or Rust
- Hang out on GitHub, HN, Reddit r/MachineLearning, Twitter/X, Discord
- Pain point: Bing API retired Aug 2025; Perplexity API too expensive at scale; Tavily acquired
- Decision cycle: hours to days (self-serve)
- Willingness to pay: $49–$299/mo without procurement approval

### Secondary: Research Professionals
- Analysts, journalists, academics, consultants
- Use AI tools for competitive intelligence, market research, due diligence
- Hang out on LinkedIn, academic Twitter, Substack
- Pain point: manual research is slow; existing AI tools lack citation quality
- Decision cycle: days to weeks
- Willingness to pay: $49–$199/mo

### Tertiary: Enterprise Teams (Phase 5+)
- AI teams at Series B+ companies
- Require SSO, SLA, audit logs
- Decision cycle: 30–90 days
- Willingness to pay: $2K–$25K/mo

---

## Channel Strategy

### 1. GitHub — Primary Distribution

**Goal:** 5,000 stars by Month 6, 20,000 stars by Month 12.

**Why it works:** 90% of AI developers check GitHub before trying any tool. Stars are social proof.

**Tactics:**
- Excellent README with animated GIF demo, benchmark table, quick-start (< 60 seconds to first result)
- GitHub Discussions enabled — every question is indexed by Google
- Issue templates that guide users to document their use case (doubles as market research)
- Changelog in GitHub Releases (every release is a marketing event)
- "Awesome Fetchium" repo: community-built integrations, plugins, use cases
- `good first issue` labels to drive contributions → contributors become advocates
- GitHub Sponsors page (even if tiny income, signals legitimacy)

**Benchmark format for README:**
```
| Tool           | Query time | AI synthesis | Video | Social | Price/1K calls |
|----------------|-----------|--------------|-------|--------|---------------|
| Fetchium       | 1.2s      | ✅           | ✅    | ✅     | $0.01         |
| Perplexity API | 2.1s      | ✅           | ❌    | ❌     | $1.00         |
| Tavily         | 1.8s      | ❌           | ❌    | ❌     | $0.04         |
| Exa            | 1.5s      | ❌           | ❌    | ❌     | $0.03         |
```

### 2. Hacker News — Credibility + Developer Reach

**Goal:** Front page twice in Phase 1, monthly posts in Phase 2+.

**Why it works:** HN readers are early adopters who share tools they love. A front-page HN post drives 500–5,000 signups in 48 hours.

**Planned posts:**
- M3: "Show HN: Fetchium — AI search API with video + social extraction"
- M6: "Show HN: I built a personal knowledge OS on top of Fetchium"
- M9: "Ask HN: Why isn't there a good search API for AI agents? (We fixed it)"
- M12: Blog post: "Lessons from building a SearXNG-backed search API"

**HN post formula:**
- Show something concrete and working (not a preview or coming soon)
- Include a live demo or benchmark
- Respond to every comment within 2 hours
- Be genuinely useful in comments regardless of upvote outcome

### 3. Reddit — Niche Community Presence

**Subreddits:**
- r/MachineLearning (950K members)
- r/LocalLLaMA (600K members)
- r/learnmachinelearning (350K members)
- r/webdev (800K members)
- r/programming (4M members)
- r/rust (300K members)
- r/Python (1.1M members)

**Content approach:**
- No promotional posts until established
- First 30 days: answer questions where Fetchium would be the honest answer
- After 30 days: post tutorials that happen to use Fetchium
- Never post "check out my tool" without substantial value
- Reddit AMAs (M6, M12) after building credibility

**Weekly time budget:** 2 hours/week monitoring and responding.

### 4. Twitter/X — Real-Time Visibility

**Account:** @fetchiumdev (register immediately)

**Content cadence:** 5 tweets/week (3 product, 1 community, 1 market/news)

**Content categories:**
- **Product demos:** animated screenshots of CLI output, benchmark results
- **Behind-the-scenes:** architecture decisions, tradeoffs, "we almost shipped X instead"
- **Algorithm explanations:** HyperFusion in a tweetstorm, QATBE visualized
- **Numbers/milestones:** DAU graphs, benchmark improvements, star counts
- **Re-sharing:** user tweets showing Fetchium in wild — quote tweet every one

**Growth hack:** Tweet at LangChain, LlamaIndex, CrewAI, Hugging Face when our integration ships. They often retweet partner tools.

### 5. Dev.to & Hashnode — SEO + Developer Discovery

**Goal:** 2 posts/month, each ranking for a target keyword.

**Why:** Dev.to posts rank in Google within days. Hashnode posts appear in their curated newsletter (100K+ subscribers).

**Post types:**
- Tutorial: "How to add web search to your LangChain agent in 5 minutes"
- Comparison: "Fetchium vs Perplexity API vs Tavily: a developer's benchmark"
- Deep dive: "How we built a 10-language search aggregator with SearXNG"
- Architecture: "Why we chose Rust for our search API core"

### 6. YouTube — Long-Form Education

**Goal:** 1 video/month starting M4. 1K subscribers by M12.

**Content:**
- Live-coding demos (search agent build-along)
- Tutorial series: "Build an AI research assistant from scratch"
- Benchmark comparisons (screen record side-by-side API calls)
- Algorithm explainers (whiteboard-style, 5-10 minutes)

**Production:** screen record + voiceover. No fancy studio needed.

### 7. Discord Community

**Server name:** Fetchium Developers
**Invite link:** discord.gg/fetchium

**Channels:**
- `#announcements` — product releases, blog posts
- `#general` — open discussion
- `#showcase` — share what you built with Fetchium
- `#help` — support (monitored daily by founder)
- `#api-discussion` — API design feedback (directly informs roadmap)
- `#contributors` — for open-source contributors

**Goal:** 500 members by M6, 2,000 by M12.
**Community rule:** Response to every message in `#help` within 24 hours.

---

## Content Calendar (Monthly Rhythm)

| Week | Content |
|------|---------|
| Week 1 | Blog post (tutorial or case study) |
| Week 1 | Twitter/X thread (algorithm deep dive) |
| Week 2 | Dev.to / Hashnode cross-post |
| Week 2 | GitHub changelog update + announcement |
| Week 3 | YouTube video (when applicable) |
| Week 3 | Reddit contribution (answer or tutorial) |
| Week 4 | Monthly newsletter (Discord + email) |
| Week 4 | HN comment monitoring + engagement |

---

## Community Programs

### Fetchium Contributor Program (Phase 2+)

Reward open-source contributors:
- 3+ merged PRs → Contributor badge in Discord + GitHub profile
- 5+ merged PRs → 3 months Pro free
- Plugin published in marketplace → Revenue share (Phase 5)
- Core contributor → advisory credit in README

### Fetchium Ambassador Program (Phase 3+)

For community members who drive signups:
- Unique referral link (track signups)
- 20% revenue share on referrals for 12 months
- Early access to all new features
- Monthly call with founder

**Target ambassadors:** Developers with YouTube channels, popular GitHub repos, or newsletters in the AI/ML space.

---

## Metrics to Track

| Metric | Phase 1 Target | Phase 3 Target | Source |
|--------|---------------|---------------|--------|
| GitHub stars | 5,000 | 20,000 | GitHub |
| Discord members | 200 | 2,000 | Discord |
| Twitter followers | 500 | 5,000 | Twitter Analytics |
| Blog monthly visitors | 2,000 | 25,000 | Plausible |
| Newsletter subscribers | 100 | 2,000 | ConvertKit |
| HN Show HN karma | 100+ | 500+ | HN |
| Organic signups/week | 10 | 100 | Auth DB |
| Signup-to-paid conversion | 5% | 15% | Stripe |

---

## Budget (Phase 1 — Solo, < $500/mo)

| Line | Monthly Cost |
|------|-------------|
| Domain + hosting | Already in infra budget |
| Plausible Analytics | $9/mo |
| ConvertKit (email) | $0 (free < 300 subscribers) |
| Canva Pro (graphics) | $12/mo |
| Buffer (social scheduling) | $0 (free plan) |
| **Total** | **$21/mo** |

No paid ads, no influencer spend, no PR agency until $1M ARR.
