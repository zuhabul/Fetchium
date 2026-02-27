# Phase 3 — Knowledge OS

**Timeline:** Months 7–12
**Theme:** Transform Fetchium from a search tool into a thinking partner that gets smarter with every session.
**Team:** Solo founder → first full-time engineer hired at M9 (if MRR > $25K)

---

## The Shift

Phases 1 and 2 made Fetchium the best way to search and extract information. Phase 3 makes Fetchium indispensable by giving it *memory*.

Every search you do, every research report you generate, every source you trust — Fetchium learns from it. Over time, your Fetchium instance becomes a personalized research assistant that knows your domain, your preferences, and your past work.

This is the "aha moment" that drives 55%+ 30-day retention.

**Competitive moat:** Knowledge OS data cannot be migrated. Once a user has 6 months of search history and a personal KB with 500 nodes, they do not switch to Perplexity.

---

## Success Criteria

| Metric | Target |
|--------|--------|
| MRR | $80,000 |
| ARR | $960,000 |
| KB sessions stored | 1,000,000+ |
| Nodes in all KBs | 10,000,000+ |
| DAU/MAU ratio | > 35% |
| 30-day retention | > 55% |
| SDK downloads/week | 5,000+ |
| Pro tier conversion | > 20% of free users |
| NPS | > 60 |

---

## Core Systems

### 1. PIE — Persistent Intelligence Engine

Cross-session learning via SQLite. Every interaction trains your personal model.

**What PIE learns:**
- Source trust scores (which domains produce accurate, relevant results for you)
- Query patterns (frequently searched topics → prefetch suggestions)
- Failure patterns (queries that returned poor results → automatic reformulation)
- Content preferences (prefers long-form vs summaries, academic vs practitioner)
- Time patterns (active search hours, topic frequency by day)

**PIE data model:**
```sql
CREATE TABLE source_trust (
  domain TEXT PRIMARY KEY,
  user_id TEXT,
  trust_score REAL DEFAULT 0.5,
  positive_signals INTEGER DEFAULT 0,
  negative_signals INTEGER DEFAULT 0,
  last_updated INTEGER
);

CREATE TABLE query_history (
  id INTEGER PRIMARY KEY,
  user_id TEXT,
  query TEXT,
  intent TEXT,
  result_count INTEGER,
  clicked_urls TEXT,  -- JSON array
  session_id TEXT,
  created_at INTEGER
);

CREATE TABLE topic_clusters (
  id INTEGER PRIMARY KEY,
  user_id TEXT,
  cluster_name TEXT,
  query_ids TEXT,  -- JSON array
  centroid_embedding BLOB  -- Phase 5: vector
);
```

**PIE effects visible to user:**
- Trusted domains ranked higher in results
- "You frequently search [topic] — start a knowledge thread?" prompts
- Pre-filled query suggestions based on history
- Research reports that reference your past findings

### 2. Personal Knowledge Base (PKB)

A searchable, linkable store of everything you've ever researched with Fetchium.

**Concepts:**
- **Node** — a saved search result, research report, or manual note
- **Thread** — a named collection of nodes around a topic
- **Link** — a relationship between nodes (supports, contradicts, elaborates)
- **Tag** — free-form labels on nodes

```bash
# Save to KB
fetchium search "transformer architecture" --save "AI architectures"

# Research with auto-save
fetchium research "quantization effects on LLM" --save-to "LLM optimization"

# Query KB
fetchium know "what did I find about quantization?"
fetchium know "summarize my AI architectures thread"

# Browse KB
fetchium kb list
fetchium kb thread "AI architectures" --show-links
fetchium kb export --format obsidian  # export as Markdown vault
```

**API endpoints:**
```
POST   /v1/kb/nodes          → create node
GET    /v1/kb/nodes/{id}     → get node
DELETE /v1/kb/nodes/{id}     → delete node
POST   /v1/kb/query          → semantic search in KB
GET    /v1/kb/threads        → list threads
POST   /v1/kb/threads        → create thread
GET    /v1/kb/export         → export full KB (JSON/Markdown)
```

**Storage:** SQLite per user (Phase 3). Migrate to PostgreSQL at > 10K users.

### 3. Learn Mode

Generate a structured learning curriculum on any topic.

```bash
fetchium learn "distributed systems" --level intermediate --duration "4 weeks"
fetchium learn "Rust async programming" --level beginner --duration "2 weeks"
```

**Output structure:**
```
Week 1: Foundations
  Day 1: What are distributed systems?
    → Resources: [3 links with summaries]
    → Exercise: Read CAP theorem paper (20 min)
  Day 2: Consistency models
    → Resources: [4 links]
    → Exercise: Compare eventual vs strong consistency (15 min)
  ...

Week 2: Coordination
  ...

Progress tracking:
  - Mark topics complete
  - Spaced repetition reminders (via webhook / email)
  - Quiz generation (AI-generated questions from content)
```

### 4. Monitor Mode

Track topics and receive alerts when new content appears.

```bash
fetchium monitor add "Claude model releases" --frequency daily
fetchium monitor add "Rust async ecosystem" --frequency weekly
fetchium monitor list
fetchium monitor pause "Rust async ecosystem"
```

**Delivery channels:**
- Email digest (daily / weekly)
- Webhook (POST to your URL)
- Dashboard notification
- CLI: `fetchium monitor inbox` shows pending alerts

**Alert format:**
```json
{
  "monitor": "Claude model releases",
  "period": "2025-11-01 to 2025-11-07",
  "new_items": 3,
  "summary": "AI summary of new content",
  "items": [
    { "title": "...", "url": "...", "published": "...", "relevance": 0.94 }
  ]
}
```

### 5. Team Spaces (Beta)

Shared knowledge bases for small teams. Full launch in Phase 5.

```bash
fetchium team create "AI Research Team"
fetchium team invite user@company.com
fetchium team kb query "what did we find about LLM fine-tuning?"
```

**Phase 3 scope (beta only):**
- Up to 5 members per space
- Shared KB with read/write for all members
- No RBAC (Phase 5)
- No SSO (Phase 5)
- Billing: flat $200/mo for beta team spaces

---

## SDK Suite — Phase 3 Launch

### Python SDK v1.0

```python
pip install fetchium

from fetchium import Fetchium

client = Fetchium(api_key="hsx_...")

# Search
results = client.search("rust async runtimes 2025", limit=10)

# AI answer
answer = client.ai("what is retrieval augmented generation?")
print(answer.text)
print(answer.citations)

# Research (async)
job = client.research("impact of quantization on LLM accuracy", depth="deep")
report = job.wait()  # polls until complete

# Knowledge base
client.kb.save(results[0], thread="AI research")
kb_results = client.kb.query("what did I find about quantization?")

# Streaming
for token in client.ai("explain transformers", stream=True):
    print(token, end="", flush=True)
```

**PyPI package:** `fetchium`
**Python support:** 3.10+
**License:** Apache 2.0

### JavaScript/TypeScript SDK v1.0

```typescript
npm install fetchium

import { Fetchium } from 'fetchium';

const client = new Fetchium({ apiKey: 'hsx_...' });

// Search
const results = await client.search('rust async runtimes 2025');

// AI answer with streaming
const stream = await client.ai('explain transformers', { stream: true });
for await (const chunk of stream) {
  process.stdout.write(chunk.content);
}

// Research
const job = await client.research('LLM quantization effects', { depth: 'deep' });
const report = await job.wait();

// Knowledge base
await client.kb.save(results[0], { thread: 'AI research' });
```

**npm package:** `fetchium`
**Node.js support:** 18+
**Deno support:** via https://deno.land/x/fetchium
**Browser:** bundled (10KB gzip, no Node.js dependencies)

---

## Infrastructure Upgrades

### Database Evolution
```
Phase 1: SQLite (auth.db, single file)
Phase 3: SQLite per user (kb/{user_id}.db) + shared auth.db
Phase 4: PostgreSQL migration (> 10K users threshold)
```

### Vector Embeddings (Phase 3 Preview)
- Enable `embeddings` feature flag in hsx-core
- ONNX Runtime (ort crate) for local embedding generation
- Model: `all-MiniLM-L6-v2` (22MB, runs on CPU, 384-dim vectors)
- Used for: KB semantic search, topic clustering, "Find related" feature
- Storage: vectors in SQLite as BLOB (< 10K users), usearch at scale

### Caching
- KB query results: 5-min TTL (user-specific)
- Monitor feed results: 1h TTL
- Redis not yet required; in-process DashMap cache sufficient at Phase 3 scale

---

## Week-by-Week Timeline

### Month 7
- W25: PIE source trust scoring active (passive, no UI yet)
- W26: Query history stored per user
- W27: Personal KB CRUD (POST/GET/DELETE nodes)
- W28: KB thread management

### Month 8
- W29: `fetchium know` — semantic KB search (BM25 Phase 3, vectors Phase 5)
- W30: KB export (JSON + Obsidian Markdown vault)
- W31: Monitor mode — tracking + email digest
- W32: PIE suggestions surfaced in CLI ("you've searched X before")

### Month 9
- W33: Learn mode — curriculum generation
- W34: Learn mode — progress tracking + spaced repetition reminders
- W35: Team spaces beta (5 users, flat billing)
- W36: First engineer hired (if MRR > $25K)

### Month 10
- W37: Python SDK v1.0 (all endpoints)
- W38: Python SDK docs + cookbook
- W39: SDK integration tests in CI
- W40: PyPI publish + announcement (HN, Dev.to, Twitter/X)

### Month 11
- W41: JavaScript/TypeScript SDK v1.0
- W42: SDK browser bundle
- W43: JS SDK docs + framework examples (Next.js, Express)
- W44: npm publish + announcement

### Month 12
- W45: API v2 design finalized (no breaking changes yet)
- W46: Performance audit: p95 < 3s all endpoints
- W47: $80K MRR milestone review
- W48: Series A materials drafted

---

## Pricing Updates (Phase 3)

| Tier | Price | Includes |
|------|-------|----------|
| Free | $0 | 100 searches/day, 5 KB nodes, 1 monitor |
| Pro | $49/mo | 5K searches/day, unlimited KB, 10 monitors, email digest |
| Pro+ | $99/mo | 20K searches/day, unlimited KB, 50 monitors, webhook delivery |
| Team | $200/mo | 5 seats, shared KB, team monitors |
| Enterprise | Custom | Unlimited, SSO, SLA, on-prem |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| KB search quality poor without vectors | BM25 as baseline (good for Phase 3), vectors in Phase 5 |
| SDK adoption slow | Write cookbooks for top 3 use cases (LangChain, n8n, Zapier) before launch |
| SQLite per-user doesn't scale | Monitor DB file size; migrate at > 100MB per user or > 10K users |
| First engineer hire fails | Start with 3-month contractor, evaluate before full-time |
| Monitor false positives | Similarity threshold tuning; user feedback thumbs up/down per alert |
