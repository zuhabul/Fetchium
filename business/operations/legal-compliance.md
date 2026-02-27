# Legal & Compliance

## Current Legal Status

**License:** MIT OR Apache-2.0 (dual license)
This is the correct choice for a developer tool. Keep it. Reasons:
- MIT: permissive, familiar to most developers, no patent grant language
- Apache-2.0: adds explicit patent grant (protects contributors and users)
- Dual-license: users choose whichever fits their project's license requirements
- Compatible with all major OSS ecosystems (LangChain, AutoGen, etc.)

**Do not change the license without a lawyer's review.** Relicensing after OSS adoption
is legally complex and community-damaging.

---

## Business Entity

### Recommended Structure

**For a solo founder building a SaaS:**
- **US founders:** Delaware C-Corp (via Stripe Atlas, $500, takes 1–3 days)
  - Why Delaware: venture-friendly, clean corporate law, standard for tech startups
  - Why C-Corp (not LLC): required for issuing stock options to employees; VCs only fund C-Corps
- **Non-US founders (e.g., Bangladesh/UK):** Wyoming LLC or UK Ltd initially; convert to Delaware C-Corp before Series A
- **Alternative (bootstrap-only):** Sole proprietorship or single-member LLC — simpler taxes, no investor optionality

**Fetchium's immediate need:** Any legal entity to accept payments and sign contracts.
Even a simple LLC works. Incorporate as soon as you have your first paying customer.

### Corporate Hygiene
- Open a dedicated business bank account (Mercury.com — free for US LLCs/C-Corps)
- Separate all business expenses from personal — critical for tax purposes
- Track all equity grants in a cap table from day one (use Carta free tier or a spreadsheet)

---

## Open Source License Compliance

### Fetchium's OSS Dependencies

All dependencies must be checked for license compatibility with MIT/Apache-2.0:

| License | Compatible? | Action Required |
|---------|------------|----------------|
| MIT | Yes | None |
| Apache-2.0 | Yes | None |
| BSD-2/3-Clause | Yes | None |
| ISC | Yes | None |
| MPL-2.0 | Conditional | File-level copyleft; keep modifications to MPL files as OSS |
| LGPL-2.1/3.0 | Conditional | Dynamic linking only; do NOT statically link |
| GPL-2.0/3.0 | **No** | Cannot use in proprietary parts; only in CLI binary if distributed |
| AGPL-3.0 | **No** | Viral to network-accessible software; avoid in hsx-api |
| CC-BY-4.0 | For content only | Fine for documentation, not code |

**Tool for checking:** `cargo deny` — run in CI on every PR:
```toml
# deny.toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-DFS-2016"]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]
```

### SearXNG License Note
SearXNG is licensed under AGPL-3.0. **This is fine because Fetchium does not distribute
or modify SearXNG** — we run it as a separate service. Our code calls SearXNG's HTTP API.
AGPL's copyleft does not propagate through HTTP API calls.

---

## GDPR Compliance

Fetchium processes data about EU residents when they use the API or web dashboard.

### What Data We Process

| Data Type | Location | Retention | Purpose |
|-----------|----------|-----------|---------|
| Email address | Auth DB (SQLite) | Until account deletion | Login, support |
| API key (hashed) | Auth DB | Until revoked | Authentication |
| IP address | Access logs | 30 days | Rate limiting, security |
| Fetch history | Usage DB | 30 days (Pro), 90 days (Teams) | Dashboard, billing |
| AI query content | Logs only | 7 days | Debugging (aggregated) |
| Payment info | Stripe (not us) | Stripe's retention | Billing |

**We do NOT store:**
- The content of fetched URLs beyond log retention
- Personal information from fetched pages (we're the fetch tool, not the data broker)
- Cookies or tracking pixels on the API

### Legal Basis for Processing (GDPR Article 6)

| Processing Activity | Legal Basis |
|--------------------|-------------|
| Account creation + auth | Contract performance (6(1)(b)) |
| Usage logging for billing | Legitimate interest (6(1)(f)) |
| Security & rate limiting | Legitimate interest (6(1)(f)) |
| Email notifications | Consent (6(1)(a)) — opt-in only |

### Rights Implementation

| Right | Implementation |
|-------|---------------|
| Right of access | Dashboard: download all your data as JSON |
| Right to erasure | Dashboard: "Delete my account" button — hard deletes within 30 days |
| Right to rectification | Dashboard: update email, name |
| Right to portability | Dashboard: export fetch history as CSV/JSON |
| Right to object | Opt-out of any non-essential data processing (just analytics) |

### Data Processing Agreement (DPA)

Required for Teams and Enterprise customers in the EU.

**Template structure:**
```
DATA PROCESSING AGREEMENT
Between: [Customer] (Controller)
And: Fetchium Inc. / [Legal Entity] (Processor)

1. Subject matter: Web fetch API services
2. Duration: Term of the subscription agreement
3. Nature and purpose: Processing queries on behalf of Controller
4. Type of personal data: As described in Schedule A
5. Categories of data subjects: Controller's end users
6. Obligations of Processor: [Standard GDPR processor obligations]
7. Sub-processors: Stripe (payments), Hetzner (infrastructure), Cloudflare (CDN)
8. Data transfers: Standard Contractual Clauses (SCCs) for non-EU transfers
9. Security measures: Encryption at rest + in transit, access controls, audit logs
10. Deletion: 30 days after contract termination
```

**Action:** Draft this with a GDPR-specialized legal template service (Iubenda, Termly, or a lawyer).
Cost: $200–$500 for a template; $2,000+ for a custom lawyer-drafted DPA.

### GDPR Checklist

- [ ] Privacy Policy published at `fetchium.com/privacy`
- [ ] Cookie consent banner (minimal — we barely use cookies)
- [ ] DPA template ready for Teams/Enterprise customers
- [ ] Data deletion mechanism working end-to-end
- [ ] Sub-processor list maintained and published
- [ ] Data breach notification procedure documented (72-hour GDPR deadline)
- [ ] DPO: not required until > 250 employees or large-scale systematic processing

---

## Terms of Service

**Publish at:** `fetchium.com/terms`

### Key Sections to Include

1. **Acceptance of Terms** — Using the API constitutes acceptance
2. **Description of Service** — What Fetchium provides and doesn't provide
3. **Account Registration** — User responsibilities for account security
4. **Acceptable Use Policy** — What users may NOT do (see below)
5. **API Usage** — Rate limits, fair use, key security
6. **Payment Terms** — Subscription billing, refund policy, failed payments
7. **Intellectual Property** — We own the platform; users own their data and outputs
8. **Disclaimer of Warranties** — AS IS, no uptime guarantee (except Enterprise SLA)
9. **Limitation of Liability** — Cap at 3 months of subscription fees paid
10. **Indemnification** — User indemnifies us for their misuse
11. **Governing Law** — Delaware or jurisdiction of incorporation
12. **Changes to Terms** — 30-day notice for material changes

**Use a template:** Termly.io ($TOS starter) or consulted from a lawyer for > $1K ARR.

---

## Acceptable Use Policy

**Users may NOT use Fetchium to:**
- Violate any applicable laws or regulations
- Scrape content in violation of a website's robots.txt or ToS at scale
- Access, scrape, or aggregate personal data without proper legal basis
- Build applications targeting children under 13 (COPPA)
- Circumvent technical access controls of target websites
- Generate spam, phishing content, or disinformation
- Conduct DDoS attacks or cause harm to third-party infrastructure
- Fetch CSAM or any illegal content
- Resell Fetchium's raw API access without a reseller agreement
- Use Fetchium to train machine learning models for direct competition with Fetchium

**Consequences of violation:** Account suspension → permanent ban → potential legal action.

---

## Privacy Policy

**Publish at:** `fetchium.com/privacy`

**Minimum required sections:**
1. What information we collect and why
2. How we use information
3. How we share information (sub-processors, legal requirements)
4. Data retention periods
5. Your rights (GDPR, CCPA)
6. Security measures
7. Contact information

**Use:** Iubenda.com (auto-generates GDPR + CCPA compliant policies, ~$60/year) or
equivalent template service. Do not copy-paste from another startup — policies must
be accurate to your actual practices.

---

## CCPA (California Consumer Privacy Act)

Applies when you have > 25K California residents OR > $25M revenue OR sell personal data.
For a startup under $1M ARR, you are **not yet required** to comply fully. However:
- **Do now:** Add a "Do Not Sell My Personal Information" link in your privacy policy
  (we don't sell data, so this is a simple "We do not sell personal information")
- **At $1M ARR:** Full CCPA compliance (California-specific disclosures, opt-out mechanism)

---

## SOC 2 Roadmap

SOC 2 Type II is required by enterprise customers with strict security requirements.
It is a 6–12 month process after controls are in place.

**Timeline:**
- **Now:** Implement the controls (security, availability, confidentiality)
- **Month 12:** Begin readiness assessment with a CPA firm ($5K–$15K)
- **Month 18:** Complete SOC 2 Type I audit ($15K–$30K)
- **Month 24:** SOC 2 Type II report available for enterprise customers ($25K–$50K)

**Controls to implement now (before the audit):**
- [ ] Encryption at rest for all databases
- [ ] TLS 1.2+ only for all traffic (Traefik config)
- [ ] API key rotation procedure documented
- [ ] Employee (contractor) access reviews quarterly
- [ ] Vulnerability scanning on every deploy (cargo audit, npm audit)
- [ ] Incident response plan documented and tested
- [ ] Background checks for any employees with database access

---

## Intellectual Property

### What Fetchium Owns
- The source code in this repository
- The Fetchium brand, logo, and domain names
- The novel algorithms documented in prd.md (trade secrets until patented or published)
- Training data and model weights derived from Fetchium's systems

### Patent Considerations
- CEP, QATBE, HyperFusion — potentially patentable algorithms
- **Recommendation:** Do not file patents at the bootstrap stage (cost: $15K–$30K per patent)
- Instead: publish detailed blog posts establishing prior art (prevents others from patenting)
- Revisit patent strategy at Series A with legal counsel

### Contributor License Agreement (CLA)
If accepting external OSS contributions:
- Use the Apache CLA (individual + corporate versions)
- CLA Assistant (cla-assistant.io) automates GitHub CLA signing
- This ensures Fetchium retains the right to relicense if needed

---

## Legal Budget Estimates

| Item | When | Estimated Cost |
|------|------|---------------|
| Business entity formation | Month 1 | $500 (Stripe Atlas) |
| Privacy Policy + ToS templates | Month 1 | $100–$300 |
| DPA template | Month 3 | $500–$1,000 |
| First legal review (contracts) | Month 6 | $500–$1,500 |
| SOC 2 Type I audit | Month 18 | $15K–$30K |
| Enterprise contract template | Month 12 | $2K–$5K |
| **Year 1 total legal budget** | — | **$2,000–$5,000** |

**Recommended legal resource:** Clerky.com for entity + equity docs; Ironclad or PandaDoc
for contract management; a startup-focused lawyer for > $1M ARR or first enterprise deal.
