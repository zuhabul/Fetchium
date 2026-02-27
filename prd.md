# Fetchium — Product Requirements Document (PRD)

> **Version:** 4.0.0 | **Last Updated:** 2026-02-23
> **Status:** Draft — Pre-Implementation
> **Tagline:** *The world's fastest, AI-native, free web search and deep research engine — built for agents, made for humans.*

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Vision & Positioning](#3-vision--positioning)
4. [Target Users](#4-target-users)
5. [Goals & Non-Goals](#5-goals--non-goals)
6. [Core Principles](#6-core-principles)
7. [Competitive Gap Analysis](#7-competitive-gap-analysis)
8. [Novel Fetchium Algorithms & Inventions](#8-novel-fetchium-algorithms--inventions)
9. [AI-Native Agent Architecture](#9-ai-native-agent-architecture)
10. [Modes of Operation](#10-modes-of-operation)
11. [CLI Interface Design](#11-cli-interface-design)
12. [System Architecture](#12-system-architecture)
13. [Machine Resource Awareness Engine](#13-machine-resource-awareness-engine)
14. [Parallel Execution Engine](#14-parallel-execution-engine)
15. [Search Backend Orchestrator](#15-search-backend-orchestrator)
16. [Content Extraction Pipeline](#16-content-extraction-pipeline)
17. [Query-Aware Token-Budgeted Extraction](#17-query-aware-token-budgeted-extraction)
18. [Semantic Content Segmentation](#18-semantic-content-segmentation)
19. [Validation & Reliability Layer](#19-validation--reliability-layer)
20. [Token Efficiency Architecture](#20-token-efficiency-architecture)
21. [Semantic Search & Hybrid Ranking](#21-semantic-search--hybrid-ranking)
22. [Cutting-Edge Research Integration](#22-cutting-edge-research-integration)
23. [AI Preview Engine](#23-ai-preview-engine)
24. [Citation & Evidence System](#24-citation--evidence-system)
25. [Agent Framework Integration](#25-agent-framework-integration)
26. [Output & Export System](#26-output--export-system)
27. [Progressive Detail Streaming](#27-progressive-detail-streaming)
28. [Caching & Local Index](#28-caching--local-index)
29. [Plugin & Extension System](#29-plugin--extension-system)
30. [MCP Server Mode](#30-mcp-server-mode)
31. [Cross-Session Learning & Persistent Intelligence](#31-cross-session-learning--persistent-intelligence)
32. [Tree-of-Thoughts & Advanced Reasoning](#32-tree-of-thoughts--advanced-reasoning)
33. [Proactive Intelligence & Anticipatory Search](#33-proactive-intelligence--anticipatory-search)
34. [Multimodal Content Understanding](#34-multimodal-content-understanding)
35. [Adversarial Robustness & Trust Verification](#35-adversarial-robustness--trust-verification)
36. [Privacy-First Architecture](#36-privacy-first-architecture)
37. [Collaborative Research Protocol](#37-collaborative-research-protocol)
38. [Domain-Specific Intelligence Modes](#38-domain-specific-intelligence-modes)
39. [Self-Evolving Architecture](#39-self-evolving-architecture)
40. [Performance Requirements](#40-performance-requirements)
41. [Security & Compliance](#41-security--compliance)
42. [300+ Advanced Features](#42-300-advanced-features)
43. [Data Model](#43-data-model)
44. [Error Handling & Fallback Chains](#44-error-handling--fallback-chains)
45. [Testing Strategy](#45-testing-strategy)
46. [Milestones & Roadmap](#46-milestones--roadmap)
47. [Success Metrics](#47-success-metrics)
48. [Technical Dependencies](#48-technical-dependencies)
49. [Appendix: Research Papers & References](#49-appendix-research-papers--references)

---

## 1. Executive Summary

**Fetchium** is a **free, open-source, AI-native web search and deep research engine** designed from the ground up for both **AI agents** and **human power users**. It is the first tool to unify search, extraction, ranking, validation, synthesis, and structured output into a single zero-cost CLI command — with novel algorithms that no existing tool provides.

### The 17 Inventions That Make Fetchium Unique

No existing tool — Tavily, Exa, Perplexity, Firecrawl, Crawl4AI, Jina, SearXNG, or any other — provides any of these:

| # | Invention | What It Does | Why No One Else Has It |
|---|-----------|-------------|----------------------|
| 1 | **HyperFusion Ranking** | Multi-signal rank fusion combining BM25 + semantic + temporal decay + authority chains + evidence density + intent weighting in a single differentiable function | All tools use basic RRF or single-signal ranking |
| 2 | **Query-Aware Token-Budgeted Extraction (QATBE)** | Fetch a URL, extract ONLY content relevant to your query, fit it within a token budget, ranked by relevance | Every tool returns the whole page — no query awareness |
| 3 | **Cascade Extraction Protocol (CEP)** | ML-predicted 5-layer extraction cascade that auto-selects the cheapest sufficient method per URL | Tools use fixed extraction — always headless or never |
| 4 | **Semantic Content Segmentation (SCS)** | Split content into typed blocks (facts, opinions, data, code, tables) each in the most token-efficient format | Everything else outputs flat markdown |
| 5 | **Speculative Research Pipelining (SRP)** | Stream answers from first results while fetching more; auto-correct if new data changes findings | All tools wait for all fetches before producing output |
| 6 | **Reflection-Augmented Research (RAR)** | Self-correcting research loop that evaluates its own findings and auto-retrieves when quality is low | No tool self-corrects — bad retrieval = bad output |
| 7 | **Evidence Graph Protocol (EGP)** | Graph-based evidence linking with claim provenance tracking and cryptographic content hashes | No tool provides verifiable evidence chains |
| 8 | **Adaptive Multi-Agent Research Swarm (AMRS)** | Dynamic sub-agent spawning: Search Agent + Extract Agent + Verify Agent + Synthesize Agent | No tool uses multi-agent collaboration for research |
| 9 | **Progressive Detail Streaming (PDS)** | Multi-tier output: 200-token summary → 1K compressed → 5K detailed → full — without re-fetching | You get everything or nothing everywhere else |
| 10 | **Query-Aware DOM Distillation (QADD)** | Reduce DOM to only query-relevant nodes BEFORE extraction, combining D2Snap + BM25 | All extractors process the entire DOM regardless of query |
| 11 | **Persistent Intelligence Engine (PIE)** | Cross-session knowledge graph that learns source trust, failure patterns, query predictions, and concept mappings over time | Every tool treats every query as isolated — zero learning |
| 12 | **Tree-of-Thoughts Research (ToTR)** | Parallel reasoning paths with branch pruning, cross-path synthesis, and self-debate for complex research questions | No tool explores multiple reasoning strategies simultaneously |
| 13 | **Contradiction Resolution Protocol (CRP)** | When sources disagree, automatically investigate: check dates, authority, context, spawn investigation agent, return weighted synthesis | Tools flag contradictions at best — none resolve them |
| 14 | **Evidence Decay Function (EDF)** | Claims have domain-calibrated half-lives — AI news decays in days, math decays in years — auto-flag stale evidence | No tool models temporal reliability of information |
| 15 | **Source Genealogy Tracker (SGT)** | Trace claim provenance: Article A cites B which cites Paper C which cites Dataset D — find the primary source automatically | No tool traces information cascades to origin |
| 16 | **Confidence Calibration Engine (CCE)** | Historically calibrated confidence: "Our 85% confidence has been accurate 87% of the time (n=1,247)" | No tool calibrates confidence against historical accuracy |
| 17 | **Adversarial Content Shield (ACS)** | Detect AI-generated content, bot farm signals, source manipulation, and coordinated inauthentic behavior in search results | No tool verifies source authenticity in 2026's AI-flooded web |

### Competitive Position

| Dimension | Industry Best | Fetchium |
|-----------|--------------|--------------|
| **Cost** | DDG (free but limited) | Free, unlimited, zero API keys |
| **Speed** | Exa Fast (350ms, $paid) | <1s cached, <3s uncached via SRP |
| **AI Agent Support** | Tavily (max_tokens only) | Full QATBE + SCS + PDS + MCP + framework adapters |
| **Content Extraction** | Firecrawl/Jina ($paid) | Free CEP with 5-layer cascade |
| **Deep Research** | Perplexity (20/month) | Unlimited RAR + AMRS + EGP |
| **Token Efficiency** | Crawl4AI (Python, no query awareness) | QADD + SCS + PDS = 97% reduction |
| **Reproducibility** | None exist | EGP with cryptographic hashes |
| **Validation** | None exist | 6-layer + RAR self-correction |
| **Ranking** | Basic RRF | HyperFusion 8-signal differentiable ranking |
| **Agent Frameworks** | Tavily (LangChain only) | LangChain + CrewAI + AutoGPT + Claude + MCP native |
| **Output Intelligence** | Flat markdown everywhere | SCS typed segments + PDS tiered detail |

### Key Commands

```bash
# For humans
hsx search "query"                    # Blazing fast search
hsx research "query"                  # Structured research report
hsx deep "query"                      # Agentic deep research
hsx ai "query"                        # AI synthesis with local model
hsx fetch <url>                       # Extract any webpage
hsx view <url>                        # Clean readable view

# For AI agents
hsx agent-search "query" --budget 2000 --schema schema.json
hsx agent-fetch <url> --query "what I need" --budget 1500 --tier compressed
hsx agent-research "query" --budget 4000 --framework langchain
hsx serve --mcp                       # Start as MCP server
hsx serve --api                       # Start as REST API

# Both
hsx compare "A vs B"                  # Comparison research
hsx monitor <url>                     # Change detection
hsx export <url> --format md,json     # Multi-format export
```

Works with: `npm` | `pnpm` | `bun` — Aliases: `hsx` | `hyper`

---

## 2. Problem Statement

### For AI Agents: The Web is Hostile

1. **Token Waste**: A typical webpage is 50K+ tokens of HTML. An agent needs ~500 tokens of relevant information. Current tools return the whole page — a 100x waste. Even "LLM-ready" tools like Firecrawl and Jina return 5-10K tokens of markdown when the agent needs 500.

2. **No Query Awareness**: No tool lets an agent say "fetch this URL but only extract content about X within Y tokens." Every tool returns everything, forcing the agent to waste its own context window filtering.

3. **No Structured Segments**: Agents need different content types in different formats — tables as JSON arrays (not markdown tables), code as fenced blocks, facts as bullet points. No tool provides semantic segmentation.

4. **No Progressive Detail**: Agents can't say "give me a 200-token summary first, then I'll decide if I need more." It's all-or-nothing everywhere.

5. **No Self-Correction**: When search returns bad results, no tool detects this. Bad retrieval = bad agent output. No existing pipeline evaluates its own retrieval quality.

6. **Framework Mismatch**: LangChain expects `Document` objects. CrewAI expects strings. Claude MCP expects specific schemas. No tool adapts its output format to the consuming framework.

### For Humans: Research is Still Broken

7. **Fragmented Pipeline**: Question → answer requires 3-5 paid tools chained together.
8. **API Paywalls**: Every serious tool costs money. Brave dropped free tier. Perplexity limits deep research to 20/month.
9. **No Reproducibility**: No evidence chains, no content hashes, no "what changed" diffs.
10. **Hallucinated Citations**: Perplexity produces citations that don't verify.
11. **No Resource Awareness**: Same fixed behavior on 4GB laptop and 128GB workstation.
12. **No Validation**: Results taken at face value.

### Fetchium Goal

**For agents**: The most token-efficient, query-aware, framework-adaptive web data layer — zero cost, zero API keys.
**For humans**: The fastest path from question to trusted, structured, reproducible knowledge artifact.

---

## 3. Vision & Positioning

### Vision

> Fetchium is the world's first AI-native web search engine — equally powerful as a human CLI tool and as a data layer for AI agents. It delivers the right information, in the right format, at the right level of detail, within the right token budget — free, fast, and validated.

### Positioning

- **AI-Native First**: Designed for agent consumption from day one — not a human tool with an API bolted on.
- **Not Offline-First**: Primary goal is **blazing-fast online search and research**. Local re-querying is a bonus.
- **Not a Browser**: A pipeline — query in, structured knowledge out.
- **Not a Chatbot**: AI is a synthesis layer, not the product. Results are evidence-backed.
- **Not Just a Scraper**: Scraping is a capability. Answers are the purpose.

---

## 4. Target Users

### Primary: AI Agent Systems

| Consumer | Needs | Pain Today |
|----------|-------|-----------|
| **LangChain/LlamaIndex Chains** | Token-efficient web context for RAG | Tavily returns too much, DDG returns too little |
| **CrewAI/AutoGPT Agents** | Structured research results for multi-step tasks | No tool provides query-aware extraction |
| **Claude Code / MCP Tools** | Web search as an MCP resource/tool | Existing MCP search tools are primitive |
| **Custom AI Agents** | REST API or library for web data | Must chain 3-5 tools, each costs money |
| **RAG Pipelines** | Clean, chunked, deduplicated web content | Raw HTML or bloated markdown from every tool |

### Primary: Human Power Users

| Persona | Needs | Pain Today |
|---------|-------|-----------|
| **Engineer / Developer** | Implementation answers, API docs, changelogs | Tab chaos, outdated StackOverflow |
| **Research Analyst** | Competitive intel, market data, reports | Expensive tools, no reproducibility |
| **AI/ML Engineer** | Token-efficient web context, structured data | Bloated APIs, expensive search |
| **Student / Academic** | Papers, citations, literature review | Paywalled everything |
| **Journalist** | Cross-source verification, evidence trails | Manual verification, scattered notes |

---

## 5. Goals & Non-Goals

### Goals (MUST)

| # | Goal | Metric |
|---|------|--------|
| G1 | **AI-native agent-first design** | Full QATBE + SCS + PDS + MCP + framework adapters |
| G2 | **Blazing fast** | <1s cached, <3s uncached search |
| G3 | **Zero API cost** | No paid APIs required |
| G4 | **Query-aware token-budgeted extraction** | Return only relevant content within budget |
| G5 | **Semantic content segmentation** | Typed blocks, not flat markdown |
| G6 | **Progressive detail streaming** | Multi-tier output without re-fetching |
| G7 | **Self-correcting research** | RAR evaluates and auto-corrects findings |
| G8 | **Machine resource awareness** | Adaptive parallelism based on CPU/RAM/network |
| G9 | **Multi-backend search fusion** | HyperFusion across DDG + SearXNG + Wikipedia + direct |
| G10 | **Reproducible research** | EGP with cryptographic hashes and audit trails |
| G11 | **Validation layer** | 6-layer verification for every result |
| G12 | **Any output format** | MD, JSON, CSV, HTML, PDF, DOCX, BibTeX, YAML, agent-native |
| G13 | **MCP server mode** | Native Model Context Protocol server |
| G14 | **Framework adapters** | LangChain, CrewAI, AutoGPT, Claude, custom |
| G15 | **npm/pnpm/bun compatible** | Standard Node.js packaging |
| G16 | **Expandable** | Plugin system for backends, extractors, rankers |

### Non-Goals

- Crawling the entire web at Google scale
- Bypassing paywalls, CAPTCHAs, or access controls
- Replacing a web browser
- Requiring cloud infrastructure or paid services
- Building a GUI (CLI + API + MCP only)

---

## 6. Core Principles

### P1: Agent-First, Human-Friendly

Every feature is designed for programmatic consumption first. Human-readable output is a formatting layer on top of structured data, not the other way around.

### P2: Token Efficiency is a First-Class Feature

Output is not "converted to markdown." It is query-filtered, relevance-ranked, semantically segmented, and budget-constrained by design. Every token earns its place.

### P3: Speed Without Compromise

Speculative pipelining, cascade extraction, predictive caching, and parallel execution deliver results before the full pipeline completes — without sacrificing quality.

### P4: Evidence Over Generation

Every claim maps to a source. The Evidence Graph Protocol ensures cryptographic verifiability. No hallucinated citations.

### P5: Self-Correcting by Default

The Reflection-Augmented Research loop evaluates retrieval quality, detects bad sources, and auto-corrects without user intervention.

### P6: Adaptive to Everything

Adapts to machine resources (CPU/RAM/GPU/network), query complexity, available backends, consuming framework, and user preferences — all automatically.

### P7: Zero Cost, Maximum Value

No API keys. No credits. No subscriptions. No rate-limited free tiers. Everything runs locally or uses free public endpoints.

---

## 7. Competitive Gap Analysis

### The 15 Gaps No Tool Fills (Fetchium Fills All)

#### Gap 1: No Query-Aware Token-Budgeted Extraction

**Problem**: No tool lets you say "fetch this URL, extract only content relevant to my query, fit within 2,000 tokens."
- Firecrawl, Crawl4AI, Jina: Return entire page as markdown
- Tavily: Has `max_tokens` for search results, not page extraction
- Exa: Has highlights but separate from extraction

**Fetchium Solution**: QATBE — single call: `agent-fetch <url> --query "what I need" --budget 2000`

#### Gap 2: No Unified Search+Extract+Structure Pipeline

**Problem**: Getting structured data from a search requires chaining 3-5 tools.
- Tavily: Search but no schema-based extraction
- Exa: Separate search and extract calls
- Firecrawl: Separate search and scrape

**Fetchium Solution**: Single call: `agent-search "query" --schema schema.json --budget 3000`

#### Gap 3: No Semantic Content Segmentation

**Problem**: Everything outputs flat markdown. Tables waste tokens as markdown. Structured data loses structure.

**Fetchium Solution**: SCS outputs `{facts: [...], tables: [{json}], code: [...], opinions: [...], metadata: {...}}`

#### Gap 4: No Progressive Detail Levels

**Problem**: All-or-nothing output. Can't request a summary first, then decide if you need more.

**Fetchium Solution**: PDS delivers 4 tiers: `key_facts` (200 tokens) → `summary` (1K) → `detailed` (5K) → `full` — without re-fetching.

#### Gap 5: No Self-Correcting Research

**Problem**: Bad retrieval = bad output. No tool evaluates its own results.

**Fetchium Solution**: RAR with Self-RAG reflection tokens + CRAG evaluator — auto-detects and corrects bad retrievals.

#### Gap 6: No Framework-Adaptive Output

**Problem**: LangChain needs `Document` objects. CrewAI needs strings. MCP needs specific schemas.

**Fetchium Solution**: Auto-detects consuming framework and returns data in the optimal format.

#### Gap 7: No Pre-Fetch Token Estimation

**Problem**: No tool says "this fetch will cost ~X tokens" before executing.

**Fetchium Solution**: `agent-fetch <url> --estimate` returns token estimate without fetching content.

#### Gap 8: No Multi-Source Synthesis with Deduplication

**Problem**: Multiple sources with overlapping content waste agent context.

**Fetchium Solution**: Cross-source deduplication and merging via SimHash — non-redundant output from N sources.

#### Gap 9: No Structured Error Taxonomy

**Problem**: Generic errors (timeout, 403) with no remediation. No fallback chains.

**Fetchium Solution**: Classified errors with automatic fallback: cache → alternative source → Wayback Machine → partial result.

#### Gap 10: No MCP-Native Composite Tools

**Problem**: MCP search servers expose individual tools. LLM must orchestrate multiple calls.

**Fetchium Solution**: Composite MCP tools: `research(query, depth, schema, budget)` handles the full pipeline internally.

#### Gap 11: No Streaming Incremental Extraction

**Problem**: No tool streams content section-by-section ranked by relevance.

**Fetchium Solution**: SRP streams chunks ranked by query relevance — agent can stop consuming when it has enough.

#### Gap 12: No Content Freshness and Temporal Awareness

**Problem**: No tool scores content by temporal relevance relative to query intent.

**Fetchium Solution**: HyperFusion includes temporal decay scoring calibrated by query intent category.

#### Gap 13: No Self-Hosted Full-Pipeline CLI

**Problem**: No single self-hostable tool does search + fetch + extract + structure + AI — without cloud APIs.

**Fetchium Solution**: Complete pipeline in a single binary. Zero external dependencies for core function.

#### Gap 14: No Verifiable Evidence Chains

**Problem**: No cryptographic verification of cited content. Sources can change after citation.

**Fetchium Solution**: EGP hashes content at fetch time. Citations are verifiable against stored hashes.

#### Gap 15: No Intelligent Extraction Method Selection

**Problem**: Tools use fixed extraction (always headless or never). No intelligence in method selection.

**Fetchium Solution**: CEP uses ML-based prediction to select the cheapest sufficient extraction method per URL.

---

## 8. Novel Fetchium Algorithms & Inventions

### 8.1 HyperFusion Ranking Algorithm

**Problem**: All existing multi-source ranking uses basic Reciprocal Rank Fusion (RRF) which treats all signals equally and ignores query context.

**Invention**: An 8-signal differentiable rank fusion function that adapts signal weights based on query intent:

```
HyperFusion(result, query) =
    w_intent[bm25]     * BM25(result, query)
  + w_intent[semantic]  * CosineSim(embed(result), embed(query))
  + w_intent[temporal]  * TemporalDecay(result.date, query.freshness_need)
  + w_intent[authority] * AuthorityChain(result.domain, result.citations)
  + w_intent[evidence]  * EvidenceDensity(result.content)
  + w_intent[diversity] * DiversityBonus(result.domain, seen_domains)
  + w_intent[depth]     * ContentDepth(result.word_count, result.structure)
  + w_intent[consensus] * ConsensusScore(result.claims, all_results.claims)
  - duplicate_penalty   * SimHash(result.content, seen_content)
```

**Key innovation**: `w_intent` is a learned weight vector per query intent category. A "current events" query upweights temporal; a "how-to" query upweights depth; a "fact-check" query upweights consensus.

**Intent categories** (auto-classified via lightweight local model):
- `factual` | `how_to` | `comparison` | `verification` | `current_events` | `deep_analysis` | `code` | `academic` | `opinion` | `data`

**Authority Chains**: Unlike simple domain scoring, HyperFusion traces citation chains — a source cited by other authoritative sources scores higher (PageRank-inspired but for individual articles).

---

### 8.2 Query-Aware Token-Budgeted Extraction (QATBE)

**Problem**: Every extraction tool returns the entire page. Agents need query-relevant content within a token budget.

**Invention**: A 4-stage extraction pipeline that accepts `(url, query, token_budget)` and returns only the most relevant content:

```
Stage 1: FETCH
  HTTP GET → raw HTML

Stage 2: SEGMENT (Semantic Content Segmentation)
  Parse HTML → typed segments: {paragraphs, tables, code, lists, headings, metadata}
  Each segment tagged with: type, position, estimated_tokens

Stage 3: RANK (Query-Aware Filtering)
  For each segment:
    relevance_score = BM25(segment.text, query) * 0.6
                    + CosineSim(embed(segment), embed(query)) * 0.4
  Sort segments by relevance_score descending

Stage 4: BUDGET (Token Packing)
  Pack highest-relevance segments into token_budget using greedy knapsack:
    while total_tokens < budget:
      add next highest-relevance segment
      if adding would exceed budget:
        truncate segment to fit remaining budget
  Return packed segments with metadata
```

**API**:
```bash
# CLI
hsx agent-fetch https://example.com --query "pricing plans" --budget 1500

# Programmatic
const result = await hsx.fetch({
  url: 'https://example.com',
  query: 'pricing plans',
  tokenBudget: 1500,
  format: 'segments'  // or 'markdown' or 'json'
});

# Output
{
  "tokens_used": 1487,
  "tokens_total_available": 12340,
  "relevance_coverage": 0.89,  // 89% of relevant content captured
  "segments": [
    { "type": "paragraph", "relevance": 0.95, "tokens": 340, "content": "..." },
    { "type": "table", "relevance": 0.91, "tokens": 210, "data": [...] },
    { "type": "list", "relevance": 0.87, "tokens": 180, "items": [...] }
  ],
  "metadata": { "title": "...", "url": "...", "fetched_at": "...", "content_hash": "..." }
}
```

**Why this is unique**: No existing tool — Firecrawl, Crawl4AI, Jina, Tavily, or any other — provides query-aware extraction with token budgeting. They all return the entire page.

---

### 8.3 Cascade Extraction Protocol (CEP)

**Problem**: Tools use fixed extraction: always headless (slow, expensive) or never (misses JS content). No intelligence in selection.

**Invention**: ML-predicted 5-layer extraction cascade with automatic escalation:

```
Layer 1: HTTP + Cheerio (cost: ~2ms, RAM: ~5MB)
  → Check: content_length > threshold AND text_ratio > 0.3
  → If pass: return. If fail: escalate.

Layer 2: HTTP + Readability (cost: ~8ms, RAM: ~10MB)
  → Check: article_detected AND main_content_length > threshold
  → If pass: return. If fail: escalate.

Layer 3: Playwright Headless Shell (cost: ~400ms, RAM: ~100MB)
  → Check: rendered_content_differs_from_static
  → If pass: return. If fail: escalate.

Layer 4: Playwright Full + Wait (cost: ~2s, RAM: ~250MB)
  → Check: dynamic_content_loaded
  → If pass: return. If fail: escalate.

Layer 5: Playwright + Scroll + Interact (cost: ~5s, RAM: ~300MB)
  → Last resort for infinite scroll, lazy load, etc.
```

**ML Method Predictor**: A lightweight classifier trained on URL patterns, domain, content-type, and HTML structural features that predicts the required extraction layer BEFORE attempting extraction — skipping unnecessary layers:

```typescript
// Trained features:
// - domain (known SPA domains list)
// - content-type header
// - HTML size vs text content ratio
// - presence of framework markers (React, Vue, Angular)
// - <noscript> content
// - script tag density
// - historical success rate per domain per layer

predictedLayer = CEPClassifier.predict(url, headers, htmlPreview);
// Jump directly to the predicted layer, saving time
```

---

### 8.4 Semantic Content Segmentation (SCS)

**Problem**: All tools output flat markdown. Tables waste tokens as markdown syntax. Structured data loses its structure. Agents must parse markdown to extract specific content types.

**Invention**: Instead of flat markdown, segment content into typed semantic blocks, each in its most token-efficient representation:

```typescript
interface SegmentedContent {
  // Each segment has a type and the most efficient encoding for that type
  segments: Segment[];
  metadata: PageMetadata;
  token_count: number;
}

type Segment =
  | { type: 'heading'; level: 1|2|3; text: string; }
  | { type: 'paragraph'; text: string; relevance: number; }
  | { type: 'fact'; claim: string; source_ref?: number; confidence: number; }
  | { type: 'opinion'; text: string; attribution?: string; }
  | { type: 'table'; headers: string[]; rows: any[][]; }  // JSON, not markdown
  | { type: 'code'; language: string; code: string; }
  | { type: 'list'; ordered: boolean; items: string[]; }  // Array, not markdown bullets
  | { type: 'quote'; text: string; attribution?: string; }
  | { type: 'data'; key: string; value: any; unit?: string; }  // Structured data points
  | { type: 'link'; text: string; url: string; context: string; }
  | { type: 'image'; alt: string; url: string; caption?: string; }
  | { type: 'definition'; term: string; definition: string; }
  | { type: 'date_event'; date: string; event: string; }
  | { type: 'entity'; name: string; type: string; description?: string; };
```

**Token efficiency comparison** (same content):

| Format | Tokens | Why |
|--------|--------|-----|
| Raw HTML | 50,000 | Tags, attributes, scripts, styles |
| Clean HTML | 12,000 | Boilerplate removed but still tags |
| Flat Markdown | 4,000 | Clean but tables/lists waste tokens |
| **SCS Segments** | **1,800** | Each type in most efficient encoding |

**Tables example**:
```
# Markdown table (wasteful):
| Name | Price | Rating |    ← 12 tokens of formatting per row
|------|-------|--------|
| A    | $10   | 4.5    |
| B    | $20   | 4.2    |

# SCS table (efficient):
{"type":"table","headers":["Name","Price","Rating"],"rows":[["A","$10","4.5"],["B","$20","4.2"]]}
                                                    ← 60% fewer tokens for same data
```

---

### 8.5 Speculative Research Pipelining (SRP)

**Problem**: All research tools wait for all fetches and processing to complete before returning any output. Users stare at a loading indicator for 30-60 seconds.

**Invention**: Start streaming answers from the first available results while asynchronously fetching more. Auto-correct if new data changes findings.

```
Timeline:
  t=0s    Query sent to all backends in parallel
  t=0.5s  DDG results arrive → start extracting top 3
  t=1.0s  Wikipedia results arrive → extract
  t=1.5s  First 3 pages extracted → STREAM initial findings
  t=2.0s  SearXNG results arrive → extract in background
  t=3.0s  More pages extracted → check for contradictions
  t=4.0s  All sources processed → STREAM corrections/additions
  t=5.0s  Validation complete → STREAM final confidence scores

Output (streaming):
  [INITIAL] Based on 3 sources: "TypeScript 6.0 was released on..."  [1][2][3]
  [UPDATE]  Additional source confirms. Confidence: HIGH               [4]
  [CORRECTION] Earlier claim about date corrected: Jan 2026 not Dec... [5]
  [FINAL]   6 sources analyzed, 5 validated. Full report below.
```

**For agents**: SRP provides a streaming API where the agent receives chunks in order of confidence, allowing it to stop consuming when it has sufficient context:

```typescript
const stream = hsx.researchStream({
  query: "...",
  onChunk: (chunk) => {
    // chunk.confidence > threshold? stop consuming
    if (chunk.confidence > 0.9) stream.stop();
  }
});
```

---

### 8.6 Reflection-Augmented Research (RAR)

**Problem**: When search retrieves irrelevant or low-quality content, no tool detects this. The garbage propagates through the entire pipeline.

**Invention**: Inspired by Self-RAG and CRAG papers, RAR adds reflection checkpoints throughout the research pipeline:

```
┌──────────────────────────────────────────────────────────────┐
│                   RAR Research Loop                           │
│                                                               │
│  Query → RETRIEVE → [R1: Need more?] ──yes──→ Retrieve more  │
│                         │ no                                  │
│                         ▼                                     │
│          EVALUATE → [R2: Relevant?] ──no──→ Reformulate query │
│                         │ yes                                 │
│                         ▼                                     │
│          EXTRACT → [R3: Sufficient?] ──no──→ Fetch more pages │
│                         │ yes                                 │
│                         ▼                                     │
│          SYNTHESIZE → [R4: Supported?] ──no──→ Find evidence  │
│                         │ yes                                 │
│                         ▼                                     │
│          VALIDATE → [R5: Consistent?] ──no──→ Flag conflicts  │
│                         │ yes                                 │
│                         ▼                                     │
│                    RETURN (confidence-scored)                  │
└──────────────────────────────────────────────────────────────┘
```

**Reflection tokens** (lightweight, no LLM needed for basic checks):
- **R1 [NEED_MORE]**: Are there fewer than N relevant results? → Expand query
- **R2 [RELEVANT]**: BM25 + semantic score between query and each result > threshold? → Discard irrelevant
- **R3 [SUFFICIENT]**: Does extracted content actually answer the query? → Fetch more if not
- **R4 [SUPPORTED]**: Does the synthesis contain claims not found in sources? → Remove unsupported claims
- **R5 [CONSISTENT]**: Do sources agree? → Flag contradictions, add consensus scores

**For complex queries (with local LLM)**: Use the LLM to evaluate retrieval quality:
```
Given query: "X"
Given retrieved content: "Y"
Is this content relevant to answering the query? (yes/no/partial)
Does it provide sufficient evidence? (yes/no/needs_more)
```

---

### 8.7 Evidence Graph Protocol (EGP)

**Problem**: No tool provides verifiable evidence chains. Citations can be hallucinated. Source content can change after citation.

**Invention**: A graph-based evidence tracking system with cryptographic verification:

```typescript
interface EvidenceGraph {
  nodes: EvidenceNode[];
  edges: EvidenceEdge[];
  rootClaim: string;
  overallConfidence: number;
  contentHashes: Map<string, string>;  // url → SHA-256 of content at fetch time
}

interface EvidenceNode {
  id: string;
  type: 'claim' | 'source' | 'fact' | 'inference';
  content: string;
  confidence: number;
  timestamp: string;
}

interface EvidenceEdge {
  from: string;  // source node
  to: string;    // claim node
  type: 'supports' | 'contradicts' | 'partially_supports' | 'inferred_from';
  quote: string; // exact supporting quote
  quoteHash: string; // SHA-256 of quote for verification
}
```

**Verification**: Any claim can be traced back to its source, with the exact supporting quote, and the content hash proves the source contained that quote at fetch time.

```bash
hsx deep "query" --evidence-graph
# Outputs: research report + evidence_graph.json with full provenance
```

---

### 8.8 Adaptive Multi-Agent Research Swarm (AMRS)

**Problem**: Deep research requires different capabilities at different stages. No tool uses specialized agents working in parallel.

**Invention**: Dynamically spawn specialized sub-agents based on query complexity:

```
Complex Query → AMRS Coordinator
                    │
        ┌───────────┼───────────────┐
        │           │               │
  Search Agent  Extract Agent  Verify Agent
  (finds URLs)  (gets content) (checks facts)
        │           │               │
        └───────────┼───────────────┘
                    │
              Synthesize Agent
              (combines findings)
                    │
               Final Output
```

**Agent types**:
- **Search Agent**: Query decomposition, backend orchestration, result fusion
- **Extract Agent**: QATBE + CEP extraction, content segmentation
- **Verify Agent**: Cross-source validation, contradiction detection, fact-checking
- **Synthesize Agent**: Evidence graph construction, report generation, citation mapping
- **Deep Agent**: Multi-hop follow-up queries, iterative refinement (deep mode only)

**Resource-aware**: AMRS only spawns agents that the machine can support. On a 4GB machine, agents run sequentially. On a 32GB machine, all run in parallel.

---

### 8.9 Progressive Detail Streaming (PDS)

**Problem**: Tools return all content at one detail level. Agents waste tokens reading full content when a summary would suffice.

**Invention**: A 4-tier content system that pre-computes all tiers at extraction time:

```
Tier 0: KEY_FACTS     (~200 tokens)   - Top 5 factual answers, one-liner each
Tier 1: SUMMARY       (~1,000 tokens) - Executive summary with key findings
Tier 2: DETAILED      (~5,000 tokens) - Full analysis with evidence
Tier 3: COMPLETE      (all tokens)    - Everything extracted, nothing omitted
```

**How it works**: At extraction time, content passes through a compression cascade:
1. Extract COMPLETE content
2. Apply relevance filtering → DETAILED
3. Apply abstractive compression → SUMMARY
4. Extract key claims only → KEY_FACTS

All 4 tiers are cached. Agent requests any tier instantly:

```bash
# CLI
hsx agent-search "query" --tier key_facts     # 200 tokens
hsx agent-search "query" --tier summary       # 1,000 tokens
hsx agent-search "query" --tier detailed      # 5,000 tokens
hsx agent-search "query" --tier complete      # everything

# API
const facts = await hsx.search({ query: "...", tier: "key_facts" });
// Not enough? Get more detail without re-searching:
const detailed = await hsx.expandTier(facts.resultId, "detailed");
```

---

### 8.10 Query-Aware DOM Distillation (QADD)

**Problem**: Traditional extraction processes the entire DOM. For a 50K-token page where the agent needs 500 tokens about a specific topic, 98% of DOM processing is wasted.

**Invention**: Reduce the DOM to only query-relevant nodes BEFORE extraction:

```
Full DOM (50K tokens)
    │
    ▼
[QADD Pipeline]
    │
    ├─ Step 1: Structural pruning
    │  Remove: <nav>, <footer>, <aside>, <script>, <style>, ads, social widgets
    │  Result: ~20K tokens
    │
    ├─ Step 2: Text node BM25 scoring
    │  Score each text node against query
    │  Prune nodes with relevance < threshold
    │  Result: ~5K tokens
    │
    ├─ Step 3: Semantic embedding check
    │  Embed remaining nodes + query
    │  Remove nodes with cosine similarity < threshold
    │  Result: ~2K tokens
    │
    ├─ Step 4: Context preservation
    │  Restore parent/sibling nodes needed for structural context
    │  Preserve table headers, list context, heading hierarchy
    │  Result: ~2.5K tokens
    │
    └─ Step 5: Token budget packing
       If still exceeds budget: greedy knapsack by relevance
       Result: fits within token_budget
```

**Combined with D2Snap**: QADD incorporates D2Snap's DOM downsampling techniques (merging container elements, dropping low-ranking sentences) for additional 5-10x reduction.

---

### 8.11 Persistent Intelligence Engine (PIE)

**Problem**: Every tool treats every query as isolated — zero memory across sessions. Source A failed 47 times, but the tool tries it again every time. The user researches "CRISPR" weekly, yet every session starts from zero.

**Invention**: A cross-session knowledge graph that learns and remembers:

```
┌─────────────────────────────────────────────────────────────┐
│                 Persistent Intelligence Engine                │
│                                                               │
│  ┌─────────────────────┐    ┌──────────────────────┐         │
│  │   Source Trust DB    │    │  Failure Pattern DB  │         │
│  │                      │    │                      │         │
│  │ reuters.com: 0.94    │    │ site-x: 403 after    │         │
│  │ blog-x.com: 0.31    │    │   5 reqs → backoff   │         │
│  │ arxiv.org: 0.97     │    │ site-y: JS-wall →    │         │
│  │ medium.com: 0.52    │    │   need headless       │         │
│  └─────────────────────┘    └──────────────────────┘         │
│                                                               │
│  ┌─────────────────────┐    ┌──────────────────────┐         │
│  │ Query Prediction DB  │    │  Concept Map DB      │         │
│  │                      │    │                      │         │
│  │ user searches CRISPR │    │ CRISPR → gene editing│         │
│  │  → predict: Cas9,    │    │   → Cas9, Cas12      │         │
│  │  off-target, ethics  │    │   → off-target       │         │
│  └─────────────────────┘    └──────────────────────┘         │
│                                                               │
│  Storage: SQLite (rusqlite) + LRU eviction                   │
│  Update: After every query — bayesian trust update           │
│  Privacy: All local, exportable, deletable                   │
└─────────────────────────────────────────────────────────────┘
```

**Key Capabilities**:
- **Source Trust Scoring**: Bayesian update of trust after each fetch (accuracy, uptime, freshness)
- **Failure Pattern Learning**: Remember what extraction method works per domain
- **Query Prediction**: Predict follow-up queries based on user history + concept graph
- **Concept Mapping**: Build entity-relationship graph across sessions for richer context
- **Cache Intelligence**: Priority-cache high-trust, frequently-accessed sources

---

### 8.12 Tree-of-Thoughts Research (ToTR)

**Problem**: All research tools follow a single linear reasoning path. If the initial approach is wrong, the entire result is wrong. Complex questions (e.g., "Is nuclear fusion economically viable by 2035?") require exploring multiple reasoning strategies simultaneously.

**Invention**: Parallel reasoning paths with branch pruning and cross-path synthesis:

```
Query: "Is nuclear fusion economically viable by 2035?"
                        │
           ┌────────────┼────────────┐
           ▼            ▼            ▼
    [Path A: Tech]  [Path B: Econ] [Path C: Policy]
    │               │               │
    ├─ ITER status  ├─ Cost/kWh    ├─ Gov funding
    ├─ Private co   ├─ vs solar    ├─ Regulation
    ├─ Breakthroughs├─ Investment  ├─ Public opinion
    │               │               │
    ▼               ▼               ▼
  Score: 0.7     Score: 0.4     Score: 0.6
    │               │               │
    │      ┌────────┘               │
    │      ▼ (pruned — low score)   │
    │                               │
    └──────────┬────────────────────┘
               ▼
    [Cross-Path Synthesis]
    "Technically approaching feasibility, but economics
     remain challenging. Policy support is key variable."
```

**Mechanism**:
1. **Branch Generation**: Decompose query into 2-5 reasoning paths (perspectives, methodologies)
2. **Parallel Exploration**: Each path runs its own search-extract-rank pipeline concurrently
3. **Branch Scoring**: Score each path on evidence quality, source diversity, coherence
4. **Pruning**: Kill low-scoring branches early to save resources
5. **Cross-Path Synthesis**: Merge surviving branches, resolve conflicts via CRP
6. **Self-Debate**: Generate counter-arguments per path, integrate strongest objections

---

### 8.13 Contradiction Resolution Protocol (CRP)

**Problem**: Sources disagree constantly. Source A says "Drug X is effective" (2023 trial), Source B says "Drug X shows no benefit" (2025 meta-analysis). Every existing tool either ignores contradictions or simply lists both. None investigate.

**Invention**: When sources contradict, automatically investigate and resolve:

```
Contradiction Detected: "Drug X effective" vs "Drug X no benefit"
    │
    ├─ Step 1: Date Check
    │  Source A: 2023 (single trial, n=200)
    │  Source B: 2025 (meta-analysis, n=12,000)
    │  → Temporal signal: newer + larger
    │
    ├─ Step 2: Authority Check
    │  Source A: pharma-sponsored trial
    │  Source B: Cochrane review
    │  → Authority signal: independent > sponsored
    │
    ├─ Step 3: Context Check
    │  Source A: specific population (adults 18-45)
    │  Source B: general population
    │  → Context: may not contradict — different populations
    │
    ├─ Step 4: Investigation Agent (spawned)
    │  Search for: "Drug X meta-analysis 2025", "Drug X population differences"
    │  → Found: 3 additional sources confirming population-specific efficacy
    │
    └─ Step 5: Weighted Synthesis
       "Drug X shows efficacy in adults 18-45 (moderate confidence)
        but no general population benefit (high confidence, Cochrane 2025).
        Population-specific effect likely."
       confidence: 0.78, resolution: "population_dependent"
```

**Resolution Strategies**: Temporal precedence, authority weighting, scope analysis, investigation spawning, Delphi-method consensus.

---

### 8.14 Evidence Decay Function (EDF)

**Problem**: A 2019 AI benchmark result is nearly meaningless in 2026. A 2019 math proof is as valid as ever. No tool models the temporal reliability of information — they treat a 7-year-old blog post and a yesterday's paper with equal weight.

**Invention**: Domain-calibrated half-lives for evidence:

```
EDF(claim, domain, age) = base_confidence × e^(-λ_domain × age)

Domain Half-Lives (λ calibration):
┌──────────────────┬──────────────┬─────────────────────────┐
│ Domain           │ Half-Life    │ Rationale               │
├──────────────────┼──────────────┼─────────────────────────┤
│ AI/ML benchmarks │ 3 months     │ SOTA changes weekly      │
│ Tech news        │ 2 weeks      │ Corrections come fast    │
│ Medical trials   │ 2 years      │ Replication takes time   │
│ Legal precedent  │ 10 years     │ Rarely overturned        │
│ Mathematics      │ 100+ years   │ Proofs don't expire      │
│ Stock prices     │ 1 day        │ Markets move constantly  │
│ Software docs    │ 6 months     │ APIs change frequently   │
│ Historical facts │ 50+ years    │ Rarely revised           │
└──────────────────┴──────────────┴─────────────────────────┘

Auto-flagging: When evidence confidence drops below threshold,
flag as "potentially stale" and optionally trigger re-search.
```

**Self-Calibrating**: EDF tracks its own accuracy — if flagged-stale evidence turns out to still be valid, it adjusts the domain half-life upward.

---

### 8.15 Source Genealogy Tracker (SGT)

**Problem**: Information cascades are everywhere — Article A cites Blog B, which cites Tweet C, which cites Paper D. The only reliable source is Paper D, but every tool treats all 4 equally. "Viral misinformation" often traces back to a single misinterpreted source.

**Invention**: Trace claim provenance to the primary source:

```
Claim: "GPT-5 achieves 98% on MMLU"
    │
    ├─ TechBlog.com (2026-02-20) ← cites →
    │   ├─ TheVerge.com (2026-02-19) ← cites →
    │   │   ├─ @researcher tweet (2026-02-18) ← cites →
    │   │   │   └─ ArXiv paper (2026-02-17) ← PRIMARY SOURCE
    │   │   │       Result: 97.3% (not 98%) on MMLU-Pro (not MMLU)
    │   │   │
    │   │   └─ MUTATION DETECTED: "97.3% on MMLU-Pro" → "98% on MMLU"
    │   │       Severity: HIGH (metric inflated + benchmark changed)
    │   │
    │   └─ Additional citation from Reddit thread (unverified)
    │
    └─ GENEALOGY REPORT:
       Primary source: ArXiv:2602.xxxxx
       Claim accuracy: DEGRADED (97.3→98, MMLU-Pro→MMLU)
       Recommendation: Cite primary source directly
       Trust cascade: 0.97 → 0.85 → 0.62 → 0.41
```

**Detection Methods**: Reference extraction, URL tracing, quote matching (fuzzy), date ordering, content similarity hashing.

---

### 8.16 Confidence Calibration Engine (CCE)

**Problem**: Every tool that provides confidence scores pulls numbers from thin air. "85% confidence" is meaningless if the tool's 85% predictions are only correct 60% of the time. No tool calibrates its confidence against historical accuracy.

**Invention**: Track historical accuracy and calibrate confidence:

```
┌─────────────────────────────────────────────────────────┐
│              Confidence Calibration Engine                │
│                                                           │
│  Historical Calibration Table (per domain):              │
│                                                           │
│  Stated   │ Actual  │ Count │ Calibrated │ Adjustment   │
│  ─────────┼─────────┼───────┼────────────┼──────────    │
│  0.90     │ 0.87    │ 1,247 │ 0.87       │ -0.03        │
│  0.80     │ 0.76    │ 2,103 │ 0.76       │ -0.04        │
│  0.70     │ 0.71    │ 1,891 │ 0.71       │ +0.01        │
│  0.60     │ 0.53    │ 1,456 │ 0.53       │ -0.07        │
│                                                           │
│  Output to user:                                         │
│  "Confidence: 85% (calibrated: 82%, n=1,247)"           │
│  "Our 85% confidence has historically been accurate      │
│   82% of the time based on 1,247 verifiable claims."    │
│                                                           │
│  Feedback loop: user corrections + automated fact-check  │
│  Isotonic regression for calibration curve fitting        │
│  Per-domain calibration (medical ≠ tech ≠ legal)        │
└─────────────────────────────────────────────────────────┘
```

**Calibration Method**: Isotonic regression on (stated_confidence, actual_accuracy) pairs. Updated incrementally with each verifiable claim.

---

### 8.17 Adversarial Content Shield (ACS)

**Problem**: The 2026 web is flooded with AI-generated content, SEO spam farms, coordinated bot campaigns, and deliberately manipulated search results. No search tool verifies source authenticity or detects manipulation.

**Invention**: Multi-layer adversarial content detection:

```
┌──────────────────────────────────────────────────────────────┐
│                  Adversarial Content Shield                    │
│                                                                │
│  Layer 1: AI Content Detection                                │
│  ├─ Perplexity scoring (burstiness, entropy patterns)         │
│  ├─ Stylometric analysis (vocabulary diversity, sentence var) │
│  ├─ Known AI watermark detection (C2PA, Content Credentials)  │
│  └─ Output: ai_probability: 0.0-1.0                          │
│                                                                │
│  Layer 2: Bot Farm Signals                                    │
│  ├─ Domain age + registration pattern                         │
│  ├─ Content publishing velocity (>50 articles/day = flag)     │
│  ├─ Cross-site content duplication (SimHash)                  │
│  ├─ Backlink network analysis (circular citation detection)   │
│  └─ Output: bot_farm_probability: 0.0-1.0                    │
│                                                                │
│  Layer 3: Source Manipulation Detection                       │
│  ├─ Sudden authority score changes                            │
│  ├─ Content modification frequency (edit wars)                │
│  ├─ Coordinated publication timing                            │
│  ├─ Astroturfing pattern detection (identical phrasing)       │
│  └─ Output: manipulation_probability: 0.0-1.0                │
│                                                                │
│  Layer 4: Trust Aggregation                                   │
│  ├─ Combine layers 1-3 with PIE historical trust              │
│  ├─ Cross-reference with known reliable source list           │
│  ├─ Apply domain-specific thresholds                          │
│  └─ Output: trust_score: 0.0-1.0, flags: [...]               │
│                                                                │
│  Actions:                                                     │
│  ├─ trust > 0.8: include normally                             │
│  ├─ 0.5 < trust < 0.8: include with warning label            │
│  ├─ trust < 0.5: exclude from results (log in audit trail)   │
│  └─ trust < 0.2: flag as adversarial, never cache            │
└──────────────────────────────────────────────────────────────┘
```

**Zero false positive design**: Conservative thresholds, human-reviewable audit trail, "shadow mode" for first 30 days (flag but don't filter, collect accuracy data before enforcing).

---

## 9. AI-Native Agent Architecture

### Design Philosophy

Fetchium is **agent-first**: every API, output format, and pipeline stage is designed for programmatic consumption by AI systems. Human CLI output is a formatting layer on top of the structured agent API.

### Agent Interface Modes

```
┌────────────────────────────────────────────────────────┐
│                  Fetchium Core                      │
│                                                         │
│  Search │ Extract │ Rank │ Validate │ Synthesize       │
├────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │   CLI    │  │  REST    │  │   MCP    │             │
│  │ (human)  │  │   API    │  │  Server  │             │
│  └──────────┘  └──────────┘  └──────────┘             │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │LangChain │  │ CrewAI   │  │ Library  │             │
│  │ Adapter  │  │ Adapter  │  │  (npm)   │             │
│  └──────────┘  └──────────┘  └──────────┘             │
└────────────────────────────────────────────────────────┘
```

### 1. CLI Mode (for humans)

```bash
hsx search "query"          # Human-readable output
hsx research "query"        # Markdown report
```

### 2. Agent CLI Mode (for scripts/agents)

```bash
hsx agent-search "query" --budget 2000 --format json --tier summary
hsx agent-fetch <url> --query "context" --budget 1500 --format segments
hsx agent-research "query" --budget 4000 --schema output.json --strict-evidence
```

### 3. REST API Mode (for any language)

```bash
hsx serve --api --port 3000
```

```http
POST /api/search
{
  "query": "best typescript ORM 2026",
  "token_budget": 2000,
  "tier": "summary",
  "format": "segments",
  "schema": { "type": "array", "items": { "name": "string", "pros": "string[]" } }
}
```

### 4. MCP Server Mode (for Claude, Claude Code, any MCP client)

```bash
hsx serve --mcp
```

Exposes composite MCP tools (see §30).

### 5. Library Mode (for Node.js/TypeScript/Bun)

```typescript
import { search, fetch, research, deep } from 'fetchium';

const results = await search({
  query: "query",
  tokenBudget: 2000,
  tier: 'summary',
  format: 'segments'
});
```

### 6. Framework Adapters

```typescript
// LangChain
import { FetchiumRetriever } from 'fetchium/langchain';
const retriever = new FetchiumRetriever({ tokenBudget: 3000 });

// CrewAI
import { FetchiumTool } from 'fetchium/crewai';
const searchTool = new FetchiumTool({ tier: 'detailed' });
```

### Pre-Fetch Token Estimation

Agents can estimate token cost before committing to a fetch:

```bash
hsx agent-fetch <url> --estimate
# Output: { "estimated_tokens": 12340, "estimated_relevant_tokens": 1850, "extraction_layer": 1 }
```

---

## 10. Modes of Operation

### Mode A: Search Mode (`hsx search`) — Blazing Fast

**Purpose**: Instant, high-precision results. <1s cached, <3s uncached.

```bash
hsx search "best rust web framework 2026"
hsx search "playwright vs puppeteer memory" --max-sources 20 --ai
hsx search "kubernetes pod eviction" --fast
```

**Agent variant**:
```bash
hsx agent-search "query" --budget 2000 --tier key_facts --format json
```

**Behavior**:
1. Query → intent classification (lightweight, <10ms)
2. Parallel multi-backend search (DDG + SearXNG + Wikipedia)
3. HyperFusion ranking with intent-adapted weights
4. Validation pass (reachability, freshness, dedup)
5. SRP: Stream first results while remaining sources load
6. Apply token budget if specified

**Output** (human):
```
[1] Actix Web vs Axum: 2026 Benchmarks — rust-lang.org
    "Axum leads at 142K req/s, Actix close at 138K..."
    Score: 0.94 | Fresh: 12d | Authority: High

[2] Choosing Rust Web Frameworks — logrocket.com
    "For most projects, Axum's ergonomics win..."
    Score: 0.91 | Fresh: 3d | Authority: Medium

Sources: [1] https://... [2] https://...
```

**Output** (agent, `--format segments --tier key_facts`):
```json
{
  "tokens": 187,
  "tier": "key_facts",
  "facts": [
    { "claim": "Axum leads Rust web benchmarks at 142K req/s", "source": 1, "confidence": 0.94 },
    { "claim": "Actix Web close at 138K req/s", "source": 1, "confidence": 0.94 },
    { "claim": "Axum recommended for most projects due to ergonomics", "source": 2, "confidence": 0.88 }
  ],
  "sources": [...]
}
```

**Latency**: <1s cached | <3s uncached

---

### Mode B: Research Mode (`hsx research`) — Structured Analysis

**Purpose**: Multi-source analysis with evidence mapping, citations, and RAR self-correction.

```bash
hsx research "GDPR implications for AI training data" --citations apa
hsx research "compare bun vs deno vs node 2026" --output report.md
```

**Agent variant**:
```bash
hsx agent-research "query" --budget 4000 --tier detailed --schema schema.json
```

**Behavior**:
1. Query decomposition (if complex)
2. Parallel multi-backend search
3. Top sources fetched via CEP
4. Content extracted via QATBE (if budget specified) or full extraction
5. RAR reflection loop validates retrieval quality
6. HyperFusion ranking
7. Evidence mapping via EGP
8. Synthesis (optional AI) with strict citation
9. Validation layer (6-layer)

**Latency**: 10-45s depending on sources

---

### Mode C: Deep Research Mode (`hsx deep`) — Agentic Investigation

**Purpose**: Multi-hop research with AMRS, RAR self-correction, and EGP evidence graphs.

```bash
hsx deep "Compare Puppeteer vs Playwright vs Crawlee at scale"
hsx deep "AI regulation: US vs EU vs China 2026" --max-depth 3
```

**Deep mode uses all novel algorithms**:
- **AMRS**: Spawns Search, Extract, Verify, Synthesize agents
- **RAR**: Self-corrects at every reflection checkpoint
- **EGP**: Builds full evidence graph with cryptographic hashes
- **SRP**: Streams findings as they're discovered
- **HyperFusion**: Intent-adaptive ranking across all sub-queries

**Deep mode features**:
- Query decomposition tree (visible in output)
- Multi-hop follow-up queries (up to `--max-depth`)
- Cross-source contradiction detection with severity scoring
- Consensus scoring for disputed claims
- "What changed since last run" diffs
- Full audit trail
- Evidence graph export

**Latency**: 1-10 minutes depending on depth

---

### Mode D: AI Preview (`hsx ai`) — Local LLM Synthesis

```bash
hsx ai "explain WebDriver BiDi protocol"
hsx ai "what's new in TypeScript 6.0" --model ollama:llama3.2
```

**Uses**: Search pipeline → QATBE extraction → context assembly with sandwich layout (best results at start/end to avoid lost-in-the-middle) → local LLM synthesis → citation injection.

**Model routing**: Auto-selects model size based on query complexity + available VRAM.

---

### Mode E: Fetch / View / Scrape — Web as Files

```bash
hsx fetch https://example.com                              # Clean extraction
hsx fetch https://example.com --query "pricing" --budget 1000  # QATBE
hsx view https://example.com                               # Terminal readable
hsx scrape https://example.com/spa --scroll                # JS + infinite scroll
hsx export https://example.com --format md,json            # Multi-format
```

**Agent variant**:
```bash
hsx agent-fetch <url> --query "what I need" --budget 1500 --format segments
hsx agent-fetch <url> --estimate     # Token estimation without fetching
```

---

### Mode F: Compare Mode — Side-by-Side

```bash
hsx compare "React vs Vue vs Svelte 2026"
```

Researches each item in parallel → merges into comparison table with per-dimension citations.

---

### Mode G: Monitor Mode — Change Detection

```bash
hsx monitor https://github.com/user/repo/releases --interval 1h --diff
hsx monitor "kubernetes CVE" --interval 6h --notify
```

Content-hash-based change detection with diff output and optional notifications.

---

### Mode H: Index Mode — Local Knowledge Base

```bash
hsx index add https://docs.example.com/sitemap.xml
hsx index build
hsx index search "authentication patterns"
```

Local hybrid index (BM25 + vector) with late chunking for superior embeddings.

---

## 11. CLI Interface Design

### Binary Names

- **Primary**: `hsx`
- **Alias**: `hyper`

### Package Manager Support

```bash
npm install -g fetchium
pnpm add -g fetchium
bun add -g fetchium
```

### Complete Command Reference

```
CORE COMMANDS (Human)
  search <query>              Fast web search
  research <query>            Structured research report
  deep <query>                Agentic deep research
  ai <query>                  AI-synthesized answer
  fetch <url>                 Fetch and extract content
  view <url>                  Clean readable view
  scrape <url>                Deep scrape with JS rendering
  compare <query>             Comparison research
  export <url|query>          Export to any format

AGENT COMMANDS (AI-Native)
  agent-search <query>        Token-budgeted search for agents
  agent-fetch <url>           Query-aware extraction for agents
  agent-research <query>      Structured research for agents
  serve --mcp                 Start MCP server
  serve --api                 Start REST API server

UTILITY COMMANDS
  monitor <url|query>         Watch for changes
  index <subcommand>          Manage local index
  cache <subcommand>          Manage cache
  config <subcommand>         Configuration
  doctor                      System health check
  version                     Version info

GLOBAL FLAGS
  --fast / --thorough         Speed vs depth tradeoff
  --max-sources <n>           Source cap (default: 10)
  --parallel <n>              Override parallelism (default: auto)
  --headless <auto|on|off>    Browser strategy (default: auto)
  --ai <on|off>               AI synthesis toggle
  --model <name>              AI model (ollama:llama3.2, etc.)
  --output <file>             Write to file
  --format <fmt>              md|json|csv|html|yaml|bibtex|segments|pdf|docx
  --citations <style>         inline|footnote|apa|ieee|chicago|bibtex
  --validate <mode>           strict|standard|fast|off
  --no-cache                  Bypass cache
  --verbose / --quiet         Output verbosity
  --profile                   Performance breakdown
  --evidence-graph            Output evidence graph

AGENT FLAGS
  --budget <n>                Token budget for output
  --tier <level>              key_facts|summary|detailed|complete
  --schema <file>             JSON schema for structured output
  --query <text>              Query context for QATBE (with fetch)
  --estimate                  Pre-fetch token estimation
  --framework <name>          Output format adapter (langchain|crewai|mcp)
  --strict-evidence           Every claim must cite a source
  --stream                    Stream results progressively
  --deduplicate               Cross-source deduplication
```

### Configuration

Location: `~/.fetchium/config.yaml`

```yaml
defaults:
  max_sources: 10
  parallel: auto
  headless: auto
  citations: inline
  format: md
  validate: true

agent:
  default_budget: 4000
  default_tier: detailed
  default_format: segments
  deduplicate: true
  estimate_before_fetch: false

ai:
  enabled: false
  provider: ollama
  model: llama3.2:8b
  endpoint: http://localhost:11434
  max_tokens: 4096
  temperature: 0.1
  sandwich_layout: true        # place best results at start/end of context

ranking:
  algorithm: hyperfusion       # hyperfusion | rrf | bm25_only
  bm25_weight: 0.25
  semantic_weight: 0.20
  temporal_weight: 0.15
  authority_weight: 0.15
  evidence_weight: 0.10
  diversity_weight: 0.10
  depth_weight: 0.03
  consensus_weight: 0.02

extraction:
  protocol: cep                # cep | always_http | always_headless
  ml_predictor: true           # use ML to predict extraction layer
  qadd_enabled: true           # query-aware DOM distillation

research:
  rar_enabled: true            # reflection-augmented research
  max_reflection_loops: 3
  amrs_enabled: true           # multi-agent swarm (deep mode)
  evidence_graph: false        # EGP (enable for deep mode)

cache:
  enabled: true
  ttl: 3600
  max_size: 500mb
  pds_tiers: true              # cache all 4 PDS tiers

backends:
  duckduckgo: true
  searxng: true
  searxng_instances:
    - https://searx.be
    - https://search.sapti.me
  wikipedia: true
  direct_fetch: true
  hackernews: false
  github: false
  arxiv: false

resource_limits:
  max_memory_percent: 70
  max_cpu_percent: 80
  min_free_memory_mb: 512
  browser_pool_max: auto

politeness:
  respect_robots_txt: true
  per_domain_delay_ms: 500
  max_per_domain_concurrent: 2
```

---

## 12. System Architecture

### High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         Interface Layer                                       │
│  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌─────────────┐  │
│  │   CLI   │  │  Agent   │  │  REST    │  │    MCP    │  │  Framework  │  │
│  │ (human) │  │   CLI    │  │   API    │  │  Server   │  │  Adapters   │  │
│  └────┬────┘  └────┬─────┘  └────┬─────┘  └─────┬─────┘  └──────┬──────┘  │
├───────┴────────────┴────────────┴──────────────┴────────────────┴──────────┤
│                          Query Router & Intent Classifier                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │          Resource Awareness Engine (§13)                             │    │
│  │  CPU | RAM | GPU | Network | Disk → ExecutionPlan                   │    │
│  └─────────────────────────────┬───────────────────────────────────────┘    │
│                                │                                            │
│  ┌─────────────────────────────┴───────────────────────────────────────┐    │
│  │          Parallel Execution Engine (§14)                             │    │
│  │  Priority Queue | Worker Pool | Backpressure | Domain Rate Limiter  │    │
│  └─────────────────────────────┬───────────────────────────────────────┘    │
│                                │                                            │
│  ┌─────────────┬───────────────┼───────────────┬───────────────────────┐    │
│  │   Search    │   Extraction  │    Ranking    │    Validation         │    │
│  │  Backend    │   Pipeline    │    Engine     │      Layer            │    │
│  │ Orchestrator│   + CEP       │  HyperFusion  │    + RAR              │    │
│  │   (§15)     │  + QADD (§16) │    (§21)      │    (§19)              │    │
│  └──────┬──────┘───────┬───────┘───────┬───────┘───────┬──────────────┘    │
│         │              │               │               │                    │
│  ┌──────┴──────────────┴───────────────┴───────────────┴──────────────┐    │
│  │              Token Efficiency Layer (§20)                           │    │
│  │  QATBE (§17) | SCS (§18) | PDS (§27) | Boilerplate Strip | BM25  │    │
│  └─────────────────────────────┬─────────────────────────────────────┘    │
│                                │                                            │
│  ┌────────────────┬────────────┴────────────┬────────────────────────┐    │
│  │  AI Preview    │  Citation & Evidence    │   Output & Export      │    │
│  │  Engine (§23)  │  EGP System (§24)      │   + PDS System (§26)   │    │
│  └────────────────┘─────────────────────────┘────────────────────────┘    │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────┐      │
│  │  Cache & Index (§28) | Plugin System (§29) | MCP Server (§30)    │      │
│  └───────────────────────────────────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Core Language** | **Rust** | Zero-cost abstractions, memory safety, fearless concurrency, no GC pauses, single static binary |
| Async Runtime | tokio | Industry-standard async runtime for Rust |
| HTTP Client | reqwest + hyper | Fastest HTTP with connection pooling, HTTP/2 |
| HTML Parsing | scraper (CSS) + lol_html (streaming) | Near-zero-copy streaming HTML parsing |
| Article Extract | readability-rs (port) + custom | Native Readability without JS overhead |
| Headless Browser | chromiumoxide / headless-chrome | CDP protocol in pure Rust, ultra-low memory |
| **Google Search** | Headless Chromium scraping | Full Google results via stealth headless |
| **Bing Search** | Headless Chromium scraping | Full Bing results via stealth headless |
| **DuckDuckGo** | HTTP scraping (html.duckduckgo.com) | Lightweight, no headless needed |
| **SearXNG** | HTTP JSON API | Meta-search aggregation |
| **Wikipedia** | REST API | Authoritative structured content |
| **Google Scholar** | Headless Chromium scraping | Academic papers and citations |
| **Brave Search** | HTTP scraping | Privacy-focused results |
| **Yandex** | HTTP scraping | Alternative perspective, non-Western results |
| Database | rusqlite (SQLite) + redb | Fastest embedded DB, zero overhead |
| Vector Store | hnswlib-rs or usearch | HNSW with SIMD-accelerated distance |
| Embeddings | ort (ONNX Runtime) + candle | Local ONNX or native Rust inference |
| BM25 | tantivy | Rust-native full-text search (Lucene-quality) |
| AI - Ollama | HTTP client to localhost:11434 | Local LLM inference |
| AI - llama.cpp | llama-cpp-rs bindings | Embedded GGUF inference, Metal/CUDA |
| AI - candle | HuggingFace candle | Pure Rust ML inference |
| CLI Framework | clap + indicatif + console | Best Rust CLI ecosystem |
| MCP | rmcp or custom MCP SDK | Rust MCP implementation |
| Config | config-rs + serde | Multi-format (YAML/JSON/TOML) |
| Serialization | serde + simd-json | SIMD-accelerated JSON parsing |
| Testing | cargo test + criterion | Built-in + benchmarking |
| Packaging | cargo-binstall + npm wrapper | Native binary + npm/pnpm/bun install via postinstall |

### Why Rust?

| Dimension | Node.js/TypeScript | **Rust** |
|-----------|-------------------|----------|
| **Speed** | V8 JIT, GC pauses | Native compiled, zero GC, SIMD |
| **Memory** | ~50MB base + V8 heap | ~5MB base, zero-copy parsing |
| **Concurrency** | Event loop + worker threads | tokio async + rayon data parallelism |
| **Binary** | Requires Node.js runtime | Single static binary, no runtime |
| **Startup** | ~200ms (V8 init) | ~5ms |
| **HTML Parsing** | cheerio: ~50ms/page | lol_html: ~2ms/page (25x faster) |
| **Headless Browser** | Playwright: ~150MB/instance | chromiumoxide: ~80MB/instance |
| **HTTP** | undici: fast | reqwest+hyper: faster, HTTP/2 native |
| **Cross-platform** | Needs Node.js installed | Single binary for Linux/macOS/Windows |

### npm/pnpm/bun Compatibility

Fetchium ships as a **Rust binary** with an **npm wrapper** for seamless installation:

```bash
# Installs pre-built native binary for your platform
npm install -g fetchium
pnpm add -g fetchium
bun add -g fetchium

# The npm package contains:
# - postinstall script that downloads platform-specific binary
# - bin stubs (hsx, hyper) that invoke the native binary
# - Supports: linux-x64, linux-arm64, darwin-x64, darwin-arm64, win-x64
```

Alternatively, install directly via Cargo:
```bash
cargo install fetchium
```

Or download pre-built binaries from GitHub Releases.

---

## 13. Machine Resource Awareness Engine

### Resource Detection

```typescript
interface ResourceProfile {
  cpu: { cores: number; model: string; usagePercent: number; };
  memory: { totalMB: number; freeMB: number; usedPercent: number; };
  network: { type: 'fast'|'moderate'|'slow'|'offline'; bandwidthMbps: number; latencyMs: number; };
  gpu: { available: boolean; memoryMB: number; type: 'metal'|'cuda'|'vulkan'|'none'; };
  disk: { freeGB: number; type: 'ssd'|'hdd'; };
  browsers: { installed: string[]; };
}
```

### Adaptive Tiers

| Tier | RAM | Parallel Fetches | Browser Pool | AI Models | AMRS Agents |
|------|-----|-----------------|-------------|-----------|-------------|
| **Minimal** | <4GB | 2-4 | 0-1 | 1-3B | Sequential |
| **Light** | 4-8GB | 4-8 | 1-2 | 3-7B | 2 parallel |
| **Standard** | 8-16GB | 8-16 | 2-4 | 7-13B | 3 parallel |
| **Power** | 16-32GB | 16-32 | 4-6 | 13-70B | 4 parallel |
| **Ultra** | 32GB+ | 32-50 | 6-8 | 70B+ | All parallel |

### Runtime Monitoring

- **Heartbeat**: Check resources every 5s during operations
- **Backpressure**: Pause new tasks when memory exceeds threshold
- **Adaptive Scaling**: Increase/decrease workers based on real-time metrics
- **OOM Prevention**: Hard ceiling — kill browser instances first, then reduce workers
- **Network Adaptation**: Adjust timeouts and fetch strategy based on measured bandwidth

```bash
hsx doctor
# CPU: Apple M2 Pro (12 cores) — 23% usage
# RAM: 32GB total, 18GB free — 44%
# GPU: Metal 16GB
# Network: Fast (245 Mbps, 12ms latency)
# → Tier: Power | Parallel: 32 | Browsers: 6 | AI: up to 70B
```

---

## 14. Parallel Execution Engine

```
┌────────────────────────────────────────────────┐
│           Priority Task Queue                   │
│  critical > high > normal > low > idle          │
├────────────────────────────────────────────────┤
│           Domain-Aware Scheduler                │
│  Per-domain concurrency caps + delay            │
│  robots.txt crawl-delay respected               │
├────────────────────────────────────────────────┤
│           Worker Pool                           │
│  HTTP Workers (lightweight)                     │
│  Browser Workers (managed Playwright)           │
│  AI Workers (model inference)                   │
├────────────────────────────────────────────────┤
│           Backpressure Controller               │
│  Memory monitor | CPU monitor | Network probe   │
│  Auto-pause | Auto-resume | Graceful degrade    │
└────────────────────────────────────────────────┘
```

---

## 15. Search Backend Orchestrator

### All Free, No API Keys — All Major Search Engines

#### Tier 1: Primary Search Engines (via efficient headless Chromium)

| Backend | Method | Strengths | Mode |
|---------|--------|-----------|------|
| **Google Search** | Headless Chromium (stealth) | Best overall index, freshest results, largest coverage | Headless |
| **Google Scholar** | Headless Chromium (stealth) | Academic papers, citation counts, related work | Headless |
| **Bing** | Headless Chromium (stealth) | Strong for technical queries, good image/video search | Headless |
| **Brave Search** | HTTP scrape | Independent index, privacy-focused, good freshness | HTTP |

Fetchium uses **chromiumoxide** (Rust CDP client) to drive a headless Chromium instance. Key optimizations:

- **Stealth mode**: Randomized fingerprints, realistic user-agent, `navigator.webdriver` patched, TLS fingerprint rotation
- **Connection reuse**: Single browser instance, multiple tabs for parallel searches
- **Resource blocking**: Block images, CSS, fonts, ads — only load HTML+JS for speed
- **DOM extraction**: Extract search results directly from rendered DOM via CSS selectors
- **Pool management**: Resource-aware browser pool (1-8 instances based on RAM)
- **Fallback chain**: If Google detects bot → fall back to Bing → fall back to DuckDuckGo → fall back to SearXNG
- **Rate limiting**: Intelligent per-engine rate limiting with jitter to avoid detection
- **Session rotation**: Rotate sessions/cookies to distribute load

#### Tier 2: Lightweight Backends (HTTP only, no headless needed)

| Backend | Method | Strengths | Mode |
|---------|--------|-----------|------|
| **DuckDuckGo** | HTTP scrape (`html.duckduckgo.com`) | Fast, private, no bot detection | HTTP |
| **SearXNG** | JSON API to public/self-hosted instances | Aggregates 244+ engines | HTTP |
| **Wikipedia** | Official REST API | Authoritative, structured, fast | HTTP |
| **Hacker News** | Algolia API (free) | Tech news and discussions | HTTP |
| **ArXiv** | Public API | Academic preprints | HTTP |
| **GitHub** | Public API | Code, repos, issues | HTTP |
| **Reddit** | Public JSON | Community discussions | HTTP |
| **StackOverflow** | Public API | Programming Q&A | HTTP |
| **Common Crawl** | CDX API | Historical web data | HTTP |

#### Tier 3: Expandable (via Plugins)

| Backend | Use Case |
|---------|----------|
| **Yandex** | Non-Western results, alternative perspective |
| **Baidu** | Chinese-language search |
| **PubMed** | Medical/biomedical research |
| **Semantic Scholar** | AI/ML paper search with citation graphs |
| **Patent search** | USPTO, EPO patent databases |
| **Legal search** | Case law, regulations |
| Custom | Any domain-specific source |

### Headless Search Engine Strategy

The key innovation is using headless Chromium efficiently in Rust for search engines that block API access:

```
┌──────────────────────────────────────────────────────┐
│           Headless Search Pool                        │
│                                                       │
│  Browser Instance Manager (chromiumoxide)              │
│  ├─ Pool size: 1-8 (based on available RAM)          │
│  ├─ Tab reuse: open tab → search → extract → close   │
│  ├─ Session rotation: new cookies every N searches    │
│  └─ Resource blocking: images/CSS/fonts disabled      │
│                                                       │
│  Stealth Layer                                        │
│  ├─ navigator.webdriver = false                       │
│  ├─ Randomized viewport, language, timezone           │
│  ├─ Realistic mouse movement simulation               │
│  ├─ TLS fingerprint matching (JA3/JA4)               │
│  └─ User-agent rotation from real browser pool        │
│                                                       │
│  Search Engine Adapters                               │
│  ├─ GoogleAdapter: CSS selectors for result parsing   │
│  ├─ BingAdapter: CSS selectors for result parsing     │
│  ├─ ScholarAdapter: CSS selectors for paper parsing   │
│  ├─ BraveAdapter: HTTP-only, no headless needed       │
│  └─ [Pluggable: add any engine]                       │
│                                                       │
│  Anti-Detection                                       │
│  ├─ Per-engine rate limits with jitter                │
│  ├─ CAPTCHA detection → auto-fallback to next engine  │
│  ├─ IP rotation via SOCKS5 proxy (if configured)      │
│  └─ Automatic cooldown on 429/503 responses           │
└──────────────────────────────────────────────────────┘
```

### Memory Efficiency of Rust Headless vs Node.js

| | Node.js (Playwright) | **Rust (chromiumoxide)** |
|--|---------------------|------------------------|
| Per instance | ~150-300MB | ~80-150MB |
| Startup | ~500ms | ~200ms |
| Search extraction | ~1s | ~300ms |
| 4 parallel searches | ~800MB | ~400MB |
| GC pauses | Yes (V8) | **None** |

### Result Fusion

```
1. PARALLEL QUERY to all enabled backends
2. COLLECT with per-backend timeout (fail-fast)
3. NORMALIZE to unified ResultItem schema
4. DEDUPLICATE via canonical URL + SimHash content
5. RANK via HyperFusion (8-signal, intent-adaptive)
6. DIVERSIFY (max 3 per domain)
7. VALIDATE via validation layer
8. RETURN top N
```

---

## 16. Content Extraction Pipeline

### Cascade Extraction Protocol (CEP)

5-layer cascade with ML-predicted method selection:

| Layer | Method | Cost | RAM | When Used |
|-------|--------|------|-----|-----------|
| 1 | HTTP + Cheerio | ~2ms | ~5MB | Static HTML pages (85% of web) |
| 2 | HTTP + Readability | ~8ms | ~10MB | Article pages |
| 3 | Playwright headless shell | ~400ms | ~100MB | JS-rendered, detected by heuristics |
| 4 | Playwright full + wait | ~2s | ~250MB | Complex SPAs |
| 5 | Playwright + scroll + interact | ~5s | ~300MB | Infinite scroll, lazy load |

### ML Method Predictor

Trained on: domain, content-type, HTML structure, framework markers, script density, historical success rate.

**Prediction accuracy target**: >90% — meaning 90% of URLs go directly to the correct extraction layer without trial-and-error.

### QADD (Query-Aware DOM Distillation)

When a query is provided, QADD prunes the DOM before extraction:
1. Structural pruning (nav, footer, ads)
2. BM25 text node scoring against query
3. Semantic embedding check for remaining nodes
4. Context preservation (table headers, list context, headings)
5. Token budget packing

---

## 17. Query-Aware Token-Budgeted Extraction (QATBE)

The single most important feature for AI agent consumption. Full specification in §8.2.

### API Surface

```bash
# Fetch with query awareness and token budget
hsx agent-fetch https://example.com \
  --query "pricing for enterprise plan" \
  --budget 1500 \
  --format segments

# Pre-fetch estimation
hsx agent-fetch https://example.com --estimate
# → { "total_tokens": 12340, "relevant_to_query": 1850, "extraction_layer": 1 }

# Batch fetch with budget across multiple URLs
hsx agent-fetch urls.txt --query "security best practices" --budget 5000 --deduplicate
```

### Programmatic API

```typescript
import { agentFetch } from 'fetchium';

const result = await agentFetch({
  url: 'https://example.com/docs',
  query: 'authentication patterns',
  tokenBudget: 2000,
  format: 'segments',
  deduplicate: true
});

// result.tokens_used: 1847
// result.relevance_coverage: 0.92
// result.segments: [{ type, relevance, tokens, content }, ...]
```

---

## 18. Semantic Content Segmentation (SCS)

Full specification in §8.4. Key segment types:

| Type | Token Efficiency vs Markdown | Best For |
|------|----------------------------|----------|
| `fact` | 40% fewer tokens | Agent reasoning |
| `table` | 60% fewer tokens (JSON vs md table) | Data extraction |
| `code` | Same | Code analysis |
| `list` | 30% fewer tokens (array vs bullets) | Enumeration |
| `data` | 50% fewer tokens (key:value vs prose) | Structured data |
| `paragraph` | Same | Narrative content |

### SCS Output Example

```json
{
  "format": "scs",
  "tokens": 1200,
  "segments": [
    { "type": "fact", "claim": "PostgreSQL supports JSON indexing since v12", "confidence": 0.95, "source_ref": 1 },
    { "type": "table", "headers": ["Feature", "PostgreSQL", "MySQL"], "rows": [["JSON", "Native JSONB", "JSON type"]] },
    { "type": "code", "language": "sql", "code": "CREATE INDEX idx ON docs USING gin(data jsonb_path_ops);" },
    { "type": "data", "key": "Max JSON document size", "value": "1GB", "unit": "bytes" }
  ]
}
```

---

## 19. Validation & Reliability Layer

### 6-Layer Validation + RAR Self-Correction

```
V1: Source Validation       → Reachability, SSL, domain reputation, redirect analysis
V2: Content Validation      → Relevance, language, dedup, paywall, error page detection
V3: Freshness Validation    → Published date, staleness, cache freshness
V4: Cross-Source Validation → Claim consistency, triangulation, contradiction detection
V5: Extraction Quality      → Completeness, structure, encoding, truncation
V6: Output Integrity        → Citation verification, link validity, format compliance
```

### RAR Integration

After V4 (cross-source validation), RAR reflection kicks in:
- **R2 [RELEVANT]**: If <50% of results are relevant, reformulate query and re-search
- **R3 [SUFFICIENT]**: If extracted content doesn't answer the query, fetch more sources
- **R5 [CONSISTENT]**: If major contradictions found, spawn Verify Agent to investigate

### Validation Modes

```bash
--validate strict    # All 6 layers + RAR (slowest, most reliable)
--validate standard  # V1-V3 + basic V4 (default)
--validate fast      # V1 only (fastest)
--validate off       # Skip validation
```

---

## 20. Token Efficiency Architecture

### Optimization Stack (cumulative savings)

| Stage | Technique | Token Savings |
|-------|-----------|--------------|
| 1 | QADD (DOM distillation) | ~60% (query-relevant nodes only) |
| 2 | Boilerplate stripping | ~30% of remaining |
| 3 | SCS (semantic segmentation) | ~30% of remaining |
| 4 | BM25 "Fit Markdown" filter | ~20% of remaining |
| 5 | Cross-source dedup (SimHash) | ~10% of remaining |
| 6 | PDS tier selection | Variable (200-5000 tokens) |
| **Total** | **QADD + Strip + SCS + BM25 + Dedup** | **~97% vs raw HTML** |

### Comparison

| Format | Tokens (typical page) |
|--------|----------------------|
| Raw HTML | 50,000 |
| Clean HTML | 12,000 |
| Flat Markdown (Firecrawl/Jina) | 4,000 |
| Fit Markdown (Crawl4AI style) | 2,500 |
| **Fetchium SCS + QATBE** | **1,500** |
| **Fetchium key_facts tier** | **200** |

### Token Budget System

```bash
hsx agent-search "query" --budget 2000     # Fit into ~2K tokens
hsx agent-fetch <url> --budget 1500        # Extract ~1.5K tokens of relevant content
hsx research "query" --budget auto         # Auto-size based on AI model context
```

---

## 21. Semantic Search & Hybrid Ranking

### HyperFusion Algorithm

8-signal differentiable rank fusion with intent-adaptive weights (full spec in §8.1).

### Components

| Signal | Method | Purpose |
|--------|--------|---------|
| BM25 | lunr.js | Lexical precision |
| Semantic | all-MiniLM-L6-v2 via ONNX | Conceptual relevance |
| Temporal | Exponential decay with intent-calibrated half-life | Freshness |
| Authority | Domain scoring + citation chain analysis | Source trust |
| Evidence | Factual claim density per 100 words | Information richness |
| Diversity | Domain diversity bonus | Breadth |
| Depth | Content structure analysis (headings, sections, length) | Thoroughness |
| Consensus | Cross-source claim agreement | Reliability |

### Cascade Retrieval (Matryoshka-inspired)

For local index search:
1. **Stage 1**: BM25 sparse retrieval → top 1000 candidates (sub-millisecond)
2. **Stage 2**: 64-dim truncated embeddings → top 100 (milliseconds)
3. **Stage 3**: Full-dim embeddings → top 20 (tens of milliseconds)
4. **Stage 4**: HyperFusion full scoring → final ranking

### Query Understanding

- **Intent classification**: Lightweight local model classifies into 10 categories
- **Query expansion**: Synonyms, acronyms, related terms
- **HyDE**: For ambiguous queries, generate hypothetical answer → embed → search

---

## 22. Cutting-Edge Research Integration

### Papers and Techniques Integrated

| Paper/Technique | How Fetchium Uses It |
|----------------|------------------------|
| **Self-RAG** (ICLR 2024) | RAR reflection tokens evaluate retrieval quality |
| **CRAG** (ICLR 2025) | Lightweight retrieval evaluator triggers re-search |
| **RAPTOR** (ICLR 2024) | Tree-organized retrieval for deep research synthesis |
| **GraphRAG** (Microsoft) | Knowledge graph construction in deep mode |
| **ReaderLM-v2** (Jina) | Optional ML-based HTML→Markdown conversion |
| **D2Snap** | DOM downsampling integrated into QADD |
| **Focused ReAct** | Research loop with query-saliency preservation |
| **Matryoshka Embeddings** | Cascade retrieval with truncated dimensions |
| **Late Chunking** (Jina) | Superior embeddings for local index |
| **ColBERT/SPLATE** | Optional late-interaction retrieval for large indexes |
| **HyDE** | Hypothetical document embeddings for ambiguous queries |
| **Mix-of-Granularity** | Dynamic chunk size routing based on query type |
| **Context Rot** (Chroma) | MECW-aware context assembly, never overfill |
| **Ms-PoE** (NeurIPS 2024) | Sandwich layout for multi-result context assembly |
| **Semantic Compression** (SrCr) | Two-tier compression preserving semantic fidelity |
| **Observation Masking** (JetBrains) | Compress observations, preserve reasoning chain |
| **RAGCache** | KV tensor caching for repeated query patterns |
| **Exp4Fuse** | LLM-augmented query variants for better RRF |
| **BGE-reranker-v2.5** | Optional cross-encoder reranking |
| **Reflexion** | Episodic memory for learning from past searches |

---

## 23. AI Preview Engine

### Architecture

```
Search Results → QATBE → Sandwich Layout Assembly → Local LLM → Citation Injection → Output
```

### Sandwich Layout (Ms-PoE inspired)

Place highest-confidence results at the beginning and end of context, lowest in the middle — mitigating the "lost in the middle" problem:

```
[CONTEXT START]
  High-confidence source 1
  High-confidence source 2
  Medium-confidence source 3  ← middle (lower attention)
  Medium-confidence source 4  ← middle
  High-confidence source 5
  High-confidence source 6
[CONTEXT END]
```

### Model Integration

| Provider | Method | When |
|----------|--------|------|
| **Ollama** | HTTP API (localhost:11434) | Recommended default |
| **node-llama-cpp** | Embedded GGUF inference | No server needed |
| **Custom endpoint** | OpenAI-compatible API | Any provider |

### Model Routing

| Query Complexity | Model Tier | Examples |
|-----------------|-----------|----------|
| Simple factual | Small (1-3B) | phi-3-mini, qwen2.5:1.5b |
| Standard | Medium (7-8B) | llama3.2:8b, mistral:7b |
| Complex synthesis | Large (13B+) | llama3.2:70b, mixtral |

---

## 24. Citation & Evidence System

### Citation Styles

6 styles: `inline` `[1]` | `footnote` ^1 | `apa` (Author, Year) | `ieee` [1] | `chicago` | `bibtex`

### Evidence Graph Protocol (EGP)

Full spec in §8.7. Graph-based evidence linking with:
- Claim → Source edges with supporting quotes
- SHA-256 content hashes for verification
- Confidence scoring per claim
- Contradiction edges with severity

### Strict Evidence Mode

```bash
hsx research "query" --strict-evidence
```
Every factual statement must cite a source. Uncitable claims marked `[unverified]`. Citation links validated against actual source content.

---

## 25. Agent Framework Integration

### LangChain

```typescript
import { FetchiumRetriever } from 'fetchium/langchain';

const retriever = new FetchiumRetriever({
  tokenBudget: 3000,
  tier: 'detailed',
  validate: true
});

// Returns LangChain Document[] objects
const docs = await retriever.getRelevantDocuments("query");
```

### CrewAI

```typescript
import { FetchiumTool } from 'fetchium/crewai';

const searchTool = new FetchiumTool({
  name: "web_search",
  tokenBudget: 2000,
  tier: 'summary'
});
// Returns string output optimized for CrewAI agents
```

### AutoGPT / Custom Agents

```typescript
import { search, agentFetch, agentResearch } from 'fetchium';

// Direct library usage with full control
const results = await search({ query: "...", tokenBudget: 2000, format: 'segments' });
```

### Claude Code / MCP

See §30 for full MCP server specification.

---

## 26. Output & Export System

### Supported Formats

| Format | Extension | Agent Use | Human Use |
|--------|-----------|-----------|-----------|
| **Segments (SCS)** | .json | Primary agent format | N/A |
| **Markdown** | .md | Secondary | Primary |
| **JSON** | .json | Structured data | Programmatic |
| **YAML** | .yaml | Config, structured | Readable structured |
| **CSV** | .csv | Data analysis | Spreadsheets |
| **HTML** | .html | Email, web | Reports |
| **Plain Text** | .txt | Universal | Minimal |
| **BibTeX** | .bib | Citation mgmt | Academic |
| **PDF** | .pdf | N/A | Formal reports |
| **DOCX** | .docx | N/A | Word docs |
| **JSONL** | .jsonl | Streaming | Log format |
| **Clipboard** | - | N/A | Quick paste |

### Multi-Format Export

```bash
hsx research "query" --format md,json,csv --output ./results/
```

---

## 27. Progressive Detail Streaming (PDS)

Full spec in §8.9. 4 tiers pre-computed at extraction time:

| Tier | Tokens | Content |
|------|--------|---------|
| `key_facts` | ~200 | Top 5 factual claims, one line each |
| `summary` | ~1,000 | Executive summary with key findings |
| `detailed` | ~5,000 | Full analysis with evidence |
| `complete` | All | Everything extracted |

### Tier Expansion API

```typescript
// Get summary first (cheap)
const summary = await hsx.agentSearch({ query: "...", tier: "key_facts" });

// Need more? Expand without re-fetching
const detailed = await hsx.expandTier(summary.resultId, "detailed");
```

---

## 28. Caching & Local Index

### Cache Layers

| Layer | Storage | TTL | Speed |
|-------|---------|-----|-------|
| L1: Memory LRU | Process memory | Session | <1ms |
| L2: Disk (SQLite) | File system | Configurable | <5ms |
| L3: PDS tier cache | SQLite | Persistent | <5ms |
| L4: Vector index | HNSW files | Persistent | <20ms |
| L5: Embedding cache | SQLite | Persistent | <1ms |

### RAGCache-Inspired Optimization

Cache query pattern → result mappings. When a similar query (>0.9 cosine similarity) is seen, return cached results immediately and asynchronously check for updates.

### Local Index

- **SQLite FTS5**: Lexical full-text search
- **HNSW vectors**: Semantic similarity search
- **Late Chunking**: Superior chunk embeddings (per Jina research, +24.47% improvement)
- **Hybrid query**: BM25 + vector + RRF fusion

---

## 29. Plugin & Extension System

### Plugin Types

| Type | Purpose | Example |
|------|---------|---------|
| **Backend** | Search sources | `hsx-plugin-arxiv`, `hsx-plugin-pubmed` |
| **Extractor** | Content extraction | `hsx-plugin-youtube-transcript`, `hsx-plugin-pdf-ocr` |
| **Ranker** | Ranking algorithms | `hsx-plugin-academic-ranker` |
| **Formatter** | Output formats | `hsx-plugin-latex`, `hsx-plugin-notion` |
| **Validator** | Quality checks | `hsx-plugin-factcheck`, `hsx-plugin-bias-detector` |
| **AI Provider** | Model integrations | `hsx-plugin-groq`, `hsx-plugin-together` |

```bash
hsx plugin install hsx-plugin-arxiv
hsx plugin list
hsx plugin create my-plugin
```

---

## 30. MCP Server Mode

### Purpose

Expose Fetchium as a Model Context Protocol server for Claude, Claude Code, and any MCP client — with **composite tools** that handle the full pipeline internally.

### Starting the Server

```bash
hsx serve --mcp                           # stdio transport
hsx serve --mcp --transport sse --port 3001  # SSE transport
```

### MCP Tools Exposed

```typescript
// COMPOSITE TOOLS (full pipeline, single call)
tools: [
  {
    name: "hypersearch_search",
    description: "Search the web and return token-efficient results",
    inputSchema: {
      query: string,
      token_budget?: number,    // max tokens in response
      tier?: "key_facts" | "summary" | "detailed" | "complete",
      max_sources?: number,
      validate?: boolean
    }
  },
  {
    name: "hypersearch_fetch",
    description: "Fetch a URL with query-aware extraction",
    inputSchema: {
      url: string,
      query?: string,           // extract only content relevant to this
      token_budget?: number,
      format?: "markdown" | "segments" | "json"
    }
  },
  {
    name: "hypersearch_research",
    description: "Conduct multi-source research with citations",
    inputSchema: {
      query: string,
      token_budget?: number,
      depth?: "shallow" | "standard" | "deep",
      strict_evidence?: boolean,
      citation_style?: string
    }
  },
  {
    name: "hypersearch_estimate",
    description: "Estimate token cost before fetching",
    inputSchema: { url: string }
  },
  {
    name: "hypersearch_expand",
    description: "Get more detail on previous results without re-fetching",
    inputSchema: { result_id: string, tier: string }
  }
]
```

### Why Composite Tools Matter

Existing MCP search servers expose individual tools (search, fetch, scrape). The LLM must orchestrate 3-5 tool calls, wasting tokens on planning. Fetchium composite tools handle the entire pipeline — search + fetch + extract + rank + validate — in **one tool call**.

---

## 31. Cross-Session Learning & Persistent Intelligence

### 31.1 Persistent Intelligence Engine (PIE)

**Problem**: Every existing tool treats every query as isolated. Zero learning across sessions. The 1000th query performs exactly like the 1st.

**Invention**: A 4-layer persistent memory architecture that compounds value over time:

```
┌─────────────────────────────────────────────────────────┐
│              Persistent Intelligence Engine               │
│                                                           │
│  Layer 1: Personal Knowledge Graph (PKG)                  │
│  ├─ Entities discovered across all searches               │
│  ├─ Relationships between concepts                        │
│  ├─ Evolves with every query                             │
│  └─ Enables: "You researched X before, Y is related"     │
│                                                           │
│  Layer 2: Source Trust Memory (STM)                       │
│  ├─ Per-domain trust scores learned from user feedback    │
│  ├─ Extraction success rates per domain per method        │
│  ├─ Rate limit patterns per engine                       │
│  └─ Enables: Smarter source selection and ranking         │
│                                                           │
│  Layer 3: Failure Pattern Memory (FPM)                    │
│  ├─ Which URLs/domains failed and why                    │
│  ├─ Which extraction methods worked where                │
│  ├─ Anti-bot detection patterns per engine               │
│  └─ Enables: Never fail the same way twice               │
│                                                           │
│  Layer 4: Query Prediction Model (QPM)                    │
│  ├─ User's research patterns and topic interests         │
│  ├─ Common follow-up query sequences                     │
│  ├─ Domain-specific query templates                      │
│  └─ Enables: Predictive prefetching and suggestions      │
└─────────────────────────────────────────────────────────┘
```

### Storage

All persistent data stored in local SQLite (encrypted at rest):
```
~/.fetchium/intelligence/
├── knowledge_graph.db     # PKG: entities, relationships
├── source_trust.db        # STM: domain scores, extraction rates
├── failure_patterns.db    # FPM: error logs, method success rates
└── query_patterns.db      # QPM: research history, predictions
```

### CLI

```bash
hsx intelligence stats          # Show learning stats
hsx intelligence reset           # Reset all learned data
hsx intelligence export          # Export for backup/sharing
hsx intelligence suggest         # Get query suggestions based on patterns
```

### How It Compounds

| Queries Completed | Intelligence Level | Capabilities Unlocked |
|-------------------|-------------------|----------------------|
| 0-50 | Baseline | Default ranking, no personalization |
| 50-200 | Learning | Source trust scoring, failure avoidance |
| 200-1000 | Adapted | Personalized ranking, query prediction |
| 1000+ | Expert | Full PKG, predictive prefetch, pattern matching |

---

## 32. Tree-of-Thoughts & Advanced Reasoning

### 32.1 Tree-of-Thoughts Research (ToTR)

**Problem**: Linear research (query → results → synthesis) fails on complex multi-faceted questions. RAR self-corrects, but doesn't explore alternative reasoning paths.

**Invention**: Parallel reasoning paths with branch evaluation and cross-path synthesis:

```
Query: "Is React or Vue better for enterprise in 2026?"

Tree-of-Thoughts:
├─ Path A: Performance Analysis
│   ├─ A1: Bundle size comparison → [search + evidence]
│   ├─ A2: Runtime benchmarks → [search + evidence]
│   └─ A3: Memory footprint → [search + evidence]
│   → Path A conclusion: "React wins on ecosystem, Vue on bundle size"
│
├─ Path B: Ecosystem & Enterprise Readiness
│   ├─ B1: Enterprise tooling → [search + evidence]
│   ├─ B2: TypeScript support quality → [search + evidence]
│   └─ B3: Long-term support guarantees → [search + evidence]
│   → Path B conclusion: "React has larger enterprise adoption"
│
├─ Path C: Developer Productivity
│   ├─ C1: Learning curve → [search + evidence]
│   ├─ C2: Hiring pool size → [search + evidence]
│   └─ C3: Documentation quality → [search + evidence]
│   → Path C conclusion: "Vue easier to learn, React easier to hire"
│
└─ SYNTHESIS: Cross-path weighted conclusion
   "React recommended for large enterprises (hiring + tooling),
    Vue recommended for smaller teams (productivity + simplicity)"
    [Confidence: 82% | Sources: 18 | Contradictions: 2]
```

### 32.2 Graph-of-Thoughts (GoT)

For non-linear reasoning where paths intersect:

```
   [Performance]──────────┐
        │                  │
   [Ecosystem]─────[Enterprise Fit]─────[Recommendation]
        │                  │
   [Productivity]─────────┘
```

Thoughts can merge, split, and loop — enabling complex synthesis impossible with linear or tree structures.

### 32.3 Self-Debate Protocol

For controversial or subjective topics, spawn two reasoning agents:
- **Advocate Agent**: Argues FOR the claim, finds supporting evidence
- **Critic Agent**: Argues AGAINST, finds contradicting evidence
- **Judge Agent**: Weighs both sides, produces balanced synthesis

```bash
hsx deep "is cryptocurrency a good investment" --self-debate
# Output includes: FOR arguments [sources], AGAINST arguments [sources], balanced synthesis
```

---

## 33. Proactive Intelligence & Anticipatory Search

### 33.1 Topic Monitoring & Research Radar

**Problem**: Fetchium is reactive — waits for queries. World-class tools anticipate needs.

```bash
# Subscribe to topics
hsx subscribe "TypeScript breaking changes" --notify webhook --digest weekly
hsx subscribe "kubernetes CVE" --notify email --threshold critical
hsx subscribe "competitor:acme-corp" --digest daily

# Research radar — suggests new relevant findings based on your history
hsx radar --limit 10
# Based on your research patterns:
# 1. [NEW] Bun v2.0 released with Node.js full compat — 3h ago
# 2. [UPDATE] React Server Components best practices updated — 12h ago
# 3. [PAPER] New RAG technique outperforms RAPTOR by 15% — 1d ago

# Intelligent digest
hsx digest --period weekly --topics "rust,webassembly,ai-agents"
```

### 33.2 Predictive Prefetching

Based on QPM (Query Prediction Model):
- When user searches "React performance", prefetch "React vs Vue" and "React optimization techniques"
- When user researches a library, prefetch its alternatives and comparisons
- Cache prefetched results for instant access

### 33.3 Anomaly Detection

```bash
hsx monitor "OpenAI API pricing" --anomaly-detect
# Alert: "OpenAI pricing page changed significantly (87% content diff) — 2h ago"
```

---

## 34. Multimodal Content Understanding

### 34.1 Beyond Text: Full Multimodal Pipeline

| Content Type | Extraction Method | Output |
|-------------|------------------|--------|
| **Images** | ONNX vision model (local) or alt text + context | Captions, OCR text, described content |
| **Videos** | YouTube transcript API / Whisper-based transcription | Timestamped text, key moments |
| **Audio** | Whisper-rs (local Rust transcription) | Full transcript, speaker detection |
| **Charts/Graphs** | Vision model → structured data | JSON data points, trend descriptions |
| **PDFs** | pdf-extract + OCR fallback | Structured text with layout preservation |
| **Screenshots** | Tesseract OCR or vision model | Extracted text, UI element detection |
| **Diagrams** | Vision model → description | Mermaid/text description of relationships |

### CLI

```bash
hsx fetch https://example.com --multimodal     # Extract images, videos, charts too
hsx view https://youtube.com/watch?v=... --transcript  # Video transcript
hsx fetch image.png --ocr                       # OCR text from image
hsx fetch chart.png --extract-data              # Chart → JSON data points
```

### 34.2 Multimodal Search

```bash
hsx search "architecture diagram microservices" --include-images
hsx search "kubernetes tutorial" --include-videos
```

---

## 35. Adversarial Robustness & Trust Verification

### 35.1 Adversarial Content Shield (ACS)

**Problem**: In 2026, the web is flooded with AI-generated content, bot farms, SEO spam, and manipulated sources.

```
┌─────────────────────────────────────────────────┐
│          Adversarial Content Shield              │
│                                                   │
│  Detector 1: AI-Generated Content Detection       │
│  ├─ Statistical analysis (perplexity, burstiness) │
│  ├─ Stylometric fingerprinting                   │
│  ├─ Watermark detection (C2PA, Content Creds)    │
│  └─ Output: ai_generated_probability: 0-1        │
│                                                   │
│  Detector 2: Bot Farm / Coordinated Inauthentic  │
│  ├─ Temporal clustering (many similar posts)     │
│  ├─ Cross-source verbatim overlap detection      │
│  ├─ Author pattern analysis                      │
│  └─ Output: coordinated_probability: 0-1         │
│                                                   │
│  Detector 3: Source Manipulation Detection        │
│  ├─ Wayback Machine diff comparison              │
│  ├─ Content hash mismatch alerts                 │
│  ├─ Semantic drift detection over time           │
│  └─ Output: manipulation_probability: 0-1        │
│                                                   │
│  Detector 4: SEO Spam Detection                   │
│  ├─ Keyword stuffing analysis                    │
│  ├─ Thin content detection                       │
│  ├─ Link farm signal detection                   │
│  └─ Output: spam_probability: 0-1                │
│                                                   │
│  Aggregate Trust Score                            │
│  trust = 1 - max(ai_gen, coordinated, manip, spam)│
└─────────────────────────────────────────────────┘
```

### CLI

```bash
hsx search "query" --trust-verify           # Enable ACS for all results
hsx fetch https://example.com --check-ai    # Check if content is AI-generated
hsx research "topic" --human-sources-only   # Prefer human-written content
```

---

## 36. Privacy-First Architecture

### 36.1 Privacy Modes

| Mode | What Happens | Use Case |
|------|-------------|----------|
| **Standard** | Normal operation, local cache | Default |
| **Private** | No persistent cache, no learning, no history | Sensitive queries |
| **Tor** | Route all requests through Tor network | Anonymous research |
| **Air-Gap** | Local index only, zero network | Classified environments |

### CLI

```bash
hsx search "query" --private           # No traces left
hsx search "query" --tor               # Route through Tor
hsx search "query" --air-gap           # Local index only
hsx research "topic" --redact-pii      # Strip PII from output
hsx config set privacy.auto-expire 7d  # Auto-delete research after 7 days
```

### 36.2 Self-Destructing Research

```bash
hsx research "sensitive topic" --auto-expire 24h
# Research artifact auto-deleted after 24 hours
# Evidence graph and cache entries purged
```

### 36.3 Zero-Knowledge Search Architecture

For the most sensitive use cases, Fetchium can be configured so that:
- No external service sees the full query (queries split across engines)
- Local synthesis means no cloud AI sees the combined results
- Cache is encrypted at rest with user key
- Research artifacts can be encrypted with GPG

---

## 37. Collaborative Research Protocol

### 37.1 Shared Research Workspaces

```bash
# Create shared workspace
hsx workspace create "project-alpha" --invite user@example.com

# Collaborative research — results shared automatically
hsx research "topic" --workspace project-alpha

# Fork a research session
hsx research fork session-abc --name "alternative-approach"

# Merge findings from multiple researchers
hsx research merge session-abc session-def --deduplicate
```

### 37.2 Shared Features

| Feature | Description |
|---------|-------------|
| **Shared Knowledge Graph** | Team-wide PKG that grows from all members' research |
| **Annotation Layer** | Highlight and annotate shared sources |
| **Research Branching** | Fork research sessions, explore different angles |
| **Evidence Merging** | Combine evidence graphs from multiple sessions |
| **Citation Network** | See how team members' research connects |
| **Conflict Detection** | Alert when researchers find contradictory evidence |

### Storage

Shared workspaces sync via:
- **Local**: Shared directory (NFS, SMB)
- **Git**: Research artifacts as git-managed files
- **Custom**: Plugin for any sync backend

---

## 38. Domain-Specific Intelligence Modes

### Pre-Configured Modes

```bash
hsx research "topic" --mode academic    # Academic mode
hsx research "topic" --mode code        # Code intelligence
hsx research "topic" --mode legal       # Legal research
hsx research "topic" --mode financial   # Financial analysis
hsx research "topic" --mode medical     # Medical/scientific
hsx research "topic" --mode security    # Cybersecurity
```

| Mode | Backends Prioritized | Ranking Adjustments | Special Features |
|------|---------------------|--------------------|--------------------|
| **Academic** | ArXiv, Scholar, Semantic Scholar, PubMed | Boost: citation count, peer review, impact factor | BibTeX export, citation graph, replication status |
| **Code** | GitHub, StackOverflow, MDN, docs sites | Boost: code examples, recent commits, stars | Code extraction, dependency analysis, license check |
| **Legal** | Case law DBs, regulation sites, .gov | Boost: jurisdiction, precedent, recency | Citation chains, jurisdiction tagging, precedent mapping |
| **Financial** | SEC EDGAR, earnings calls, market data | Boost: financial data density, official filings | Table extraction, trend analysis, ticker detection |
| **Medical** | PubMed, WHO, clinical trials, FDA | Boost: evidence level, peer review, sample size | Evidence grading (I-V), methodology analysis |
| **Security** | CVE DBs, NVD, security advisories | Boost: severity, exploitability, affected versions | CVSS scoring, affected package detection, patch status |

---

## 39. Self-Evolving Architecture

### 39.1 AutoML for Ranking

HyperFusion weights auto-optimize based on user feedback:

```
User clicks result #3 instead of #1
→ Implicit signal: #3 was more relevant
→ Adjust HyperFusion weights for this query type
→ Future similar queries rank better
```

### 39.2 Extraction Method Evolution

CEP ML predictor continuously improves:
- Track extraction success/failure per domain per method
- Retrain predictor periodically with new data
- A/B test new extraction strategies on subset of queries

### 39.3 Confidence Calibration Engine (CCE)

**Problem**: When a system says "85% confident," is it really 85%? Most tools have uncalibrated confidence.

**Invention**: Track prediction accuracy over time and calibrate:

```
Reported confidence: 85%
Historical accuracy at 85%: 87% (n=1,247 similar predictions)
Calibrated confidence: 87% ± 3%

Calibration table (built over time):
| Reported | Actual (n) | Calibrated |
|----------|-----------|------------|
| 50%      | 53% (892) | 53%        |
| 70%      | 68% (1,104) | 68%      |
| 85%      | 87% (1,247) | 87%      |
| 95%      | 91% (634) | 91%        |
```

### 39.4 Community Feedback Loop (opt-in)

With user consent, anonymized signals improve the global model:
- Which domains are reliable
- Which extraction methods work
- Common failure patterns
- All privacy-preserving (differential privacy)

---

## 40. Performance Requirements

### Latency Targets

| Operation | Cached | Uncached |
|-----------|--------|----------|
| `search` | <1s | <3s |
| `agent-search` (key_facts) | <500ms | <2s |
| `agent-fetch` (QATBE) | <300ms | <2s |
| `research` | <5s | <45s |
| `deep` | N/A | 1-10 min |
| `ai` | <1s + model time | <3s + model time |
| `fetch` | <200ms | <2s |
| Token estimation | <100ms | <500ms |

### Reliability

- 99% fetch success on non-protected pages
- <0.1% false positive on SPA detection (CEP ML predictor)
- <5% citation verification failures
- RAR catches and corrects >80% of bad retrievals
- Never crash — always degrade gracefully
- Automatic retry with exponential backoff (max 3)

---

## 41. Security & Compliance

- **No credential storage** for third-party services
- **No data exfiltration** — all data stays local
- **Sanitized output** — HTML sanitized before display
- **TLS enforcement** — HTTPS required, warn on cert issues
- **robots.txt respected** — always, configurable but strict default
- **Rate limiting** — per-domain delays, never abuse
- **No CAPTCHA bypass** — never attempt
- **No paywall bypass** — never attempt
- **GDPR-friendly** — no tracking, no telemetry
- **PII redaction** — optional `--redact-pii` flag

---

## 42. 300+ Advanced Features

### Search & Retrieval (1-35)

1. HyperFusion 8-signal intent-adaptive ranking
2. Query intent classification (10 categories)
3. Query decomposition into parallel sub-questions
4. Multi-hop retrieval with follow-up queries
5. Source diversity enforcement
6. SimHash near-duplicate detection
7. Canonical URL normalization
8. Redirect chain recording
9. Contradiction detection with severity scoring
10. Consensus scoring (% agreement across sources)
11. Query expansion (synonyms, acronyms, related terms)
12. Typo-tolerant fuzzy search
13. Boolean query support (AND, OR, NOT, quotes)
14. Site-specific search (`--site github.com`)
15. Filetype filtering (`--filetype pdf`)
16. Date-range filtering (`--after 2025-01-01`)
17. Multi-language search with translation
18. Region-specific result boosting
19. Safe search filtering
20. "Find primary sources" mode
21. "Find datasets" mode
22. "Find benchmarks" mode
23. "Find implementation repos" mode
24. "Find standards / RFCs" mode
25. Academic paper search (ArXiv, Semantic Scholar)
26. Code-aware search (detect code queries)
27. Real-time search for breaking news
28. Search history with replay
29. Saved queries with scheduling
30. HyDE hypothetical document embeddings
31. LLM-augmented query variants (Exp4Fuse-inspired)
32. Cascade retrieval (Matryoshka 64-dim → full-dim)
33. Late interaction retrieval (ColBERT/SPLATE)
34. Mix-of-Granularity dynamic chunk routing
35. Authority chain analysis (who cites whom)

### AI-Native Agent Features (36-65)

36. Query-Aware Token-Budgeted Extraction (QATBE)
37. Semantic Content Segmentation (SCS) — typed blocks
38. Progressive Detail Streaming (PDS) — 4 tiers
39. Pre-fetch token estimation
40. Query-Aware DOM Distillation (QADD)
41. Framework-adaptive output (LangChain, CrewAI, MCP)
42. MCP server with composite tools
43. REST API server mode
44. Library mode (npm import)
45. Streaming search results ranked by confidence
46. Tier expansion without re-fetching
47. Batch fetch with cross-source deduplication
48. Schema-based structured extraction
49. Agent CLI (`agent-search`, `agent-fetch`, `agent-research`)
50. LangChain `Retriever` adapter
51. CrewAI `Tool` adapter
52. AutoGPT-compatible tool interface
53. Pipe-friendly JSON output for shell pipelines
54. Context-window-aware output sizing
55. Sandwich layout for multi-result context assembly
56. MECW-aware context filling (never overfill)
57. Token accounting per operation
58. Cost-free operation guarantee (no hidden API calls)
59. Agent session management (multi-turn research)
60. Structured error responses with remediation guidance
61. Fallback chains (cache → alt source → Wayback → partial)
62. Content-type-aware extraction routing (PDF → parser, video → transcript)
63. Authenticated fetch support (cookies, tokens)
64. Agent-friendly progress events (for status tracking)
65. Composite operations (search+extract+structure in one call)

### Content Extraction (66-90)

66. Cascade Extraction Protocol (CEP) — 5-layer
67. ML-based extraction method prediction
68. SPA / JS-render detection heuristics
69. Readability article extraction
70. CSS selector targeting
71. XPath targeting
72. Table extraction → JSON arrays (not markdown)
73. Code block preservation with language detection
74. Image URL + alt text extraction
75. Link extraction with context
76. Metadata extraction (title, author, date, og:tags, JSON-LD)
77. PDF text extraction
78. RSS/Atom feed parsing
79. Infinite scroll handling
80. Multi-page article stitching
81. Wayback Machine fallback for dead links
82. Character encoding detection + normalization
83. Content language detection
84. Boilerplate removal (configurable aggressiveness)
85. Structured data extraction (JSON-LD, Microdata, RDFa)
86. Navigation structure extraction
87. Form field detection
88. Video transcript extraction (YouTube)
89. Content truncation detection
90. Source code extraction from GitHub/GitLab pages

### AI & Synthesis (91-110)

91. Local AI via Ollama HTTP API
92. Embedded LLM via node-llama-cpp (Metal/CUDA/Vulkan)
93. Custom OpenAI-compatible endpoint
94. Multi-model routing (small → large based on complexity)
95. Auto model selection based on VRAM
96. Streaming AI responses
97. JSON-schema constrained outputs
98. AI-powered query decomposition
99. AI contradiction analysis
100. AI executive summary generation
101. AI evidence table generation
102. Token budgeting for AI operations
103. AI response caching
104. Model health checks + failover
105. Customizable synthesis prompts
106. Multi-turn research conversations
107. AI query suggestions
108. AI topic clustering
109. AI bias detection in sources
110. Sandwich layout for context assembly (Ms-PoE)

### Validation & Self-Correction (111-130)

111. 6-layer validation pipeline
112. RAR self-correcting research loop
113. Self-RAG reflection tokens
114. CRAG retrieval quality evaluator
115. Source reachability validation
116. SSL certificate checking
117. Domain reputation scoring
118. Soft-404 detection
119. Content relevance scoring
120. Near-duplicate detection (SimHash)
121. Paywall detection
122. Published date extraction
123. Staleness detection
124. Cross-source claim consistency
125. Fact triangulation (3+ sources)
126. Extraction completeness checking
127. Citation verification (source contains cited claim)
128. Link validity checking
129. Format compliance checking
130. Confidence calibration per claim

### Evidence & Citation (131-145)

131. Evidence Graph Protocol (EGP)
132. Cryptographic content hashes (SHA-256)
133. Claim provenance tracking
134. 6 citation styles (inline, footnote, APA, IEEE, Chicago, BibTeX)
135. Strict evidence mode
136. Citation verification against source content
137. Contradiction reports with severity
138. Consensus meter
139. Evidence tables (claim → source → quote)
140. Annotated bibliography generation
141. Source age labeling
142. Claim propagation tracking
143. Source network analysis
144. Evidence chain building across sessions
145. Verifiable research artifacts

### Output & Format (146-165)

146. 12+ output formats (MD, JSON, YAML, CSV, HTML, TXT, BibTeX, PDF, DOCX, XML, JSONL, clipboard)
147. Multi-format simultaneous export
148. Semantic Content Segmentation (SCS) output
149. Progressive Detail Streaming (PDS) tiers
150. Structured JSON with full metadata
151. Agent-optimized segments format
152. Framework-adaptive formatting
153. Streaming output
154. Multi-format research bundles
155. Research notebook mode (append runs)
156. Export to clipboard
157. Pipe-friendly stdout (--quiet --json)
158. Custom output templates
159. Report customization (sections to include/exclude)
160. Evidence graph export (JSON)
161. Audit trail export
162. Comparison table generation
163. Timeline generation
164. Entity extraction output
165. Relationship graph output

### Performance & Resource (166-185)

166. Machine resource profiling (CPU/RAM/GPU/network/disk)
167. Adaptive parallelism (5 tiers)
168. Dynamic worker pool sizing
169. Per-domain rate limiting
170. Backpressure control
171. Graceful degradation
172. OOM prevention
173. Network bandwidth estimation
174. GPU/VRAM detection for AI
175. Connection pooling
176. Browser instance recycling
177. Cache warming for frequent sources
178. Prefetch hints for follow-up queries
179. Performance profiling (`--profile`)
180. Latency breakdown (per-stage)
181. Deterministic mode for benchmarks
182. Queue prioritization
183. Timeout management per source
184. Retry with exponential backoff
185. Circuit breaker for failing sources

### Research Workflow (186-210)

186. Research notebook mode
187. "What changed since last run" diffs
188. Session replay / audit trail
189. Snapshot hashing for integrity
190. Re-run reproducibility mode
191. Workspace profiles (engineering, academic, legal)
192. Domain allowlist/denylist
193. Saved queries with tags
194. Watch mode with alerts
195. Diff summaries for monitored pages
196. Changelog monitoring
197. GitHub release monitoring
198. Security advisory monitoring
199. Side-by-side source comparison
200. Research timeline construction
201. Entity extraction (people, orgs, products)
202. Relationship graph discovery
203. Automatic outline builder
204. Critical reading mode (gaps, biases)
205. Counterargument retrieval
206. "No hallucination" mode
207. Auto glossary of key terms
208. Topic map clustering
209. Research progress tracking
210. Collaborative research export

### System & Developer (211-230)

211. `hsx doctor` system check
212. Plugin system (npm-based)
213. Custom ranking rules DSL
214. Scoring explainability
215. YAML/JSON config files
216. Environment variable overrides
217. Per-project config (`.fetchium.yaml`)
218. Verbose/debug logging
219. Structured log output
220. Benchmark harness
221. Extraction regression tests
222. API server mode
223. MCP server mode
224. Shell completions (bash, zsh, fish)
225. Interactive TUI for deep research
226. Progress bars with ETA
227. Color-coded output with themes
228. Quiet mode for scripting
229. Reflexion memory (learn from past searches)
230. Self-evolving strategy selection

### Cross-Session Learning & Persistent Intelligence (231-245)

231. Personal Knowledge Graph (PKG) growing across sessions
232. Source Trust Memory (per-domain learned trust scores)
233. Failure Pattern Memory (never fail the same way twice)
234. Query Prediction Model (anticipate follow-up queries)
235. Predictive prefetching based on research patterns
236. Compounding intelligence (more usage = better results)
237. Exportable/importable intelligence profiles
238. Per-workspace intelligence isolation
239. Intelligence statistics dashboard
240. Research pattern analysis and visualization
241. Automatic source allowlist/denylist learning
242. Extraction method success tracking per domain
243. Anti-bot pattern memory per search engine
244. User preference learning (formats, depth, citation style)
245. Cross-project knowledge transfer

### Advanced Reasoning (246-260)

246. Tree-of-Thoughts parallel reasoning paths
247. Graph-of-Thoughts non-linear reasoning
248. Self-Debate Protocol (advocate vs critic vs judge)
249. Recursive query decomposition with branch pruning
250. Cross-path synthesis for multi-faceted questions
251. Contradiction Resolution Protocol (CRP) — auto-investigate
252. Evidence Decay Function (domain-calibrated half-lives)
253. Source Genealogy Tracker (trace claims to primary source)
254. Confidence Calibration Engine (historically calibrated)
255. Query Difficulty Estimation (predict time/cost before execution)
256. Multi-agent debate for controversial topics
257. Weighted synthesis with per-path confidence
258. Research completeness scoring
259. Assumption surfacing in deep research
260. Counterfactual analysis mode

### Proactive Intelligence (261-270)

261. Topic subscription with alert thresholds
262. Research radar (suggestions from history)
263. Intelligent weekly/daily digests
264. Anomaly detection on monitored sources
265. Predictive prefetching of likely follow-up results
266. "What you missed" summaries for subscribed topics
267. Trending topic detection in your domains
268. Automatic re-verification of stale research
269. Competitive intelligence monitoring
270. Change impact analysis for monitored sources

### Multimodal Content (271-280)

271. Image content understanding via local vision model
272. Video transcript extraction (YouTube API + Whisper)
273. Audio transcription via whisper-rs
274. Chart/graph interpretation → structured data
275. Screenshot OCR via Tesseract or vision model
276. Diagram → text description conversion
277. Table image → structured JSON extraction
278. Multimodal search (include images/videos in results)
279. Infographic data extraction
280. PDF layout-aware extraction with visual understanding

### Adversarial Robustness & Trust (281-290)

281. AI-generated content detection (perplexity, burstiness)
282. Bot farm / coordinated inauthentic behavior detection
283. Source manipulation detection via Wayback diff
284. SEO spam detection and filtering
285. Content watermark detection (C2PA)
286. Aggregate trust scoring per source
287. Human-sources-only mode
288. Stylometric fingerprinting for author verification
289. Cross-source verbatim overlap detection
290. Temporal anomaly detection (sudden content changes)

### Privacy & Security (291-300)

291. Private mode (zero persistence)
292. Tor routing for anonymous research
293. Air-gap mode (local index only)
294. Self-destructing research (auto-expire)
295. Encrypted cache at rest
296. GPG-encrypted research artifacts
297. Split queries across engines (no single engine sees full query)
298. PII redaction in all outputs
299. Differential privacy for community feedback
300. Zero-knowledge search architecture

### Collaboration (301-310)

301. Shared research workspaces
302. Real-time collaborative annotation
303. Research session forking
304. Evidence graph merging
305. Team knowledge graph
306. Conflict detection across researchers
307. Citation network visualization
308. Git-based research sync
309. Role-based access control for workspaces
310. Collaborative evidence curation

### Domain-Specific Intelligence (311-318)

311. Academic mode (Scholar, ArXiv, citation analysis)
312. Code intelligence mode (GitHub, StackOverflow, docs)
313. Legal research mode (case law, precedent chains)
314. Financial analysis mode (SEC, earnings, market data)
315. Medical/scientific mode (PubMed, evidence grading)
316. Cybersecurity mode (CVE, NVD, advisories, CVSS)
317. Patent search mode (USPTO, EPO)
318. Custom domain mode (user-defined backend + ranking)

### Self-Evolving Architecture (319-328)

319. AutoML ranking weight optimization
320. Extraction method auto-tuning
321. Confidence calibration engine
322. A/B testing framework for algorithms
323. Community feedback loop (opt-in, differential privacy)
324. Self-healing extraction (auto-adapt to site changes)
325. Performance regression detection
326. Query success rate tracking
327. Automatic benchmark on ranking changes
328. Evolutionary algorithm selection

### Experimental & Future (329-350)

329. RAPTOR tree-organized retrieval for deep synthesis
330. GraphRAG knowledge graph construction
331. Focused ReAct research loop
332. Multi-agent research swarm (AMRS)
333. Graph-of-Thoughts research planning
334. Cross-lingual information retrieval
335. Time-travel search (Wayback Machine)
336. Content evolution tracking
337. Semantic dedup across sessions
338. Source network analysis
339. Claim propagation tracking
340. Research quality scoring
341. Automatic literature review generation
342. Speculative research pipelining (SRP)
343. RAGCache KV tensor caching
344. Embedding cache for frequent queries
345. Observation masking for agent contexts
346. D2Snap DOM downsampling
347. Edge-cloud hybrid computation
348. Peer-to-peer research network (future)
349. Federated knowledge graph (cross-user, privacy-preserving)
350. Neuro-symbolic reasoning integration

---

## 43. Data Model

### Core Entities

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSearchResult {
    pub meta: SearchMeta,
    pub segments: Vec<Segment>,       // SCS typed segments
    pub findings: Vec<Finding>,
    pub evidence: Vec<EvidenceLink>,
    pub contradictions: Vec<Contradiction>,
    pub sources: Vec<Source>,
    pub evidence_graph: Option<EvidenceGraph>,
    pub audit_trail: Vec<AuditEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMeta {
    pub query: String,
    pub mode: String,
    pub tier: String,
    pub tokens_used: u32,
    pub tokens_budget: u32,
    pub sources_fetched: u32,
    pub sources_validated: u32,
    pub validation_pass_rate: f64,
    pub duration_ms: u64,
    pub resource_tier: String,
    pub timestamp: String,
    pub result_id: String,            // for PDS tier expansion
    pub content_hashes: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SegmentType {
    Heading, Paragraph, Fact, Opinion, Table, Code, List,
    Quote, Data, Link, Image, Definition, DateEvent, Entity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    pub seg_type: SegmentType,
    pub relevance: f64,
    pub tokens: u32,
    pub content: serde_json::Value,   // type-specific structure
    pub source_ref: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EvidenceType { Supports, Contradicts, PartiallySupports }

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceLink {
    pub claim: String,
    pub source_id: u32,
    pub quote: String,
    pub quote_hash: String,           // SHA-256
    pub confidence: f64,
    pub evidence_type: EvidenceType,
}
```

---

## 44. Error Handling & Fallback Chains

### Structured Error Taxonomy

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorType {
    NetworkTimeout, DnsFailure, Http403, Http429, Http5xx,
    AntiBot, Paywall, ContentNotFound, ExtractionFailed,
    BrowserCrash, AiUnavailable, ValidationFailed, BudgetExceeded,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructuredError {
    pub error_type: ErrorType,
    pub retryable: bool,
    pub message: String,
    pub source_url: Option<String>,
    pub suggested_action: String,
    pub alternatives: Vec<String>,    // fallback URLs or strategies
}
```

### Automatic Fallback Chain

```
Primary fetch fails →
  1. Check cache (serve stale if available) →
  2. Try alternative backend (DDG failed? Try SearXNG) →
  3. Try Wayback Machine snapshot →
  4. Return partial results with clear explanation
```

### Principles

1. **Never crash** — always degrade gracefully
2. **Never hang** — every operation has a timeout
3. **Never lose data** — partial results > no results
4. **Always explain** — structured errors with suggested actions
5. **Always fallback** — automated fallback chain before giving up

---

## 45. Testing Strategy

| Level | Framework | Coverage |
|-------|-----------|----------|
| Unit | `cargo test` | Core: ranker, chunker, QATBE, SCS, CEP |
| Integration | `cargo test` + `tokio::test` | Pipeline: fetch → extract → rank → output |
| E2E | `assert_cmd` + `predicates` + mock servers | Full CLI + agent commands |
| Benchmark | `criterion` | Latency, throughput, token efficiency |
| Extraction | `insta` (snapshot tests) | Known pages → expected output |
| Agent | mock LLM (wiremock-rs) | Framework adapter correctness |
| MCP | MCP test client | Tool schema + response format |
| Fuzz | `cargo-fuzz` / `libfuzzer` | HTML parsing, URL handling, JSON deserialization |
| Concurrency | `loom` | Lock-free data structure correctness |

---

## 46. Milestones & Roadmap

### MVP (Weeks 1-6)

- [ ] Project scaffolding (Cargo workspace: `hsx-core`, `hsx-cli`, `hsx-mcp`, `hsx-api`)
- [ ] `hsx search` with DuckDuckGo backend (reqwest)
- [ ] `hsx fetch` / `hsx view` with HTTP + scraper/lol_html (CEP Layer 1-2)
- [ ] Basic QATBE (query + budget for fetch)
- [ ] Basic SCS (paragraphs, tables, code blocks)
- [ ] PDS tier 0-1 (key_facts, summary)
- [ ] BM25 ranking (tantivy)
- [ ] Memory LRU cache (moka)
- [ ] Resource profiling (sysinfo crate)
- [ ] `hsx doctor`
- [ ] npm/pnpm/bun packaging (platform-specific pre-built binaries)
- [ ] `agent-search` and `agent-fetch` commands
- [ ] JSON segments output format (serde_json / simd-json)
- [ ] Pre-fetch token estimation
- [ ] CI/CD with cross-compilation (Linux x64/arm64, macOS x64/arm64, Windows x64)

### V1.0 (Weeks 7-14)

- [ ] Google + Bing + Scholar via headless Chromium (chromiumoxide)
- [ ] SearXNG + Wikipedia backends
- [ ] Multi-backend orchestrator + HyperFusion ranking
- [ ] Full CEP (5 layers with ML predictor via ort/candle)
- [ ] Full QATBE with QADD
- [ ] Full SCS (all 14 segment types)
- [ ] PDS all 4 tiers
- [ ] `hsx research` with evidence mapping
- [ ] Validation layers V1-V3
- [ ] `hsx ai` with Ollama (HTTP API) + llama-cpp-rs
- [ ] Semantic search (ONNX embeddings via ort)
- [ ] Citation system (6 styles)
- [ ] SQLite disk cache (rusqlite)
- [ ] MCP server mode (rmcp)
- [ ] LangChain adapter
- [ ] REST API server (axum)
- [ ] Shell completions (clap_complete)
- [ ] Stealth mode for headless (fingerprint randomization, resource blocking)

### V1.5 (Weeks 15-22)

- [ ] `hsx deep` with AMRS (tokio task spawning + channels)
- [ ] RAR self-correction loop
- [ ] EGP evidence graphs
- [ ] SRP speculative pipelining
- [ ] Cross-source validation (V4-V6)
- [ ] HyDE query expansion
- [ ] Cascade retrieval (Matryoshka embeddings)
- [ ] `hsx compare` and `hsx monitor`
- [ ] Plugin system (dynamic loading via libloading or WASM via wasmtime)
- [ ] CrewAI adapter
- [ ] Strict evidence mode
- [ ] Local vector index with late chunking (hnswlib-rs)
- [ ] PDF/DOCX export (pdf-extract + Pandoc)
- [ ] Reflexion memory
- [ ] Persistent Intelligence Engine (PIE) — cross-session knowledge graph (rusqlite)
- [ ] Tree-of-Thoughts Research (ToTR) — parallel reasoning paths

### V2.0 (Months 6-9)

- [ ] RAPTOR/GraphRAG for deep synthesis
- [ ] Interactive TUI (ratatui)
- [ ] Plugin marketplace
- [ ] Cross-lingual search
- [ ] Time-travel search (Wayback Machine integration)
- [ ] Community plugins
- [ ] Benchmark suite (criterion + custom harness)
- [ ] Documentation site
- [ ] Adversarial Content Shield (ACS) — AI content detection, bot farm signals
- [ ] Contradiction Resolution Protocol (CRP) — automated source disagreement investigation
- [ ] Evidence Decay Function (EDF) — domain-calibrated temporal reliability
- [ ] Source Genealogy Tracker (SGT) — claim provenance tracing
- [ ] Confidence Calibration Engine (CCE) — historically calibrated confidence scores
- [ ] Proactive Intelligence Engine — anticipatory search + query prediction
- [ ] Collaborative Research Protocol — multi-user research sessions
- [ ] Domain-Specific Intelligence Modes (legal, medical, academic, financial)
- [ ] Self-Evolving Architecture — automatic algorithm optimization
- [ ] Multimodal content understanding (image alt-text, chart extraction, audio transcription)

---

## 47. Success Metrics

| Metric | Target |
|--------|--------|
| Search latency (cached) | <1s |
| Search latency (uncached) | <3s |
| Token efficiency vs raw HTML | >97% reduction |
| Token efficiency vs flat markdown | >60% reduction |
| QATBE relevance coverage | >85% of relevant content captured |
| Citation accuracy | >95% verification pass rate |
| CEP method prediction accuracy | >90% |
| RAR self-correction rate | >80% of bad retrievals caught |
| Fetch success (non-protected) | >99% |
| npm weekly downloads (6 months) | >10,000 |
| GitHub stars (6 months) | >5,000 |
| MCP adoption (12 months) | Top 10 MCP tool by usage |

---

## 48. Technical Dependencies

### Core Rust Crates (Required)

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `reqwest` | HTTP client with connection pooling |
| `hyper` | Low-level HTTP (server mode) |
| `scraper` | CSS selector-based HTML parsing |
| `lol_html` | Streaming HTML rewriter (Cloudflare) |
| `chromiumoxide` | CDP-based headless Chromium (Google/Bing search) |
| `clap` | CLI argument parsing |
| `indicatif` | Progress bars and spinners |
| `console` | Terminal colors and formatting |
| `rusqlite` | SQLite bindings (cache, index) |
| `tantivy` | Full-text search engine (BM25) |
| `serde` + `serde_json` | Serialization |
| `simd-json` | SIMD-accelerated JSON parsing |
| `config` | Multi-format configuration |
| `tracing` | Structured logging |

### Optional Crates (Feature-gated)

| Crate | Feature Flag | Purpose |
|-------|-------------|---------|
| `ort` | `embeddings` | ONNX Runtime for local embeddings |
| `candle-core` | `candle` | Pure Rust ML inference |
| `hnswlib-rs` | `vector-search` | HNSW vector index |
| `llama-cpp-rs` | `llama` | Embedded GGUF model inference |
| `pdf-extract` | `pdf` | PDF text extraction |
| `rmcp` | `mcp` | MCP server protocol |

### External (System)

| Tool | Required | Purpose |
|------|----------|---------|
| Chromium / Chrome | Recommended | Headless search (Google, Bing, Scholar) — auto-detected |
| Ollama | Optional | Local AI model server |
| Pandoc | Optional | PDF/DOCX export |

### npm Wrapper Package

The npm package is a thin wrapper that downloads the correct platform binary:

```json
{
  "name": "fetchium",
  "version": "1.0.0",
  "bin": { "hsx": "bin/hsx", "hyper": "bin/hyper" },
  "scripts": { "postinstall": "node scripts/install-binary.js" },
  "optionalDependencies": {
    "@fetchium/linux-x64": "1.0.0",
    "@fetchium/linux-arm64": "1.0.0",
    "@fetchium/darwin-x64": "1.0.0",
    "@fetchium/darwin-arm64": "1.0.0",
    "@fetchium/win-x64": "1.0.0"
  }
}
```

This pattern (used by esbuild, turbo, SWC) ensures:
- `npm install -g fetchium` works seamlessly
- `pnpm add -g fetchium` works seamlessly
- `bun add -g fetchium` works seamlessly
- No Rust toolchain needed for end users
- Pre-built native binary for each platform

---

## 49. Appendix: Research Papers & References

### Integrated Research

| Paper | Venue | Integration |
|-------|-------|-------------|
| Self-RAG | ICLR 2024 (Oral) | RAR reflection tokens |
| CRAG | ICLR 2025 | Retrieval quality evaluator |
| RAPTOR | ICLR 2024 | Tree-organized deep synthesis |
| GraphRAG | Microsoft 2024 | Knowledge graph in deep mode |
| ReaderLM-v2 | Jina 2025 | Optional ML HTML→MD |
| D2Snap | 2025 | QADD DOM downsampling |
| Matryoshka RL | NeurIPS 2022 | Cascade retrieval |
| Late Chunking | Jina 2024 | Local index embeddings |
| Ms-PoE | NeurIPS 2024 | Sandwich context layout |
| Context Rot | Chroma 2025 | MECW-aware filling |
| Focused ReAct | 2024 | Research loop saliency |
| Reflexion | NeurIPS 2023 | Episodic memory |
| HyDE | 2023 | Hypothetical embeddings |
| SPLATE | SIGIR 2024 | Sparse late interaction |
| Mix-of-Granularity | 2024 | Dynamic chunk routing |
| BGE-reranker | BAAI 2024 | Cross-encoder reranking |
| Exp4Fuse | 2025 | LLM-augmented RRF |
| RAGCache | 2025 | KV tensor caching |
| WebAgent-R1 | EMNLP 2025 | Agent RL training |
| Semantic Compression | 2025 | SrCr metric |

### Competitive Tools Analyzed

Tavily, Exa, Perplexity, Serper, Brave, googler, SearXNG, Crawl4AI, Firecrawl, Jina Reader, ReaderLM-v2, Browser-Use, Stagehand, MultiOn, Perplexica, DuckDuckGo-search, Websurfx, MarkItDown, Markdowner, Trafilatura, ScrapeGraphAI, Skyvern, Cloudflare Markdown for Agents, Google WebMCP.

---

*Fetchium — AI-native. Agent-first. Human-friendly. The fastest path from question to knowledge. Free. Open-source. Yours.*
