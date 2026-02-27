# Go-to-Market Strategy

## Overview

Fetchium targets a developer-first PLG (product-led growth) motion. The product sells
itself when developers integrate it into an agent workflow and immediately see structured,
AI-enriched results with zero prompt engineering overhead. Every activated user is a
potential evangelist in a Slack, a GitHub issue thread, or a conference talk.

**Core thesis:** Build the fetch layer every AI agent eventually needs. Let developers
discover it organically, fall in love with it, and then inevitably pull their team or
employer in behind them.

---

## Target Segments (Priority Order)

| Segment | Description | Size | Urgency |
|---------|-------------|------|---------|
| **AI agent builders** | Devs building LangChain/CrewAI/AutoGen workflows | ~2M devs | High — need reliable fetch now |
| **Research engineers** | ML researchers needing structured web data | ~500K | High — tedious work today |
| **Indie hackers** | Solo builders shipping AI products | ~300K | Medium — cost-sensitive |
| **Enterprise AI teams** | Internal AI tooling at >500-person companies | ~10K teams | Medium — long sales cycle |
| **Data engineers** | ETL pipelines incorporating live web content | ~1M devs | Low — less urgent |

---

## Top of Funnel — Awareness

### GitHub Stars (Primary)
- Ship a polished README with a 30-second GIF: query in, structured JSON out
- Add Fetchium to `awesome-llm-tools`, `awesome-agents`, `awesome-search` lists
- Star-hunt: post to r/LocalLLaMA, r/MachineLearning with real benchmark comparisons
- Target: 500 stars in first 30 days, 5K by month 6

### Hacker News
- "Show HN: Fetchium — typed, AI-enriched web fetch for AI agents" — launch post
- "Ask HN: How do you give your agents reliable web access?" — community question
- Technical deep-dives as blog posts that surface on HN front page organically
- Respond to every mention of Perplexity API, Tavily, or Exa with a calm, factual comparison

### Developer Blog
- Post 1: "Why every AI agent eventually reinvents a web fetcher (and gets it wrong)"
- Post 2: "Benchmarking AI search APIs: Perplexity vs Tavily vs Exa vs Fetchium"
- Post 3: "How we get 10x token reduction from raw HTML using QADD"
- Post 4: "Building a research agent with LangChain + Fetchium in 50 lines"
- Post 5: "The real cost of Bing API retiring: what agents should do instead"
- Publish to: personal blog, dev.to, Hashnode, Medium (with canonical link)

### OSS Contributions & Visibility
- Contribute a `fetchium` integration to LangChain community tools
- Open issues in popular agent frameworks pointing to Fetchium as a solution
- Sponsor or contribute to SearXNG (upstream dependency respect)
- Add Fetchium examples to popular LLM cookbooks (OpenAI, Anthropic, Google)

### Community Presence
- Discord: join LangChain, CrewAI, AutoGen, LocalLLaMA servers — be genuinely helpful
- Twitter/X: post weekly agent-building tips; include Fetchium where naturally relevant
- YouTube: 3-5 min screencasts showing real agent workflows — no sales pitch, just value

---

## Middle of Funnel — Activation

The moment of truth is the **first successful fetch**: a developer runs one command or
makes one API call and gets back clean, structured, AI-enriched content. This must take
less than 5 minutes from install to first result.

### Activation Flow
```
1. See Fetchium mentioned (GitHub / HN / Discord)
2. README → `pip install fetchium` or `cargo install hsx` (< 30 seconds)
3. `fetchium fetch "https://example.com"` → beautiful JSON in terminal
4. Try `fetchium ai "what is the latest on X?"` → gets a real answer
5. Integrate into their existing agent — it just works
6. Hit the free tier limit (10 fetches/day) → considers upgrading
```

### Reducing Activation Friction
- Zero-config default: works without API keys using built-in SearXNG
- Instant gratification: first fetch returns in < 3 seconds on average
- Typed output: JSON schema in docs, TypeScript types published as npm package
- Error messages are actionable: "Rate limit hit — upgrade to Pro for unlimited fetches"
- `fetchium quickstart` command walks through a 3-step tutorial interactively

### Content for Activation
- 5-minute quickstart doc (the most important page on the docs site)
- Integration guides: LangChain, CrewAI, AutoGen, raw OpenAI function calling
- Example agents in a `fetchium-examples` GitHub repo (50+ stars target)
- Jupyter notebooks for research engineer use cases

---

## Bottom of Funnel — Conversion

### Self-Serve Upgrade Path
1. User hits free tier limit (10 fetches/day or 3 AI queries/day)
2. Clear in-product message: "You've used your free fetches today. Pro is $12/month."
3. Click → Stripe checkout → immediate unlock, no sales call needed
4. Welcome email with 3 power-user tips + link to Discord

### Team Expansion
1. Pro user is building something in a team context
2. Fetches results they share with colleagues — colleagues see Fetchium branding
3. "Invite your team" CTA in the dashboard — Teams plan unlocks shared knowledge base
4. One admin manages billing; all team members get unlocked access

### Enterprise Path
1. Enterprise users typically start with individual Pro accounts (2-5 people)
2. Signal: same email domain on multiple Pro accounts → trigger outreach email
3. "Looks like your team is using Fetchium — let's talk about an enterprise plan"
4. Enterprise offer: on-prem, SSO, SLA, dedicated Slack channel, custom limits
5. Typical deal size: $20K–$100K/year; close via async Loom + proposal doc

---

## PLG Metrics (Weekly Review)

| Metric | Target (Month 1) | Target (Month 6) | Target (Month 12) |
|--------|-----------------|-----------------|------------------|
| New signups / week | 50 | 500 | 2,000 |
| Activation rate (first fetch / 24h) | 40% | 55% | 65% |
| D7 retention | 25% | 35% | 45% |
| Free → Pro conversion | 2% | 5% | 8% |
| MRR | $0 | $5K | $30K |
| GitHub stars | 200 | 3K | 10K |

---

## Launch Sequence

### Week -2: Soft Launch (Friends & Colleagues)
- Share with 20-30 trusted developers for early feedback
- Fix the top 5 friction points before public launch

### Week 0: Show HN Launch
- Post "Show HN: Fetchium" at 9am EST Tuesday (historically best slot)
- Have 3-5 friends ready to comment with real use cases
- Monitor and respond to every comment within 1 hour

### Week 1: Product Hunt Launch
- Submit to Product Hunt same week — different audience, different day
- Prepare hunter + maker assets: tagline, thumbnail, first comment, demo GIF
- Goal: Top 5 of the day

### Week 2+: Sustained Growth
- Publish benchmark blog post: "Fetchium vs Tavily vs Exa — real numbers"
- Reach out to 5 AI newsletter authors with a guest post or sponsorship offer
- Start weekly changelog posts ("What shipped this week at Fetchium")

---

## Competitive Positioning

| vs. Perplexity API | We are the fetch layer, not the answer layer. Complement, don't compete. |
|--------------------|-------------------------------------------------------------------------|
| vs. Tavily | Open-source core, self-hostable, 10x cheaper at scale, better modes |
| vs. Exa | We support all content types (PDF, headless JS), not just clean web |
| vs. raw requests | Zero-config content extraction, AI enrichment, typed output |
| vs. BeautifulSoup | Async, AI-aware, multi-mode, structured output — not a scraping library |

**Positioning statement:** "Fetchium is the typed web fetch layer for AI agents — structured
results, AI-enriched summaries, and 7 extraction modes in one SDK."

---

## Year 1 GTM Budget

| Channel | Monthly Budget | Expected Output |
|---------|---------------|-----------------|
| Blog content (time) | $0 (founder time) | 2 posts/month |
| OSS bounties | $200 | Community PRs |
| Conference travel | $500/quarter | 1-2 talks/quarter |
| Newsletter sponsorships | $500 | 1 sponsor slot/month |
| Total cash | ~$1,000/month | — |

The majority of early growth is time-invested, not money-invested. This is intentional
for a bootstrapped launch — convert attention to revenue before spending on paid channels.
