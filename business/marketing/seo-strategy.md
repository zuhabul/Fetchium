# Fetchium — SEO Strategy

**Last updated:** 2026-02-27
**Goal:** 50K monthly organic visitors by Month 12. 200K by Month 24.
**Tools:** Google Search Console (free), Plausible Analytics ($9/mo), Ahrefs (when > $10K MRR)

---

## SEO Philosophy

Developer tools rank through **genuine helpfulness and technical depth**, not keyword stuffing. Google's HCU (Helpful Content Update) rewards content that demonstrates real expertise.

**Three principles:**
1. Every page must be the best answer to its target query on the internet
2. Technical depth is SEO — developers search for specific things; answer specifically
3. Speed is SEO — Core Web Vitals matter, and fast pages convert better

---

## Site Architecture

### Domain Strategy

```
fetchium.dev              → Landing page (product overview, pricing, sign up)
docs.fetchium.dev         → Documentation site
app.fetchium.dev          → Dashboard (authenticated, not indexed)
api.fetchium.dev          → API (not indexed)
plugins.fetchium.dev      → Plugin marketplace (Phase 4, indexed)
status.fetchium.dev       → Status page (not indexed)
```

**Why separate docs subdomain?**
- Cleaner URL structure (`docs.fetchium.dev/cli/search` vs `fetchium.dev/docs/cli/search`)
- Independent deployment (docs can update without marketing site deploy)
- Search Console can track docs separately

### URL Structure

```
fetchium.dev/
  /pricing
  /docs → redirects to docs.fetchium.dev
  /blog/
    /blog/[slug]
  /changelog
  /vs/
    /vs/perplexity
    /vs/tavily
    /vs/exa
    /vs/firecrawl

docs.fetchium.dev/
  /quickstart
  /cli/
    /cli/search
    /cli/fetch
    /cli/ai
    /cli/research
    /cli/video (Phase 2)
    /cli/social (Phase 2)
  /api/
    /api/reference
    /api/authentication
    /api/rate-limits
    /api/errors
    /api/endpoints/
      /api/endpoints/search
      /api/endpoints/fetch
      /api/endpoints/ai
      /api/endpoints/research
  /sdks/
    /sdks/python
    /sdks/javascript
    /sdks/go
    /sdks/rust
  /integrations/
    /integrations/langchain
    /integrations/llamaindex
    /integrations/crewai
    /integrations/n8n
  /cookbook/
    /cookbook/ai-agent
    /cookbook/research-pipeline
    /cookbook/competitor-monitoring
```

### Comparison Pages (`/vs/`)

High-intent pages for searchers actively evaluating alternatives.

**Target queries:**
- "fetchium vs perplexity" → `/vs/perplexity`
- "tavily alternative" → `/vs/tavily`
- "exa search alternative" → `/vs/exa`
- "firecrawl alternative" → `/vs/firecrawl`
- "bing api replacement" → `/vs/bing-api` (very high intent since Bing retired Aug 2025)

**Page structure (each /vs/ page):**
1. H1: "Fetchium vs [Competitor]: [year] Comparison"
2. Quick summary table (5 rows, key differences)
3. Feature comparison (full table)
4. Pricing comparison (side by side)
5. Performance benchmark (latency, accuracy — link to open methodology)
6. When to choose Fetchium vs when to choose [competitor]
7. Migration guide (if they're switching from competitor)
8. CTA: "Start free — no credit card required"

---

## Keyword Research

### Tier 1 — High Intent, Medium Volume

These searchers are ready to try or buy. Convert at 5-15%.

| Keyword | Monthly Volume | Difficulty | Target Page |
|---------|--------------|------------|------------|
| search api for ai agents | 800 | Low | fetchium.dev |
| langchain web search tool | 1,200 | Low | /integrations/langchain |
| tavily alternative | 400 | Low | /vs/tavily |
| bing api replacement 2025 | 600 | Low | /vs/bing-api |
| web search api python | 2,000 | Medium | /sdks/python |
| perplexity api alternative | 300 | Low | /vs/perplexity |
| add web search to chatbot | 500 | Low | /cookbook/ai-agent |

### Tier 2 — Medium Intent, High Volume

These searchers are researching. Convert at 1-3%.

| Keyword | Monthly Volume | Difficulty | Target Page |
|---------|--------------|------------|------------|
| ai search api | 3,000 | High | fetchium.dev |
| web scraping api | 15,000 | High | /vs/firecrawl |
| url to text api | 2,000 | Medium | /api/endpoints/fetch |
| langchain retriever | 4,000 | High | /integrations/langchain |
| research automation tool | 1,500 | Medium | /cookbook/research-pipeline |
| ai agent tools | 5,000 | High | fetchium.dev |

### Tier 3 — Educational, High Volume

Broad reach. Convert at 0.1-0.5%, but drive brand awareness and backlinks.

| Keyword | Monthly Volume | Difficulty | Target Page |
|---------|--------------|------------|------------|
| how to add web search to ai | 8,000 | Medium | /blog/ai-agent-web-search |
| bm25 vs semantic search | 2,000 | Medium | /blog/bm25-vs-semantic |
| web extraction techniques | 3,000 | Medium | /blog/cep-extraction |
| langchain tutorial 2026 | 10,000 | High | /blog/langchain-tutorial |
| rag vs fine tuning | 20,000 | High | /blog/rag-vs-fine-tuning (informational) |

### Long-Tail (Phase 3+)

The long tail drives 70%+ of organic traffic at scale. Capture with deep documentation and tutorial content.

Examples:
- "fetchium python sdk example" → /sdks/python
- "how to use searxng as search backend" → /blog/searxng-backend
- "langchain tavily vs fetchium" → /vs/tavily
- "ai agent that searches the web python" → /cookbook/ai-agent
- "monitor website changes with api" → /cli/monitor

---

## Technical SEO

### Core Web Vitals Targets

| Metric | Target | How |
|--------|--------|-----|
| LCP (Largest Contentful Paint) | < 2.5s | Next.js Image optimization, CDN |
| INP (Interaction to Next Paint) | < 200ms | Minimal JS on landing page |
| CLS (Cumulative Layout Shift) | < 0.1 | Explicit image dimensions |
| TTFB (Time to First Byte) | < 200ms | Cloudflare caching |

### Implementation Checklist

**Landing page (fetchium.dev):**
- [ ] `robots.txt` allows all crawlers, blocks `/dashboard`, `/api`
- [ ] `sitemap.xml` auto-generated, submitted to Search Console
- [ ] Canonical URLs set on all pages (`<link rel="canonical">`)
- [ ] Open Graph tags: `og:title`, `og:description`, `og:image` on every page
- [ ] Twitter Card meta tags: `twitter:card`, `twitter:title`, `twitter:description`
- [ ] JSON-LD structured data: `SoftwareApplication` schema on landing page
- [ ] `hreflang` tags for future i18n (Phase 6)
- [ ] No broken internal links (CI check with `next-sitemap`)
- [ ] 301 redirects for any old URLs (never 302)
- [ ] HTTPS enforced (Cloudflare)
- [ ] `gzip` / `brotli` compression enabled

**Docs site (docs.fetchium.dev):**
- [ ] Every page has a unique `<title>` (`Page Title | Fetchium Docs`)
- [ ] Meta description on every page (120-150 chars)
- [ ] Code blocks have `lang` attribute for syntax highlighting
- [ ] "Last updated" timestamp on every doc page
- [ ] Breadcrumb navigation (helps Google understand structure)
- [ ] Search (Algolia DocSearch — free for open-source docs)
- [ ] Version switcher (when API v2 ships — Phase 4)

**Blog (`fetchium.dev/blog`):**
- [ ] Article schema (`@type: "Article"`) with author, date, image
- [ ] Estimated reading time in post header
- [ ] Table of contents for posts > 1,500 words
- [ ] "Related posts" section at bottom (increases session depth)
- [ ] RSS feed (`/blog/rss.xml`) — developers subscribe to these

### Page Speed

**Next.js optimizations:**
```js
// next.config.js
module.exports = {
  images: { domains: ['fetchium.dev'], formats: ['image/avif', 'image/webp'] },
  experimental: { optimizeCss: true },
  compress: true,
}
```

**Cloudflare rules:**
- Cache landing page HTML: 4 hours
- Cache static assets (JS, CSS, images): 30 days
- Cache blog posts: 1 hour

**Image handling:**
- All screenshots: convert to WebP (50% smaller than PNG)
- Hero images: AVIF first, WebP fallback
- Animated demos: use `<video>` not GIF (10x smaller)

---

## Link Building Strategy

### Tier 1 — High Authority, Relevant (Priority)

**Strategy:** Build integrations → get linked from integration docs.

| Target site | Page | Why it matters |
|------------|------|---------------|
| LangChain docs | /docs/integrations/tools | 50K+ monthly visitors who need search |
| LlamaIndex docs | /docs/tools | Same audience |
| Hugging Face | /spaces or /datasets | ML developer hub |
| Papers With Code | listing page | Academic + practitioner crossover |
| dev.to organization | profile | Follow-back links from authors |

**How to earn:** Ship the integration, submit a PR to their docs with a Fetchium page, then request a listing.

### Tier 2 — Developer Directories

- **Awesome lists:** Submit to `awesome-langchain`, `awesome-llm`, `awesome-selfhosted`
- **AlternativeTo:** Create Fetchium listing, list alternatives to Bing API, Tavily
- **Product Hunt:** Listing links back to fetchium.dev (PageRank from PH)
- **npm:** `fetchium` package links to fetchium.dev
- **crates.io:** `fetchium` crate links to fetchium.dev
- **PyPI:** `fetchium` package links to fetchium.dev + docs

### Tier 3 — Content Marketing Links

- Guest posts on AI newsletters (The Batch, TLDR AI, The Rundown)
- arXiv preprint: publish HyperFusion algorithm paper → academic citations
- Open-source benchmark repo: "AI Search API Benchmark" → referenced in other blog posts
- GitHub sponsorship of popular AI repos → mentions in their README

### Disavow Policy

Monitor backlinks monthly (Google Search Console). Disavow:
- Spam directories
- Link farms
- PBNs (private blog networks)

---

## Measurement & Reporting

### Monthly SEO Report (takes 20 min to compile)

**Data sources:**
- Google Search Console: clicks, impressions, CTR, position
- Plausible Analytics: sessions, top pages, bounce rate, conversions
- Manual: GitHub stars correlation with organic traffic

**Key metrics to track:**

| Metric | M3 | M6 | M12 |
|--------|----|----|-----|
| Organic sessions/month | 1,000 | 8,000 | 50,000 |
| Indexed pages | 30 | 80 | 200 |
| Pages in top 10 (Search Console) | 5 | 20 | 60 |
| Average position (all keywords) | 25 | 15 | 10 |
| Domain Rating (Ahrefs, when subscribed) | — | 20 | 35 |
| Backlinks | 50 | 200 | 800 |
| Organic signups/month | 10 | 80 | 400 |

### Search Console Setup

1. Add `fetchium.dev` and `docs.fetchium.dev` as separate properties
2. Submit `sitemap.xml` for both
3. Set up email alerts for coverage issues
4. Review weekly: new queries ranking on page 2 → optimize those pages

### Quick Wins to Prioritize in Month 1

1. **Submit sitemap** to Search Console (immediate indexing improvement)
2. **Fix Core Web Vitals** — Next.js often has LCP issues with unoptimized images
3. **Write one /vs/ page** — "Bing API Replacement 2025" is near-zero competition with high intent
4. **Add Article schema** to every blog post — helps Google display rich results
5. **Get listed on AlternativeTo** — free, immediate referral traffic
6. **Submit to Algolia DocSearch** — improves docs UX and signals quality to Google
