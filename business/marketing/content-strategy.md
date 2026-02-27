# Fetchium — Content Strategy

**Last updated:** 2026-02-27
**Goal:** 50K monthly blog visitors by Month 12. Become the authoritative resource on AI search APIs and agent web search.

---

## Content Mission

Every piece of content Fetchium publishes should do one of three things:
1. **Teach** something concrete (a tutorial, a technique, an architecture)
2. **Prove** something with data (a benchmark, a case study, a metric)
3. **Entertain** the developer audience (behind-the-scenes, war stories, contrarian takes)

Content that doesn't do any of these is not worth publishing.

---

## Audience-First Content Pillars

### Pillar 1 — AI Agent Web Search (Primary)
The core topic. Every AI developer building agents needs web search. This is the exact problem Fetchium solves.

**Target reader:** AI engineer building a LangChain / LlamaIndex / AutoGen agent that needs live web data.

**Content examples:**
- "How to add web search to your AI agent in 5 minutes" (tutorial)
- "Why your AI agent gets stale answers (and how to fix it)" (problem/solution)
- "LangChain web search tools compared: Tavily vs Exa vs Fetchium" (comparison)
- "Building a self-improving research agent with Fetchium + GPT-4" (advanced tutorial)

### Pillar 2 — Search Engineering (Authority)
Deep technical content on how search works. Attracts senior developers and establishes credibility.

**Target reader:** Senior developer, ML engineer, or researcher who wants to understand the internals.

**Content examples:**
- "HyperFusion: how we combine 8 ranking signals" (algorithm deep dive)
- "Why BM25 still beats neural search for most queries" (contrarian take)
- "QATBE: query-aware token budgeting for LLM context windows" (original research)
- "Anatomy of a web extraction pipeline: CSS → readability → headless JS" (architecture)

### Pillar 3 — The AI Search Market (Thought Leadership)
Big-picture analysis of where the market is going. Attracts VCs, founders, and senior decision-makers.

**Content examples:**
- "The Bing API retirement created a $1B opportunity" (market analysis)
- "Why Perplexity's token pricing doesn't scale for agent use cases" (analysis)
- "Predicting the AI search market in 2027" (forecast)
- "Why the search API market is winner-take-most, not winner-take-all" (contrarian)

### Pillar 4 — Developer Productivity (Broad Reach)
Tutorial content on topics adjacent to Fetchium. Attracts developers who don't yet know they need Fetchium.

**Content examples:**
- "10 ways to make your AI agent smarter with live web data" (list + tutorial)
- "How to build a competitive intelligence tool with Python" (tutorial using Fetchium)
- "Automating research reports with AI: a complete guide" (long-form tutorial)
- "Real-time web data in your n8n workflow: step-by-step" (workflow tutorial)

---

## Content Types & Cadence

### Blog Posts (primary channel)
**Frequency:** 2 posts/month (Phase 1), 4/month (Phase 3+)
**Length:** 1,200–3,000 words
**Format:** Tutorial, comparison, deep dive, or case study
**SEO:** Every post targets one primary keyword (researched in advance)
**CTAs:** Always end with a code snippet + "try it now" link to playground

| Month | Post 1 | Post 2 |
|-------|--------|--------|
| M1 | "Introducing Fetchium" | "How we built a search API in Rust" |
| M2 | "HyperFusion ranking explained" | "CEP: extracting content from any webpage" |
| M3 | "AI agent web search in 5 minutes (LangChain)" | "Fetchium vs Perplexity API: a benchmark" |
| M4 | "Adding video search to your AI pipeline" | "Why SearXNG is the best search backend you've never heard of" |
| M5 | "Social media signal aggregation with Fetchium" | "Multi-agent research: how AMRS works" |
| M6 | "6 months of Fetchium: lessons from 1M API calls" | "AI search market: where we are in 2026" |

### Tutorial Videos (YouTube)
**Frequency:** 1/month starting M4
**Length:** 8–15 minutes
**Format:** Screen record + voiceover, no face camera required
**Style:** Fast-paced, skip the filler, get to the working code

| Month | Video Topic |
|-------|------------|
| M4 | "Fetchium in 5 minutes — full demo" |
| M5 | "Build a research agent with Fetchium + LangChain" |
| M6 | "Fetchium social + video mode walkthrough" |
| M7 | "Fetchium Knowledge Base — demo and tutorial" |
| M8 | "Building a monitor bot with Fetchium Webhooks" |

### Newsletter (monthly)
**Frequency:** Monthly
**Name:** "The Fetch" (Fetchium's newsletter)
**Platform:** ConvertKit (free < 300 subscribers, $29/mo at 300–1K)
**Length:** 600–1,000 words

**Newsletter sections:**
1. **Month in review** — what shipped (features, metrics, milestones)
2. **Deep dive** — one algorithmic or architectural topic explained
3. **Community spotlight** — best thing someone built with Fetchium this month
4. **Market intelligence** — one interesting development in AI search
5. **Quick tips** — 3 Fetchium tips/tricks power users don't know

**Growth tactics:**
- Newsletter signup at end of every blog post
- "Subscribe for the monthly benchmark" in README
- Link in Discord welcome message

### Code Examples & Cookbooks
**Where:** `github.com/fetchium/fetchium-examples` repo + docs.fetchium.dev/cookbook

**Examples to ship by Month 3:**
- [ ] LangChain agent with web search (Python)
- [ ] LlamaIndex retriever (Python)
- [ ] Next.js app with Fetchium search bar
- [ ] Python research pipeline (fetch → extract → synthesize → save)
- [ ] CLI one-liners cheat sheet
- [ ] n8n workflow JSON (import and use)

**Examples to ship by Month 6:**
- [ ] CrewAI research team with Fetchium
- [ ] Real-time competitor monitoring script
- [ ] Podcast transcript search tool
- [ ] Academic paper aggregator (arXiv + Semantic Scholar via Fetchium)

### Twitter/X Threads
**Frequency:** 1 thread/week
**Format:** 6–10 tweets, starts with a bold claim or number, ends with a CTA

**Thread templates:**
```
Thread: Algorithm deep dive
Tweet 1: "HyperFusion ranks search results using 8 signals simultaneously. Here's how it works: [thread]"
Tweets 2-8: One signal per tweet, with concrete example
Tweet 9: "The result: [before/after ranking example]"
Tweet 10: "Full code on GitHub: [link]. Docs: [link]. Questions? → [Discord]"
```

```
Thread: Benchmark
Tweet 1: "I benchmarked Fetchium, Perplexity API, Tavily, and Exa on 100 queries. Results: [image]"
Tweets 2-5: Deep dive on specific categories (factual, research, code, real-time)
Tweet 6: "Methodology: [link to GitHub benchmark repo]. Run it yourself."
Tweet 7: "TL;DR: [2-sentence summary]. Try Fetchium: [link]"
```

---

## SEO Strategy (integrated with content)

### Keyword Clusters

**Cluster 1: AI agent search (high intent)**
- Primary: "search api for ai agents" (1K/mo, medium competition)
- Secondary: "langchain web search tool", "add web search to langchain", "tavily alternative"
- Content: Tutorial posts, comparison posts

**Cluster 2: Web extraction (medium intent)**
- Primary: "web extraction api" (2K/mo, high competition)
- Secondary: "firecrawl alternative", "url to markdown api", "web scraping api python"
- Content: Comparison posts, tutorial posts

**Cluster 3: Developer tools (broad reach)**
- Primary: "ai research tool" (5K/mo, high competition)
- Secondary: "research automation api", "automated research report"
- Content: Product overview, use case posts

**Cluster 4: Brand/competitor (bottom of funnel)**
- Primary: "fetchium" (grows with brand)
- Secondary: "perplexity api alternative", "bing api replacement"
- Content: Landing page, comparison pages

### On-Page SEO Rules
- H1 = exact target keyword (every post)
- Meta description: 150 chars, includes keyword + value proposition
- Internal links: every post links to 3+ other Fetchium pages
- Code snippets: include `fetchium` CLI commands in most posts (anchors to brand)
- Images: all have alt text describing the visual + keyword context
- URL slugs: short and keyword-rich (`/blog/langchain-web-search-api`, not `/blog/2026-02-27-how-to-add-web-search-to-your-langchain-agent-in-five-minutes`)

### Link Building
**Phase 1 targets:**
- GitHub README → docs.fetchium.dev (massive referral traffic)
- HN posts → fetchium.dev (domain authority boost)
- Dev.to posts → fetchium.dev (rel="nofollow" but still traffic)
- Partner integrations → mutual links (LangChain docs, LlamaIndex docs)

**Phase 3 targets:**
- Guest posts on popular AI newsletters (The Batch, The Rundown, TLDR AI)
- Academic citation: publish an arXiv preprint on HyperFusion algorithm

---

## Content Production Process

### For each blog post:

**Day 1 (30 min):** Outline + keyword research
- Choose target keyword (check Search Console volume)
- 5-point outline
- Identify 3 internal links + 3 external authoritative links

**Day 2 (2 hours):** First draft
- Write without editing (get to 1,500+ words)
- Include at least one code snippet, one image/diagram, one concrete result

**Day 3 (1 hour):** Edit + publish
- Cut 20% of words (remove filler)
- Add SEO metadata (title tag, meta description, slug)
- Schedule on Dev.to and Hashnode (24 hours after fetchium.dev to avoid duplicate content)

**Day 4:** Distribution
- Twitter thread summarizing the post
- Discord announcement
- HN submission if high-quality
- LinkedIn post (plain text summary, no link in first comment)

---

## Content Metrics

| Metric | M3 Target | M6 Target | M12 Target |
|--------|-----------|-----------|-----------|
| Monthly blog visitors | 2,000 | 15,000 | 50,000 |
| Newsletter subscribers | 100 | 500 | 2,000 |
| YouTube subscribers | — | 200 | 1,000 |
| YouTube monthly views | — | 2,000 | 10,000 |
| Twitter followers | 300 | 1,500 | 5,000 |
| Organic signups from content | 20/mo | 100/mo | 400/mo |

**Attribution:** UTM parameters on all content links. Track `?utm_source=blog&utm_medium=organic&utm_campaign=[post-slug]` → Stripe signup conversion.
