import { redirect } from 'next/navigation'
import { ADMIN_PAGE_PADDING } from '@/lib/layout'
import { getSession } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import ApiExplorerClient from './ApiExplorerClient'

export interface Route {
  method: 'GET' | 'POST' | 'PATCH' | 'DELETE' | 'PUT'
  path: string
  auth: string
  rateLimit: string
  handler: string
}

const ROUTES: Route[] = [
  // ── Public ────────────────────────────────────────────────────────────────
  { method: 'GET',    path: '/v1/health',                                  auth: 'none',          rateLimit: 'unlimited',  handler: 'health' },

  // ── Customer API (api_key) ─────────────────────────────────────────────
  { method: 'POST',   path: '/v1/search',                                  auth: 'api_key',       rateLimit: 'plan_limit', handler: 'search' },
  { method: 'POST',   path: '/v1/scrape',                                  auth: 'api_key',       rateLimit: 'plan_limit', handler: 'scrape' },
  { method: 'POST',   path: '/v1/fetch',                                   auth: 'api_key',       rateLimit: 'plan_limit', handler: 'fetch' },
  { method: 'POST',   path: '/v1/research',                                auth: 'api_key',       rateLimit: 'plan_limit', handler: 'research' },
  { method: 'POST',   path: '/v1/research/jobs',                           auth: 'api_key',       rateLimit: 'plan_limit', handler: 'research::submit_job' },
  { method: 'POST',   path: '/v1/estimate',                                auth: 'api_key',       rateLimit: 'plan_limit', handler: 'estimate' },
  { method: 'GET',    path: '/v1/jobs/:id',                                auth: 'api_key',       rateLimit: 'plan_limit', handler: 'jobs::get_status' },
  { method: 'POST',   path: '/v1/youtube/search',                          auth: 'api_key',       rateLimit: 'plan_limit', handler: 'youtube::search' },
  { method: 'POST',   path: '/v1/youtube/search/jobs',                     auth: 'api_key',       rateLimit: 'plan_limit', handler: 'youtube::submit_search_job' },
  { method: 'POST',   path: '/v1/youtube/analyze',                         auth: 'api_key',       rateLimit: 'plan_limit', handler: 'youtube::analyze' },
  { method: 'POST',   path: '/v1/youtube/analyze/jobs',                    auth: 'api_key',       rateLimit: 'plan_limit', handler: 'youtube::submit_analyze_job' },
  { method: 'POST',   path: '/v1/social/research',                         auth: 'api_key',       rateLimit: 'plan_limit', handler: 'social::research' },
  { method: 'POST',   path: '/v1/social/research/jobs',                    auth: 'api_key',       rateLimit: 'plan_limit', handler: 'social::submit_research_job' },
  { method: 'POST',   path: '/v1/social/reddit',                           auth: 'api_key',       rateLimit: 'plan_limit', handler: 'social::reddit_search' },
  { method: 'POST',   path: '/v1/social/reddit/jobs',                      auth: 'api_key',       rateLimit: 'plan_limit', handler: 'social::submit_reddit_job' },
  { method: 'POST',   path: '/v1/social/hackernews',                       auth: 'api_key',       rateLimit: 'plan_limit', handler: 'social::hackernews_search' },
  { method: 'POST',   path: '/v1/social/hackernews/jobs',                  auth: 'api_key',       rateLimit: 'plan_limit', handler: 'social::submit_hackernews_job' },
  { method: 'GET',    path: '/v1/usage',                                   auth: 'api_key',       rateLimit: 'plan_limit', handler: 'usage::self' },
  { method: 'GET',    path: '/v1/meta/routes',                             auth: 'api_key',       rateLimit: 'plan_limit', handler: 'meta::routes' },
  { method: 'GET',    path: '/v1/dashboard/overview',                      auth: 'api_key',       rateLimit: 'plan_limit', handler: 'dashboard::overview' },
  { method: 'GET',    path: '/v1/dashboard/billing',                       auth: 'api_key',       rateLimit: 'plan_limit', handler: 'dashboard::billing' },
  { method: 'GET',    path: '/v1/dashboard/usage',                         auth: 'api_key',       rateLimit: 'plan_limit', handler: 'dashboard::usage' },
  { method: 'GET',    path: '/v1/dashboard/quickstart',                    auth: 'api_key',       rateLimit: 'plan_limit', handler: 'dashboard::quickstart' },
  { method: 'GET',    path: '/v1/dashboard/settings',                      auth: 'api_key',       rateLimit: 'plan_limit', handler: 'dashboard::get_settings' },
  { method: 'PATCH',  path: '/v1/dashboard/settings',                      auth: 'api_key',       rateLimit: 'plan_limit', handler: 'dashboard::update_settings' },

  // ── v1 Admin (X-Admin-Secret) ──────────────────────────────────────────
  { method: 'GET',    path: '/v1/keys',                                    auth: 'admin_secret',  rateLimit: 'admin',      handler: 'keys::list_v1' },
  { method: 'POST',   path: '/v1/keys',                                    auth: 'admin_secret',  rateLimit: 'admin',      handler: 'keys::create_v1' },
  { method: 'DELETE', path: '/v1/keys/:id',                                auth: 'admin_secret',  rateLimit: 'admin',      handler: 'keys::revoke_v1' },
  { method: 'GET',    path: '/v1/proxy/stats',                             auth: 'admin_secret',  rateLimit: 'admin',      handler: 'proxy::stats_v1' },
  { method: 'POST',   path: '/v1/proxy/reset',                             auth: 'admin_secret',  rateLimit: 'admin',      handler: 'proxy::reset_v1' },
  { method: 'POST',   path: '/v1/proxy/purge',                             auth: 'admin_secret',  rateLimit: 'admin',      handler: 'proxy::purge_v1' },
  { method: 'POST',   path: '/v1/proxy/test',                              auth: 'admin_secret',  rateLimit: 'admin',      handler: 'proxy::test_v1' },

  // ── Internal Admin — Auth & Sessions ──────────────────────────────────
  { method: 'POST',   path: '/internal/admin/auth/bootstrap',              auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::bootstrap' },
  { method: 'POST',   path: '/internal/admin/auth/login',                  auth: 'none',          rateLimit: 'admin',      handler: 'auth::login' },
  { method: 'POST',   path: '/internal/admin/auth/logout',                 auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::logout' },
  { method: 'GET',    path: '/internal/admin/auth/me',                     auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::me' },
  { method: 'POST',   path: '/internal/admin/auth/totp/setup',             auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::totp_setup' },
  { method: 'POST',   path: '/internal/admin/auth/totp/confirm',           auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::totp_confirm' },
  { method: 'GET',    path: '/internal/admin/sessions',                    auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::list_sessions' },
  { method: 'DELETE', path: '/internal/admin/sessions/:id',                auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::revoke_session' },

  // ── Internal Admin — Staff ────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/staff',                       auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::list_staff' },
  { method: 'POST',   path: '/internal/admin/staff',                       auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::create_staff' },
  { method: 'PATCH',  path: '/internal/admin/staff/:id',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::update_staff' },
  { method: 'DELETE', path: '/internal/admin/staff/:id',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::remove_staff' },
  { method: 'GET',    path: '/internal/admin/staff/:id/sessions',          auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::staff_sessions' },
  { method: 'DELETE', path: '/internal/admin/staff/:id/sessions',          auth: 'admin_session', rateLimit: 'admin',      handler: 'auth::revoke_all_sessions' },

  // ── Internal Admin — Orgs ─────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/orgs',                        auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::list' },
  { method: 'POST',   path: '/internal/admin/orgs',                        auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::create' },
  { method: 'GET',    path: '/internal/admin/orgs/:id',                    auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::get' },
  { method: 'PATCH',  path: '/internal/admin/orgs/:id',                    auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::update' },
  { method: 'POST',   path: '/internal/admin/orgs/:id/suspend',            auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::suspend' },
  { method: 'POST',   path: '/internal/admin/orgs/:id/reactivate',         auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::reactivate' },
  { method: 'PATCH',  path: '/internal/admin/orgs/:id/plan',               auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::change_plan' },
  { method: 'PATCH',  path: '/internal/admin/orgs/:id/quota',              auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::override_quota' },
  { method: 'GET',    path: '/internal/admin/orgs/:id/keys',               auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::list_keys' },
  { method: 'GET',    path: '/internal/admin/orgs/:id/usage',              auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::usage' },
  { method: 'GET',    path: '/internal/admin/orgs/:id/billing',            auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::billing' },
  { method: 'GET',    path: '/internal/admin/orgs/:id/tickets',            auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::tickets' },
  { method: 'GET',    path: '/internal/admin/orgs/:id/members',            auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::members' },
  { method: 'GET',    path: '/internal/admin/orgs/:id/audit',              auth: 'admin_session', rateLimit: 'admin',      handler: 'orgs::audit' },
  { method: 'GET',    path: '/internal/admin/orgs/:id/crm',                auth: 'admin_session', rateLimit: 'admin',      handler: 'crm::get_for_org' },
  { method: 'PATCH',  path: '/internal/admin/orgs/:id/crm',                auth: 'admin_session', rateLimit: 'admin',      handler: 'crm::update_for_org' },

  // ── Internal Admin — Users ────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/users',                       auth: 'admin_session', rateLimit: 'admin',      handler: 'users::list' },
  { method: 'GET',    path: '/internal/admin/users/:id',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'users::get' },
  { method: 'POST',   path: '/internal/admin/users/:id/suspend',           auth: 'admin_session', rateLimit: 'admin',      handler: 'users::suspend' },
  { method: 'POST',   path: '/internal/admin/users/:id/force-logout',      auth: 'admin_session', rateLimit: 'admin',      handler: 'users::force_logout' },

  // ── Internal Admin — Keys ─────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/keys',                        auth: 'admin_session', rateLimit: 'admin',      handler: 'keys::list' },
  { method: 'POST',   path: '/internal/admin/keys',                        auth: 'admin_session', rateLimit: 'admin',      handler: 'keys::create' },
  { method: 'GET',    path: '/internal/admin/keys/:id',                    auth: 'admin_session', rateLimit: 'admin',      handler: 'keys::get' },
  { method: 'DELETE', path: '/internal/admin/keys/:id',                    auth: 'admin_session', rateLimit: 'admin',      handler: 'keys::revoke' },
  { method: 'POST',   path: '/internal/admin/keys/:id/rotate',             auth: 'admin_session', rateLimit: 'admin',      handler: 'keys::rotate' },

  // ── Internal Admin — Usage ────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/usage',                       auth: 'admin_session', rateLimit: 'admin',      handler: 'usage::summary' },
  { method: 'GET',    path: '/internal/admin/usage/forensics/:request_id', auth: 'admin_session', rateLimit: 'admin',      handler: 'usage::forensics' },
  { method: 'GET',    path: '/internal/admin/usage/top-orgs',              auth: 'admin_session', rateLimit: 'admin',      handler: 'usage::top_orgs' },
  { method: 'GET',    path: '/internal/admin/usage/heatmap',               auth: 'admin_session', rateLimit: 'admin',      handler: 'usage::endpoint_heatmap' },

  // ── Internal Admin — Billing ──────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/billing',                     auth: 'admin_session', rateLimit: 'admin',      handler: 'billing::list_subscriptions' },
  { method: 'GET',    path: '/internal/admin/billing/webhooks',            auth: 'admin_session', rateLimit: 'admin',      handler: 'billing::webhook_log' },
  { method: 'POST',   path: '/internal/admin/billing/webhooks/:id/replay', auth: 'admin_session', rateLimit: 'admin',      handler: 'billing::webhook_replay' },
  { method: 'GET',    path: '/internal/admin/billing/:org_id/invoices',    auth: 'admin_session', rateLimit: 'admin',      handler: 'billing::invoices' },
  { method: 'POST',   path: '/internal/admin/billing/:org_id/refund',      auth: 'admin_session', rateLimit: 'admin',      handler: 'billing::refund' },
  { method: 'POST',   path: '/internal/admin/billing/:org_id/credit',      auth: 'admin_session', rateLimit: 'admin',      handler: 'billing::credit' },

  // ── Internal Admin — CRM ──────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/crm/accounts',                auth: 'admin_session', rateLimit: 'admin',      handler: 'crm::list' },
  { method: 'GET',    path: '/internal/admin/crm/accounts/:org_id',        auth: 'admin_session', rateLimit: 'admin',      handler: 'crm::get' },
  { method: 'PATCH',  path: '/internal/admin/crm/accounts/:org_id',        auth: 'admin_session', rateLimit: 'admin',      handler: 'crm::update' },
  { method: 'POST',   path: '/internal/admin/crm/accounts/:org_id/notes',  auth: 'admin_session', rateLimit: 'admin',      handler: 'crm::add_note' },

  // ── Internal Admin — Support ──────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/support/tickets',             auth: 'admin_session', rateLimit: 'admin',      handler: 'support::list' },
  { method: 'GET',    path: '/internal/admin/support/tickets/:id',         auth: 'admin_session', rateLimit: 'admin',      handler: 'support::get' },
  { method: 'POST',   path: '/internal/admin/support/tickets/:id/notes',   auth: 'admin_session', rateLimit: 'admin',      handler: 'support::add_note' },
  { method: 'PATCH',  path: '/internal/admin/support/tickets/:id/assign',  auth: 'admin_session', rateLimit: 'admin',      handler: 'support::assign' },
  { method: 'PATCH',  path: '/internal/admin/support/tickets/:id/status',  auth: 'admin_session', rateLimit: 'admin',      handler: 'support::set_status' },
  { method: 'GET',    path: '/internal/admin/support/macros',              auth: 'admin_session', rateLimit: 'admin',      handler: 'support::list_macros' },
  { method: 'POST',   path: '/internal/admin/support/macros',              auth: 'admin_session', rateLimit: 'admin',      handler: 'support::create_macro' },

  // ── Internal Admin — Incidents ────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/incidents',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'incidents::list' },
  { method: 'POST',   path: '/internal/admin/incidents',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'incidents::create' },
  { method: 'GET',    path: '/internal/admin/incidents/:id',               auth: 'admin_session', rateLimit: 'admin',      handler: 'incidents::get' },
  { method: 'PATCH',  path: '/internal/admin/incidents/:id',               auth: 'admin_session', rateLimit: 'admin',      handler: 'incidents::update' },
  { method: 'POST',   path: '/internal/admin/incidents/:id/timeline',      auth: 'admin_session', rateLimit: 'admin',      handler: 'incidents::add_timeline' },
  { method: 'POST',   path: '/internal/admin/incidents/:id/resolve',       auth: 'admin_session', rateLimit: 'admin',      handler: 'incidents::resolve' },
  { method: 'POST',   path: '/internal/admin/incidents/:id/postmortem',    auth: 'admin_session', rateLimit: 'admin',      handler: 'incidents::postmortem' },

  // ── Internal Admin — Campaigns ────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/campaigns',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'campaigns::list' },
  { method: 'POST',   path: '/internal/admin/campaigns',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'campaigns::create' },
  { method: 'GET',    path: '/internal/admin/campaigns/attribution',       auth: 'admin_session', rateLimit: 'admin',      handler: 'campaigns::attribution_report' },
  { method: 'GET',    path: '/internal/admin/campaigns/funnel',            auth: 'admin_session', rateLimit: 'admin',      handler: 'campaigns::funnel' },
  { method: 'GET',    path: '/internal/admin/campaigns/:id',               auth: 'admin_session', rateLimit: 'admin',      handler: 'campaigns::get' },

  // ── Internal Admin — Audit ────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/audit',                       auth: 'admin_session', rateLimit: 'admin',      handler: 'audit::list' },
  { method: 'GET',    path: '/internal/admin/audit/:id',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'audit::get' },

  // ── Internal Admin — Feature Flags ───────────────────────────────────
  { method: 'GET',    path: '/internal/admin/flags',                       auth: 'admin_session', rateLimit: 'admin',      handler: 'flags::list' },
  { method: 'POST',   path: '/internal/admin/flags',                       auth: 'admin_session', rateLimit: 'admin',      handler: 'flags::create' },
  { method: 'GET',    path: '/internal/admin/flags/:id',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'flags::get' },
  { method: 'PATCH',  path: '/internal/admin/flags/:id',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'flags::update' },

  // ── Internal Admin — Metrics & System ────────────────────────────────
  { method: 'GET',    path: '/internal/admin/metrics/summary',             auth: 'admin_session', rateLimit: 'admin',      handler: 'metrics::summary' },
  { method: 'GET',    path: '/internal/admin/metrics/realtime',            auth: 'admin_session', rateLimit: 'admin',      handler: 'metrics::realtime' },
  { method: 'GET',    path: '/internal/admin/metrics/providers',           auth: 'admin_session', rateLimit: 'admin',      handler: 'metrics::provider_health' },
  { method: 'GET',    path: '/internal/admin/system/stats',                auth: 'admin_session', rateLimit: 'admin',      handler: 'metrics::system_stats' },
  { method: 'GET',    path: '/internal/admin/system/logs',                 auth: 'admin_session', rateLimit: 'admin',      handler: 'metrics::system_logs' },
  { method: 'GET',    path: '/internal/admin/system/jobs',                 auth: 'admin_session', rateLimit: 'admin',      handler: 'metrics::system_jobs' },

  // ── Internal Admin — Proxy ────────────────────────────────────────────
  { method: 'GET',    path: '/internal/admin/proxy/stats',                 auth: 'admin_session', rateLimit: 'admin',      handler: 'proxy::stats' },
  { method: 'POST',   path: '/internal/admin/proxy/reset',                 auth: 'admin_session', rateLimit: 'admin',      handler: 'proxy::reset' },
  { method: 'POST',   path: '/internal/admin/proxy/purge',                 auth: 'admin_session', rateLimit: 'admin',      handler: 'proxy::purge' },
  { method: 'GET',    path: '/internal/admin/proxy/geo',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'proxy::geo_distribution' },

  // ── Internal Admin — DB, Search, Anomaly, Export, Approvals ──────────
  { method: 'POST',   path: '/internal/admin/db/query',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'db::query' },
  { method: 'GET',    path: '/internal/admin/search',                      auth: 'admin_session', rateLimit: 'admin',      handler: 'search::universal' },
  { method: 'GET',    path: '/internal/admin/anomaly/alerts',              auth: 'admin_session', rateLimit: 'admin',      handler: 'anomaly::alerts' },
  { method: 'GET',    path: '/internal/admin/anomaly/tenants',             auth: 'admin_session', rateLimit: 'admin',      handler: 'anomaly::suspicious_tenants' },
  { method: 'GET',    path: '/internal/admin/export/:entity',              auth: 'admin_session', rateLimit: 'admin',      handler: 'export::csv' },
  { method: 'GET',    path: '/internal/admin/approvals',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'approval::list' },
  { method: 'POST',   path: '/internal/admin/approvals',                   auth: 'admin_session', rateLimit: 'admin',      handler: 'approval::create' },
  { method: 'POST',   path: '/internal/admin/approvals/:id/approve',       auth: 'admin_session', rateLimit: 'admin',      handler: 'approval::approve' },
  { method: 'POST',   path: '/internal/admin/approvals/:id/reject',        auth: 'admin_session', rateLimit: 'admin',      handler: 'approval::reject' },
]

export default async function ApiExplorerPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  return (
    <>
      <TopBar title="API Explorer" subtitle="All routes — method, auth, rate limit" />
      <div className={`${ADMIN_PAGE_PADDING} max-w-6xl`}>
        <ApiExplorerClient routes={ROUTES} />
      </div>
    </>
  )
}
