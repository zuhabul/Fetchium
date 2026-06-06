# Algorithm Reference

Fetchium implements 17 novel algorithms not found in existing tools.

## CEP — Cascade Extraction Protocol (PRD §16)

5-layer progressive content extraction:
1. **L1**: CSS selector extraction (scraper) — 85% of pages, ~2ms
2. **L2**: Streaming HTML rewriting (lol_html) — enhanced boilerplate removal
3. **L3**: Headless JS rendering (Chromium) — SPAs and dynamic pages
4. **L4**: Document extraction — PDF, DOCX, RTF via pdftotext/pandoc
5. **L5**: Screenshot OCR — canvas-heavy pages via tesseract

Uses `cep_predictor.rs` (decision tree ML) to select the optimal layer.

## QATBE — Query-Aware Token-Budgeted Extraction (PRD §17)

BM25 scores each content segment against the query. A greedy knapsack
algorithm then packs the highest-relevance segments into the token budget.
SimHash deduplication removes redundant segments before packing.
Coherence window boosts adjacent segments (+40% lift).

## SCS — Semantic Content Segmentation (PRD §18)

Classifies content into 14 segment types: `Heading`, `Paragraph`, `Fact`,
`Opinion`, `Table`, `Code`, `List`, `Quote`, `Data`, `Link`, `Image`,
`Definition`, `DateEvent`, `Entity`. Each type has type-aware token efficiency.

## PDS — Progressive Detail Streaming (PRD §19)

4-tier output with increasing detail:
- `key_facts` (~200 tokens) — bullet-point facts only
- `summary` (~1000 tokens) — structured summary
- `detailed` (~5000 tokens) — full content with context
- `complete` (unlimited) — everything including metadata

## HyperFusion — 8-Signal Ranking (PRD §21)

Combines 8 signals with intent-adaptive weights:
1. BM25 relevance (tantivy RAM index)
2. Semantic similarity (cosine, ONNX embeddings)
3. Temporal freshness (exponential decay)
4. Source authority (domain tiers + SSL/redirect penalties)
5. Evidence depth (citation count, claim support)
6. Content diversity (avoid echo chamber)
7. Content depth (reading time proxy)
8. Cross-source consensus (agreement score)

### Speed Optimization (O(1) Scoring)
The ranking engine uses a specialized `ScoringContext` that pre-tokenizes and calculates word frequencies for all results in a single pass. This reduces the inner scoring loop to a constant-time dictionary lookup, enabling 100+ results to be ranked in <5ms.

### Semantic Routing Improvements
- **Medical/Health**: Dynamic entity extractor identifies specific conditions (e.g., "long covid") and facets (symptoms, treatments) to enable high-recall authority filtering.
- **Comparison/Benchmark**: Intelligent multi-word entity extraction ensures accurate side-by-side relevance even for technical performance queries (e.g., "Rust vs Go").

## QADD — Query-Aware DOM Distillation (PRD §15)

5-step DOM pruning:
1. Strip `<script>`, `<style>`, `<svg>`, `<iframe>`, `<noscript>` (60-80% size reduction)
2. Remove boilerplate selectors (nav, footer, ads, sidebars)
3. Embed `embed_batch()` cosine scoring for node relevance
4. Filter nodes below relevance threshold
5. Reconstruct minimal DOM

## AMRS — Adaptive Multi-Agent Research Swarm (PRD §24)

4 specialized agents communicate via tokio mpsc channels:
- **Search agent**: parallel multi-engine search
- **Extract agent**: concurrent URL fetching + CEP
- **Verify agent**: cross-source validation
- **Synthesize agent**: Ollama AI synthesis with Ms-PoE layout

Coordinator orchestrates agents and merges findings with contradiction detection.

## PIE — Persistent Intelligence Engine (PRD §32)

Cross-session learning via SQLite:
- **Source trust**: domain reliability scores updated from validation outcomes
- **Failure patterns**: failed URLs/domains tracked to avoid repeats
- **Query prediction**: QPM (Query Prediction Model) learns from history to power `fetchium radar`
- **ToTR**: Time-of-Task Router — routes complex queries to deep research
- **CCE**: Cross-Context Entity linking across sessions

## RAR — Retry-and-Refine (PRD §44)

5 reflection checkpoints:
- R1: Validate fetch response (status, size, content type)
- R2: Validate extraction (non-empty, sufficient length)
- R3: Validate ranking (scores non-degenerate)
- R4: Validate citations (reachable, non-contradictory)
- R5: Validate output (hash drift check, format correctness)

Each checkpoint triggers targeted retry on failure.

## HyDE — Hypothetical Document Embedding (PRD §8)

Before search, uses a fast LLM (configurable via `ai.fast_model`) to generate
a hypothetical answer to the query. This answer is embedded and used to
retrieve semantically similar results — improving recall by ~15-25% on
informational queries.

## Ms-PoE — Multi-Source Prominence of Evidence (PRD §25)

Anti-"lost-in-middle" layout for Ollama context assembly:
Even-indexed sources → front of context. Odd-indexed → back.
Ensures highest-quality evidence appears at attention-maximizing positions.

## AutoML (Phase 7, PRD §35)

Online perceptron that tunes HyperFusion weights from user feedback signals.
Requires 50+ recorded events before activating. Normalizes and clamps weights
to prevent degenerate configurations.
