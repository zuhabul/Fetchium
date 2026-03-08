import { redirect } from 'next/navigation'
import { getSession, adminFetch } from '@/lib/session'
import TopBar from '@/components/layout/TopBar'
import ConfigEditorClient from './ConfigEditorClient'

interface Flag {
  id: string
  name: string
  description: string
  enabled: boolean
  is_dangerous: boolean
  rollout_percent?: number
}

export default async function ConfigPage() {
  const session = await getSession()
  if (!session) redirect('/login')
  if (session.role !== 'owner') {
    return (
      <>
        <TopBar title="Config Editor" subtitle="Feature flags & kill switches" />
        <div className="p-6">
          <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-8 text-center">
            <p className="text-sm font-semibold text-red-400">Access denied — owner only</p>
          </div>
        </div>
      </>
    )
  }

  let flags: Flag[] = []
  try {
    const res = await adminFetch('/internal/admin/flags')
    if (res.ok) {
      const data = await res.json()
      flags = Array.isArray(data) ? data : (data.flags ?? [])
    }
  } catch { /* non-fatal */ }

  // Fallback demo flags if API returns empty
  if (flags.length === 0) {
    flags = [
      { id: 'kill_search', name: 'Kill Search', description: 'Disable all /v1/search endpoints immediately', enabled: false, is_dangerous: true },
      { id: 'kill_research', name: 'Kill Research', description: 'Disable all /v1/research endpoints immediately', enabled: false, is_dangerous: true },
      { id: 'maintenance_mode', name: 'Maintenance Mode', description: 'Return 503 to all API consumers with retry-after header', enabled: false, is_dangerous: true },
      { id: 'semantic_rerank', name: 'Semantic Reranking', description: 'Enable nomic-embed-text reranking via Ollama', enabled: true, is_dangerous: false, rollout_percent: 100 },
      { id: 'fact_fusion', name: 'Fact Fusion', description: 'Concurrent Qwen fact extraction for research queries', enabled: true, is_dangerous: false, rollout_percent: 80 },
      { id: 'reddit_boost', name: 'Reddit Boost', description: 'Increase Reddit result weighting for comparison queries', enabled: true, is_dangerous: false, rollout_percent: 100 },
      { id: 'new_ranker', name: 'New HyperFusion Ranker v2', description: 'Experimental ranking model — 30% rollout only', enabled: false, is_dangerous: false, rollout_percent: 30 },
    ]
  }

  return (
    <>
      <TopBar title="Config Editor" subtitle="Feature flags & kill switches" />
      <div className="p-6 max-w-3xl">
        <ConfigEditorClient flags={flags} />
      </div>
    </>
  )
}
