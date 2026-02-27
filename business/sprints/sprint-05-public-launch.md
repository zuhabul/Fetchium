# Sprint 05: Public Launch

**Duration:** 2 weeks
**Theme:** HN front page, Product Hunt, 1K installs, 500 GitHub stars
**Goal:** Fetchium goes from "private beta" to "widely known developer tool" in a 2-week coordinated launch
**Dependency:** Sprint 04 complete (NPS >= 40, P0/P1 bugs fixed, 100 active beta users)

---

## Context

The public launch is a one-time opportunity to generate concentrated attention. Done well,
it creates organic momentum that compounds for months. Done poorly, you get a day of
traffic and silence.

**The goal is not to make a sale. The goal is to make developers curious enough to install.**

One install leads to one activation, leads to one recommendation, leads to 5 more installs.
Concentrated launch attention bootstraps the organic growth flywheel.

---

## Pre-Launch Checklist (Complete Before Day 1)

**Product:**
- [ ] `fetchium --version` shows a clean version number (not a pre-release)
- [ ] `fetchium quickstart` runs without errors on macOS, Linux, Windows (WSL)
- [ ] API is live at `api.fetchium.com` with 99.9%+ uptime in the last 7 days
- [ ] Docs site is live, all 5 tutorials are published
- [ ] LangChain adapter is published to PyPI: `pip install langchain-fetchium`
- [ ] Benchmark blog post is written and ready to publish (see Task 5.7)

**Infrastructure:**
- [ ] Auto-scaling is configured to handle 10x normal traffic
- [ ] Redis cache is enabled for hot endpoints
- [ ] Cloudflare cache rules are set for static content
- [ ] Error alerting is configured (Grafana → Discord webhook)
- [ ] Runbooks are written for the 3 most likely failure modes

**Social:**
- [ ] Launch tweets are drafted and scheduled for Day 0 and Day 1
- [ ] Beta users have been briefed: "Launch day is [date] — here's how you can help"
- [ ] Product Hunt submission is prepared (see Task 5.3)
- [ ] GitHub repo stars are at baseline (100+) from beta users

---

## Week 1: Launch Execution

### Day 0 (Sunday Night): Pre-Launch

**Task 5.1 — Notify beta users**

Email to all 100 beta users (send Sunday at 8pm EST):

> Subject: Fetchium launches tomorrow — here's how you can help
>
> Hey [name],
>
> Thanks for being in the Fetchium beta. Tomorrow is our public launch day.
>
> If you've found Fetchium useful, the biggest thing you can do:
> 1. Star the GitHub repo: github.com/fetchium/fetchium
> 2. Upvote on Product Hunt: producthunt.com/posts/fetchium (link goes live at 12:01am PST)
> 3. Retweet our launch tweet (link in tomorrow's email)
>
> That's it. No pressure — and thank you for helping us get here.
> — [Founder name]

### Day 1 (Monday): Show HN Launch

**Task 5.2 — Show HN post**

**Best time:** 9:00–10:00am EST on a Tuesday or Wednesday (A/B test is impossible — pick one)

**Post title:**
> "Show HN: Fetchium – typed web fetch for AI agents (open source)"

**Post body (keep it under 200 words):**
> Fetchium is a web fetch tool designed specifically for AI agents. It does three things:
>
> 1. Extracts clean content from any URL (HTML, PDF, JavaScript-heavy pages) using a 5-layer cascade
> 2. Reduces raw HTML to 10-20x fewer tokens with query-aware DOM distillation
> 3. Returns typed JSON with AI-enriched summaries, so your agent doesn't need to parse HTML
>
> We built this because every agent framework eventually needs a reliable web fetch layer —
> and the existing options (Tavily, Exa, raw requests) each have significant limitations.
>
> Key features:
> - 7 fetch modes: fast, extract, clean, AI-enriched, deep research, headless JS, PDF
> - Native adapters for LangChain and CrewAI
> - Self-hostable with SearXNG (zero cost, zero rate limits)
> - Rust core for reliability, Python/TypeScript SDKs for usability
>
> `cargo install fetchium` or `pip install fetchium-py`
>
> Would love feedback from anyone building AI agents. Happy to answer questions.

**Monitor and respond:**
- Check HN every 15–30 minutes for the first 4 hours
- Respond to every comment within 60 minutes
- Be substantive, not defensive — engage with criticism openly
- Do not ask friends to upvote (against HN rules)

**Task 5.3 — Product Hunt launch**

Product Hunt submissions go live at 12:01am PST. Launch on Tuesday (different audience from HN).

**Assets to prepare:**
- Thumbnail: 240×240px — clean logo on dark background
- Gallery: 4 screenshots showing CLI output, docs, LangChain example, dashboard
- Tagline (< 60 chars): "Typed web fetch for AI agents"
- First comment (write as maker): 200-word honest description of why you built it
- Product description: 300-word version of the HN post

**Tasks on PH launch day:**
- Post at 12:01am PST (manually, using PH scheduler)
- Send the beta user email at 9am PST with the PH link
- Post on Twitter/X at 9am PST: "We just launched on @ProductHunt..."
- Target: Top 5 product of the day

### Day 2–3: Twitter / Content Distribution

**Task 5.4 — Twitter launch thread**

**Thread format (5–7 tweets):**

Tweet 1:
> "Introducing Fetchium — the web fetch layer AI agents deserve.
>
> 7 modes. 10-20x token reduction. Open source. Self-hostable.
>
> Here's what it does and why we built it 🧵"

Tweet 2:
> "Every AI agent eventually needs to fetch web content.
>
> The options today:
> - Raw requests → you parse 100K tokens of HTML
> - Tavily / Exa → expensive, no self-host, opaque
> - LangChain loaders → inconsistent, hard to debug
>
> We needed something better."

Tweet 3:
> "Fetchium's QADD algorithm distills web pages to 10-20x fewer tokens
> by pruning everything a language model doesn't need:
> - Nav bars
> - Ads
> - Sidebars
> - Cookie banners
> - Tracking scripts
>
> Your agent sees the signal, not the noise."

Tweet 4:
> "7 fetch modes for every use case:
> - fast: 1–2 seconds, snippets only
> - extract: clean body text
> - ai: AI-enriched summary + key points
> - deep: 3-5 sources + synthesis
> - research: multi-agent 10-20 sources
> - headless: JavaScript-rendered pages
> - pdf: full document extraction"

Tweet 5:
> "Self-hosted = $0/fetch
>
> `fetchium setup` installs SearXNG + Chrome in one command.
> Your searches never leave your machine.
>
> For teams: $49/seat on our hosted version."

Tweet 6 (CTA):
> "`cargo install fetchium`
>
> Or try the hosted API:
> `curl api.fetchium.com/fetch?url=example.com`
>
> Star us on GitHub → github.com/fetchium/fetchium
> Docs → docs.fetchium.com"

**Task 5.5 — Cross-posting**

Post the launch announcement to:
- r/LocalLLaMA: "Fetchium: typed web fetch for AI agents (open source, self-hostable)"
- r/MachineLearning: same post
- LangChain Discord `#tools` channel
- CrewAI Discord `#announcements` (request from mods)
- AutoGen GitHub Discussions
- dev.to article version of the launch post

### Day 4–5: Benchmark Blog Post

**Task 5.7 — Publish "Fetchium vs Tavily vs Exa: Real Numbers"**

This is the highest-converting content piece for a developer tool. Developers Google
"[alternative] vs [alternative]" and this post captures that traffic forever.

**Structure:**
1. **Test methodology** (transparent, reproducible)
   - 50 URLs across: news, technical docs, research papers, paywalled content
   - Metrics: content quality, token count, latency, cost, self-hostability
2. **Results table** (a visual table is worth 10 paragraphs)
   - Fetchium wins on: cost, self-hostability, token efficiency, mode variety
   - Competitors win on: simplicity, existing ecosystem
3. **Detailed analysis** for each dimension
4. **When to use which tool** (be honest — some use cases favor competitors)
5. **Try it yourself** — code to reproduce the benchmark

**Where to publish:**
- Primary: `fetchium.com/blog/benchmarks` (SEO juice stays with us)
- Syndicated: dev.to, Hashnode (canonical link pointing back)
- Posted to HN as a new submission: "Fetchium vs Tavily vs Exa: 50-URL benchmark results"

---

## Week 2: Amplification & Iteration

### Day 6–7: Respond to Inbound

After a successful launch, there is inbound:
- GitHub Issues: questions, feature requests, bugs
- Discord: new members asking setup questions
- Twitter: replies, mentions, DMs
- Email: some developers email for help

**Task 5.8 — Rapid response protocol (48 hours post-launch)**

Respond to everything within 2 hours. First impressions stick.

Priority order:
1. Bug reports (fix if P0, acknowledge and timeline if P1)
2. Setup help (answer or point to docs)
3. Integration questions (often become tutorial topics)
4. Feature requests (thank them, add to GitHub Discussions)

### Day 8–10: Measure and Iterate

**Task 5.9 — Launch metrics review**

48 and 96 hours after launch, review:
- GitHub stars (target: 500)
- CLI installs (target: 1,000 new installs)
- API signups (target: 200 new accounts)
- HN score and comments
- Product Hunt final ranking and vote count
- Blog post unique visitors

**Where we missed target:** Understand why.
- Low installs despite high HN engagement → onboarding friction (fix immediately)
- Low HN engagement → post timing, title, or content (less actionable after the fact)
- Low PH votes → beta users didn't mobilize (process issue, not product issue)

**Task 5.10 — Press kit**

Prepare a press kit for any journalists or newsletter writers who reach out:
- `fetchium.com/press` page with:
  - Company fact sheet (1 page)
  - High-res logo and screenshots
  - Founder bio + headshot
  - Key metrics (GitHub stars, install count)
  - Media contact: `press@fetchium.com`

### Day 11–14: Sustain Momentum

**Task 5.11 — Week 2 content**

The launch creates a 48-hour spike. Week 2 content sustains it:

- **Tuesday:** "How we built Fetchium's QADD algorithm" (technical deep-dive, HN-friendly)
- **Thursday:** Demo video (3 minutes, screen-share showing `fetchium quickstart` → LangChain integration)
- **Friday:** Weekly changelog post ("What shipped this week at Fetchium")

**Task 5.12 — Reach out to 5 newsletters**

Developer newsletters to pitch a guest post or sponsor slot:
- tldr.tech (developer-focused, 500K+ subscribers)
- The Pragmatic Engineer (ML/AI edition)
- Import AI (Jack Clark — AI researchers, 50K+ subscribers)
- The Batch (Andrew Ng — AI practitioner audience)
- TLDR AI (AI-specific, 250K+ subscribers)

Pitch: "Would you feature Fetchium in an upcoming issue? Here's a 200-word write-up."

---

## Launch Day Runbook

**9:00am EST — Go time**

```
[ ] HN post submitted
[ ] Beta user email sent
[ ] Twitter thread posted
[ ] Discord announcements posted
[ ] Grafana dashboard monitoring started
[ ] On-call phone notifications enabled

Every 30 min for first 4 hours:
[ ] Check HN post ranking — respond to comments
[ ] Check error rate in Grafana — < 0.5% target
[ ] Check API latency — < 3s P95 target
[ ] Check disk/memory on server — no resource exhaustion

End of Day 1:
[ ] GitHub stars count
[ ] New signups count
[ ] HN score and rank
[ ] Post summary to Discord #announcements
```

---

## Definition of Done

Sprint 05 is complete when:
- [ ] GitHub stars >= 500
- [ ] CLI installs >= 1,000 (new, during sprint)
- [ ] API signups >= 200 (new, during sprint)
- [ ] HN Show HN post reached >= 50 points
- [ ] Product Hunt top 10 of the day
- [ ] Benchmark blog post published
- [ ] Zero P0 incidents during launch week (or resolved within 1 hour)
- [ ] Discord has >= 200 members

---

## What Success Looks Like vs. What It Doesn't

**Success:**
- HN front page, 200+ points, thoughtful comments from real developers
- 5K GitHub stars in 30 days post-launch
- Inbound from developers saying "I've been looking for this"

**Failure mode 1 — Traffic with no activation:**
HN hits, installs, but people don't come back. Signal: activation rate < 30%.
Fix: onboarding sprint immediately post-launch.

**Failure mode 2 — No traction at all:**
< 50 HN points, < 100 installs. This is a framing problem, not a product problem.
The product works (beta proved it). Change the angle: different HN title, different
positioning, different distribution channel.

**Never interpret silence as rejection.** One failed launch doesn't mean the product
is dead — it means that particular channel + angle didn't resonate. Try again in 4 weeks.
