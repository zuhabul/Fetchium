import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'

interface MetricsSummary {
  jobs_active?: number
  jobs_queued?: number
  jobs_completed_today?: number
  jobs_failed_today?: number
}

interface Job {
  id: string
  type: string
  org: string
  status: 'running' | 'completed' | 'failed' | 'queued'
  started: string
  duration: string
}

const MOCK_JOBS: Job[] = [
  { id: 'job_001', type: 'research', org: 'Acme Corp', status: 'completed', started: '2m ago', duration: '8.4s' },
  { id: 'job_002', type: 'youtube_analyze', org: 'TechStart', status: 'running', started: '30s ago', duration: '...' },
  { id: 'job_003', type: 'health_recompute', org: 'system', status: 'completed', started: '6h ago', duration: '2.1s' },
  { id: 'job_004', type: 'research', org: 'DevCo', status: 'failed', started: '1h ago', duration: '30.0s' },
  { id: 'job_005', type: 'batch_scrape', org: 'Startup Inc', status: 'queued', started: '—', duration: '—' },
  { id: 'job_006', type: 'research', org: 'Acme Corp', status: 'completed', started: '3h ago', duration: '11.2s' },
]

const STATUS_BADGE: Record<Job['status'], string> = {
  running:   'bg-blue-500/20 text-blue-400 border-blue-500/30',
  completed: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
  failed:    'bg-red-500/20 text-red-400 border-red-500/30',
  queued:    'bg-amber-500/20 text-amber-400 border-amber-500/30',
}

function StatCard({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
      <p className="text-xs text-zinc-500 mb-1">{label}</p>
      <p className="text-2xl font-semibold text-zinc-100">{value}</p>
    </div>
  )
}

export default async function JobsPage() {
  const session = await getSession()
  if (!session) redirect('/login')

  let summary: MetricsSummary = {}
  try {
    const res = await adminFetch('/internal/admin/metrics/summary')
    if (res.ok) summary = await res.json()
  } catch { /* non-fatal */ }

  return (
    <>
      <TopBar title="Job Monitor" subtitle="Background job queue" />
      <div className="p-6 space-y-6 max-w-5xl">

        {/* Stat cards */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard label="Active Jobs"      value={summary.jobs_active ?? 1} />
          <StatCard label="Queued Jobs"      value={summary.jobs_queued ?? 1} />
          <StatCard label="Completed Today"  value={summary.jobs_completed_today ?? 24} />
          <StatCard label="Failed Today"     value={summary.jobs_failed_today ?? 1} />
        </div>

        {/* Notice */}
        <div className="bg-zinc-800/50 border border-zinc-700 rounded-lg px-4 py-2.5 text-xs text-zinc-500">
          Showing recent jobs. Live job listing endpoint coming in Phase 20.
        </div>

        {/* Jobs table */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
          <div className="px-4 py-3 border-b border-zinc-800">
            <p className="text-sm font-semibold text-zinc-300">Recent Jobs</p>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-xs">
              <thead>
                <tr className="border-b border-zinc-800">
                  <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Job ID</th>
                  <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Type</th>
                  <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Org</th>
                  <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Status</th>
                  <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Started</th>
                  <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Duration</th>
                  <th className="px-4 py-2.5 text-left font-medium text-zinc-500">Actions</th>
                </tr>
              </thead>
              <tbody>
                {MOCK_JOBS.map(job => (
                  <tr key={job.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/20">
                    <td className="px-4 py-3 font-mono text-zinc-500">{job.id}</td>
                    <td className="px-4 py-3 text-zinc-300">{job.type}</td>
                    <td className="px-4 py-3 text-zinc-400">{job.org}</td>
                    <td className="px-4 py-3">
                      <span className={`inline-flex items-center gap-1.5 text-xs font-medium px-2 py-0.5 rounded-full border ${STATUS_BADGE[job.status]}`}>
                        {job.status === 'running' && (
                          <span className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-pulse" />
                        )}
                        {job.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-zinc-500">{job.started}</td>
                    <td className="px-4 py-3 font-mono text-zinc-400">{job.duration}</td>
                    <td className="px-4 py-3">
                      <div className="flex gap-2">
                        {job.status === 'failed' && (
                          <button
                            onClick={() => alert('Not yet implemented')}
                            className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
                          >
                            Retry
                          </button>
                        )}
                        {job.status === 'running' && (
                          <button
                            onClick={() => alert('Not yet implemented')}
                            className="text-xs text-red-400 hover:text-red-300 transition-colors"
                          >
                            Cancel
                          </button>
                        )}
                        {job.status !== 'failed' && job.status !== 'running' && (
                          <span className="text-zinc-700">—</span>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </>
  )
}
