# Fetchium — Brand Guidelines

**Last updated:** 2026-02-27
**Version:** 1.0

---

## Brand Foundation

### Name

**Fetchium** — a portmanteau of "fetch" (retrieve) and the "-ium" suffix (evoking elements, precision, scientific rigor).

| Context | Correct Usage | Incorrect |
|---------|--------------|-----------|
| Prose, marketing, headings | **Fetchium** (capital F) | fetchium, FETCHIUM, Fetch-ium |
| Code identifiers | `fetchium` (lowercase) | Fetchium, FETCHIUM |
| CLI binary | `fetchium` | Fetchium |
| Package names | `fetchium` | Fetchium |
| Domain | `fetchium.dev` | fetchium.com, FetchiumAI.com |
| Twitter/X | `@fetchiumdev` | @Fetchium, @FetchiumAI |

**Never:**
- "Fetchium AI" (in prose — only acceptable in a product tier name)
- "The Fetchium" (no definite article)
- "FetchiumX" or other portmanteau variants
- Abbreviation "FX" — not official

### Tagline

**Primary tagline:**
> Fetch anything. Verified. Fast.

**Secondary taglines (contextual):**

| Context | Tagline |
|---------|---------|
| Developer landing page | "The search API that gets smarter with every query." |
| Agent/LLM context | "Give your AI agent the internet." |
| Research context | "Research at machine speed." |
| Social search context | "What the internet actually thinks." |
| Video context | "Watch nothing. Know everything." |

**Tagline rules:**
- Never use "AI-powered" (overused, meaningless)
- Never use "revolutionary" or "game-changing"
- Never use exclamation marks in taglines
- Keep under 7 words where possible

---

## Visual Identity

### Logo

**Wordmark:** "Fetchium" set in Inter (Bold weight).

**Symbol:** A stylized atom/orbit mark with an arrow suggesting fetch/retrieval. The atom alludes to precision, elements, and the "-ium" suffix. The arrow alludes to retrieval and speed.

**Logo variants:**
| Variant | Use case |
|---------|---------|
| Full wordmark (horizontal) | Primary; use wherever space allows |
| Symbol only | App icon, favicon, social avatar |
| Stacked (symbol above wordmark) | Square contexts (og:image, badges) |
| White on dark | Dark backgrounds, terminal contexts |
| Dark on light | Light backgrounds, print |

**Logo file formats:**
- SVG (primary — scalable, crisp at all sizes)
- PNG @1x, @2x, @3x (for contexts that don't support SVG)
- Favicon: 16px, 32px, 180px (Apple touch icon)

**Minimum size:** 80px wide for wordmark; 16px for symbol alone.

**Clear space:** Minimum padding around logo = height of the "F" in the wordmark.

### Color Palette

#### Primary Colors

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| Void | `#0A0E1A` | 10, 14, 26 | Dark backgrounds, hero sections |
| Deep Blue | `#0F172A` | 15, 23, 42 | Card backgrounds, secondary dark surfaces |
| Electric Indigo | `#6366F1` | 99, 102, 241 | Primary brand color, CTAs, links |
| Bright Cyan | `#22D3EE` | 34, 211, 238 | Accent, highlights, terminal output |

#### Secondary Colors

| Name | Hex | Usage |
|------|-----|-------|
| White | `#FFFFFF` | Text on dark, logos |
| Slate 100 | `#F1F5F9` | Light backgrounds |
| Slate 400 | `#94A3B8` | Muted text, captions |
| Slate 700 | `#334155` | Secondary text on light |
| Emerald 400 | `#34D399` | Success states, "Built" badges |
| Amber 400 | `#FBBF24` | Warning states |
| Rose 400 | `#FB7185` | Error states, "breaking change" badges |

#### Gradient (primary hero gradient)
```css
background: linear-gradient(135deg, #0A0E1A 0%, #1a1040 50%, #0f172a 100%);
```

#### Accent glow (for interactive elements)
```css
box-shadow: 0 0 30px rgba(99, 102, 241, 0.3);
```

#### Terminal/code backgrounds
```css
background: #0D1117;  /* GitHub dark — familiar to developers */
color: #22D3EE;       /* Cyan for output */
color: #E2E8F0;       /* Off-white for input */
```

### Typography

**Heading font:** Inter (Bold, 700-weight)
- Fallback: system-ui, -apple-system, sans-serif
- Source: Google Fonts (free) or self-hosted via @next/font

**Body font:** Inter (Regular, 400; Medium, 500)
- Same family as headings for simplicity

**Code font:** JetBrains Mono (Regular, 400)
- Fallback: 'Fira Code', 'Cascadia Code', monospace
- Used for: code blocks, CLI examples, API responses, inline code

**Type scale (web):**
| Level | Size | Weight | Line Height |
|-------|------|--------|------------|
| Display | 64px | 700 | 1.1 |
| H1 | 48px | 700 | 1.2 |
| H2 | 36px | 700 | 1.3 |
| H3 | 24px | 600 | 1.4 |
| Body | 16px | 400 | 1.6 |
| Small | 14px | 400 | 1.5 |
| Code | 14px | 400 | 1.7 |

---

## Voice & Tone

### Brand Personality

Fetchium speaks like **a senior developer who's also a good teacher** — confident about what they know, honest about limitations, and able to explain hard things clearly without condescension.

**Four core traits:**
1. **Precise** — we use exact numbers, not vague claims
2. **Direct** — we say what we mean, no filler
3. **Genuine** — we acknowledge competitors fairly, admit limitations
4. **Understated** — results speak; we don't oversell

### Voice Examples

**Good:**
> Fetchium searches 10 web engines simultaneously and synthesizes an AI answer with citations. Average latency: 1.2 seconds.

**Bad:**
> Fetchium is a revolutionary AI-powered search platform that completely transforms how you find information on the web! Our cutting-edge technology gives you instant access to verified, accurate results!

---

**Good:**
> We're slower than a direct SearXNG query because we verify citations. That tradeoff is intentional.

**Bad:**
> Fetchium is the fastest search API available. We never compromise on speed.

---

**Good:**
> Perplexity is excellent for consumer use. For building agents at scale, their token-based pricing gets expensive fast. Fetchium is flat-rate.

**Bad:**
> Unlike other inferior solutions, Fetchium dominates the competition in every metric.

### Tone by Context

| Context | Tone | Example |
|---------|------|---------|
| Landing page | Confident, minimal | "Search everything. Know anything." |
| Docs | Precise, helpful | "The `--fast` flag skips full-page extraction and uses snippets only." |
| Error messages | Clear, actionable | "Rate limit exceeded. Upgrade to Pro for 5K searches/day." |
| Blog posts | Conversational, direct | "Here's what we learned after 1M API calls." |
| Release notes | Factual, organized | "v1.2.0: Added streaming for `/v1/ai`. Breaking: `result_id` field renamed to `id`." |
| Discord support | Warm, prompt | "Got it — that's a known issue with sites that use lazy-loading. Here's the workaround:" |
| Tweets | Punchy, concrete | "New benchmark: Fetchium p95 latency = 1.4s. Perplexity Sonar = 2.8s. Source: our open test suite." |

### Do's and Don'ts

**Do:**
- Include actual numbers whenever possible
- Admit when a competitor does something better (and explain our tradeoff)
- Use code snippets in every developer-facing piece of content
- Write short sentences (< 25 words)
- Use active voice
- Name specific tools, specific algorithms, specific mechanisms

**Don't:**
- Use "leverage" as a verb
- Say "game-changing," "revolutionary," "world-class," "cutting-edge"
- Use vague superlatives ("the best," "the fastest") without a cited benchmark
- Start marketing copy with "We"
- Use "simple" or "easy" (let the user decide)
- Apologize for being a startup or for limitations — be matter-of-fact

---

## Logo Usage Rules

**Acceptable:**
- Fetchium logo on any background where contrast ratio > 4.5:1
- Logo alongside partner logos at equal or smaller size
- Logo in grayscale when color printing is unavailable

**Not acceptable:**
- Stretching or distorting the logo in any dimension
- Rotating the logo
- Applying drop shadows or gradients to the logo
- Placing the logo on a busy photographic background without a clear background
- Recreating the logo in a different font
- Using a competitor's logo directly adjacent without clear context ("vs" comparisons)

---

## Social Media Assets

**Profile photo:** Symbol-only logo on Void (#0A0E1A) background, 800×800px.

**Twitter/X banner:** 1500×500px. Tagline + CLI demo screenshot on dark gradient.

**OG image (web):** 1200×630px. Wordmark + tagline + one-line code snippet.

**GitHub social preview:** 1280×640px. README-style: logo + benchmark table.

**Discord server icon:** Symbol-only logo, 512×512px, on Electric Indigo background.

---

## Badge & Status Indicators (for README + docs)

```markdown
![Fetchium](https://img.shields.io/badge/fetchium-1.x-6366f1?style=flat-square)
![License](https://img.shields.io/badge/license-Apache%202.0-22d3ee?style=flat-square)
![Build](https://img.shields.io/github/actions/workflow/status/fetchium/fetchium/ci.yml?style=flat-square)
```

Use the Electric Indigo (#6366F1) for brand badges and Bright Cyan (#22D3EE) for license/status badges.
