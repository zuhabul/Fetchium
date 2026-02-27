# Fetchium — Competitive Analysis

**Last updated:** 2026-02-27
**Purpose:** Deep teardown of every major competitor. Know their pricing, limitations, and where we win.

---

## Market Landscape

The AI search and web extraction market is fragmenting into four categories:

1. **Consumer AI search** — Perplexity, You.com, Kagi (B2C, subscription)
2. **Developer/agent search APIs** — Tavily (acquired), Exa, Brave Search API, Serper
3. **Web extraction** — Firecrawl, Jina.ai, Diffbot
4. **Traditional search APIs** — Google Custom Search, Bing (retired Aug 2025)

Fetchium competes across categories 2 and 3 simultaneously, which is our structural advantage.

---

## 1. Perplexity AI

**Category:** Consumer AI search + developer API
**Revenue:** $656M ARR (2025)
**Users:** 45M+ MAU
**Valuation:** $20B (Series E)

### What They Do Well
- Best-in-class AI synthesis quality (Pro Search mode)
- Consumer UX is polished and fast
- Strong brand recognition
- Sonar API (developer product) has traction
- Citations are clean and verifiable

### Pricing (API — Sonar)
| Tier | Price | Notes |
|------|-------|-------|
| Sonar | $1 per 1K tokens | Context window costs extra |
| Sonar Pro | $3 per 1K tokens | Better model |
| Sonar Reasoning | $5 per 1K tokens | Slower, chain-of-thought |

**Effective cost for a typical query:** $0.005–$0.05 per query (token-dependent).
At 1M queries/month: $5K–$50K. Expensive at scale.

### Limitations
- API is token-priced, not query-priced — unpredictable costs
- No web extraction mode (fetch a specific URL)
- No social media aggregation
- No video content extraction
- No personal knowledge base
- No multi-agent research
- No on-prem option
- Rate limits poorly documented
- No streaming in cheaper tiers

### Our Advantage vs Perplexity
- Flat pricing: $49/mo Pro vs $5–$50K/mo at API scale
- Multi-modal: video + social + research in one call
- Extraction: CEP can fetch and extract any specific URL
- Memory: PIE cross-session learning (Perplexity is stateless)
- Agent-native: purpose-built for LangChain/LlamaIndex
- On-prem: available Phase 5 (Perplexity never will be)

---

## 2. Tavily

**Category:** Developer search API for AI agents
**Status:** Acquired by Nebius (GPU cloud) for ~$25M
**Users:** 800K+ developers before acquisition

### What They Do Well
- First-mover in "search API for AI agents" positioning
- LangChain integration drove most adoption
- Simple, predictable pricing
- Clean REST API with good docs
- Fast response times

### Pricing (estimated pre-acquisition)
| Tier | Price | Searches |
|------|-------|---------|
| Free | $0 | 1,000/month |
| Basic | $20/mo | 15,000/month |
| Pro | $50/mo | 50,000/month |
| Business | $200/mo | 250,000/month |

### Limitations (significant post-acquisition concerns)
- Nebius is a GPU cloud company — search API is not their core business
- Development likely to slow or stagnate
- No video extraction
- No social aggregation
- No personal KB or memory
- No CLI tool
- No research mode
- Extraction quality lower than Firecrawl
- Future pricing and availability uncertain

### Our Advantage vs Tavily
- Actively developed; Tavily is now a side product at a GPU company
- Feature breadth: video, social, KB, research — Tavily has none of these
- Better extraction quality (CEP vs Tavily's basic fetch)
- CLI for developers who prefer terminal workflows
- More transparent roadmap and pricing

---

## 3. Exa (formerly Metaphor)

**Category:** Neural/semantic web search API
**Funding:** $17M Series A
**Focus:** Semantic search (find pages *similar to* a given URL or description)

### What They Do Well
- Unique neural search approach (embedding-based, not keyword)
- Excellent for "find me more like this" queries
- Good at finding specific types of content (papers, blog posts)
- Clean API and good documentation
- Highlights feature: extracts the most relevant text from results

### Pricing
| Tier | Price | API Calls |
|------|-------|----------|
| Free | $0 | 1,000/month |
| Developer | $20/mo | 5,000/month |
| Startup | $50/mo | 20,000/month |
| Growth | $150/mo | 100,000/month |
| Business | $500/mo | 500,000/month |

### Limitations
- Neural search only — no keyword/BM25 fallback
- Can miss exact keyword queries that BM25 handles well
- No AI synthesis (just search + extract)
- No social aggregation
- No video
- No personal KB
- No research mode
- Smaller index than Google/Bing (built their own)
- Index freshness can lag for very recent events

### Our Advantage vs Exa
- Hybrid ranking (HyperFusion: BM25 + semantic + 6 other signals)
- AI synthesis built-in (Exa only returns results, no answer)
- Multi-modal (video, social, research)
- Fresh index via SearXNG (real-time from Google/Bing engines)
- Flat pricing vs per-call pricing at scale

---

## 4. Firecrawl

**Category:** Web scraping and extraction API
**Focus:** Clean content extraction from any URL, site crawling
**Users:** 10K+ developers (estimated)

### What They Do Well
- Best-in-class extraction quality (Markdown output is clean)
- Site crawl mode (crawl entire domains)
- LLM-friendly output format
- Scraping actions (click, scroll, fill forms)
- Map feature: extract sitemap structure

### Pricing
| Tier | Price | Pages |
|------|-------|-------|
| Free | $0 | 500 pages/month |
| Hobby | $16/mo | 3,000 pages/month |
| Standard | $83/mo | 100,000 pages/month |
| Growth | $333/mo | 500,000 pages/month |

### Limitations
- Extraction only — no search capability
- No AI synthesis
- No social aggregation
- No video
- No personal KB
- No research mode
- Rate limits can be hit quickly
- Some sites' dynamic content still requires manual configuration

### Our Advantage vs Firecrawl
- Search + extract in one API call (Firecrawl requires a separate search API)
- AI synthesis on extracted content
- Social and video modes
- CEP quality comparable for most use cases (Phase 2+)
- Cheaper at low-to-medium volume

### Where Firecrawl Wins
- Pure extraction quality: Firecrawl edges us for complex SPAs and JS-heavy sites
- Site crawl depth: Firecrawl's crawler is more mature
- **Positioning:** We don't compete head-on on extraction. We win on "search + extract + synthesize" in one call.

---

## 5. Jina.ai

**Category:** Multi-modal AI and web extraction
**Focus:** Embeddings API + reader API (URL → clean text)
**Notable:** r.jina.ai is a free URL-to-text service popular with developers

### What They Do Well
- `r.jina.ai/URL` prefix trick is extremely viral (zero signup required)
- Good extraction quality
- Multi-modal models (text + image embeddings)
- Free tier is generous
- Rerank API useful for RAG pipelines

### Pricing (Reader API)
| Tier | Price |
|------|-------|
| Free | 20 RPM, 200 RPD |
| Paid | $0.02 per 1K tokens |

### Limitations
- Reader API is not search — it only fetches a specific URL
- No web search capability
- No AI synthesis
- No social, video, research modes
- Token pricing is expensive at scale
- Their focus is shifting to embeddings, not search

### Our Advantage vs Jina
- Search + extract: Jina only does extract
- All-in-one API: one key, one SDK, search → extract → synthesize
- Flat pricing at Pro tier vs token-based

---

## 6. Google (Custom Search + Gemini)

**Category:** Enterprise search API (paid) + consumer AI (free)
**Status:** Google Custom Search JSON API is their only pure search API

### Custom Search API Pricing
| Usage | Price |
|-------|-------|
| First 100 queries/day | Free |
| 101–10,000 queries/day | $5 per 1,000 queries |

**Cost at 1M queries/month:** $5,000.
**Index quality:** Excellent (it's Google).
**Freshness:** Excellent.

### Limitations
- $5/1K queries is expensive
- No AI synthesis in the API
- Returns 10 results max per query
- No extraction mode
- No social, video, research modes
- Requires Google Cloud billing account
- CSE (Custom Search Engine) setup is complex
- Rate limits are strict and poorly documented

### Gemini (separate product)
- Gemini Advanced: consumer product, not an extraction/search API
- Gemini API does have grounding (Google Search integration) — closest to Fetchium
- Grounding cost: additional $35 per 1K queries on top of token cost

### Our Advantage vs Google
- 10x cheaper for AI search (we're $49/mo flat vs $5K/mo at 1M queries)
- Social + video + research: Google API has none of these
- Simpler setup: one API key, no GCP account required
- Better developer experience (CLI, better docs, Discord support)

---

## 7. Brave Search API

**Category:** Independent web search index
**Status:** Active, growing
**Key strength:** Completely independent index (no Google/Bing data)

### Pricing
| Tier | Price | Queries |
|------|-------|---------|
| Free | $0 | 2,000/month |
| Basic | $3/month | 500/month |
| Pro | $5–9/month | variable |
| Data for AI | $5 per 1K queries | |

### Why We Use Brave (Instead of Competing With It)
Brave Search is in Fetchium's **backend chain** as a SearXNG engine. We don't compete — we build on top of it.

When SearXNG aggregates search results, Brave is one of the engines. This gives us:
- Freshness from an independent index
- Privacy-respecting results
- No Bing dependency

**We add value on top:** HyperFusion ranking, AI synthesis, social modes, etc.

---

## Competitive Matrix

| Capability | Fetchium | Perplexity API | Tavily | Exa | Firecrawl | Jina |
|------------|----------|---------------|--------|-----|-----------|------|
| Web search | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| AI synthesis | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| URL extraction | ✅ | ❌ | Partial | ✅ | ✅ | ✅ |
| Video extraction | ✅ (P2) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Social aggregation | ✅ (P2) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Research mode | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Personal KB | ✅ (P3) | ❌ | ❌ | ❌ | ❌ | ❌ |
| CLI | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Streaming | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| On-prem | ✅ (P5) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Flat pricing | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ |
| LangChain | ✅ (P4) | ✅ | ✅ | ✅ | ✅ | ✅ |

**Unique to Fetchium (no competitor offers all three):**
- Video extraction + Social aggregation + Personal KB in one API

---

## Positioning Statement

**For AI developers and research teams** who need reliable, affordable web search and content extraction for their agents, pipelines, and workflows — **Fetchium** is the only API that combines web search, video extraction, social aggregation, and multi-agent research with a flat monthly price, open-source transparency, and a knowledge base that learns from your queries.

Unlike Perplexity (expensive token pricing, no extraction), Tavily (acquired, stagnating, limited features), or Exa (neural-only, no synthesis), Fetchium is actively developed, multi-modal from day one, and built for the agent era.

---

## Win/Loss Themes

**We win when:**
- The buyer needs more than one data type (web + social, or web + video)
- Cost predictability matters (our flat pricing vs Perplexity's token pricing)
- The team uses a CLI (unique to Fetchium)
- On-prem or data residency is required
- The buyer wants an actively-developed product (vs acquired Tavily)

**We lose when:**
- Pure extraction quality is the only criterion (Firecrawl edges us on complex JS sites)
- The buyer already uses Perplexity for search and just needs one simple endpoint
- The buyer only does 100 queries/day (Google CSE free tier is sufficient)

**Response to "why not just use Perplexity Sonar?":**
> At 100K queries/month, Sonar costs $500–$5,000 depending on token usage. Fetchium Pro+ is $99/month — flat. Sonar also can't extract a specific URL, aggregate Reddit discussions, or transcribe a YouTube video. Fetchium does all of that in one API call.
