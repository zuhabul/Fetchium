import { redirect } from 'next/navigation'
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
  { method: 'GET',    path: '/v1/health',                        auth: 'none',          rateLimit: 'unlimited',   handler: 'health' },
  { method: 'POST',   path: '/v1/search',                        auth: 'api_key',       rateLimit: 'plan_limit',  handler: 'search' },
  { method: 'POST',   path: '/v1/scrape',                        auth: 'api_key',       rateLimit: 'plan_limit',  handler: 'scrape' },
  { method: 'POST',   path: '/v1/research',                      auth: 'api_key',       rateLimit: 'plan_limit',  handler: 'research' },
  { method: 'GET',    path: '/v1/usage',                         auth: 'api_key',       rateLimit: 'plan_limit',  handler: 'usage::self' },
  { method: 'GET',    path: '/internal/admin/orgs',              auth: 'admin_session', rateLimit: 'admin',       handler: 'orgs::list' },
  { method: 'GET',    path: '/internal/admin/orgs/:id',          auth: 'admin_session', rateLimit: 'admin',       handler: 'orgs::get' },
  { method: 'PATCH',  path: '/internal/admin/orgs/:id',          auth: 'admin_session', rateLimit: 'admin',       handler: 'orgs::update' },
  { method: 'DELETE', path: '/internal/admin/orgs/:id',          auth: 'admin_session', rateLimit: 'admin',       handler: 'orgs::delete' },
  { method: 'GET',    path: '/internal/admin/users',             auth: 'admin_session', rateLimit: 'admin',       handler: 'users::list' },
  { method: 'GET',    path: '/internal/admin/users/:id',         auth: 'admin_session', rateLimit: 'admin',       handler: 'users::get' },
  { method: 'PATCH',  path: '/internal/admin/users/:id',         auth: 'admin_session', rateLimit: 'admin',       handler: 'users::update' },
  { method: 'GET',    path: '/internal/admin/keys',              auth: 'admin_session', rateLimit: 'admin',       handler: 'keys::list' },
  { method: 'POST',   path: '/internal/admin/keys',              auth: 'admin_session', rateLimit: 'admin',       handler: 'keys::create' },
  { method: 'DELETE', path: '/internal/admin/keys/:id',          auth: 'admin_session', rateLimit: 'admin',       handler: 'keys::revoke' },
  { method: 'GET',    path: '/internal/admin/metrics/summary',   auth: 'admin_session', rateLimit: 'admin',       handler: 'metrics::summary' },
  { method: 'GET',    path: '/internal/admin/metrics/realtime',  auth: 'admin_session', rateLimit: 'admin',       handler: 'metrics::realtime' },
  { method: 'POST',   path: '/internal/admin/proxy/reset',       auth: 'admin_session', rateLimit: 'admin',       handler: 'proxy::reset' },
  { method: 'GET',    path: '/internal/admin/flags',             auth: 'admin_session', rateLimit: 'admin',       handler: 'flags::list' },
  { method: 'PATCH',  path: '/internal/admin/flags/:id',         auth: 'admin_session', rateLimit: 'admin',       handler: 'flags::update' },
  { method: 'GET',    path: '/internal/admin/audit',             auth: 'admin_session', rateLimit: 'admin',       handler: 'audit::list' },
  { method: 'GET',    path: '/internal/admin/incidents',         auth: 'admin_session', rateLimit: 'admin',       handler: 'incidents::list' },
  { method: 'POST',   path: '/internal/admin/incidents',         auth: 'admin_session', rateLimit: 'admin',       handler: 'incidents::create' },
  { method: 'PATCH',  path: '/internal/admin/incidents/:id',     auth: 'admin_session', rateLimit: 'admin',       handler: 'incidents::update' },
  { method: 'GET',    path: '/internal/admin/support/tickets',   auth: 'admin_session', rateLimit: 'admin',       handler: 'support::list' },
  { method: 'POST',   path: '/internal/admin/db/query',          auth: 'admin_session', rateLimit: 'admin',       handler: 'db::query' },
  { method: 'POST',   path: '/internal/admin/cache/clear',       auth: 'admin_session', rateLimit: 'admin',       handler: 'cache::clear' },
]

export default async function ApiExplorerPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  return (
    <>
      <TopBar title="API Explorer" subtitle="All routes — method, auth, rate limit" />
      <div className="p-6 max-w-6xl">
        <ApiExplorerClient routes={ROUTES} />
      </div>
    </>
  )
}
