# Phase 5 — Teams & Enterprise

**Timeline:** Months 19–24
**Theme:** Land enterprise contracts. One $10K/mo deal equals 200 Pro users.
**Team:** 5 people (founder + 2 engineers + 1 sales + 1 customer success)

---

## The Enterprise Inflection

At $3M ARR (Phase 4 end), Fetchium has proven product-market fit with developers. Phase 5 moves upmarket. Enterprise buyers pay 10-100x more per seat, churn far less (annual contracts), and bring entire teams.

**Enterprise buyer profile:**
- AI/ML teams at Series B+ startups building research-heavy products
- Analyst teams at consulting firms (McKinsey, BCG, Deloitte)
- Newsrooms and research departments (NYT, Bloomberg, think tanks)
- Legal teams needing deep research + citation verification
- Government contractors with FedRAMP requirements (Phase 6+)

**Why they pay:**
- SSO required for IT compliance
- Audit logs required for SOC 2 / ISO 27001
- SLA with credits required for production use
- On-prem option required for data sovereignty
- Dedicated support required when something breaks at 3am

---

## Success Criteria

| Metric | Target |
|--------|--------|
| ARR | $10,000,000 |
| MRR | $833,000 |
| Enterprise accounts (> $1K/mo) | 25 |
| Average contract value (ACV) | $25,000/year |
| Net Revenue Retention (NRR) | > 120% |
| Gross margin | > 75% |
| Churn rate (monthly) | < 2% |
| Sales cycle (SMB, < $500/mo) | < 14 days |
| Sales cycle (Enterprise, > $1K/mo) | < 60 days |
| SOC 2 Type II | Certified |

---

## What Ships

### 1. Team Workspaces (GA)

Full general availability, graduating from Phase 3 beta.

**Features:**
- Unlimited team members (seat-based billing)
- Shared Knowledge Base with thread organization
- Real-time collaboration: see who's researching what
- Shared monitor subscriptions and digest delivery
- Team-wide API key management (admins issue/revoke developer keys)
- Activity feed: recent searches, saved notes, research reports

**Roles:**
| Role | Permissions |
|------|-------------|
| Owner | Billing, members, all data |
| Admin | Members, API keys, settings |
| Member | Search, KB read/write, research |
| Viewer | KB read only, no search |

**CLI multi-profile support:**
```bash
fetchium profile add --name "work" --api-key hsx_work_...
fetchium profile use work
fetchium search "query" --profile work
```

### 2. SSO — SAML 2.0 + OIDC

Single sign-on required by enterprise IT. This is the most common enterprise deal blocker.

**Supported identity providers:**
- Okta (SAML 2.0 + OIDC)
- Google Workspace (OIDC)
- Microsoft Azure AD / Entra ID (SAML 2.0 + OIDC)
- OneLogin (SAML 2.0)
- Any SAML 2.0 or OIDC-compliant IdP

**SCIM provisioning:**
- Automatic user provisioning/deprovisioning via SCIM 2.0
- Group sync: map IdP groups to Fetchium roles
- Just-in-time (JIT) provisioning as fallback

**Setup flow:**
1. Admin goes to Settings → Security → SSO
2. Downloads Fetchium SAML metadata XML
3. Uploads IdP metadata or enters OIDC discovery URL
4. Tests connection with a test user
5. Enforces SSO (disables password login for domain)

### 3. Audit Logs

Every action logged, exportable, queryable.

**Logged events:**
- User login / logout / SSO events
- API key creation / rotation / deletion
- Search queries (with IP, key, timestamp)
- KB node creation / deletion
- Webhook creation / delivery / failure
- Plan changes, billing events
- Admin actions (member add/remove, role change)

**Retention:** 1 year by default, configurable up to 7 years.

**Export formats:** JSON (line-delimited), CSV, SIEM integration (Splunk, Datadog, Elastic).

**API:**
```
GET /v2/audit-logs?from=...&to=...&event_type=...&user_id=...
```

### 4. Compliance — SOC 2 Type II

**Timeline:**
- M19: Engage Vanta (compliance automation platform, ~$10K/year)
- M20: Vanta continuous monitoring live (100+ controls)
- M21: SOC 2 Type II audit window begins (6-month observation period)
- M24: SOC 2 Type II report issued

**Controls implemented:**
- Encryption at rest (AES-256) and in transit (TLS 1.3)
- Access control reviews (quarterly)
- Vulnerability management (Snyk, monthly scans)
- Incident response playbook (documented, tested annually)
- Employee security training (annual, documented)
- Vendor risk assessments (annual for all critical vendors)
- Change management (PR reviews, staging before prod)
- Business continuity plan (documented, tested annually)

**GDPR compliance:**
- DPA (Data Processing Agreement) template for EU customers
- Data residency: EU customers routed to Frankfurt region
- Right to erasure: user deletion cascade (all data wiped in 72h)
- Privacy policy updated with GDPR-specific language
- Cookie consent for web app

### 5. SLA Tiers

| Tier | Uptime SLA | Support | Credits |
|------|-----------|---------|---------|
| Free | No SLA | Community | None |
| Pro | 99.5% | Email 48h | None |
| Pro+ | 99.9% | Email 24h | 10% per 0.1% below |
| Growth | 99.95% | Email 8h | 15% per 0.1% below |
| Enterprise | 99.99% | Dedicated CSM + Slack | 20% per 0.1% below |

**99.99% = 52 minutes downtime/year.**
This requires:
- Multi-region active-passive failover (US + EU)
- Database replication with < 5s RPO
- Automated failover (no manual intervention)
- Status page: https://fetchium.com/status (Instatus or Better Uptime)

### 6. On-Prem Deployment

Enterprise customers with strict data sovereignty requirements.

**Options:**

**A. Docker Compose (SMB enterprise)**
```bash
curl -sSf https://install.fetchium.com/enterprise | sh
# Pulls: fetchium-api, fetchium-web, fetchium-worker, postgres, redis, searxng
# Configures Traefik, generates self-signed cert, starts all services
fetchium-admin setup  # interactive wizard
```

**B. Kubernetes / Helm chart (large enterprise)**
```bash
helm repo add fetchium https://charts.fetchium.com
helm install fetchium fetchium/fetchium \
  --set license.key="..." \
  --set database.host="..." \
  --values custom-values.yaml
```

**License model:** Annual license fee (included in Enterprise contract) + support contract.

**On-prem update process:**
- Monthly patch releases
- Customer pulls via `fetchium-admin update`
- LTS releases: 18-month support window

### 7. Enterprise Sales Infrastructure

**CRM:** HubSpot (free tier → Starter at $50/mo when pipeline > 20 deals)

**Inbound motion:**
1. Developer finds Fetchium via GitHub / HN / docs
2. Signs up for free → hits limits → upgrades to Pro
3. Shares with team → team admin requests Enterprise trial
4. Sales emails within 4 hours of Enterprise trial signup
5. Demo call → custom contract → close

**Outbound motion (M21+):**
- Target: AI team leads + VP Engineering at Series B-D startups
- Channels: LinkedIn outreach, conference networking, warm intros
- Sequence: LinkedIn connect → email → follow-up → call

**Sales deck structure:**
1. The problem (Bing API dead, Perplexity API expensive, Tavily acquired)
2. The solution (Fetchium: fastest, most accurate, most affordable)
3. The benchmark (real latency/accuracy comparison)
4. The platform (SDK, marketplace, KB, agents)
5. Enterprise readiness (SOC 2, SSO, SLA, on-prem)
6. Pricing + ROI calculator
7. Customer stories (3 dev team case studies)

**Legal templates (drafted by M19):**
- Master Service Agreement (MSA)
- Data Processing Agreement (DPA)
- Security review questionnaire response template
- Enterprise SLA agreement

---

## Infrastructure for Enterprise-Grade Reliability

### Multi-Region Active-Passive

```
Primary region: US-East (Hetzner Ashburn)
  ├── fetchium-api (3 instances, load balanced)
  ├── PostgreSQL primary
  ├── Redis primary
  └── SearXNG cluster

Failover region: EU-Central (Hetzner Frankfurt)
  ├── fetchium-api (standby, cold start < 60s)
  ├── PostgreSQL replica (streaming, < 5s lag)
  ├── Redis replica
  └── SearXNG cluster

Failover trigger:
  Primary health check fails 3 consecutive times (30s window)
  → Cloudflare DNS switches to EU region automatically
  → Estimated RTO: < 2 minutes
```

### Observability (Enterprise Grade)
- Grafana Cloud (managed Prometheus + Loki + Tempo)
- PagerDuty: P0 (production down) → immediate page → 15-min response SLA
- Status page: https://fetchium.com/status with historical uptime
- Customer-facing incident communications: email + status page updates within 15 minutes

---

## Week-by-Week Timeline

### Month 19
- W73: Team workspaces GA (roles, shared KB, activity feed)
- W74: Okta + Google Workspace SSO live
- W75: SCIM provisioning (Okta)
- W76: Vanta compliance monitoring live

### Month 20
- W77: Azure AD SSO + OIDC generic provider
- W78: Audit logs full implementation + export
- W79: DPA template + GDPR right-to-erasure
- W80: Multi-region infrastructure (US + EU) deployed

### Month 21
- W81: SOC 2 Type II audit window begins
- W82: Docker Compose enterprise deployment
- W83: Helm chart (Kubernetes)
- W84: Hire Account Executive (sales) — first sales hire

### Month 22
- W85: Sales deck + demo environment
- W86: Enterprise pricing page + ROI calculator
- W87: First 5 Enterprise trials (outbound + inbound)
- W88: HubSpot CRM pipeline live

### Month 23
- W89: First Enterprise contracts signed (target: 10)
- W90: CSM hired — dedicated customer success
- W91: Enterprise onboarding playbook + documentation
- W92: 99.99% SLA infrastructure testing

### Month 24
- W93: SOC 2 Type II report issued
- W94: $833K MRR milestone review
- W95: Series B materials preparation
- W96: Team at 5 people — ready for Phase 6 scale

---

## Enterprise Pricing

| Tier | Price | Seats | Usage |
|------|-------|-------|-------|
| Team | $200/mo | 5 | 50K API calls/day |
| Team+ | $500/mo | 15 | 200K API calls/day |
| Business | $2,000/mo | 50 | 1M API calls/day |
| Enterprise | Custom (min $5K/mo) | Unlimited | Unlimited |

**Enterprise add-ons:**
- On-prem license: $25K/year
- Dedicated support SLA: $1K/mo
- Custom model fine-tuning: $5K one-time
- Security review / pen test report: $2K one-time
