# Fetchium Admin Dashboard — Full Production Execution Plan

> **Status legend:** `[ ]` = not started · `[~]` = in progress · `[x]` = done
> **Task IDs:** `AD-P{phase}-T{task}` (AD = Admin Dashboard)
> **Owner:** This file is the single source of truth. Mark tasks as you go.

---

## Table of Contents

1. [Monorepo Structure](#monorepo-structure)
2. [Phase 0 — Data Model & Permission Spec](#phase-0)
3. [Phase 1 — Admin App Scaffold](#phase-1)
4. [Phase 2 — Admin Auth (TOTP + Sessions)](#phase-2)
5. [Phase 3 — RBAC & Policy Enforcement](#phase-3)
6. [Phase 4 — Admin Backend Namespace (Rust)](#phase-4)
7. [Phase 5 — Admin SQLite Schema](#phase-5)
8. [Phase 6 — Org & User Management](#phase-6)
9. [Phase 7 — API Key & Usage Operations](#phase-7)
10. [Phase 8 — Billing & Finance](#phase-8)
11. [Phase 9 — CRM & Revenue Ops](#phase-9)
12. [Phase 10 — Support Operations](#phase-10)
13. [Phase 17 — Observability & Incident Command](#phase-11)
14. [Phase 12 — Ads & Attribution Analytics](#phase-12)
15. [Phase 13 — Security, Audit & Compliance](#phase-13)
16. [Phase 14 — Feature Flags & Operational Controls](#phase-14)
17. [Phase 15 — Advanced UX (Command Palette, Live Data, Shortcuts)](#phase-15)
18. [Phase 16 — 20+ World-Class Advanced Features](#phase-16)
19. [Phase 17 — Deployment & Infra](#phase-17)
20. [Phase 18 — Tests & Release Gates](#phase-18)
21. [Environment Variables Reference](#env-vars)
22. [Production Readiness Checklist](#prod-checklist)

---

## Monorepo Structure {#monorepo-structure}

### New directories to create

```
apps/
  admin/                                  ← NEW: admin.fetchium.com app
    package.json
    next.config.ts
    tsconfig.json
    tailwind.config.ts
    postcss.config.mjs
    .env.example
    src/
      app/
        layout.tsx                        ← Root layout (dark theme, admin brand)
        login/
          page.tsx                        ← Staff login (email + password + TOTP)
        (admin)/
          layout.tsx                      ← Admin shell (sidebar + topbar + RBAC gate)
          overview/page.tsx               ← KPI command center
          orgs/
            page.tsx                      ← Org directory
            [id]/page.tsx                 ← Org profile (deep detail)
          users/
            page.tsx                      ← User directory
            [id]/page.tsx                 ← User profile
          keys/
            page.tsx                      ← API key inventory
            [id]/page.tsx                 ← Key detail + trace
          usage/
            page.tsx                      ← Usage explorer (multi-dim)
            forensics/page.tsx            ← Request forensics (by request ID)
          billing/
            page.tsx                      ← Billing operations
            [orgId]/page.tsx              ← Per-org billing detail
            invoices/page.tsx             ← Invoice history
            webhooks/page.tsx             ← Webhook event log + replay
          crm/
            page.tsx                      ← Account health + lifecycle
            [orgId]/page.tsx              ← Account timeline
          support/
            page.tsx                      ← Ticket queue
            [ticketId]/page.tsx           ← Ticket thread
          incidents/
            page.tsx                      ← Incident list
            [id]/page.tsx                 ← Incident timeline + postmortem
          campaigns/
            page.tsx                      ← Campaign + attribution
            funnels/page.tsx              ← Conversion funnel drilldown
          proxy/
            page.tsx                      ← Proxy pool ops + geo map
          audit/
            page.tsx                      ← Immutable audit log
          flags/
            page.tsx                      ← Feature flags + rollout controls
          settings/
            page.tsx                      ← Admin app settings
            staff/page.tsx               ← Staff user management
            sessions/page.tsx            ← Active sessions + revocation
        api/
          auth/[...nextauth]/route.ts     ← NextAuth for admin staff
          admin/
            orgs/route.ts
            orgs/[id]/route.ts
            users/route.ts
            users/[id]/route.ts
            keys/route.ts
            keys/[id]/route.ts
            usage/route.ts
            usage/forensics/route.ts
            billing/route.ts
            billing/[orgId]/route.ts
            billing/webhooks/route.ts
            crm/route.ts
            crm/[orgId]/route.ts
            support/route.ts
            support/[id]/route.ts
            incidents/route.ts
            incidents/[id]/route.ts
            campaigns/route.ts
            audit/route.ts
            flags/route.ts
            metrics/route.ts             ← Real-time metrics feed
            search/route.ts              ← Universal entity search
            export/route.ts             ← CSV/JSON export endpoint
      components/
        ui/                              ← Primitive components (button, badge, input)
        layout/
          AdminShell.tsx
          Sidebar.tsx
          TopBar.tsx
          CommandPalette.tsx             ← Cmd+K universal search + actions
          NotificationCenter.tsx
        tables/
          DataTable.tsx                  ← Sortable, filterable, exportable
          BulkActions.tsx
        charts/
          UsageTrendChart.tsx
          FunnelChart.tsx
          HeatmapChart.tsx
          RealtimeChart.tsx             ← WebSocket-fed live chart
        entity/
          OrgCard.tsx
          UserCard.tsx
          KeyBadge.tsx
          HealthScore.tsx
          AuditDiffViewer.tsx
        modals/
          ConfirmDestructive.tsx        ← Step-up auth re-confirm modal
          ApprovalWorkflow.tsx
          ExportModal.tsx
          BulkOperationModal.tsx
      hooks/
        useAdminSession.ts
        usePermission.ts
        useRealtimeMetrics.ts           ← WebSocket hook
        useCommandPalette.ts
        useSavedFilters.ts
        useKeyboardShortcuts.ts
        useBulkSelect.ts
        useLiveTail.ts
      lib/
        admin-api.ts                    ← Type-safe admin API client
        permissions.ts                  ← Client-side permission helpers
        health-score.ts                 ← Health score calculation
        export.ts                       ← CSV/JSON export logic
        shortcuts.ts                    ← Keyboard shortcut registry
        realtime.ts                     ← WebSocket connection manager
      types/
        admin.ts                        ← All admin entity types
        permissions.ts                  ← Permission atom types
        next-auth.d.ts
      auth.ts                           ← NextAuth staff config
      middleware.ts                     ← Admin route protection

crates/fetchium-api/src/
  admin/                               ← NEW: admin backend module
    mod.rs
    auth.rs                            ← Admin user auth + TOTP + sessions
    rbac.rs                            ← Permission enforcement
    db.rs                              ← AdminDb (SQLite) + migrations
    orgs.rs                            ← Org CRUD + search
    users.rs                           ← User ops (suspend, force-logout)
    keys.rs                            ← Key inventory + revoke
    usage.rs                           ← Usage deep-dive + forensics
    billing.rs                         ← Billing ops + credits
    crm.rs                             ← CRM lifecycle + health score
    support.rs                         ← Ticket ops + notes
    incidents.rs                       ← Incident management
    campaigns.rs                       ← Attribution + UTM tracking
    audit.rs                           ← Audit log writer + reader
    flags.rs                           ← Feature flag CRUD
    metrics.rs                         ← Real-time metrics aggregator
    proxy_ops.rs                       ← Advanced proxy management
    anomaly.rs                         ← Anomaly detection engine
    websocket.rs                       ← WebSocket handler for live feeds
    export.rs                          ← Data export engine
    approval.rs                        ← Multi-step approval workflows

infra/
  admin/
    docker-compose.yml                 ← Admin app container
    Dockerfile                         ← Admin Next.js build
    nginx.conf                         ← Admin-specific nginx
    traefik-admin.yml                  ← Traefik routing for admin.fetchium.com

scripts/
  admin-seed.sh                        ← Seed admin DB with test data
  admin-first-user.sh                  ← Create first owner account
```

---

## Phase 0 — Data Model & Permission Spec {#phase-0}

**Goal:** Write the canonical data model and permission matrix before touching any code.

### AD-P0-T1: Define canonical entity model

- [ ] Document in `docs/admin-data-model.md`:

```
Entities and relationships:

admin_user
  id, email, password_hash, totp_secret, totp_enabled,
  role (owner|ops|support|finance|growth|readonly),
  name, created_at, last_login_at, is_active

admin_session
  id, admin_user_id, token_hash, ip, user_agent,
  created_at, expires_at, revoked_at, step_up_at

organization (maps 1-to-1 with an account in auth.db)
  id, name, slug, plan, status (active|suspended|churned|trial),
  mrr_cents, created_at, owner_email,
  lifecycle_stage, csm_admin_id, health_score, health_signals

organization_member
  org_id, user_id, role (owner|member|viewer), joined_at

customer_user
  id, email, org_id, role, created_at, last_active_at,
  email_verified, is_suspended, suspension_reason

subscription
  id, org_id, plan, status, stripe_subscription_id (or paddle),
  current_period_start, current_period_end, trial_end,
  cancel_at, canceled_at, mrr_cents

invoice
  id, org_id, subscription_id, amount_cents, currency,
  status (draft|open|paid|void|uncollectible),
  due_date, paid_at, stripe_invoice_id, pdf_url

credits_ledger
  id, org_id, actor_admin_id, delta_cents, reason,
  invoice_id (nullable), created_at

payment_event
  id, org_id, invoice_id, event_type, amount_cents,
  failure_reason, provider_event_id, created_at

support_ticket
  id, org_id, user_id, subject, status (open|pending|resolved|closed),
  priority (low|normal|high|urgent), assignee_admin_id,
  sla_due_at, resolved_at, tags[], created_at

support_note
  id, ticket_id, author_admin_id, body, is_internal,
  created_at, edited_at

support_macro
  id, name, body, tags[], author_admin_id, use_count

campaign
  id, name, source, medium, utm_content, utm_term,
  budget_cents, status, start_date, end_date

attribution_touch
  id, org_id, campaign_id, touch_type (first|last|assist),
  event_name (visit|signup|key_created|first_request|upgrade),
  landed_at

incident
  id, title, severity (critical|high|medium|low),
  status (investigating|identified|monitoring|resolved),
  owner_admin_id, impacted_endpoints[], affected_org_count,
  started_at, resolved_at, postmortem_url

incident_timeline
  id, incident_id, actor_admin_id, event_type, body, created_at

audit_event
  id, admin_user_id, role_at_time, target_type, target_id,
  action, before_snapshot (JSON), after_snapshot (JSON),
  ip, user_agent, timestamp

feature_flag
  id, key, description, enabled_globally, rollout_pct,
  plan_overrides (JSON), org_overrides (JSON),
  owner_admin_id, updated_at

crm_account (extends organization lifecycle data)
  org_id, lifecycle_stage (lead|prospect|trial|active|expansion|churn_risk|churned),
  health_score (0-100), csm_id, arr_cents, renewal_date,
  churn_probability, last_contacted_at, activation_milestone

crm_note
  id, org_id, author_admin_id, body, created_at

crm_activity
  id, org_id, activity_type, description, created_at
```

### AD-P0-T2: Define permission atoms

- [ ] Document in `docs/admin-permissions.md`:

```
Permission atoms (format: resource.action):

Keys:
  keys.read           View any API key (masked)
  keys.reveal         Reveal full prefix (support+)
  keys.create         Create key for org
  keys.revoke         Revoke key
  keys.rotate         Rotate key

Orgs:
  orgs.read           View org list + profiles
  orgs.suspend        Suspend/reactivate org
  orgs.plan_change    Change plan
  orgs.quota_override Override rate/quota limits
  orgs.delete         Permanent deletion (owner only)

Users:
  users.read          View user list + profiles
  users.suspend       Suspend/reactivate user
  users.force_logout  Invalidate all sessions

Billing:
  billing.read        View billing state
  billing.refund      Issue refund
  billing.credit      Apply credit
  billing.plan_override Manually set plan state
  billing.invoice_void  Void invoice

Support:
  support.read        View tickets + notes
  support.reply       Create internal notes + assign
  support.close       Resolve/close ticket
  support.delete      Delete note (owner only)

CRM:
  crm.read            View CRM data
  crm.write           Update lifecycle stage, notes

Incidents:
  incidents.read      View incident list
  incidents.manage    Create/update/resolve incidents
  incidents.postmortem Write postmortem

Audit:
  audit.read          View audit log

Flags:
  flags.read          View feature flags
  flags.write         Modify flags
  flags.dangerous     Toggle kill-switch flags (owner only)

Proxy:
  proxy.read          View proxy stats
  proxy.reset         Reset proxy pool
  proxy.purge         Purge all cached IPs

Campaigns:
  campaigns.read      View attribution data
  campaigns.write     Create/edit campaigns

Admin Settings:
  admin.staff_manage  Add/remove staff, change roles
  admin.session_revoke Revoke any admin session
  admin.secrets_rotate Rotate admin secrets
```

- [ ] Document permission-to-role matrix:

```
Permission          owner  ops  support  finance  growth  readonly
keys.read             ✓     ✓      ✓       ✓        -        ✓
keys.reveal           ✓     ✓      ✓       -        -        -
keys.create           ✓     ✓      ✓       -        -        -
keys.revoke           ✓     ✓      -       -        -        -
keys.rotate           ✓     ✓      -       -        -        -
orgs.read             ✓     ✓      ✓       ✓        ✓        ✓
orgs.suspend          ✓     ✓      -       -        -        -
orgs.plan_change      ✓     ✓      -       ✓        -        -
orgs.quota_override   ✓     ✓      -       -        -        -
orgs.delete           ✓     -      -       -        -        -
users.read            ✓     ✓      ✓       ✓        ✓        ✓
users.suspend         ✓     ✓      -       -        -        -
users.force_logout    ✓     ✓      -       -        -        -
billing.read          ✓     -      -       ✓        -        -
billing.refund        ✓     -      -       ✓        -        -
billing.credit        ✓     -      -       ✓        -        -
billing.plan_override ✓     -      -       -        -        -
billing.invoice_void  ✓     -      -       ✓        -        -
support.read          ✓     ✓      ✓       -        -        ✓
support.reply         ✓     ✓      ✓       -        -        -
support.close         ✓     ✓      ✓       -        -        -
support.delete        ✓     -      -       -        -        -
crm.read              ✓     -      ✓       ✓        ✓        ✓
crm.write             ✓     -      ✓       -        ✓        -
incidents.read        ✓     ✓      ✓       -        -        ✓
incidents.manage      ✓     ✓      -       -        -        -
incidents.postmortem  ✓     ✓      -       -        -        -
audit.read            ✓     ✓      -       -        -        ✓
flags.read            ✓     ✓      -       -        -        ✓
flags.write           ✓     ✓      -       -        -        -
flags.dangerous       ✓     -      -       -        -        -
proxy.read            ✓     ✓      -       -        -        -
proxy.reset           ✓     ✓      -       -        -        -
proxy.purge           ✓     -      -       -        -        -
campaigns.read        ✓     -      -       ✓        ✓        ✓
campaigns.write       ✓     -      -       -        ✓        -
admin.staff_manage    ✓     -      -       -        -        -
admin.session_revoke  ✓     ✓      -       -        -        -
admin.secrets_rotate  ✓     -      -       -        -        -
```

**Acceptance criteria:**
- [ ] Data model doc covers every entity and its foreign key relationships
- [ ] Permission atoms cover every mutable admin action
- [ ] Role matrix has no ambiguous overlaps
- [ ] Every planned admin page maps to at least one permission atom

---

## Phase 1 — Admin App Scaffold {#phase-1}

**Goal:** Create `apps/admin` as a separate Next.js 15 app in the Turborepo monorepo.

### AD-P1-T1: Initialize apps/admin package

- [ ] Create `apps/admin/package.json`:

```json
{
  "name": "@fetchium/admin",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev --port 3300",
    "build": "next build",
    "start": "next start --port 3300",
    "lint": "next lint",
    "type-check": "tsc --noEmit"
  },
  "dependencies": {
    "next": "15.1.6",
    "react": "19.0.0",
    "react-dom": "19.0.0",
    "next-auth": "5.0.0-beta.25",
    "@radix-ui/react-dialog": "^1.1.4",
    "@radix-ui/react-dropdown-menu": "^2.1.4",
    "@radix-ui/react-popover": "^1.1.4",
    "@radix-ui/react-select": "^2.1.4",
    "@radix-ui/react-tabs": "^1.1.2",
    "@radix-ui/react-tooltip": "^1.1.6",
    "@radix-ui/react-switch": "^1.1.2",
    "@radix-ui/react-avatar": "^1.1.2",
    "@radix-ui/react-badge": "^1.1.2",
    "recharts": "^2.15.0",
    "swr": "^2.3.2",
    "cmdk": "^1.0.4",
    "date-fns": "^4.1.0",
    "zod": "^3.24.1",
    "otpauth": "^9.3.6",
    "qrcode": "^1.5.4",
    "@tanstack/react-table": "^8.21.3",
    "papaparse": "^5.4.1",
    "fuse.js": "^7.0.0",
    "lucide-react": "^0.469.0",
    "clsx": "^2.1.1",
    "tailwind-merge": "^2.6.0"
  },
  "devDependencies": {
    "typescript": "^5.7.3",
    "@types/node": "^22.10.7",
    "@types/react": "^19.0.7",
    "@types/react-dom": "^19.0.3",
    "@types/qrcode": "^1.5.5",
    "@types/papaparse": "^5.3.15",
    "tailwindcss": "^3.4.17",
    "postcss": "^8.5.1",
    "autoprefixer": "^10.4.20",
    "eslint": "^9.17.0",
    "eslint-config-next": "15.1.6"
  }
}
```

### AD-P1-T2: Configure Next.js for admin domain

- [ ] Create `apps/admin/next.config.ts`:

```typescript
import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          { key: 'X-Frame-Options', value: 'DENY' },
          { key: 'X-Content-Type-Options', value: 'nosniff' },
          { key: 'X-XSS-Protection', value: '1; mode=block' },
          { key: 'Referrer-Policy', value: 'strict-origin' },
          { key: 'Strict-Transport-Security', value: 'max-age=63072000; includeSubDomains; preload' },
          { key: 'X-Robots-Tag', value: 'noindex, nofollow, noarchive' },
          {
            key: 'Content-Security-Policy',
            value: [
              "default-src 'self'",
              "script-src 'self' 'unsafe-inline'",
              "style-src 'self' 'unsafe-inline'",
              "img-src 'self' data: blob:",
              "connect-src 'self' wss://admin.fetchium.com ***REMOVED***",
              "font-src 'self'",
              "frame-src 'none'",
              "object-src 'none'",
              "base-uri 'self'",
            ].join('; '),
          },
          { key: 'Permissions-Policy', value: 'camera=(); microphone=(); geolocation=(); payment=()' },
        ],
      },
    ]
  },
  async rewrites() {
    return [{ source: '/', destination: '/overview' }]
  },
  // Never expose admin routes to public search
  async redirects() {
    return []
  },
}

export default nextConfig
```

### AD-P1-T3: Add admin to Turborepo pipeline

- [ ] Edit `turbo.json` to include `@fetchium/admin` in build and dev tasks:

```json
{
  "$schema": "https://turbo.build/schema.json",
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": [".next/**", "!.next/cache/**"]
    },
    "dev": {
      "persistent": true,
      "cache": false
    }
  }
}
```

### AD-P1-T4: Create admin root layout with dark operational theme

- [ ] Create `apps/admin/src/app/layout.tsx`:

```typescript
import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import './globals.css'

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' })

export const metadata: Metadata = {
  title: 'Fetchium Admin',
  description: 'Internal operations console',
  robots: { index: false, follow: false },
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.variable} font-sans bg-zinc-950 text-zinc-100 antialiased`}>
        {children}
      </body>
    </html>
  )
}
```

### AD-P1-T5: Create robots.txt blocking all crawlers

- [ ] Create `apps/admin/src/app/robots.txt/route.ts`:

```typescript
export async function GET() {
  return new Response('User-agent: *\nDisallow: /\n', {
    headers: { 'Content-Type': 'text/plain' },
  })
}
```

### AD-P1-T6: Create .env.example

- [ ] Create `apps/admin/.env.example`:

```env
# Admin app URL
NEXTAUTH_URL=https://admin.fetchium.com
NEXTAUTH_SECRET=                        # 32+ random bytes, different from dashboard

# Fetchium API (internal, never exposed to browser)
***REMOVED***=***REMOVED***
FETCHIUM_***REMOVED***=         # For admin session signing

# Admin DB (separate from auth.db)
ADMIN_DB_PATH=/data/admin.db

# Email (for password reset + alerts)
SMTP_HOST=
SMTP_PORT=587
SMTP_USER=
SMTP_PASS=
SMTP_FROM=admin-alerts@fetchium.com

# Optional: Sentry
SENTRY_DSN=
SENTRY_ENVIRONMENT=production

# Optional: Slack alerts
SLACK_WEBHOOK_SECURITY=
SLACK_WEBHOOK_INCIDENTS=
```

**Acceptance criteria:**
- [ ] `cd apps/admin && npm run dev` starts on port 3300
- [ ] `npm run build` completes with no TypeScript errors
- [ ] Admin app responds on `localhost:3300`
- [ ] robots.txt returns `Disallow: /` for all agents
- [ ] CSP header present on all responses, blocks frame embedding

---

## Phase 2 — Admin Auth (TOTP + Sessions) {#phase-2}

**Goal:** Staff login with email/password + TOTP 2FA + session management.

### AD-P2-T1: Create admin user auth module in Rust

- [ ] Create `crates/fetchium-api/src/admin/auth.rs`:

```rust
// Tables managed: admin_users, admin_sessions, admin_backup_codes
// Key types: AdminUser, AdminSession, AdminRole

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct AdminUser {
    pub id: String,              // ulid
    pub email: String,
    pub password_hash: String,   // argon2id
    pub role: AdminRole,
    pub name: String,
    pub totp_secret: Option<String>,  // base32 encoded
    pub totp_enabled: bool,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum AdminRole {
    Owner,
    Ops,
    Support,
    Finance,
    Growth,
    Readonly,
}

pub struct AdminSession {
    pub id: String,             // ulid session ID
    pub admin_user_id: String,
    pub token_hash: String,     // SHA-256 of the session token
    pub ip: String,
    pub user_agent: String,
    pub created_at: String,
    pub expires_at: String,     // 8h rolling, extend on activity
    pub revoked_at: Option<String>,
    pub step_up_at: Option<String>,  // last step-up auth timestamp
    pub step_up_expires_at: Option<String>,
}

// Key functions:
// authenticate(email, password) -> Result<AdminUser>
// validate_totp(user_id, code) -> Result<bool>
// create_session(user_id, ip, ua) -> Result<(session_id, token)>
// validate_session(token) -> Result<AdminUser>
// revoke_session(session_id) -> Result<()>
// require_step_up(session_id) -> Result<bool>  (within 5 min window)
// generate_backup_codes(user_id) -> Result<Vec<String>>
// invalidate_totp_code(user_id, code) -> Result<()>  (prevent replay)
```

### AD-P2-T2: Create NextAuth config for staff

- [ ] Create `apps/admin/src/auth.ts`:

```typescript
import NextAuth from 'next-auth'
import Credentials from 'next-auth/providers/credentials'
import { z } from 'zod'

const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  totpCode: z.string().optional(),
})

export const { handlers, auth, signIn, signOut } = NextAuth({
  session: { strategy: 'jwt', maxAge: 8 * 60 * 60 },
  pages: { signIn: '/login' },
  providers: [
    Credentials({
      async authorize(credentials) {
        const parsed = loginSchema.safeParse(credentials)
        if (!parsed.success) return null

        const res = await fetch(`${process.env.***REMOVED***}/internal/admin/auth/login`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(parsed.data),
        })

        if (!res.ok) return null

        const data = await res.json()
        // data: { id, email, name, role, sessionToken, totpRequired }
        if (data.totpRequired && !credentials.totpCode) {
          throw new Error('TOTP_REQUIRED')
        }
        return data
      },
    }),
  ],
  callbacks: {
    jwt({ token, user }) {
      if (user) {
        token.adminId = user.id
        token.role = user.role
        token.sessionToken = user.sessionToken
        token.email = user.email
        token.name = user.name
      }
      return token
    },
    session({ session, token }) {
      session.user.adminId = token.adminId as string
      session.user.role = token.role as string
      session.user.sessionToken = token.sessionToken as string
      return session
    },
  },
})
```

### AD-P2-T3: Admin middleware (protect all routes)

- [ ] Create `apps/admin/src/middleware.ts`:

```typescript
import { auth } from './auth'
import { NextResponse } from 'next/server'

export default auth((req) => {
  const { pathname } = req.nextUrl
  if (pathname === '/login' || pathname.startsWith('/api/auth')) return

  if (!req.auth) {
    const loginUrl = new URL('/login', req.url)
    loginUrl.searchParams.set('callbackUrl', pathname)
    return NextResponse.redirect(loginUrl)
  }
})

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico|robots.txt).*)'],
}
```

### AD-P2-T4: Login page with TOTP step

- [ ] Create `apps/admin/src/app/login/page.tsx`:
  - Email + password fields
  - TOTP code field (shown after first step validates)
  - Two-step flow: step 1 = credentials, step 2 = TOTP (if enabled)
  - Error states: invalid credentials, TOTP required, TOTP invalid, account suspended
  - Rate limit feedback (too many attempts)
  - No registration link (staff accounts created by owner only)

### AD-P2-T5: TOTP enrollment page

- [ ] Create `apps/admin/src/app/(admin)/settings/sessions/page.tsx`:
  - Show QR code for TOTP app (Google Authenticator, Authy)
  - Confirm with first code before enabling
  - Generate + show backup codes (one-time, hash-stored)
  - Session list: IP, UA, created, last active + revoke button
  - Step-up auth required for all mutations on this page

### AD-P2-T6: Rust admin auth API routes

- [ ] Add to `crates/fetchium-api/src/routes.rs`:

```rust
// Internal admin auth routes (no X-Admin-Secret, uses session cookies)
router.route("/internal/admin/auth/login", post(admin::auth::login))
router.route("/internal/admin/auth/logout", post(admin::auth::logout))
router.route("/internal/admin/auth/totp/setup", post(admin::auth::totp_setup))
router.route("/internal/admin/auth/totp/confirm", post(admin::auth::totp_confirm))
router.route("/internal/admin/auth/totp/verify", post(admin::auth::totp_verify))
router.route("/internal/admin/auth/step-up", post(admin::auth::step_up))
router.route("/internal/admin/auth/backup-codes", post(admin::auth::gen_backup_codes))
router.route("/internal/admin/sessions", get(admin::auth::list_sessions))
router.route("/internal/admin/sessions/:id", delete(admin::auth::revoke_session))
```

**Acceptance criteria:**
- [ ] Login with wrong credentials returns 401 with no info leakage
- [ ] TOTP setup generates valid QR code scannable by Authenticator app
- [ ] TOTP replay attacks blocked (codes invalidated after use)
- [ ] Session expires after 8 hours of inactivity
- [ ] Revoking session in UI immediately blocks further requests
- [ ] Backup codes work as one-time TOTP replacements
- [ ] Step-up auth required before destructive actions (re-confirms TOTP)
- [ ] Suspicious login (new IP/country) triggers Slack alert

---

## Phase 3 — RBAC & Policy Enforcement {#phase-3}

**Goal:** Single permission layer used by both UI and API backend.

### AD-P3-T1: Permission enforcement middleware (Rust)

- [ ] Create `crates/fetchium-api/src/admin/rbac.rs`:

```rust
pub struct AdminPermission(&'static str);

pub const KEYS_READ: AdminPermission = AdminPermission("keys.read");
pub const KEYS_REVOKE: AdminPermission = AdminPermission("keys.revoke");
pub const ORGS_SUSPEND: AdminPermission = AdminPermission("orgs.suspend");
pub const BILLING_REFUND: AdminPermission = AdminPermission("billing.refund");
// ... all atoms from Phase 0

pub fn has_permission(role: &AdminRole, perm: &AdminPermission) -> bool {
    // Static permission matrix lookup — compile-time checked
}

// Axum extractor: requires auth + permission
pub struct RequirePermission<const PERM: &'static str>(pub AdminUser);

impl<S, const PERM: &'static str> FromRequestParts<S> for RequirePermission<PERM> {
    // 1. Extract admin session from Authorization header
    // 2. Validate session in admin.db
    // 3. Check has_permission(role, PERM)
    // 4. Return 403 + write audit event if denied
}
```

### AD-P3-T2: Permission hook for Next.js UI

- [ ] Create `apps/admin/src/hooks/usePermission.ts`:

```typescript
import { useSession } from 'next-auth/react'
import { PERMISSION_MATRIX } from '@/lib/permissions'

export function usePermission(atom: string): boolean {
  const { data: session } = useSession()
  if (!session?.user?.role) return false
  return PERMISSION_MATRIX[session.user.role]?.includes(atom) ?? false
}
```

### AD-P3-T3: Permission-aware sidebar nav

- [ ] Create `apps/admin/src/components/layout/Sidebar.tsx`:
  - Nav items conditionally rendered based on `usePermission`
  - Role badge displayed next to user name
  - Items: Overview, Orgs, Users, API Keys, Usage, Billing, CRM, Support, Campaigns, Incidents, Proxy, Audit, Flags, Settings

### AD-P3-T4: Server-side page guards

- [ ] Every `(admin)/**/page.tsx` must call:

```typescript
import { auth } from '@/auth'
import { requirePermission } from '@/lib/permissions'
import { redirect } from 'next/navigation'

export default async function Page() {
  const session = await auth()
  if (!session) redirect('/login')
  requirePermission(session.user.role, 'orgs.read') // throws if denied
  // ... render
}
```

**Acceptance criteria:**
- [ ] Support role cannot see billing nav items
- [ ] Finance role cannot see proxy ops
- [ ] Forbidden API calls return 403 (not 404, not 500)
- [ ] Every permission-denied action is written to audit_events
- [ ] Role removal invalidates session within 60 seconds (polling or WebSocket)

---

## Phase 4 — Admin Backend Namespace (Rust) {#phase-4}

**Goal:** Dedicated `/internal/admin/*` routes, session-authenticated, separate from public API.

### AD-P4-T1: Admin module structure

- [ ] Create `crates/fetchium-api/src/admin/mod.rs`:

```rust
pub mod auth;
pub mod rbac;
pub mod db;
pub mod orgs;
pub mod users;
pub mod keys;
pub mod usage;
pub mod billing;
pub mod crm;
pub mod support;
pub mod incidents;
pub mod campaigns;
pub mod audit;
pub mod flags;
pub mod metrics;
pub mod proxy_ops;
pub mod anomaly;
pub mod websocket;
pub mod export;
pub mod approval;
```

### AD-P4-T2: Register admin routes in routes.rs

- [ ] Add admin router to `crates/fetchium-api/src/routes.rs`:

```rust
let admin_router = Router::new()
    // Orgs
    .route("/orgs", get(admin::orgs::list).post(admin::orgs::create))
    .route("/orgs/:id", get(admin::orgs::get).patch(admin::orgs::update))
    .route("/orgs/:id/suspend", post(admin::orgs::suspend))
    .route("/orgs/:id/reactivate", post(admin::orgs::reactivate))
    .route("/orgs/:id/plan", patch(admin::orgs::change_plan))
    .route("/orgs/:id/quota", patch(admin::orgs::override_quota))
    .route("/orgs/:id/keys", get(admin::keys::list_for_org))
    .route("/orgs/:id/usage", get(admin::usage::for_org))
    .route("/orgs/:id/billing", get(admin::billing::for_org))
    .route("/orgs/:id/tickets", get(admin::support::for_org))
    .route("/orgs/:id/crm", get(admin::crm::get).patch(admin::crm::update))
    // Users
    .route("/users", get(admin::users::list))
    .route("/users/:id", get(admin::users::get))
    .route("/users/:id/suspend", post(admin::users::suspend))
    .route("/users/:id/force-logout", post(admin::users::force_logout))
    // Keys
    .route("/keys", get(admin::keys::list).post(admin::keys::create))
    .route("/keys/:id", get(admin::keys::get).delete(admin::keys::revoke))
    .route("/keys/:id/rotate", post(admin::keys::rotate))
    // Usage
    .route("/usage", get(admin::usage::summary))
    .route("/usage/forensics/:request_id", get(admin::usage::forensics))
    .route("/usage/top-orgs", get(admin::usage::top_orgs))
    .route("/usage/heatmap", get(admin::usage::endpoint_heatmap))
    // Billing
    .route("/billing", get(admin::billing::list_subscriptions))
    .route("/billing/:org_id/refund", post(admin::billing::refund))
    .route("/billing/:org_id/credit", post(admin::billing::credit))
    .route("/billing/:org_id/invoices", get(admin::billing::invoices))
    .route("/billing/webhooks", get(admin::billing::webhook_log))
    .route("/billing/webhooks/:id/replay", post(admin::billing::webhook_replay))
    // CRM
    .route("/crm/accounts", get(admin::crm::list))
    .route("/crm/accounts/:org_id", get(admin::crm::get).patch(admin::crm::update))
    .route("/crm/accounts/:org_id/notes", post(admin::crm::add_note))
    // Support
    .route("/support/tickets", get(admin::support::list))
    .route("/support/tickets/:id", get(admin::support::get))
    .route("/support/tickets/:id/notes", post(admin::support::add_note))
    .route("/support/tickets/:id/assign", patch(admin::support::assign))
    .route("/support/tickets/:id/status", patch(admin::support::set_status))
    .route("/support/macros", get(admin::support::list_macros).post(admin::support::create_macro))
    // Incidents
    .route("/incidents", get(admin::incidents::list).post(admin::incidents::create))
    .route("/incidents/:id", get(admin::incidents::get).patch(admin::incidents::update))
    .route("/incidents/:id/timeline", post(admin::incidents::add_timeline))
    .route("/incidents/:id/resolve", post(admin::incidents::resolve))
    // Campaigns
    .route("/campaigns", get(admin::campaigns::list).post(admin::campaigns::create))
    .route("/campaigns/:id", get(admin::campaigns::get))
    .route("/campaigns/attribution", get(admin::campaigns::attribution_report))
    .route("/campaigns/funnel", get(admin::campaigns::funnel))
    // Audit
    .route("/audit", get(admin::audit::list))
    .route("/audit/:id", get(admin::audit::get))
    // Flags
    .route("/flags", get(admin::flags::list).post(admin::flags::create))
    .route("/flags/:id", get(admin::flags::get).patch(admin::flags::update))
    // Metrics
    .route("/metrics/realtime", get(admin::metrics::realtime))
    .route("/metrics/summary", get(admin::metrics::summary))
    .route("/metrics/providers", get(admin::metrics::provider_health))
    // Proxy ops
    .route("/proxy/stats", get(admin::proxy_ops::stats))
    .route("/proxy/reset", post(admin::proxy_ops::reset))
    .route("/proxy/purge", post(admin::proxy_ops::purge))
    .route("/proxy/geo", get(admin::proxy_ops::geo_distribution))
    // Anomaly
    .route("/anomaly/alerts", get(admin::anomaly::alerts))
    .route("/anomaly/tenants", get(admin::anomaly::suspicious_tenants))
    // WebSocket live feeds
    .route("/ws/metrics", get(admin::websocket::metrics_feed))
    .route("/ws/logs/:org_id", get(admin::websocket::live_log_feed))
    // Export
    .route("/export/:entity", get(admin::export::export_csv))
    // Admin staff management
    .route("/staff", get(admin::auth::list_staff).post(admin::auth::create_staff))
    .route("/staff/:id", patch(admin::auth::update_staff).delete(admin::auth::remove_staff))
    .route("/staff/:id/sessions", get(admin::auth::staff_sessions).delete(admin::auth::revoke_all_sessions))
    // Approval workflows
    .route("/approvals", get(admin::approval::list).post(admin::approval::create))
    .route("/approvals/:id/approve", post(admin::approval::approve))
    .route("/approvals/:id/reject", post(admin::approval::reject))
    // Require admin session on all routes
    .layer(AdminSessionLayer);

app.nest("/internal/admin", admin_router)
```

**Acceptance criteria:**
- [ ] All `/internal/admin/*` routes require a valid admin session JWT
- [ ] Every route checks the appropriate permission atom
- [ ] Failed permission checks write to audit_events
- [ ] Public `/v1/*` routes unchanged
- [ ] No admin route is reachable without `Authorization: Bearer <admin_session_token>`

---

## Phase 5 — Admin SQLite Schema {#phase-5}

**Goal:** Separate `admin.db` with all admin-domain tables + migrations.

### AD-P5-T1: Create AdminDb with migrations

- [ ] Create `crates/fetchium-api/src/admin/db.rs`:

```rust
// Migration 0001: admin_users + admin_sessions + admin_backup_codes
CREATE TABLE admin_users (
    id                  TEXT PRIMARY KEY,       -- ulid
    email               TEXT NOT NULL UNIQUE,
    password_hash       TEXT NOT NULL,          -- argon2id
    role                TEXT NOT NULL,
    name                TEXT NOT NULL,
    totp_secret         TEXT,                   -- base32 encoded
    totp_enabled        INTEGER NOT NULL DEFAULT 0,
    is_active           INTEGER NOT NULL DEFAULT 1,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    last_login_at       TEXT,
    last_login_ip       TEXT
);

CREATE TABLE admin_sessions (
    id                  TEXT PRIMARY KEY,
    admin_user_id       TEXT NOT NULL REFERENCES admin_users(id),
    token_hash          TEXT NOT NULL UNIQUE,
    ip                  TEXT NOT NULL,
    user_agent          TEXT NOT NULL,
    created_at          TEXT NOT NULL,
    last_active_at      TEXT NOT NULL,
    expires_at          TEXT NOT NULL,
    revoked_at          TEXT,
    step_up_at          TEXT,
    step_up_expires_at  TEXT
);

CREATE TABLE admin_backup_codes (
    id                  TEXT PRIMARY KEY,
    admin_user_id       TEXT NOT NULL REFERENCES admin_users(id),
    code_hash           TEXT NOT NULL,
    used_at             TEXT
);

CREATE TABLE admin_totp_used_codes (
    admin_user_id       TEXT NOT NULL,
    code                TEXT NOT NULL,
    used_at             TEXT NOT NULL,
    PRIMARY KEY (admin_user_id, code)
);

-- Migration 0002: organizations + members
CREATE TABLE organizations (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    slug                TEXT NOT NULL UNIQUE,
    plan                TEXT NOT NULL DEFAULT 'free',
    status              TEXT NOT NULL DEFAULT 'active',
    mrr_cents           INTEGER NOT NULL DEFAULT 0,
    owner_email         TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE organization_members (
    org_id              TEXT NOT NULL REFERENCES organizations(id),
    user_id             TEXT NOT NULL,
    role                TEXT NOT NULL DEFAULT 'member',
    joined_at           TEXT NOT NULL,
    PRIMARY KEY (org_id, user_id)
);

CREATE TABLE customer_users (
    id                  TEXT PRIMARY KEY,
    email               TEXT NOT NULL,
    org_id              TEXT REFERENCES organizations(id),
    role                TEXT NOT NULL DEFAULT 'member',
    created_at          TEXT NOT NULL,
    last_active_at      TEXT,
    email_verified      INTEGER NOT NULL DEFAULT 0,
    is_suspended        INTEGER NOT NULL DEFAULT 0,
    suspension_reason   TEXT
);

-- Migration 0003: subscriptions + billing
CREATE TABLE subscriptions (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT NOT NULL REFERENCES organizations(id),
    plan                TEXT NOT NULL,
    status              TEXT NOT NULL,
    provider_id         TEXT,               -- stripe/paddle subscription id
    current_period_start TEXT NOT NULL,
    current_period_end  TEXT NOT NULL,
    trial_end           TEXT,
    cancel_at           TEXT,
    canceled_at         TEXT,
    mrr_cents           INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE invoices (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT NOT NULL REFERENCES organizations(id),
    subscription_id     TEXT REFERENCES subscriptions(id),
    amount_cents        INTEGER NOT NULL,
    currency            TEXT NOT NULL DEFAULT 'usd',
    status              TEXT NOT NULL,
    due_date            TEXT,
    paid_at             TEXT,
    provider_invoice_id TEXT,
    pdf_url             TEXT,
    created_at          TEXT NOT NULL
);

CREATE TABLE credits_ledger (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT NOT NULL REFERENCES organizations(id),
    actor_admin_id      TEXT REFERENCES admin_users(id),
    delta_cents         INTEGER NOT NULL,
    reason              TEXT NOT NULL,
    invoice_id          TEXT REFERENCES invoices(id),
    created_at          TEXT NOT NULL
);

CREATE TABLE payment_events (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT NOT NULL,
    invoice_id          TEXT,
    event_type          TEXT NOT NULL,
    amount_cents        INTEGER,
    failure_reason      TEXT,
    provider_event_id   TEXT UNIQUE,
    raw_payload         TEXT,               -- JSON, for webhook replay
    processed_at        TEXT,
    created_at          TEXT NOT NULL
);

-- Migration 0004: support
CREATE TABLE support_tickets (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT REFERENCES organizations(id),
    user_id             TEXT,
    subject             TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'open',
    priority            TEXT NOT NULL DEFAULT 'normal',
    assignee_admin_id   TEXT REFERENCES admin_users(id),
    tags                TEXT NOT NULL DEFAULT '[]',  -- JSON array
    sla_due_at          TEXT,
    resolved_at         TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE support_notes (
    id                  TEXT PRIMARY KEY,
    ticket_id           TEXT NOT NULL REFERENCES support_tickets(id),
    author_admin_id     TEXT NOT NULL REFERENCES admin_users(id),
    body                TEXT NOT NULL,
    is_internal         INTEGER NOT NULL DEFAULT 1,
    created_at          TEXT NOT NULL,
    edited_at           TEXT
);

CREATE TABLE support_macros (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    body                TEXT NOT NULL,
    tags                TEXT NOT NULL DEFAULT '[]',
    author_admin_id     TEXT REFERENCES admin_users(id),
    use_count           INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL
);

-- Migration 0005: CRM
CREATE TABLE crm_accounts (
    org_id              TEXT PRIMARY KEY REFERENCES organizations(id),
    lifecycle_stage     TEXT NOT NULL DEFAULT 'prospect',
    health_score        INTEGER NOT NULL DEFAULT 50,
    health_signals      TEXT NOT NULL DEFAULT '{}',  -- JSON
    csm_id              TEXT REFERENCES admin_users(id),
    arr_cents           INTEGER NOT NULL DEFAULT 0,
    renewal_date        TEXT,
    churn_probability   REAL NOT NULL DEFAULT 0.0,
    last_contacted_at   TEXT,
    activation_milestone TEXT,
    updated_at          TEXT NOT NULL
);

CREATE TABLE crm_notes (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT NOT NULL REFERENCES organizations(id),
    author_admin_id     TEXT NOT NULL REFERENCES admin_users(id),
    body                TEXT NOT NULL,
    created_at          TEXT NOT NULL
);

-- Migration 0006: campaigns + attribution
CREATE TABLE campaigns (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    source              TEXT,
    medium              TEXT,
    utm_content         TEXT,
    utm_term            TEXT,
    budget_cents        INTEGER,
    status              TEXT NOT NULL DEFAULT 'active',
    start_date          TEXT,
    end_date            TEXT,
    created_at          TEXT NOT NULL
);

CREATE TABLE attribution_touches (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT REFERENCES organizations(id),
    campaign_id         TEXT REFERENCES campaigns(id),
    touch_type          TEXT NOT NULL,  -- first|last|assist
    event_name          TEXT NOT NULL,  -- visit|signup|key_created|first_request|upgrade
    metadata            TEXT,           -- JSON (page, referrer, etc.)
    landed_at           TEXT NOT NULL
);

-- Migration 0007: incidents
CREATE TABLE incidents (
    id                  TEXT PRIMARY KEY,
    title               TEXT NOT NULL,
    severity            TEXT NOT NULL DEFAULT 'medium',
    status              TEXT NOT NULL DEFAULT 'investigating',
    owner_admin_id      TEXT REFERENCES admin_users(id),
    impacted_endpoints  TEXT NOT NULL DEFAULT '[]',  -- JSON
    affected_org_count  INTEGER,
    started_at          TEXT NOT NULL,
    resolved_at         TEXT,
    postmortem_url      TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE incident_timeline (
    id                  TEXT PRIMARY KEY,
    incident_id         TEXT NOT NULL REFERENCES incidents(id),
    actor_admin_id      TEXT REFERENCES admin_users(id),
    event_type          TEXT NOT NULL,   -- update|status_change|mitigation|resolution
    body                TEXT NOT NULL,
    created_at          TEXT NOT NULL
);

-- Migration 0008: audit + flags
CREATE TABLE audit_events (
    id                  TEXT PRIMARY KEY,
    admin_user_id       TEXT,            -- null for system events
    role_at_time        TEXT,
    target_type         TEXT NOT NULL,
    target_id           TEXT,
    action              TEXT NOT NULL,
    before_snapshot     TEXT,            -- JSON
    after_snapshot      TEXT,            -- JSON
    ip                  TEXT,
    user_agent          TEXT,
    metadata            TEXT,            -- JSON (extra context)
    timestamp           TEXT NOT NULL
);

CREATE INDEX idx_audit_timestamp ON audit_events(timestamp DESC);
CREATE INDEX idx_audit_admin ON audit_events(admin_user_id, timestamp DESC);
CREATE INDEX idx_audit_target ON audit_events(target_type, target_id);

CREATE TABLE feature_flags (
    id                  TEXT PRIMARY KEY,
    key                 TEXT NOT NULL UNIQUE,
    description         TEXT,
    enabled_globally    INTEGER NOT NULL DEFAULT 0,
    rollout_pct         INTEGER NOT NULL DEFAULT 0,
    plan_overrides      TEXT NOT NULL DEFAULT '{}',  -- JSON
    org_overrides       TEXT NOT NULL DEFAULT '{}',  -- JSON
    is_dangerous        INTEGER NOT NULL DEFAULT 0,
    owner_admin_id      TEXT REFERENCES admin_users(id),
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

-- Migration 0009: approval workflows
CREATE TABLE approval_requests (
    id                  TEXT PRIMARY KEY,
    requester_id        TEXT NOT NULL REFERENCES admin_users(id),
    action_type         TEXT NOT NULL,   -- refund|plan_override|bulk_revoke|secret_rotation|etc
    target_type         TEXT,
    target_id           TEXT,
    payload             TEXT NOT NULL,   -- JSON: action parameters
    justification       TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending',
    reviewer_id         TEXT REFERENCES admin_users(id),
    review_note         TEXT,
    created_at          TEXT NOT NULL,
    reviewed_at         TEXT,
    expires_at          TEXT NOT NULL
);
```

**Acceptance criteria:**
- [ ] All migrations run cleanly on fresh DB
- [ ] Every table has appropriate indexes
- [ ] Foreign key constraints enforced (`PRAGMA foreign_keys = ON`)
- [ ] All TEXT timestamps use RFC3339 format consistently
- [ ] JSON columns always store valid JSON (validated at insert)

---

## Phase 6 — Org & User Management {#phase-6}

**Goal:** Full org/user lifecycle CRUD with search, filtering, and bulk ops.

### AD-P6-T1: Org directory page

- [ ] Create `apps/admin/src/app/(admin)/orgs/page.tsx`:
  - Search by name, email, plan, status
  - Columns: Name, Plan, MRR, Status, Health, Members, Created, Actions
  - Filters: plan (free/starter/pro/enterprise), status (active/suspended/trial/churned), health (<50 / 50-75 / >75)
  - Sortable columns (server-side)
  - Pagination (50/page)
  - Bulk select → Bulk suspend / bulk plan change / CSV export
  - "New Org" button (owner only)

### AD-P6-T2: Org profile page

- [ ] Create `apps/admin/src/app/(admin)/orgs/[id]/page.tsx`:
  - Header: org name, plan badge, status badge, MRR, health score gauge
  - Tabs: Overview | Members | API Keys | Usage | Billing | CRM | Support | Incidents | Audit
  - Overview tab: created date, owner email, lifecycle stage, CSM, quota overrides, feature flag overrides
  - Actions sidebar:
    - Suspend / Reactivate (ops.orgs.suspend)
    - Change Plan → modal with plan selector (finance)
    - Override Quota → modal (ops)
    - Add Credit → modal with amount + reason (finance)
    - Revoke All Keys → confirmation (owner)
    - Force Logout All Users → confirmation (ops)
    - View Audit Trail → filtered to this org

### AD-P6-T3: Rust org handlers

- [ ] Create `crates/fetchium-api/src/admin/orgs.rs`:

```rust
// list() — search + filter + paginate
// GET /internal/admin/orgs?q=&plan=&status=&sort=mrr&dir=desc&page=1&per_page=50
pub async fn list(
    RequirePermission<"orgs.read">: admin_user,
    Query(params): Query<OrgListParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Join organizations + subscriptions + crm_accounts for health score
    // Full-text search on name, slug, owner_email
    // Write audit event: orgs.list with filter params
}

// get() — org detail with all related data
pub async fn get(
    RequirePermission<"orgs.read">: admin_user,
    Path(org_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Fetch org + subscription + crm + members + key count + recent usage
}

// suspend() / reactivate()
pub async fn suspend(RequirePermission<"orgs.suspend">: admin_user, ...) {
    // 1. Write before_snapshot to audit_events
    // 2. Set organizations.status = 'suspended'
    // 3. Write after_snapshot
    // 4. Notify via Slack webhook
}

// change_plan()
pub async fn change_plan(RequirePermission<"orgs.plan_change">: admin_user, ...) {
    // 1. Validate new plan
    // 2. Audit before/after
    // 3. Update auth.db api_keys.plan for all org keys
    // 4. Update admin.db subscriptions.plan
}

// override_quota()
pub async fn override_quota(RequirePermission<"orgs.quota_override">: admin_user, ...) {
    // Store in org_quota_overrides table (to be added to auth.db migration)
    // Checked by rate limiter before plan limits
}
```

### AD-P6-T4: User directory and profile

- [ ] Create `apps/admin/src/app/(admin)/users/page.tsx` — similar to orgs but for users
- [ ] Create `apps/admin/src/app/(admin)/users/[id]/page.tsx`:
  - Email, org, role, last active, suspension status
  - Actions: Suspend, Force Logout, Verify Email, Reset Onboarding
  - Timeline: recent requests, support tickets, org membership history

**Acceptance criteria:**
- [ ] Org search returns results within 200ms for 10k orgs
- [ ] Suspending org immediately blocks all API key auth for that org
- [ ] Plan change reflected in rate limiter within 60 seconds
- [ ] Bulk operations show dry-run preview before execution
- [ ] All mutations produce audit_event rows with before/after snapshots

---

## Phase 7 — API Key & Usage Operations {#phase-7}

**Goal:** Full key inventory + multi-dimensional usage explorer + request forensics.

### AD-P7-T1: API key inventory page

- [ ] Create `apps/admin/src/app/(admin)/keys/page.tsx`:
  - Columns: Key ID, Prefix (masked), Org, Plan, Created, Last Used, Requests/Month, Anomaly Flag, Status
  - Filters: plan, status (active/revoked), has_anomaly, org
  - Search by prefix or org name
  - Actions: Revoke, Rotate, View Usage, View Org

### AD-P7-T2: Usage explorer page

- [ ] Create `apps/admin/src/app/(admin)/usage/page.tsx`:
  - Time range picker (1h / 6h / 24h / 7d / 30d / custom)
  - Breakdowns: by org / by key / by endpoint / by status code / by latency bucket
  - Charts:
    - Request volume over time (line)
    - Status code distribution (bar)
    - Endpoint popularity (horizontal bar)
    - Latency percentiles (p50/p95/p99)
    - Top 10 orgs by volume table
  - Rate-limit event timeline

### AD-P7-T3: Request forensics page

- [ ] Create `apps/admin/src/app/(admin)/usage/forensics/page.tsx`:
  - Search by request_id (jump directly to trace)
  - Shows: timestamp, org, key prefix, endpoint, status, duration_ms, tokens, rate_limit_hit, error
  - Link to org profile from request trace
  - "Copy request ID" button

### AD-P7-T4: Usage Rust handler

- [ ] Create `crates/fetchium-api/src/admin/usage.rs`:

```rust
// summary() — aggregated usage metrics
// GET /internal/admin/usage?from=&to=&group_by=org|key|endpoint|status

// for_org() — per-org usage deep-dive
// GET /internal/admin/usage?org_id=&from=&to=

// forensics() — single request trace
// GET /internal/admin/usage/forensics/:request_id
// Joins usage_logs with api_keys, organizations

// top_orgs() — ranked by request volume
// GET /internal/admin/usage/top-orgs?from=&to=&limit=25

// endpoint_heatmap() — endpoint × hour-of-day saturation matrix
// GET /internal/admin/usage/heatmap
```

**Acceptance criteria:**
- [ ] Request forensics resolves any request ID to org + key within 500ms
- [ ] Revoking a key from the keys page blocks auth within 1 second
- [ ] Usage charts load within 1 second for 30-day queries
- [ ] Export to CSV works for all usage views

---

## Phase 8 — Billing & Finance Operations {#phase-8}

**Goal:** Full billing visibility + manual operations + webhook replay.

### AD-P8-T1: Billing overview page

- [ ] Create `apps/admin/src/app/(admin)/billing/page.tsx`:
  - MRR trend chart (30 days)
  - Failed payments queue with failure reason + retry button
  - Subscription status breakdown (active/trialing/past_due/canceled)
  - New subscriptions vs churn this month
  - Trial conversions rate

### AD-P8-T2: Per-org billing detail

- [ ] Create `apps/admin/src/app/(admin)/billing/[orgId]/page.tsx`:
  - Current subscription: plan, status, period, MRR, trial end
  - Invoice history: downloadable PDFs
  - Credits ledger: all credits applied + reasons
  - Payment events: all webhook events for this org
  - Actions:
    - Issue refund (finance.billing.refund + step-up auth)
    - Apply credit (finance.billing.credit)
    - Change plan (finance.billing.plan_override + approval workflow)
    - Void invoice (finance.billing.invoice_void)

### AD-P8-T3: Webhook event log + replay

- [ ] Create `apps/admin/src/app/(admin)/billing/webhooks/page.tsx`:
  - All billing provider webhook events (Stripe/Paddle)
  - Status: processed/failed/skipped
  - Raw payload viewer (collapsible JSON)
  - Replay button for failed events (idempotent)

### AD-P8-T4: Rust billing handlers

- [ ] Create `crates/fetchium-api/src/admin/billing.rs`:

```rust
// All mutations require:
// 1. RequirePermission<"billing.refund"> (or appropriate atom)
// 2. audit_event write before + after
// 3. Step-up auth for refunds > $100 (configurable threshold)
// 4. Approval workflow for plan_override

pub async fn refund(admin: RequirePermission<"billing.refund">, ...) {
    // Validate step-up token in session
    // Call billing provider API
    // Write credits_ledger row (negative delta)
    // Audit before/after
}

pub async fn webhook_replay(admin: RequirePermission<"billing.read">, ...) {
    // Retrieve raw_payload from payment_events
    // Re-process through billing webhook handler
    // Idempotency via provider_event_id check
}
```

**Acceptance criteria:**
- [ ] Refunds require step-up TOTP re-confirmation
- [ ] All billing mutations produce audit rows with USD amounts
- [ ] Webhook replay is idempotent (cannot double-process)
- [ ] Finance role can see billing but not change plans without approval
- [ ] Credits ledger balances correctly across all operations

---

## Phase 9 — CRM & Revenue Operations {#phase-9}

**Goal:** Lightweight CRM with account health scoring and lifecycle management.

### AD-P9-T1: CRM account list

- [ ] Create `apps/admin/src/app/(admin)/crm/page.tsx`:
  - Columns: Org, Stage, Health Score, ARR, CSM, Last Contacted, Churn Risk
  - Filter by stage, health range, CSM
  - Sort by ARR, health score, last contacted
  - Bulk lifecycle stage update

### AD-P9-T2: Account health score engine

- [ ] Create `crates/fetchium-api/src/admin/crm.rs`:

```rust
pub struct HealthSignals {
    pub request_volume_trend: f32,      // 0-1: rising vs dropping
    pub payment_health: f32,            // 0-1: no failures = 1
    pub support_load: f32,              // 0-1: fewer tickets = higher
    pub feature_adoption: f32,          // 0-1: endpoints used / total
    pub activation_complete: f32,       // 0-1: milestone hit
    pub days_since_last_request: f32,   // 0-1: recent = higher
}

pub fn compute_health_score(signals: &HealthSignals) -> u8 {
    let weights = [0.30, 0.25, 0.15, 0.15, 0.10, 0.05];
    let values = [
        signals.request_volume_trend,
        signals.payment_health,
        signals.support_load,
        signals.feature_adoption,
        signals.activation_complete,
        signals.days_since_last_request,
    ];
    let score = weights.iter().zip(values.iter()).map(|(w, v)| w * v).sum::<f32>();
    (score * 100.0) as u8
}
```

- [ ] Scheduled job: recompute health scores every 6 hours
- [ ] CRM account timeline: org notes + recent tickets + billing events + usage anomalies

### AD-P9-T3: Account timeline view

- [ ] Create `apps/admin/src/app/(admin)/crm/[orgId]/page.tsx`:
  - Unified timeline: notes, tickets, billing events, usage spikes, plan changes
  - Add note button
  - Change lifecycle stage
  - Assign CSM
  - Health score breakdown (drillable signals)

**Acceptance criteria:**
- [ ] Health score updates within 6 hours of usage change
- [ ] CRM data links to org (not orphan records)
- [ ] Growth role can update lifecycle but cannot see billing details
- [ ] Health score breakdown shows all input signals with values

---

## Phase 10 — Support Operations {#phase-10}

**Goal:** Full ticket queue with SLA timers, macros, and customer timeline.

### AD-P10-T1: Ticket queue page

- [ ] Create `apps/admin/src/app/(admin)/support/page.tsx`:
  - Tabs: All / Open / Pending / Urgent / Mine / Unassigned
  - SLA timer countdown (red when breached)
  - Columns: ID, Subject, Org, Priority, Assignee, SLA, Created
  - Bulk assign / bulk close

### AD-P10-T2: Ticket detail page

- [ ] Create `apps/admin/src/app/(admin)/support/[ticketId]/page.tsx`:
  - Thread: notes (internal vs customer-visible indicator)
  - Quick org summary sidebar (plan, health, recent tickets)
  - Actions: Add Note, Assign, Change Priority, Change Status, Escalate
  - Macro selector (saved reply templates)
  - Attach incident reference
  - Link request IDs to forensics
  - Tag taxonomy

### AD-P10-T3: Rust support handlers

- [ ] Create `crates/fetchium-api/src/admin/support.rs`:

```rust
// SLA calculation:
// normal: 24h | high: 8h | urgent: 2h
// sla_due_at = created_at + sla_hours(priority)

// add_note(): writes support_notes, checks is_internal flag
// Increments macro use_count if macro_id provided
// Sends Slack alert to #support-escalations if priority=urgent

// Escalation trigger: auto-create incident if ticket tagged with
// "service-outage" and priority=urgent
```

**Acceptance criteria:**
- [ ] SLA breach triggers Slack alert to `#support-escalations`
- [ ] Internal notes not visible via any customer-facing endpoint
- [ ] Support role cannot delete notes (owner only)
- [ ] Ticket → org → usage forensics reachable in 3 clicks

---

## Phase 17 — Observability & Incident Command {#phase-11}

**Goal:** Real-time metrics dashboard + structured incident management.

### AD-P11-T1: Observability overview page

- [ ] Create `apps/admin/src/app/(admin)/overview/page.tsx`:
  - Live metrics (WebSocket-fed, updates every 5s):
    - Request/s (sparkline)
    - Error rate % (p95/p99 latency)
    - Active jobs (search, research, youtube)
    - Rate limit events/min
    - Queue depth
  - Provider health grid:
    - Google, DDG, Bing, Brave, SearXNG, Gemini, Serper, Exa
    - Each: last response time, last success, current status (green/yellow/red)
  - Proxy pool health:
    - Active IPs, rotation rate, block rate
  - Deploy markers (vertical lines on timeline)
  - Open incidents widget

### AD-P11-T2: Real-time metrics WebSocket (Rust)

- [ ] Create `crates/fetchium-api/src/admin/websocket.rs`:

```rust
// WebSocket handler: /internal/admin/ws/metrics
// Pushes every 5s: {
//   requests_per_second, error_rate, p50_ms, p95_ms, p99_ms,
//   active_jobs, rate_limit_events, queue_depth,
//   provider_health: HashMap<provider, ProviderStatus>,
//   proxy: { active_ips, rotation_rate, block_rate }
// }

// WebSocket handler: /internal/admin/ws/logs/:org_id
// Live tail of usage_logs for a specific org (for support forensics)
// Pushes every new log row matching org_id
```

### AD-P11-T3: Incident management page

- [ ] Create `apps/admin/src/app/(admin)/incidents/page.tsx`:
  - List: open incidents sorted by severity
  - Status badges: INVESTIGATING / IDENTIFIED / MONITORING / RESOLVED
  - Create incident button
  - Auto-open trigger: configurable threshold on error rate

- [ ] Create `apps/admin/src/app/(admin)/incidents/[id]/page.tsx`:
  - Timeline (chronological updates)
  - Add update (body + event_type)
  - Change severity / status
  - Affected endpoint tagger
  - Affected org count (from usage logs during incident period)
  - Resolve with RCA summary
  - Postmortem template auto-generation (AI-assisted)

### AD-P11-T4: Auto-open incident rule

- [ ] Create `crates/fetchium-api/src/admin/anomaly.rs`:

```rust
// Runs every 60s in background task
// If error_rate > 5% for 3 consecutive checks: auto-create incident (severity=high)
// If p99 > 10000ms for 2 checks: auto-create incident (severity=medium)
// Post to Slack #incidents channel with link to admin incident page
// Write to audit_events as system action

pub async fn check_thresholds(state: &AppState) {
    let metrics = collect_current_metrics(state).await;
    if metrics.error_rate > 0.05 {
        create_auto_incident(state, "High error rate", Severity::High, metrics).await;
    }
}
```

**Acceptance criteria:**
- [ ] WebSocket metrics update within 5 seconds of actual traffic changes
- [ ] Incident creation sends Slack notification to #incidents
- [ ] Resolving incident auto-computes affected org count from usage logs
- [ ] Postmortem template includes timeline, severity, affected endpoints, RCA placeholder
- [ ] Provider health shows real data (not stubs)

---

## Phase 12 — Ads & Attribution Analytics {#phase-12}

**Goal:** UTM capture → signup → activation → revenue funnel tracking.

### AD-P12-T1: Attribution ingestion

- [ ] Add to `crates/fetchium-api/src/routes.rs` (PUBLIC, no auth):

```rust
// Called by landing page + docs site + app signup flow
router.route("/v1/track/attribution", post(track_attribution))
// Body: { event, utm_source, utm_medium, utm_campaign, utm_content, utm_term,
//         user_id (if known), org_id (if known), page_url, referrer }
```

- [ ] Store in `attribution_touches` table
- [ ] Cookie-based first-touch preservation (7 day TTL)
- [ ] Link to org_id when signup completes

### AD-P12-T2: Campaign management page

- [ ] Create `apps/admin/src/app/(admin)/campaigns/page.tsx`:
  - Campaign list with: name, source, medium, spend, signups, conversions, MRR attributed
  - Create campaign

### AD-P12-T3: Conversion funnel page

- [ ] Create `apps/admin/src/app/(admin)/campaigns/funnels/page.tsx`:
  - Funnel steps: Visit → Signup → Key Created → First Request → Upgrade
  - Drop-off rates at each step
  - Filter by campaign / time range / plan
  - Cohort comparison (this week vs last week)

### AD-P12-T4: Attribution report (Rust)

- [ ] Create `crates/fetchium-api/src/admin/campaigns.rs`:

```rust
// attribution_report():
// Groups attribution_touches by campaign, counts by event_name
// Joins with subscriptions for revenue attribution
// Returns: { campaign_id, name, visits, signups, conversions, mrr_attributed }

// funnel():
// Counts distinct org_ids at each funnel step
// Calculates step-over-step conversion rates
// Supports time range filtering and campaign filtering
```

**Acceptance criteria:**
- [ ] UTM params captured from all signup and landing page flows
- [ ] Attribution tied to org_id, not orphan visitor records
- [ ] Funnel shows real conversion rates from stored events
- [ ] Campaign attribution report matches between admin and billing data

---

## Phase 13 — Security, Audit & Compliance {#phase-13}

**Goal:** Immutable audit log + security review pages + step-up auth gates.

### AD-P13-T1: Audit log page

- [ ] Create `apps/admin/src/app/(admin)/audit/page.tsx`:
  - Filterable by: admin user, action type, target type, date range
  - Columns: Timestamp, Actor, Role, Action, Target, IP, Result
  - Click row → diff viewer (before/after JSON diff)
  - Export to CSV for compliance

### AD-P13-T2: Audit diff viewer component

- [ ] Create `apps/admin/src/components/entity/AuditDiffViewer.tsx`:
  - Side-by-side JSON diff (before_snapshot vs after_snapshot)
  - Color-coded: added (green), removed (red), changed (yellow)
  - Collapsed by default, expand sections

### AD-P13-T3: Security review dashboard

- [ ] Add security panel to overview:
  - Failed login attempts (last 24h) with IP + UA
  - Revoked key usage attempts (blocked, with IP)
  - Suspicious org behavior (rate limit violations, scraping patterns)
  - Admin actions in last 24h (privileged action count)

### AD-P13-T4: Step-up auth middleware

- [ ] Create `apps/admin/src/components/modals/ConfirmDestructive.tsx`:
  - Modal that appears before any destructive action
  - Requires TOTP code entry (or password for owners without TOTP)
  - Calls `/internal/admin/auth/step-up` to validate + refresh step_up_at
  - Step-up valid for 5 minutes (no re-prompt within window)

### AD-P13-T5: Break-glass access

- [ ] Add to `crates/fetchium-api/src/admin/auth.rs`:

```rust
// break_glass_access(): owner-only
// Generates a single-use 15-minute elevated token with all permissions
// Records to audit_events: { action: "break_glass_activated", mandatory_justification }
// Sends immediate Slack alert to #security channel
```

**Acceptance criteria:**
- [ ] Audit log is append-only (no delete/update endpoints, even for owner)
- [ ] Every mutable admin API call produces an audit row
- [ ] Step-up required for: refunds, plan overrides, bulk revokes, break-glass
- [ ] Failed step-up attempts are logged and counted
- [ ] Raw secrets / full API keys never appear in audit snapshots (masked)

---

## Phase 14 — Feature Flags & Operational Controls {#phase-14}

**Goal:** Admin-managed feature flags with per-org, per-plan overrides + kill switches.

### AD-P14-T1: Feature flags page

- [ ] Create `apps/admin/src/app/(admin)/flags/page.tsx`:
  - List all flags with: key, enabled_globally, rollout_pct, plan_overrides, org_overrides
  - Toggle global enable/disable
  - Set rollout percentage (0-100%)
  - Add plan override: `{ pro: true, free: false }`
  - Add org override: `{ org_id: true/false }`
  - Dangerous flags (kill switches): require owner role + confirmation modal

### AD-P14-T2: Rust flag handler

- [ ] Create `crates/fetchium-api/src/admin/flags.rs`:

```rust
pub async fn evaluate_flag(key: &str, org_id: &str, plan: &str) -> bool {
    let flag = get_flag(key).await?;
    // 1. Check org_overrides first (highest priority)
    // 2. Check plan_overrides
    // 3. Check rollout_pct (hash org_id to 0-100, deterministic)
    // 4. Fall back to enabled_globally
}
```

- [ ] Add flag evaluation endpoint used by API handlers:
  - `GET /internal/flags/:key?org_id=&plan=`

### AD-P14-T3: Operational kill switches

- [ ] Pre-define flags in DB seed:
  - `search.enabled` — disable all search endpoints
  - `research.enabled` — disable research pipeline
  - `proxy.residential_enabled` — force proxy bypass
  - `rate_limit.strict` — lower all rate limits to 10/min emergency mode
  - `new_signups.enabled` — block new key creation
  - `beta.headless_extraction` — enable/disable headless JS extraction

**Acceptance criteria:**
- [ ] Flag evaluation order: org > plan > rollout > global
- [ ] Changing a flag propagates to API within 30 seconds (cache TTL)
- [ ] Dangerous flags require owner role + TOTP step-up
- [ ] All flag changes produce audit rows with before/after values

---

## Phase 15 — Advanced UX {#phase-15}

**Goal:** Command palette, real-time live data, keyboard shortcuts, compact density.

### AD-P15-T1: Command palette (Cmd+K)

- [ ] Create `apps/admin/src/components/layout/CommandPalette.tsx`:
  - Using `cmdk` library
  - Groups:
    - **Navigate**: Overview, Orgs, Users, Keys, Usage, Billing, Support, Incidents, Audit
    - **Search**: "org: <query>", "key: <query>", "user: <query>", "ticket: #<id>", "request: <uuid>"
    - **Actions**: Create Incident, Create Ticket, Add Credit, Revoke Key
  - Keyboard: `Cmd+K` to open, `Esc` to close, arrows to navigate, Enter to select
  - Recent + pinned items
  - Entity jump: type request ID → goes directly to forensics

### AD-P15-T2: Keyboard shortcuts system

- [ ] Create `apps/admin/src/lib/shortcuts.ts`:

```typescript
export const shortcuts = {
  'g o': { label: 'Go to Overview', action: () => navigate('/overview') },
  'g r': { label: 'Go to Orgs', action: () => navigate('/orgs') },
  'g k': { label: 'Go to Keys', action: () => navigate('/keys') },
  'g u': { label: 'Go to Usage', action: () => navigate('/usage') },
  'g b': { label: 'Go to Billing', action: () => navigate('/billing') },
  'g s': { label: 'Go to Support', action: () => navigate('/support') },
  'g i': { label: 'Go to Incidents', action: () => navigate('/incidents') },
  'g a': { label: 'Go to Audit', action: () => navigate('/audit') },
  '/':   { label: 'Focus search', action: () => focusSearch() },
  '?':   { label: 'Show shortcuts', action: () => showShortcutsModal() },
  'n i': { label: 'New Incident', action: () => openCreateIncident() },
  'n t': { label: 'New Ticket Note', action: () => openAddNote() },
  'e':   { label: 'Export current view', action: () => triggerExport() },
  'r':   { label: 'Refresh data', action: () => refreshData() },
}
```

### AD-P15-T3: Real-time charts (WebSocket)

- [ ] Create `apps/admin/src/hooks/useRealtimeMetrics.ts`:

```typescript
export function useRealtimeMetrics() {
  const [metrics, setMetrics] = useState<Metrics | null>(null)

  useEffect(() => {
    const ws = new WebSocket(
      `${process.env.NEXT_PUBLIC_ADMIN_WS_URL}/internal/admin/ws/metrics`,
      ['admin-session', session.user.sessionToken]
    )
    ws.onmessage = (e) => setMetrics(JSON.parse(e.data))
    ws.onclose = () => setTimeout(() => reconnect(), 3000)
    return () => ws.close()
  }, [])

  return metrics
}
```

### AD-P15-T4: Live log tail component

- [ ] Create `apps/admin/src/hooks/useLiveTail.ts`:
  - WebSocket connection to `/internal/admin/ws/logs/:org_id`
  - Ring buffer of last 200 log lines
  - Pause/resume button
  - Filter by endpoint or status code
  - Auto-scroll with "scroll to bottom" anchor

### AD-P15-T5: Data density mode

- [ ] CSS class `data-density="compact"` on body reduces:
  - Row heights from 48px → 36px
  - Font size from 14px → 12px
  - Padding from 16px → 8px
  - User preference stored in localStorage
  - Toggle in top bar settings

### AD-P15-T6: Saved filters per admin user

- [ ] Store in localStorage per-page filter presets
- [ ] Name + save current filter state
- [ ] Load saved filter from dropdown
- [ ] Shared filters (saved to server, available to all staff)

### AD-P15-T7: Entity pinning

- [ ] Pin any org/user/key to quick-access sidebar section
- [ ] Pins stored per admin user in admin_users.preferences (JSON)
- [ ] Max 10 pinned entities
- [ ] Pin from any entity page with ⭐ button

### AD-P15-T8: Contextual right-side drawer

- [ ] Clicking table rows opens slide-in drawer instead of full page nav
- [ ] Drawer shows: entity summary, quick actions, recent activity
- [ ] "Open full page" link in drawer header

**Acceptance criteria:**
- [ ] Command palette opens in < 100ms
- [ ] All keyboard shortcuts work without mouse
- [ ] Real-time metrics update visible within 5 seconds
- [ ] Live log tail shows entries within 1 second of API request
- [ ] Density mode persists across sessions

---

## Phase 16 — 20+ World-Class Advanced Features {#phase-16}

### Feature 1: AI-Powered Anomaly Detection

- [ ] Create `crates/fetchium-api/src/admin/anomaly.rs`:
  - Track per-org request volume as time series (sliding 1h windows)
  - Detect: sudden spike (>3σ above baseline), sustained drop, unusual endpoint pattern
  - Score each org's anomaly level (0-100)
  - Flag high-score orgs in key inventory + org list
  - Suspicious patterns: all requests to /research (bulk abuse), many 429s (scraping)
  - Auto-create support ticket suggestion for high anomaly orgs

### Feature 2: Multi-Step Approval Workflows

- [ ] `crates/fetchium-api/src/admin/approval.rs`:
  - Actions requiring approval: plan_downgrade, bulk_revoke, manual_invoice_void
  - Approval request: requester creates, reviewer approves/rejects
  - Slack notification to reviewer on creation
  - Auto-expire after 24 hours
  - Audit: request creation + review decision both logged

### Feature 3: Bulk Operations with Dry-Run Preview

- [ ] `BulkOperationModal.tsx`:
  - Select N orgs/keys/users via checkboxes
  - Choose action: suspend / plan change / add credit / revoke keys
  - Dry run: shows preview of all changes + affected entities + warnings
  - Confirm → execute → progress bar → result summary
  - Undo available for 60 seconds (soft delete / reversible state)

### Feature 4: CSV/JSON Export for Any View

- [ ] `apps/admin/src/components/modals/ExportModal.tsx`:
  - Column selector (check which fields to include)
  - Format: CSV or JSON
  - Scope: current page / all pages / filtered set
  - Server-side streaming export for large datasets
  - Audit: all exports logged with admin_user_id + column selection

### Feature 5: Admin Action Diff Viewer

- [ ] `AuditDiffViewer.tsx`:
  - Side-by-side before/after rendering
  - Syntax-highlighted JSON
  - Only changed keys highlighted in yellow
  - Added keys in green, removed keys in red

### Feature 6: Incident Impact Calculator

- [ ] `crates/fetchium-api/src/admin/incidents.rs`:
  - When creating/resolving incident: query usage_logs during incident time range
  - Count: affected_requests, affected_orgs, estimated_revenue_impact_cents
  - Revenue impact = (affected_requests / avg_requests_per_dollar)
  - Show in incident detail header

### Feature 7: AI-Assisted Postmortem Generator

- [ ] `crates/fetchium-api/src/admin/incidents.rs::generate_postmortem()`:
  - Calls Gemini Flash with incident timeline + metrics data
  - Generates: incident summary, timeline, root cause hypothesis, mitigation steps, prevention recommendations
  - Returns markdown, admin edits before publishing
  - Prompt tuned for technical SRE postmortem format

### Feature 8: Provider Dependency Health Rollup

- [ ] `crates/fetchium-api/src/admin/metrics.rs`:
  - Per-provider metrics: success_rate, p95_ms, last_success_at, consecutive_errors
  - Providers: Google, DDG, Bing, Brave, SearXNG, Gemini, Serper, Exa, Firecrawl, Tavily
  - Health = (success_rate * 0.5) + (latency_score * 0.3) + (freshness_score * 0.2)
  - UI: color-coded grid with sparklines per provider
  - Alert if any provider health drops below 60 for 5 consecutive minutes

### Feature 9: Proxy Pool Geographic Distribution Visualization

- [ ] `apps/admin/src/app/(admin)/proxy/page.tsx`:
  - World map heat map of active proxy IPs by country
  - Table: country, IP count, success rate, avg latency, block rate
  - Actions: purge IPs for specific country, test connectivity, force rotation
  - Historical block rate chart (which domains trigger blocks most)

### Feature 10: Tenant Abuse Detection

- [ ] `crates/fetchium-api/src/admin/anomaly.rs`:
  - Pattern: > 90% requests hitting same endpoint in 1 hour → flag as automation/scraping
  - Pattern: > 100 rate limit violations in 1 hour → flag as limit circumvention attempt
  - Pattern: many requests with identical query strings → flag as batch abuse
  - Suspicious tenants appear in security review panel
  - Admin can: add to blocklist / reduce quota / flag for review / send warning email

### Feature 11: Revenue Waterfall Chart

- [ ] `apps/admin/src/app/(admin)/billing/page.tsx`:
  - Monthly revenue waterfall: new MRR + expansion − contraction − churn = net MRR
  - Color-coded bars: new (green), expansion (blue), contraction (orange), churn (red)
  - Click any bar to see contributing orgs

### Feature 12: Cohort Analysis View

- [ ] `apps/admin/src/app/(admin)/campaigns/page.tsx`:
  - Define cohort by signup week/month
  - Track retention: % still active at week 1/2/4/8/12
  - Compare cohorts: pre-feature vs post-feature
  - Revenue by cohort over time

### Feature 13: Mobile-Responsive Incident Response Mode

- [ ] Admin app fully responsive for on-call mobile use:
  - Incident list page optimized for phone viewport
  - Swipe to change incident status
  - Push notification support (PWA web push for incident alerts)
  - Quick actions: update status, post timeline update, change severity
  - Bottom navigation bar on mobile

### Feature 14: Universal Entity Search

- [ ] Server-side: `GET /internal/admin/search?q=<query>`:
  - Searches: orgs (name, slug, email), users (email), keys (prefix), tickets (subject), incidents (title)
  - Returns typed results with entity_type + id + display_name
  - Powered by SQLite FTS5 full-text search
  - Debounced client-side (300ms delay)
  - Results in command palette AND dedicated `/search?q=` page

### Feature 15: Saved Admin Reports

- [ ] Admin users can save any filtered view as a named report
- [ ] Reports are personal (per admin user) or shared (team-wide)
- [ ] Scheduled reports: run weekly, export CSV to email
- [ ] Report types: org health summary, usage anomaly digest, support SLA report, billing status

### Feature 16: SLA Breach Predictor

- [ ] `crates/fetchium-api/src/admin/support.rs`:
  - For each open ticket: calculate SLA remaining time
  - Predict if likely to breach based on assignee's average response time
  - Flag predicted breaches 1 hour before deadline
  - Slack alert for predicted breach: "@assignee: ticket #X predicted to breach SLA in 47 min"

### Feature 17: Churn Prediction Signals

- [ ] `crates/fetchium-api/src/admin/crm.rs`:
  - Compute churn_probability weekly per org using:
    - Declining request volume (last 2 weeks vs prior 2 weeks)
    - Increasing support ticket frequency
    - Payment failures
    - No login in last 14 days
    - Trial approaching expiry without upgrade
  - Surface churn-risk orgs in CRM list (red badge if >70%)
  - Automated task: assign CSM when churn_probability crosses 0.6

### Feature 18: API Endpoint Saturation Heatmap

- [ ] `apps/admin/src/app/(admin)/usage/page.tsx`:
  - X-axis: hour of day (0-23), Y-axis: endpoint
  - Cell value: request volume (color intensity)
  - Shows which endpoints are busiest at which hours
  - Used for capacity planning and rate limit tuning

### Feature 19: Webhook Event Replay + Idempotency Test

- [ ] `crates/fetchium-api/src/admin/billing.rs`:
  - Replay any stored billing webhook event (uses raw_payload from payment_events)
  - Pre-flight: check provider_event_id → reject if already processed (idempotency)
  - Test mode: replay without executing side effects (dry-run billing webhook)
  - Result: success/conflict/error with diff of what changed

### Feature 20: Admin Notification Center

- [ ] `apps/admin/src/components/layout/NotificationCenter.tsx`:
  - Bell icon in top bar with unread count
  - Notification types:
    - New incident auto-opened
    - SLA breach (ticket you own)
    - Approval request pending your review
    - Churn-risk org assigned to you
    - New support ticket assigned
    - Health score dropped below 40 for your CSM accounts
  - Mark as read / dismiss all
  - Link to entity from notification

### Feature 21: Admin Activity Feed

- [ ] `apps/admin/src/app/(admin)/overview/page.tsx`:
  - Live feed of all admin actions (last 50) from audit_events
  - WebSocket-fed, updates in real time
  - Format: "ops_user suspended org Acme Corp 2 min ago"
  - Click → org profile or audit event detail

### Feature 22: Dark Operational Theme + Branding

- [ ] Distinct visual identity from `app.fetchium.com`:
  - Deep zinc-950 background
  - Zinc-800 borders, zinc-700 hover states
  - Red accents for destructive/danger actions
  - Amber for warnings
  - Emerald for success
  - Blue for informational
  - Monospace font for IDs, hashes, request IDs
  - Dense data tables (no whitespace padding)
  - "ADMIN" badge in sidebar header to clearly distinguish from customer dashboard

### Feature 23: Rate Limit Override Controls

- [ ] `apps/admin/src/app/(admin)/keys/[id]/page.tsx`:
  - Override rate limits for a specific key:
    - requests_per_minute (override plan limit)
    - requests_per_month (override quota)
  - Time-limited overrides (expires in N hours)
  - Override reason required (audited)
  - Used for: enterprise negotiations, beta testing, incident remediation

### Feature 24: Automated Escalation Engine

- [ ] `crates/fetchium-api/src/admin/support.rs` background task:
  - Every 5 minutes: scan for escalation triggers
  - Triggers:
    - Ticket priority=urgent + unassigned for >15 min → Slack to #support-oncall
    - Ticket SLA breached → escalate to next tier
    - Org health_score drops below 20 → create CRM task for CSM
    - Org payment failed + has enterprise plan → create high-priority ticket
  - All escalations produce audit events

### Feature 25: Zero-Downtime Config Hot-Reload

- [ ] `crates/fetchium-api/src/admin/flags.rs`:
  - Feature flags cached in-process with 30s TTL
  - On flag update via admin API: broadcast invalidation via tokio broadcast channel
  - All in-flight request handlers check flag cache (non-blocking read)
  - Config changes propagate without Rust process restart

---

## Phase 17 — Deployment & Infra {#phase-17}

### AD-P17-T1: Docker setup for admin app

- [ ] Create `infra/admin/Dockerfile`:

```dockerfile
FROM node:22-alpine AS builder
WORKDIR /app
COPY apps/admin/package*.json ./
RUN npm ci
COPY apps/admin/ .
RUN npm run build

FROM node:22-alpine AS runner
WORKDIR /app
RUN addgroup -S admin && adduser -S admin -G admin
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public
USER admin
EXPOSE 3300
ENV NODE_ENV=production
CMD ["node", "server.js"]
```

### AD-P17-T2: Docker Compose for admin

- [ ] Create `infra/admin/docker-compose.yml`:

```yaml
services:
  admin:
    build:
      context: ../..
      dockerfile: infra/admin/Dockerfile
    ports:
      - "127.0.0.1:3300:3300"
    environment:
      NEXTAUTH_URL: ${NEXTAUTH_URL_ADMIN}
      NEXTAUTH_SECRET: ${NEXTAUTH_SECRET_ADMIN}
      ***REMOVED***: http://api:3050
      ADMIN_DB_PATH: /data/admin.db
    volumes:
      - admin-data:/data
    restart: unless-stopped
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.admin.rule=Host(`admin.fetchium.com`)"
      - "traefik.http.routers.admin.tls=true"
      - "traefik.http.routers.admin.tls.certresolver=letsencrypt"
      - "traefik.http.routers.admin.middlewares=admin-ip-restrict"
      # IP allowlist: office IP + VPN range only
      - "traefik.http.middlewares.admin-ip-restrict.ipwhitelist.sourcerange=10.0.0.0/8,203.0.113.0/24"
    networks:
      - traefik-net

volumes:
  admin-data:

networks:
  traefik-net:
    external: true
```

### AD-P17-T3: Update INFRASTRUCTURE_SOURCE_OF_TRUTH.md

- [ ] Add to `/home/echo/INFRASTRUCTURE_SOURCE_OF_TRUTH.md`:

```markdown
## Fetchium Admin Console

- **Domain**: admin.fetchium.com
- **Port**: 3300
- **Stack**: Next.js 15 + NextAuth v5
- **Container**: fetchium-admin
- **Data**: /data/admin.db (SQLite)
- **IP Restriction**: VPN/office IPs only via Traefik middleware
- **Auth**: Email + TOTP 2FA (staff accounts only, no self-serve)
- **Roles**: owner, ops, support, finance, growth, readonly
- **Environment secrets**: NEXTAUTH_SECRET_ADMIN, ***REMOVED***
- **Monitoring**: Sentry project fetchium-admin, Slack #admin-alerts
```

### AD-P17-T4: Update INFRASTRUCTURE_ENDPOINTS.tsv

- [ ] Add row:
  ```
  admin.fetchium.com  3300  https  internal  fetchium-admin  Next.js admin console  VPN-only
  ```

### AD-P17-T5: Admin DB path in API server

- [ ] Add `FETCHIUM_ADMIN_DB_PATH` env var to API server
- [ ] Default: `$FETCHIUM_DATA_DIR/admin.db`
- [ ] Separate from `auth.db` (never mixed)

### AD-P17-T6: First-run setup script

- [ ] Create `scripts/admin-first-user.sh`:

```bash
#!/bin/bash
# Create the first owner admin user
# Usage: ./scripts/admin-first-user.sh --email admin@fetchium.com --name "Admin User"
curl -X POST http://localhost:3050/internal/admin/auth/bootstrap \
  -H "X-Bootstrap-Secret: $FETCHIUM_BOOTSTRAP_SECRET" \
  -d '{"email":"'"$EMAIL"'","name":"'"$NAME"'","role":"owner"}'
# Prints temporary password, must change on first login
```

**Acceptance criteria:**
- [ ] `admin.fetchium.com` resolves and serves admin app only
- [ ] IP allowlist blocks non-VPN access (403 from Traefik)
- [ ] TLS certificate provisioned by Let's Encrypt
- [ ] Admin DB stored separately from auth.db
- [ ] INFRA SSOT and endpoints TSV updated in same deploy
- [ ] Sentry error tracking active for admin app errors

---

## Phase 18 — Tests & Release Gates {#phase-18}

### AD-P18-T1: Rust unit tests

- [ ] `crates/fetchium-api/src/admin/auth.rs` tests:
  - `test_password_hash_and_verify()`
  - `test_totp_valid_code_accepted()`
  - `test_totp_replayed_code_rejected()`
  - `test_session_expiry()`
  - `test_step_up_window()`
  - `test_backup_code_single_use()`

- [ ] `crates/fetchium-api/src/admin/rbac.rs` tests:
  - `test_all_role_permission_matrix()` (exhaustive: all roles × all atoms)
  - `test_denied_access_writes_audit_event()`

- [ ] `crates/fetchium-api/src/admin/billing.rs` tests:
  - `test_refund_requires_step_up()`
  - `test_webhook_replay_idempotent()`
  - `test_credits_ledger_balance()`

- [ ] `crates/fetchium-api/src/admin/anomaly.rs` tests:
  - `test_spike_detection_above_3_sigma()`
  - `test_no_false_positive_on_normal_growth()`

### AD-P18-T2: Integration tests (axum TestClient)

- [ ] Login flow: valid creds → session token
- [ ] TOTP verification: valid + invalid + replay
- [ ] RBAC enforcement: support role cannot call billing.refund
- [ ] Audit log: org suspend produces before/after audit row
- [ ] Key revoke: key unusable after revoke
- [ ] Feature flag: org override beats global flag

### AD-P18-T3: Next.js E2E tests (Playwright)

- [ ] `apps/admin/e2e/`:
  - `auth.spec.ts`: login → dashboard, wrong password, TOTP flow
  - `orgs.spec.ts`: list orgs, view profile, suspend + audit check
  - `keys.spec.ts`: revoke key, verify blocked
  - `support.spec.ts`: create note, assign ticket, resolve
  - `incidents.spec.ts`: create, update, resolve

### AD-P18-T4: Seed realistic staging data

- [ ] `scripts/admin-seed.sh`:
  - 100 organizations (mix of plans + lifecycle stages)
  - 3-5 users per org
  - 500 API keys
  - 30 days of usage logs (50k rows)
  - 20 support tickets (mix of status)
  - 5 incidents (mix of resolved + open)
  - 3 campaigns with attribution touches

### AD-P18-T5: Pre-production checklist

- [ ] Auth: all flows pass
- [ ] 2FA: required for owner + ops
- [ ] RBAC: permission matrix verified against test matrix
- [ ] Audit: every mutation produces audit row
- [ ] No client-shipped secrets (check Next.js bundle)
- [ ] No localhost fallbacks in production config
- [ ] No hardcoded fake metrics (all charts use real data)
- [ ] Billing webhooks idempotent (replay test passes)
- [ ] Support/finance permissions isolated
- [ ] Observability screens show real data
- [ ] Deploy documented in SSOT
- [ ] Rollback: previous Docker image tagged and tested
- [ ] IP allowlist active on admin domain
- [ ] CSP headers present (verified with browser devtools)
- [ ] robots.txt returns `Disallow: /`
- [ ] Admin DB backed up (same schedule as auth.db)

---

## Environment Variables Reference {#env-vars}

### apps/admin runtime

| Variable | Purpose | Required |
|----------|---------|----------|
| `NEXTAUTH_URL` | Full URL of admin app | ✓ |
| `NEXTAUTH_SECRET` | JWT signing secret (≥32 chars) | ✓ |
| `***REMOVED***` | Internal API base URL | ✓ |
| `ADMIN_DB_PATH` | Path to admin.db | ✓ |
| `SMTP_HOST` | Email for alerts | optional |
| `SMTP_PORT` | Email port | optional |
| `SMTP_USER` | Email user | optional |
| `SMTP_PASS` | Email password | optional |
| `SMTP_FROM` | From address | optional |
| `SENTRY_DSN` | Error tracking | recommended |
| `SENTRY_ENVIRONMENT` | production/staging | recommended |
| `SLACK_WEBHOOK_SECURITY` | Security alerts channel | recommended |
| `SLACK_WEBHOOK_INCIDENTS` | Incident alerts channel | recommended |
| `SLACK_WEBHOOK_SUPPORT` | Support SLA alerts | recommended |

### crates/fetchium-api admin module

| Variable | Purpose | Required |
|----------|---------|----------|
| `FETCHIUM_ADMIN_DB_PATH` | Path to admin.db | ✓ |
| `FETCHIUM_BOOTSTRAP_SECRET` | First-user creation secret | ✓ (setup only) |
| `ADMIN_SESSION_TTL_SECONDS` | Session lifetime (default: 28800) | optional |
| `ADMIN_STEP_UP_TTL_SECONDS` | Step-up window (default: 300) | optional |
| `ADMIN_REFUND_STEP_UP_THRESHOLD_CENTS` | Refund amount requiring step-up (default: 10000) | optional |
| `ADMIN_ANOMALY_CHECK_INTERVAL_SECONDS` | Anomaly detection interval (default: 60) | optional |
| `ADMIN_HEALTH_RECOMPUTE_INTERVAL_SECONDS` | CRM health score refresh (default: 21600) | optional |
| `SLACK_WEBHOOK_SECURITY` | Security alert webhook | recommended |
| `SLACK_WEBHOOK_INCIDENTS` | Incident alert webhook | recommended |
| `SLACK_WEBHOOK_SUPPORT` | Support escalation webhook | recommended |

---

## Production Readiness Checklist {#prod-checklist}

### Security
- [ ] Admin app IP-restricted at infrastructure level (Traefik allowlist)
- [ ] All admin sessions use secure, HttpOnly, SameSite=Strict cookies
- [ ] Session cookies scoped to `admin.fetchium.com` only
- [ ] TOTP enforced for owner and ops roles (cannot disable)
- [ ] Step-up auth required for all financial mutations
- [ ] Break-glass access sends immediate Slack alert
- [ ] robots.txt returns `Disallow: /`
- [ ] `X-Robots-Tag: noindex, nofollow` on all admin responses
- [ ] CSP blocks frame embedding and external script sources
- [ ] API keys never stored in plain text or logged
- [ ] Audit log append-only (no delete/update routes)
- [ ] Failed login attempts rate-limited (5 per minute per IP)

### Data
- [ ] Admin DB backed up daily with point-in-time recovery
- [ ] auth.db and admin.db are separate files, separate pools
- [ ] All text timestamps in RFC3339 UTC format
- [ ] JSON columns validated at insert (no raw string concatenation)
- [ ] SQLite WAL mode enabled on admin.db
- [ ] foreign_keys pragma enabled

### Observability
- [ ] Sentry active for admin Next.js app
- [ ] Sentry active for Rust admin API routes
- [ ] All admin API routes emit structured logs (tracing spans)
- [ ] Slow admin query alerts (>1s)
- [ ] Admin app health endpoint: `/api/health`
- [ ] Uptime monitoring for admin.fetchium.com

### Operations
- [ ] INFRA SSOT updated
- [ ] INFRA ENDPOINTS TSV updated
- [ ] Rollback: previous Docker image tagged and documented
- [ ] First-user setup script documented and tested
- [ ] Staff onboarding runbook: how to add new admin user
- [ ] On-call runbook: how to respond to admin app down

---

## Build Order Summary

Execute phases in this order for maximum parallelism after Phase 0:

```
Phase 0: Data model spec [blocking — must finish before all others]
         ↓
Phase 1: App scaffold + Phase 5: DB schema  [parallel]
         ↓
Phase 2: Auth         + Phase 4: Admin backend  [parallel]
         ↓
Phase 3: RBAC  [depends on Phase 2 + 4]
         ↓
Phase 6: Orgs/Users  + Phase 7: Keys/Usage  + Phase 8: Billing  [parallel]
         ↓
Phase 9: CRM  + Phase 10: Support  + Phase 11: Observability  [parallel]
         ↓
Phase 12: Campaigns  + Phase 13: Audit  + Phase 14: Flags  [parallel]
         ↓
Phase 15: UX + Phase 16: Advanced Features  [parallel, can partially ship earlier]
         ↓
Phase 17: Deployment  [depends on all above]
         ↓
Phase 18: Tests + production checklist  [final gate]
```

---

## Phase 19 — Full Control & Visibility Layer {#phase-19}

**Goal:** Complete god-mode visibility and control over every system component — no black boxes.

### AD-P19-T1: System Control Panel

- [ ] `apps/admin/src/app/(admin)/system/page.tsx` — Owner only:
  - **Live server stats**: CPU%, RAM used/total, disk usage, open file handles, goroutine count
  - **API process health**: uptime, requests served total, errors total, restart count
  - **Database stats**: auth.db size, admin.db size, WAL file size, connection pool usage, slow query count
  - **Cache stats**: hit rate, eviction count, entry count, memory used
  - **Active connections**: current WebSocket connections, HTTP keep-alive count
  - **Actions**: Trigger GC, Clear cache, Reload config, Graceful restart, Emergency shutdown

- [ ] Rust endpoint: `GET /internal/admin/system/stats`
  ```rust
  // Returns: { cpu_pct, mem_used_mb, mem_total_mb, disk_used_gb,
  //   uptime_secs, total_requests, total_errors, db_sizes,
  //   cache_hits, cache_misses, active_ws_connections }
  // Collected via /proc/self/status + sysinfo crate
  ```

### AD-P19-T2: Live Request Inspector (Full Visibility)

- [ ] `apps/admin/src/app/(admin)/usage/inspector/page.tsx`:
  - **Real-time stream of ALL requests** across all orgs (WebSocket, 50/s cap)
  - Columns: time, org, key prefix, endpoint, status, latency_ms, tokens, rate_limited
  - Toggle filters: endpoint, org, status code, min_latency
  - Click any row → full request detail drawer:
    - Request headers (sanitized — no Bearer token)
    - Query params / body summary
    - Response status + timing breakdown (queue_ms, search_ms, rank_ms, output_ms)
    - Provider backend used (Google/DDG/Serper/etc.)
    - Cache hit/miss
    - Token budget allocation
  - "Freeze" mode: pause stream, inspect, resume

- [ ] Rust WebSocket: `GET /internal/admin/ws/inspector`:
  ```rust
  // Broadcasts every completed request as JSON to all connected admin inspectors
  // Uses tokio broadcast channel (capacity: 1000, drops oldest on overflow)
  // Never logs Bearer tokens, full query text truncated to 200 chars
  ```

### AD-P19-T3: Customer Impersonation (Controlled)

- [ ] Owner-only action on org/user profile:
  - "View as customer" → generates a time-limited (15 min) read-only session for `app.fetchium.com`
  - Impersonation token scoped to: read-only, specific org, no key creation/revoke
  - Visible banner in customer app: "You are viewing as [Org Name] (Admin session)"
  - All actions during impersonation tagged in audit log as `impersonated_by: admin_user_id`
  - Automatic expiry + audit event on end

- [ ] Rust: `POST /internal/admin/impersonate/:org_id`:
  ```rust
  // Requires: owner role + TOTP step-up
  // Creates: impersonation_token (signed JWT, 15min, readonly claim)
  // Audit: { action: "impersonation_started", target_org_id, actor: admin_user_id }
  // Customer app validates impersonation_token via /internal/validate-impersonation
  ```

### AD-P19-T4: Config Editor (Live Hot-Reload)

- [ ] `apps/admin/src/app/(admin)/system/config/page.tsx` — Owner/Ops:
  - Display all runtime-configurable settings in structured form:
    - Rate limits per plan (requests/min, requests/month)
    - Search concurrency semaphore size
    - Proxy pool size per country
    - AI model selection (Gemini Flash/Pro)
    - Research pipeline timeout
    - Cache TTL values
    - Backend enable/disable toggles (same as feature flags but config-level)
  - Edit in form → preview diff → Apply (hot-reload, no restart)
  - Config stored in admin.db `runtime_config` table (key-value)
  - Changes propagate to Rust via flag evaluation (30s TTL cache bust)
  - Full audit trail of every config change

### AD-P19-T5: Database Inspector (Read-Only)

- [ ] `apps/admin/src/app/(admin)/system/db/page.tsx` — Owner only:
  - Table browser: select table → paginated row view
  - Read-only SQL query runner (SELECT only, validated by Rust before execution)
  - Query results exportable to CSV
  - Table stats: row count, size, last vacuum, index usage
  - Useful for: debugging production issues, verifying data integrity, customer escalations

- [ ] Rust: `POST /internal/admin/db/query`:
  ```rust
  // Validates query is SELECT only (SQL parser check, not just string match)
  // Enforces: LIMIT ≤ 1000, timeout 5s
  // Audit: logs every query + admin_user_id + row_count returned
  // Never allows: INSERT, UPDATE, DELETE, DROP, PRAGMA write ops
  ```

### AD-P19-T6: Log Streaming Center

- [ ] `apps/admin/src/app/(admin)/system/logs/page.tsx`:
  - Live tail of Rust API server structured logs (WebSocket)
  - Filter by: log level (ERROR/WARN/INFO/DEBUG), module, org_id, request_id
  - Log levels color-coded (red=ERROR, amber=WARN, zinc=INFO)
  - Download last 1000 lines as JSON
  - Search within buffered lines (client-side Fuse.js)
  - Separate tabs: API logs | Admin logs | Anomaly detector logs | Background job logs

- [ ] Rust WebSocket: `GET /internal/admin/ws/logs`:
  ```rust
  // Taps into the tracing subscriber via a broadcast channel appender
  // Filters before sending (respects admin's log level filter)
  // Redacts: Bearer tokens, password fields, TOTP codes in log output
  ```

### AD-P19-T7: Background Job Monitor

- [ ] `apps/admin/src/app/(admin)/system/jobs/page.tsx`:
  - List all running + recent background jobs:
    - YouTube analysis jobs
    - Social research jobs
    - Research pipeline jobs
    - Health score recompute
    - Anomaly detection runs
    - Scheduled report generation
  - Columns: job_id, type, org, status, started_at, duration, progress, error
  - Actions: Cancel running job, Retry failed job, View job output log
  - Historical: last 100 completed/failed jobs

### AD-P19-T8: API Schema & Route Explorer

- [ ] `apps/admin/src/app/(admin)/system/api/page.tsx`:
  - Generated from Rust router introspection
  - Lists all routes: method, path, auth type, rate limit, handler name
  - Shows which feature flag gates each route
  - "Test this endpoint" → opens API playground with admin auth pre-filled
  - Useful for: debugging, documentation, support escalations requiring API testing

### AD-P19-T9: Admin Audit of Admin Actions (Meta-Audit)

- [ ] Dedicated view filtered to `target_type = 'admin_user'`:
  - Who created which staff accounts
  - Who changed role assignments
  - Who accessed break-glass
  - Who ran DB queries
  - Who performed impersonation
  - Who changed runtime config
  - Immutable — not even owner can delete entries

### AD-P19-T10: Full Revenue Dashboard

- [ ] `apps/admin/src/app/(admin)/billing/page.tsx` expanded:
  - **MRR**: total, by plan tier, by acquisition channel
  - **ARR**: annualized, with 90-day trend
  - **ARPU**: average revenue per org, trending
  - **NRR (Net Revenue Retention)**: expansion − contraction − churn
  - **Logo churn rate**: orgs canceled / total orgs (monthly)
  - **Revenue churn rate**: MRR lost / total MRR
  - **LTV**: average org lifetime value (MRR × avg lifetime months)
  - **CAC**: cost per acquired customer (if ad spend data available)
  - **LTV:CAC ratio**: target >3x
  - **Payback period**: CAC / monthly ARPU
  - **Trial conversion rate**: trials → paid (daily + weekly trend)
  - All metrics exportable, all charts drill-down to contributing orgs

### AD-P19-T11: Customer 360 View

- [ ] Every org profile page aggregates ALL data in one view:
  - **Identity**: name, slug, owner, created, lifecycle stage, CSM
  - **Financial**: plan, MRR, ARR, payment status, last invoice, credits balance
  - **Usage**: requests today/7d/30d, top endpoints, last active key
  - **Health**: score (0-100), signal breakdown, churn probability
  - **Support**: open tickets, SLA status, escalations
  - **Incidents**: incidents affecting this org (correlated by time + endpoint)
  - **Attribution**: how they found Fetchium, first touch campaign
  - **Audit**: all admin actions on this org (last 20)
  - **Feature flags**: overrides active for this org
  - **Notes**: CRM notes + support notes unified timeline
  - **Active sessions**: how many users actively using the API right now

### AD-P19-T12: Proxy Control Center (Full Visibility)

- [ ] `apps/admin/src/app/(admin)/proxy/page.tsx` expanded:
  - **Per-domain proxy assignment**: which domains use residential vs direct
  - **Per-country IP pool**: count, health, last rotation, avg latency
  - **Block detection log**: which IPs got blocked, when, by which domain, how resolved
  - **Bandwidth usage**: GB consumed by DataImpulse today/month vs budget
  - **Cost estimator**: projected monthly DataImpulse cost based on current usage rate
  - **Controls**:
    - Force rotate all IPs for a country
    - Ban a specific domain from residential (move to direct)
    - Set per-domain max_requests_before_rotate
    - Pause residential proxy entirely (emergency)
    - Test proxy health: sends test request through each country pool

**Acceptance criteria for Phase 19:**
- [ ] System stats update every 5 seconds via WebSocket
- [ ] Request inspector shows real requests within 500ms of completion
- [ ] Impersonation sessions expire automatically at 15 min
- [ ] DB query runner rejects all non-SELECT statements (parse-level, not just string match)
- [ ] Log streaming redacts all secrets before broadcast
- [ ] Revenue metrics match billing webhook data (verified monthly)
- [ ] Customer 360 loads full profile within 800ms
- [ ] Proxy control center shows real DataImpulse pool state

**Minimum Viable Production Admin (updated):**

Complete in order:
1. Phase 0 (spec)
2. Phase 1 (scaffold)
3. Phase 2 (auth with TOTP)
4. Phase 3 (RBAC)
5. Phase 4 (backend routes)
6. Phase 5 (DB schema)
7. Phase 6 (org + user CRUD)
8. Phase 7 (key + usage)
9. Phase 17 (observability overview only)
10. Phase 13 (audit log)
11. Phase 17 (deploy)
12. Phase 18 (tests + checklist)

Then ship: billing, CRM, support, campaigns, advanced features as follow-on.
