# Strategic Partnerships

## Partnership Philosophy

Partnerships for a solo-founder developer tool must be asymmetric by design: we do
the integration work, they get a better product for their users, we get distribution.
No co-marketing budgets, no joint press releases, no months of negotiation — just a
clean adapter that their community can discover and use.

**Priority rule:** Start with integrations (code), not relationships (people). Ship the
adapter first, then reach out to maintainers with a working PR.

---

## Tier 1: AI Framework Integrations (Immediate Priority)

These are the frameworks where Fetchium's target users already live. An integration
here means Fetchium appears in every getting-started tutorial, cookbook, and example.

### LangChain

**Opportunity:** LangChain has ~95K GitHub stars and is the most widely used agent
framework. Many tutorials use `WebBaseLoader` or `TavilySearchResults` — both inferior
to what Fetchium provides.

**Integration plan:**
- Build `langchain-fetchium` Python package (separate repo: `fetchium/langchain-fetchium`)
- Implements `BaseRetriever` and `BaseTool` interfaces
- Two tools: `FetchiumFetchTool` (single URL) and `FetchiumSearchTool` (query → results)
- Submit to `langchain-community` as a PR — maintain in our repo, they list in their docs

**Example:**
```python
from langchain_fetchium import FetchiumSearchTool

tool = FetchiumSearchTool(api_key="hsx_...")
agent = create_react_agent(llm, [tool], prompt)
result = agent.invoke({"input": "latest AI research on RAG"})
```

**Timeline:** 2 weeks to build, 2 weeks to review. Target: in langchain-community docs by Month 3.

**Expected impact:** 500–2,000 new signups from LangChain community within 6 months of inclusion.

---

### CrewAI

**Opportunity:** CrewAI is the fastest-growing agent framework (30K+ stars), focused on
multi-agent workflows. Web fetch is a core primitive in every crew.

**Integration plan:**
- Build `crewai-fetchium` Python package
- Implements `BaseTool` with `FetchiumFetchTool` and `FetchiumResearchTool`
- `FetchiumResearchTool` uses the research mode — gives CrewAI a native deep-research capability
- Reach out to João Moura (creator) with a working integration

**Example:**
```python
from crewai_tools import FetchiumResearchTool

researcher = Agent(
    role="Research Analyst",
    tools=[FetchiumResearchTool()],
    ...
)
```

**Timeline:** 1 week (shares codebase with LangChain adapter, just different interface).

---

### AutoGen (Microsoft)

**Opportunity:** AutoGen is Microsoft's agent framework — enterprise-credibility and
a different audience than LangChain/CrewAI (more corporate, more Windows/.NET users).

**Integration plan:**
- Build Python `fetchium` tool compatible with `autogen.ConversableAgent`
- Register as a `FunctionTool` with typed input/output schemas
- Submit example to AutoGen's cookbook repository

**Timeline:** 1 week. Lower priority than LangChain/CrewAI — pursue after those ship.

---

### Semantic Kernel (Microsoft)

**Opportunity:** Semantic Kernel is used by Microsoft enterprise customers. Enterprise
distribution channel for Fetchium.

**Integration plan:**
- Build `FetchiumPlugin` as a C# NuGet package and Python package
- Implements `KernelPlugin` with `FetchAsync` and `SearchAsync` functions
- Target audience: enterprise .NET teams — different from typical Fetchium user

**Timeline:** Month 6+ (lower priority; requires C# expertise or contributor).

---

### DSPy (Stanford)

**Opportunity:** DSPy is gaining significant traction in research and serious ML
engineering communities — exactly Fetchium's research engineer segment.

**Integration plan:**
- Build `fetchium-dspy` adapter implementing `dspy.Retrieve` interface
- Create a DSPy program example: `FetchiumRetriever → ChainOfThought → Answer`
- Submit to DSPy examples gallery

**Timeline:** Month 3 (small adapter, high-value community).

---

## Tier 2: Cloud Marketplace Listings

Cloud marketplace listings provide enterprise credibility and access to budget-holder
customers who prefer to procure through existing vendor relationships.

### AWS Marketplace

**Product type:** SaaS listing (API subscription)
**Setup:** Register as AWS Marketplace seller; connect Stripe → AWS Marketplace payment flow
**Listing:** "Fetchium AI Web Fetch API — Structured web content for AI agents"
**Value:** Enterprise customers can pay via their existing AWS commitment (EDP programs)

**Timeline:** Month 8 (requires $1K+ MRR to justify the listing fee overhead)
**Cost:** 3% of revenue through marketplace
**Expected uplift:** 10–15% of enterprise deals come through marketplace once listed

### GCP Marketplace

**Product type:** SaaS listing via Google Cloud Partner Advantage
**Setup:** Partner Advantage portal enrollment; OAuth flow for GCP customers
**Value:** GCP customers get Fetchium credits applied to their cloud bill

**Timeline:** Month 10 (pursue after AWS listing is live and generating leads)

### Azure Marketplace

**Product type:** SaaS listing via Microsoft Partner Network
**Notes:** Higher barrier to entry; best pursued when enterprise deals exceed $100K ARR
**Timeline:** Year 2

---

## Tier 3: Data Provider Partnerships

These partnerships enhance Fetchium's data quality and unlock use cases that pure
web scraping can't address.

### Academic Data

**Semantic Scholar API (Allen AI)**
- Free tier: 100 req/min; higher tiers by arrangement
- Integration: `fetchium fetch --mode academic <doi>` routes through Semantic Scholar
- Value: Structured paper metadata, citations, full-text links for research agents
- Contact: partnerships@semanticscholar.org

**arXiv API**
- Free, open API — no partnership needed; just integrate
- Integration: Auto-detect arXiv URLs; return paper metadata + abstract as structured JSON
- Value: Research agents get consistent arXiv data without parsing LaTeX HTML

**CrossRef**
- Free metadata API for DOI resolution — no partnership needed
- Integration: Resolve DOIs to full citation metadata in fetch results

### News & Media

**NewsAPI.org**
- Free tier: 100 requests/day; paid: $449/month for 100K req
- Integration: `fetchium search --mode news "query"` routes through NewsAPI
- Value: Real-time news access; better for current events than SearXNG alone
- Consider: Fetching our own news sources is cheaper; NewsAPI as fallback

**The Guardian API**
- Free tier: 500 requests/day
- Integration: Guardian articles as a premium content source
- Value: High-quality, structured journalism content for news agents

**Common Crawl**
- Free, petabyte-scale web crawl index
- Integration: Historical content retrieval for URLs that no longer exist live
- Value: Unique capability no competitor offers — "fetch this page as it was in 2023"
- Note: Complex infrastructure; explore as a Phase 3+ feature

---

## Tier 4: AI Model Provider Relationships

### Anthropic (Claude)

**Opportunity:** Fetchium as the recommended web-fetch layer for Claude-powered agents.
Claude's tools use (`computer_use`, `web_search`) are limited; Fetchium extends them.

**Approach:**
- Build `fetchium-claude` Python adapter with `Tool` schemas matching Claude's tool use format
- Submit to Anthropic cookbook as a community recipe
- Long-term: Apply to Anthropic's startup program (API credits, potential co-promotion)

**Timeline:** Month 2 (high priority — Claude users are sophisticated and pay for quality)

### Google (Gemini)

**Opportunity:** Gemini function calling + Fetchium = powerful research agent.
Google has developer programs with API credits and promotion.

**Approach:**
- Build Gemini-compatible function definition for Fetchium tools
- Submit to Google's AI cookbook and demos repository
- Apply to Google for Startups Cloud Program (GCP credits)

**Timeline:** Month 2 (in parallel with Anthropic adapter)

### OpenAI

**Opportunity:** ChatGPT Plugin → Custom GPT Action → Assistant tool
**Approach:**
- Build OpenAI function calling schema for Fetchium
- Publish as a "Custom GPT Action" — accessible via GPT Store to ChatGPT users
- This is a significant distribution channel: millions of ChatGPT users

**Timeline:** Month 3

---

## Partnership Prioritization Matrix

| Partner | Dev Effort | Expected Signups | Strategic Value | Priority |
|---------|-----------|-----------------|----------------|----------|
| LangChain | 2 weeks | 2,000 | Very High | P0 |
| CrewAI | 1 week | 1,500 | Very High | P0 |
| Anthropic cookbook | 3 days | 800 | High | P1 |
| Google cookbook | 3 days | 600 | High | P1 |
| DSPy | 1 week | 400 | High (research) | P1 |
| AutoGen | 1 week | 500 | Medium | P2 |
| OpenAI GPT Actions | 1 week | 1,000 | High | P2 |
| Semantic Scholar | 3 days | 300 | Medium | P2 |
| AWS Marketplace | 3 weeks | Enterprise | Very High | P3 |
| Semantic Kernel | 2 weeks | 200 | Medium | P3 |
| GCP Marketplace | 3 weeks | Enterprise | High | P4 |

---

## Outreach Process

**For OSS framework integrations:**
1. Build the working adapter (never cold-pitch without code)
2. Open a PR to their repo or examples gallery
3. Post in their Discord/Slack: "Built a Fetchium integration — feedback welcome"
4. Follow up with maintainer on GitHub after PR review

**For cloud marketplaces:**
1. Reach $1K MRR first — proves product viability
2. Complete vendor registration (1-2 weeks of paperwork)
3. Submit listing for review (2-4 week review cycle)

**For data providers:**
1. Start with free tiers — no outreach needed
2. When free tier limits are hit, email the partnerships contact with usage stats
3. Volume discounts typically kick in at $500+/month spend

---

## Partner Success Metrics

| Metric | Target |
|--------|--------|
| LangChain integration shipped | Month 3 |
| CrewAI integration shipped | Month 3 |
| DSPy integration shipped | Month 4 |
| AI cookbook entries (Anthropic/Google/OpenAI) | Month 3 |
| AWS Marketplace listed | Month 9 |
| Signups attributed to partner integrations | 30% of total by Month 12 |
