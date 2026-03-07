# Infrastructure Scaling Plan

## Current Baseline (Day 1)

**Hardware:** Single Ubuntu server — `server.zuhabul.com`
- CPU: 8-core (likely AMD/Intel Xeon-class VPS or dedicated)
- GPU: NVIDIA RTX 3060 (12GB VRAM) — for local AI inference
- RAM: 32GB (estimated)
- Storage: 500GB SSD
- Network: 1Gbps uplink

**Software stack:**
- OS: Ubuntu 24.04 LTS
- Reverse proxy: Traefik (TLS termination, routing)
- Auth: Authelia (SSO)
- Search backend: SearXNG (Docker, port 4040)
- API: fetchium-api (axum, port 3050)
- Web apps: Next.js on Node 24 (ports 3100, 3200)
- Database: SQLite (auth.db, usage.db) — local files

**Current capacity:**
- Estimated max: ~50 concurrent users, ~500 requests/minute before degradation
- AI inference: local Gemini CLI or API (no local GPU inference yet)
- SearXNG: single instance, no redundancy

---

## Phase 1: Hardening (Month 1–3, 0–500 users)

**Goal:** Make the current server production-reliable. Add observability before adding scale.

### CDN — Cloudflare (Free → Pro)
- Enable Cloudflare proxy on `fetchium.com` and `*.fetchium.com`
- Configure page rules: cache static assets (60min TTL), pass-through for API routes
- Enable Cloudflare Bot Management (free tier) to block scraper abuse
- DDoS protection is automatic with Cloudflare proxy
- **Cost:** $0 (free tier) → $20/month (Pro for analytics + custom rules)

### Observability — Grafana Stack
- Prometheus for metrics scraping (fetchium-api exposes `/metrics` endpoint)
- Loki for log aggregation (structured JSON from all services)
- Grafana for dashboards (already partially deployed on server)
- Alertmanager for PagerDuty/Discord alerts on error rate spikes
- **Dashboards to build:**
  - Requests/second by endpoint
  - P95/P99 latency heatmap
  - Error rate by error type
  - SearXNG health (response time, success rate)
  - Database size and query latency

### Backup Strategy
- SQLite databases: daily backup to Backblaze B2 (< $1/month)
- Configuration files: versioned in a private Git repo
- Server snapshot: weekly Hetzner/Contabo snapshot if VPS (or `rsync` to a $5/month Hetzner box)
- RTO (Recovery Time Objective): < 4 hours for full restore from backup

### Security Hardening
- Fail2ban on SSH and API endpoints (already partially configured)
- UFW firewall: only ports 22, 80, 443 open externally
- API rate limiting: already in fetchium-api (`fetchium-ip-rate-limit` middleware via Traefik)
- Secret rotation: document rotation procedure for ***REMOVED***, API keys
- Dependency auditing: `cargo audit` in CI weekly

**Phase 1 Total Cost:** ~$50–100/month (existing server + Cloudflare Pro + backup storage)

---

## Phase 2: Multi-Region Capability (Month 4–9, 500–5,000 users)

**Goal:** Eliminate single points of failure. Add read replicas and caching. Reduce global latency.

### Redis Cache Layer
- Deploy Redis (Docker) on the primary server for:
  - Fetch result caching (TTL: 1 hour for fast mode, 4 hours for AI mode)
  - Session/API key validation caching (avoid auth DB hits on every request)
  - Rate limit counters (replace in-memory state with Redis for multi-instance readiness)
- **Why Redis before second server:** Enables the app to scale horizontally without state conflicts
- **Cost:** $0 (self-hosted Docker container)

### Second Region — EU-West
- Deploy a second server in Europe (Hetzner Frankfurt or Contabo EU)
- Run: fetchium-api + SearXNG + Redis
- Configure: US server for US/Americas, EU server for EU/APAC (Cloudflare Load Balancing geo-routing)
- Database: SQLite replicated via Litestream to S3-compatible storage (Backblaze B2)
  - Primary: US server; EU is read-replica until writes scale enough for primary-primary
- **Cost:** ~$40–80/month for a 4-core/8GB EU VPS

### Database Migration Path: SQLite → PostgreSQL
- **Trigger:** > 100 concurrent users OR > 10K auth lookups/minute
- Deploy managed PostgreSQL (Supabase free tier → Neon → managed Hetzner Postgres)
- Migrate auth.db and usage.db; keep SQLite for config/local state
- Litestream handles SQLite streaming replication during the transition period

### SearXNG Scaling
- Deploy dedicated SearXNG instance per region (4040 stays on each server)
- Configure SearXNG with a pool of search engines per region (avoid regional blocks)
- Add SearXNG health check to `/health` endpoint — return 503 if SearXNG is down

### Monitoring Upgrades
- Add synthetic monitoring: Uptime Kuma (self-hosted) checks every 60 seconds
- External monitoring: BetterStack free tier for independent uptime checks
- On-call rotation: just the founder at this stage; PagerDuty free tier (1 user)
- **Target SLA:** 99.9% uptime (< 9 hours downtime/year)

**Phase 2 Total Cost:** ~$150–250/month (second server + Redis + PostgreSQL + monitoring tools)

---

## Phase 3: Kubernetes & Auto-scaling (Month 10–18, 5,000–50,000 users)

**Goal:** Remove manual scaling decisions. Handle traffic spikes automatically.

### Kubernetes Migration
- Choose: k3s (lightweight, self-hosted) vs. managed Kubernetes (GKE Autopilot, EKS)
- **Recommendation for solo-founder phase:** k3s on 2-3 VPS nodes — no cloud lock-in, lower cost
- Deploy all services as Kubernetes Deployments with resource limits
- HPA (Horizontal Pod Autoscaler) for fetchium-api based on CPU + request queue depth

### Service Architecture on Kubernetes

```
                    Cloudflare (CDN + DDoS + Geo-routing)
                            ↓
                    Traefik Ingress Controller
                     ↙            ↘
             US Cluster           EU Cluster
             (3 nodes)            (2 nodes)
                ↓                     ↓
          fetchium-api (3 pods)      fetchium-api (2 pods)
          SearXNG (2 pods)      SearXNG (1 pod)
          Redis (1 pod)         Redis (1 pod)
                ↓
         PostgreSQL (primary)
                ↓
         read-replica (EU)
```

### Multi-Tenant Isolation
- API key scoping: each request is isolated to the API key's tenant context
- Data isolation: separate database schemas per enterprise tenant
- Rate limiting: per-API-key and per-IP, enforced at ingress level
- Audit logging: every API request logged with tenant ID for compliance

### Content Delivery
- Static assets (docs, landing) on Cloudflare Pages (free) — no server needed
- API responses cached at Cloudflare edge for common public queries (TTL: 5 min)
- Result artifacts (PDF extractions, screenshots) stored in Cloudflare R2 (S3-compatible, $0.015/GB)

### Cost at Phase 3
| Item | Monthly Cost |
|------|-------------|
| 3-node US k3s cluster (Hetzner CCX) | $150 |
| 2-node EU k3s cluster | $100 |
| Managed PostgreSQL (Neon or Supabase Pro) | $25 |
| Cloudflare Pro + Load Balancing | $50 |
| Cloudflare R2 storage | $10 |
| Monitoring (Grafana Cloud Pro) | $30 |
| Backups (B2) | $5 |
| **Total** | **~$370/month** |

---

## Phase 4: Edge Compute & Global PoPs (Month 18+, 50,000+ users)

**Goal:** Sub-100ms response times globally. Enterprise SLA capability.

### Edge Compute Strategy
- **Cloudflare Workers:** Run lightweight fetch routing logic at the edge
  - Route request to nearest healthy region
  - Handle rate limiting at edge (avoid origin hits for rate-limited requests)
  - Cache hit serving without reaching origin
- **Cloudflare Workers AI:** Offload small AI tasks (intent classification, snippet scoring) to edge

### Global PoPs (Points of Presence)
- Add 2 more regions: APAC (Singapore or Tokyo) and LATAM (São Paulo)
- Each PoP: 2-3 k3s nodes, local SearXNG, Redis
- Traffic: Cloudflare geo-routes to nearest PoP; failover is automatic

### Enterprise Isolation Tier
- Dedicated namespace in Kubernetes for enterprise customers
- Enterprise customers get their own SearXNG instance (no shared search traffic)
- Optional: on-prem Kubernetes chart for maximum data sovereignty
- VPN connectivity for on-prem SearXNG to cloud API (WireGuard)

### Cost Projections at Scale

| Phase | Monthly Infra Cost | Revenue Required for 20% Margin |
|-------|--------------------|-------------------------------|
| Phase 1 | $50–100 | $500 |
| Phase 2 | $150–250 | $1,500 |
| Phase 3 | $350–500 | $3,500 |
| Phase 4 | $1,000–3,000 | $10,000 |

**Key insight:** Infrastructure costs scale sub-linearly relative to revenue. At 50K users,
infra is < 5% of revenue — healthy for a SaaS business.

---

## Disaster Recovery Plan

### RTO / RPO Targets (by Phase)

| Phase | RTO | RPO | Strategy |
|-------|-----|-----|---------|
| Phase 1 | 4 hours | 24 hours | Manual restore from backups |
| Phase 2 | 1 hour | 1 hour | Failover to EU replica |
| Phase 3 | 15 minutes | 5 minutes | Kubernetes auto-reschedule + streaming replication |
| Phase 4 | 5 minutes | 1 minute | Multi-region active-active |

### Runbooks (Document Before You Need Them)
- `runbook-api-down.md`: SearXNG failure, API pod crash, database connection failure
- `runbook-high-latency.md`: Redis cache miss storm, database slow queries, search backend timeout
- `runbook-data-restore.md`: Steps to restore from B2 backup, verify integrity
- `runbook-security-incident.md`: API key compromise, database breach response

---

## Infrastructure Decision Log

| Decision | Rationale | Revisit When |
|----------|-----------|-------------|
| SQLite over PostgreSQL (Phase 1) | Zero ops overhead, sufficient for < 100 req/s | > 100 concurrent users |
| Self-hosted over managed cloud | Cost: 5x cheaper at this scale | Team > 5 engineers |
| k3s over managed K8s | Cost: $0 control plane vs. $70+/month | Ops burden exceeds 5h/week |
| Cloudflare CDN | Best free tier, global network, DDoS | If pricing changes significantly |
| Hetzner VPS | Best price/performance EU VPS provider | If APAC/US latency becomes critical |
