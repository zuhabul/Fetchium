# Sprint 01: Foundation

**Duration:** 2 weeks
**Theme:** Domain, brand assets, legal entity, social presence
**Goal:** Fetchium exists as a real, discoverable brand before a single line of product code ships under the new name

---

## Context

Before executing the code rebrand (Sprint 02), the brand infrastructure must be in place.
Domain purchased, legal entity registered, social handles claimed, design system defined.
This sprint is intentionally non-technical — it's the foundation everything else is built on.

**Budget:** ~$500

---

## Week 1: Domain, Legal, Identity

### Day 1–2: Domain & Hosting Infrastructure

**Task 1.1 — Register fetchium.com**
- Check: https://www.namecheap.com or https://porkbun.com (both cheaper than GoDaddy)
- Register: `fetchium.com` (~$12/year)
- Also register: `getfetchium.com`, `fetchium.io` as defensive registrations (~$30 total)
- Point DNS to Cloudflare immediately after registration (free nameservers, DDoS protection)
- **Deliverable:** DNS propagated, `fetchium.com` → server IP

**Task 1.2 — Cloudflare Setup**
- Create Cloudflare account with a dedicated email (not personal Gmail)
- Transfer nameservers to Cloudflare
- Enable "Full (Strict)" SSL/TLS mode
- Set up Page Rules: `www.fetchium.com` → 301 redirect to `fetchium.com`
- Enable bot protection (free tier)
- **Deliverable:** Cloudflare dashboard showing green SSL status

**Task 1.3 — Email Setup**
- Create `hello@fetchium.com`, `support@fetchium.com`, `team@fetchium.com`
- Options: Cloudflare Email Routing (free) → forward to existing Gmail, OR Proton for Business ($4/month)
- Recommendation: Cloudflare Email Routing for Year 1 — zero cost, professional address
- Set up email signature
- **Deliverable:** Email addresses working and forwarding

### Day 3–4: Legal Entity

**Task 1.4 — Business Registration**
- Option A (US): Stripe Atlas ($500) — Delaware C-Corp in 1–3 business days
  - Includes: Articles of Incorporation, EIN, initial stock issuance, founder agreement
  - Best if planning to raise VC funding
- Option B (non-US or bootstrap): Sole proprietorship or local LLC equivalent
  - File in your country/state — typically < $100 in most jurisdictions
  - Register "Fetchium" as a DBA (Doing Business As) if operating under a personal entity
- **Deliverable:** Business entity registered, EIN obtained, bank account opened

**Task 1.5 — Bank Account**
- Mercury Bank (mercury.com) — free for US startups, no minimums, clean UI
- Deposit any operating funds
- Note account number for Stripe setup
- **Deliverable:** Business bank account open and funded

**Task 1.6 — Cap Table Spreadsheet**
- Create a Google Sheet or Notion table tracking:
  - Founder equity (typically 100% at start)
  - Option pool (reserve 15–20% before first hire)
  - Future: investor SAFE notes
- This is the source of truth until you can afford Carta
- **Deliverable:** Cap table document created and secured

### Day 5: Brand Identity

**Task 1.7 — Logo Design**
- Brief: Fetchium is a developer tool for web fetch + AI. Clean, technical, no clip art.
- Style reference: Vercel, Linear, Supabase logos — minimal, dark-mode-first
- Process:
  1. Use Figma (free) to sketch concepts
  2. Or use Looka.com ($20–$65) for AI-generated logo options
  3. Or brief a designer on Contra or Fiverr ($100–$200 for quality work)
- Output formats needed: SVG, PNG (light + dark variants), favicon (16×16, 32×32, 180×180)
- **Deliverable:** Logo in SVG + PNG formats, favicon set

**Task 1.8 — Color Palette & Typography**
- Primary color: Choose 1 accent color (recommendation: electric blue `#0066FF` or teal `#00BFA5`)
- Neutral palette: slate/zinc grays (Tailwind defaults work well)
- Typography: Inter (Google Fonts, free) for body; JetBrains Mono for code
- Document in a Figma file or Notion page: hex codes, font weights, usage guidelines
- **Deliverable:** Brand style guide (1 page, can be a Notion doc)

---

## Week 2: Social Presence & GitHub Setup

### Day 6–7: GitHub Organization

**Task 1.9 — Create GitHub Organization**
- Name: `fetchium` (if available) or `fetchium-ai`
- Create: https://github.com/organizations/new
- Plan: Free tier is sufficient; consider GitHub Teams ($4/user/month) after first hire
- Transfer main repository from personal account to the org
- **Deliverable:** `github.com/fetchium` organization live

**Task 1.10 — GitHub Profile & Repository Setup**
- Add organization profile README (`.github/profile/README.md`)
  - What Fetchium is (2 sentences)
  - Quick install command
  - Links to: docs, Discord, Twitter
- Pin the main `fetchium` repository
- Set up GitHub Discussions for community Q&A
- Enable GitHub Sponsors (even if not yet active — builds trust)
- **Deliverable:** `github.com/fetchium` looks professional and complete

**Task 1.11 — Repository Preparation**
- Update `README.md` with new name, logo, and install instructions
- Add `CONTRIBUTING.md` — how to contribute, code style, PR process
- Add `CODE_OF_CONDUCT.md` — Contributor Covenant is fine (copy from contributor-covenant.org)
- Update `LICENSE` file — ensure it lists the correct entity name
- Add GitHub issue templates: bug report, feature request, documentation
- **Deliverable:** Repository ready for public visitors

### Day 8–9: Social Media

**Task 1.12 — Twitter/X Account**
- Username: `@fetchium` (preferred) or `@fetchiumai` or `@getfetchium`
- Profile: headshot or logo, bio: "Typed web fetch for AI agents. Open source."
- Link: `fetchium.com`
- Pin tweet: "Introducing Fetchium — [tagline] 🧵" (draft now, post on launch)
- Schedule welcome tweet for launch day
- **Deliverable:** @fetchium (or equivalent) account created and profile complete

**Task 1.13 — LinkedIn Company Page**
- Create company page: `linkedin.com/company/fetchium`
- Add logo, cover image, description, website, founding year
- Connect personal LinkedIn with "works at Fetchium"
- **Deliverable:** LinkedIn page live

**Task 1.14 — Discord Server**
- Create Discord server: "Fetchium Community"
- Channel structure:
  - `#announcements` (read-only, important updates)
  - `#general` (open discussion)
  - `#help` (support questions)
  - `#feedback` (feature requests)
  - `#showcase` (what people built with Fetchium)
  - `#contributors` (for OSS contributors)
- Add Discord bot: Carl-bot or MEE6 for moderation
- Create invite link and put it in README + landing page
- **Deliverable:** Discord server live with invite link

**Task 1.15 — Product Hunt Profile**
- Create maker account at producthunt.com
- Set up Fetchium as an "upcoming product"
- This starts collecting upvotes before launch
- **Deliverable:** Fetchium "upcoming" page live on Product Hunt

### Day 10: Documentation Skeleton

**Task 1.16 — Docs Site Skeleton**
- Technology: Next.js (already in the stack) or Mintlify (free tier, no maintenance)
- Recommendation: Mintlify — zero maintenance, auto-generates from MDX files
- Register at mintlify.com, connect the GitHub repo
- Create pages: Introduction, Quickstart, Fetch Modes, API Reference, Self-Hosting
- Content: placeholder with "Coming soon" is fine — structure matters now
- **Deliverable:** `docs.fetchium.com` is live (even if sparse)

---

## Budget Breakdown

| Item | Cost |
|------|------|
| fetchium.com domain | $12 |
| Defensive domains (2) | $28 |
| Business entity (Stripe Atlas) | $500 |
| Logo design (Looka or freelancer) | $65–$200 |
| Figma (free tier) | $0 |
| Email (Cloudflare routing) | $0 |
| Discord, GitHub, Twitter | $0 |
| Mintlify docs (free tier) | $0 |
| **Total** | **$605–$740** |

Note: Business entity is the biggest cost. If bootstrapping lean, use local sole
proprietorship (< $100) and defer formal incorporation until first significant revenue.

---

## Deliverables Checklist

- [ ] `fetchium.com` registered and pointing to server via Cloudflare
- [ ] `hello@fetchium.com` email working
- [ ] Legal entity registered (or explicitly deferred with reasoning)
- [ ] Business bank account open
- [ ] Logo: SVG + PNG (light/dark) + favicon set
- [ ] Brand color + typography documented (1-page Notion doc)
- [ ] `github.com/fetchium` organization created
- [ ] GitHub repository transferred and profile README added
- [ ] `@fetchium` Twitter/X account created and profile complete
- [ ] Discord server live with invite link
- [ ] LinkedIn company page live
- [ ] Product Hunt upcoming page created
- [ ] `docs.fetchium.com` skeleton live (even if sparse)

---

## Definition of Done

Sprint 01 is complete when:
1. Someone can Google "Fetchium" and find a real, professional brand presence
2. A developer can click from Twitter → GitHub → Docs and understand what Fetchium is
3. All social handles are secured — even if not yet active
4. The legal and financial foundation exists to accept money and sign contracts

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| `fetchium.com` is taken | Fallback: `getfetchium.com` |
| Logo design takes too long | Use Looka for a quick v1; iterate later |
| Business registration is slow | Start legal entity on Day 1; sprint runs in parallel |
| `@fetchium` Twitter handle taken | Try `@fetchiumai` or `@getfetchium` |
