# Sprint 03: Product Polish

**Duration:** 2 weeks
**Theme:** UX improvements, onboarding, and documentation
**Goal:** A new developer can install Fetchium and successfully complete their first meaningful fetch in under 5 minutes
**Dependency:** Sprint 02 complete (rebrand done, CI green)

---

## Context

The technical foundation is solid. The product works. But "working" and "delightful"
are different things. This sprint closes the gap between a project that developers can
use and one they want to use and recommend to others.

**The onboarding funnel problem today:**
1. Developer finds Fetchium on GitHub
2. Reads README → unclear what "7 fetch modes" means in practice
3. Installs the binary
4. Runs `fetchium --help` → wall of text
5. Tries `fetchium fetch "https://example.com"` → JSON output but unclear what to do next
6. No tutorial → developer gives up

**After this sprint:** The developer sees a clear quickstart, runs `fetchium quickstart`,
gets a working result in 90 seconds, and understands how to use the tool in their agent.

---

## Week 1: CLI UX & Onboarding

### Day 1–2: `fetchium quickstart` Command

**Task 3.1 — Implement the quickstart command**

This is the most important task of the sprint. `fetchium quickstart` is an interactive
walkthrough that activates new users.

**Flow:**
```
$ fetchium quickstart

Welcome to Fetchium! Let's fetch something together.

Step 1/3: Fast fetch (get page content in ~2 seconds)
  Fetching https://news.ycombinator.com...
  ✓ Got 142 items, 1,847 tokens of content (1.2s)

Step 2/3: AI-enriched fetch (summarize + extract key info)
  Fetching + analyzing https://news.ycombinator.com/item?id=...
  ✓ Summary: [1-2 sentence summary of the HN post] (6.3s)

Step 3/3: Use Fetchium in your code
  Python:  pip install fetchium-py && fetchium-py quickstart
  Rust:    cargo add fetchium && see docs.fetchium.com/sdk/rust
  API:     curl -H "Authorization: Bearer YOUR_KEY" \
                ***REMOVED***/fetch?url=...

You're all set! Run `fetchium --help` to explore all 7 modes.
Docs: https://docs.fetchium.com
Discord: https://discord.gg/fetchium
```

**Implementation in `fetchium-cli/src/commands/quickstart.rs`:**
- Steps are sequential with real fetches to real URLs
- If no AI keys configured, Step 2 shows "AI not configured — run `fetchium setup` to enable"
- Timing shown for each step
- Final step shows the relevant code snippet for the user's ecosystem

**Task 3.2 — Add quickstart to `--help` prominently**

Update the CLI help output so `fetchium quickstart` is the first suggested command
for new users.

### Day 2–3: Output Formatting Improvements

**Task 3.3 — Human-friendly default output**

Currently: raw JSON dump
Target: formatted, readable output with color

```bash
$ fetchium fetch "https://example.com"

# Before (current)
{"url":"https://example.com","title":"Example Domain","content":"...","tokens":42}

# After
Title: Example Domain
URL:   https://example.com
Mode:  fast (1.3s)
Tokens: 42

Content:
  This domain is for use in illustrative examples in documents.
  You may use this domain in literature without prior coordination...

Run with --json for raw JSON output.
```

**Implementation:**
- Default output: human-friendly formatted text (using `comfy-table` or manual formatting)
- `--json` flag: raw JSON (existing behavior, now opt-in)
- `--quiet` flag: content only, no metadata headers
- Color: use `colored` crate; respect `NO_COLOR` env var (https://no-color.org)

**Task 3.4 — Progress indicators for slow operations**

Deep fetch and research mode can take 10–30 seconds. Without feedback, users ctrl-C.

```bash
$ fetchium ai "latest developments in AI reasoning"
Searching the web...          ████░░░░░░  (searching)
Fetching top 3 results...     ████████░░  (fetching)
Analyzing with Gemini...      ██████████  ✓ done (8.2s)
```

Use the `indicatif` crate for progress bars. Disable when stdout is not a TTY
(for piping: `fetchium ai "..." | jq .`).

**Task 3.5 — Error messages are actionable**

Current error messages are Rust error chains. Target: user-friendly with next steps.

```
# Before
Error: reqwest error: error sending request for url (https://...): error trying to connect: ...

# After
Error: Could not reach https://example.com

  Possible causes:
  • The URL is unreachable from this network
  • The site blocks automated requests (try --mode headless)
  • Your internet connection may be down

  Run `fetchium doctor` to check your setup.
```

### Day 4–5: `fetchium doctor` Improvements

**Task 3.6 — Comprehensive health check output**

Current `doctor` command is basic. Expand to check and clearly report:

```
$ fetchium doctor

Fetchium v0.2.0 — Environment Check
────────────────────────────────────
✓ CLI binary:        fetchium 0.2.0 (/home/user/.cargo/bin/fetchium)
✓ Config file:       ~/.fetchium/config.toml (found)
✓ Data directory:    ~/.fetchium/ (writable)

Search Backends:
✓ SearXNG:           ***REMOVED*** (12ms response)
✗ Bing API:          Not configured (retired Aug 2025 — see docs)
✓ DuckDuckGo:        Available (fallback)

AI Providers:
✓ Gemini:            3 keys configured (pool active)
✗ OpenAI:            Not configured
✗ Anthropic:         Not configured

Browser:
✓ Chrome:            ~/.fetchium/chromium/chrome (118.0.5993.70)

Network:
✓ Internet:          Connected (8.8.8.8 reachable)
✓ DNS:               Working (google.com resolves)

Status: Ready (some optional features not configured)
Run `fetchium setup` to configure missing components.
```

---

## Week 2: Documentation — 5 Tutorial Posts

Each tutorial is both a docs page and a blog post. Written for developers who have
just installed Fetchium and want to build something real.

### Task 3.7 — Tutorial 1: "Your First AI Search in 5 Minutes"

**Target reader:** Developer who just installed Fetchium, no prior context
**Length:** 500–800 words
**File:** `docs/tutorials/01-first-ai-search.md`

**Outline:**
1. Install Fetchium (30 seconds)
2. Run `fetchium quickstart` — see it work
3. Try a real query: `fetchium ai "what is retrieval augmented generation"`
4. Understand the output: search results + AI summary + sources
5. Next steps: integrate into Python script

**Deliverable:** Published at `docs.fetchium.com/tutorials/first-ai-search`

### Task 3.8 — Tutorial 2: "Building a Research Agent with LangChain"

**Target reader:** Python developer building agents
**Length:** 800–1200 words
**File:** `docs/tutorials/02-langchain-research-agent.md`

**Outline:**
1. Install LangChain + fetchium adapter
2. Create a simple research agent
3. Run: "Research the latest developments in AI reasoning and write a summary"
4. Compare output to raw LangChain web tools
5. Customize: switch to deep mode, adjust token budget

**Code example:**
```python
from langchain_fetchium import FetchiumSearchTool
from langchain.agents import create_react_agent

tool = FetchiumSearchTool(api_key="fxm_...")
agent = create_react_agent(llm=ChatOpenAI(), tools=[tool], prompt=hub.pull("hwchase17/react"))
result = agent.invoke({"input": "Latest breakthroughs in multimodal AI, 2025"})
print(result["output"])
```

### Task 3.9 — Tutorial 3: "Token-Efficient Web Fetch for LLMs"

**Target reader:** Developer trying to reduce LLM costs
**Length:** 600–900 words
**File:** `docs/tutorials/03-token-efficient-fetch.md`

**Outline:**
1. The problem: raw HTML is 50K tokens; your context window is 8K
2. Fetchium's QADD algorithm: 10–20x reduction
3. Comparing modes by token count: fast vs extract vs clean
4. Using `--budget 2000` to cap token usage
5. Real numbers: benchmark table (raw HTML vs each mode)

### Task 3.10 — Tutorial 4: "Self-Hosting Fetchium with SearXNG"

**Target reader:** Developer who wants zero-cost, private AI search
**Length:** 800–1000 words
**File:** `docs/tutorials/04-self-hosting.md`

**Outline:**
1. Why self-host: privacy, no API costs, no rate limits
2. One-command setup: `fetchium setup` (installs SearXNG + Chrome)
3. Configure your AI provider (Gemini free tier)
4. Run a local benchmark: cost = $0 per fetch
5. Docker Compose for running on a VPS

### Task 3.11 — Tutorial 5: "Real-Time Data for CrewAI Agents"

**Target reader:** Developer building multi-agent pipelines
**Length:** 700–1000 words
**File:** `docs/tutorials/05-crewai-integration.md`

**Outline:**
1. CrewAI's limitation: no reliable web fetch by default
2. Install the fetchium-crewai adapter
3. Build a 3-agent crew: Researcher → Analyst → Writer
4. The Researcher uses FetchiumResearchTool for deep web queries
5. Run end-to-end: crew produces a 1-page research brief

---

## Docs Site Polish

### Task 3.12 — Improve the Quickstart Page

The docs quickstart (`docs.fetchium.com/quickstart`) must take under 5 minutes:

1. Install (1 command)
2. Run quickstart command
3. See your first result
4. Next steps

**No configuration required before getting the first result.**

### Task 3.13 — Add "Fetch Modes" Reference Page

A single page explaining all 7 modes with:
- When to use each
- Example command
- Typical latency and token count
- Which AI providers it works with

### Task 3.14 — API Reference Page

Auto-generate from the OpenAPI spec (the fetchium-api already has typed request/response
structs — generate `openapi.json` and render with Redoc or Scalar).

---

## Definition of Done

Sprint 03 is complete when:

1. `fetchium quickstart` runs end-to-end without errors on a fresh install
2. Default CLI output is readable by a human (not raw JSON)
3. Progress indicators are shown for operations > 2 seconds
4. Error messages include actionable next steps
5. `fetchium doctor` passes all checks in the standard environment
6. 5 tutorial posts are published and linked from the docs home page
7. Onboarding time (install → first successful AI result) is < 5 minutes
8. Median time measured: test with 3 developers unfamiliar with Fetchium

---

## Success Metrics

| Metric | Baseline | Sprint Target |
|--------|---------|--------------|
| Time to first fetch | ~15 min | < 5 min |
| Quickstart completion rate | N/A | > 60% |
| Docs bounce rate | N/A | < 50% |
| GitHub issues about "how do I" | N/A (estimate: 5/week) | < 2/week |
| NPS from early users | N/A | > 35 |
