'use client'

import { useState, useEffect, useRef } from 'react'
import TopBar from '@/components/layout/TopBar'

type LogLevel = 'ERROR' | 'WARN' | 'INFO' | 'DEBUG'

interface LogLine { id: number; line: string; level: LogLevel }

const LEVEL_COLORS: Record<LogLevel, string> = {
  ERROR: 'text-red-400', WARN: 'text-amber-400', INFO: 'text-zinc-400', DEBUG: 'text-zinc-600',
}
const LEVEL_BTN_ACTIVE: Record<LogLevel, string> = {
  ERROR: 'bg-red-500/20 text-red-400 border-red-500/30',
  WARN:  'bg-amber-500/20 text-amber-400 border-amber-500/30',
  INFO:  'bg-zinc-700 text-zinc-300 border-zinc-600',
  DEBUG: 'bg-zinc-800 text-zinc-500 border-zinc-700',
}

let _id = 0

export default function LogsPage() {
  const [logs, setLogs] = useState<LogLine[]>([])
  const [paused, setPaused] = useState(false)
  const [service, setService] = useState<'fetchium-api' | 'fetchium-admin'>('fetchium-api')
  const [levels, setLevels] = useState<Set<LogLevel>>(new Set(['ERROR', 'WARN', 'INFO', 'DEBUG']))
  const [error, setError] = useState<string | null>(null)
  const bufferRef = useRef<LogLine[]>([])
  const pausedRef = useRef(false)
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => { pausedRef.current = paused }, [paused])

  async function fetchLogs(svc: string) {
    try {
      const res = await fetch('/api/admin/system/logs?service=' + svc + '&lines=200')
      if (!res.ok) { setError('Failed to fetch logs'); return }
      const body = await res.json()
      if (!body.ok) { setError(body.error ?? 'API error'); return }
      setError(null)
      const lines: LogLine[] = (body.lines ?? []).map((l: { line: string; level: string }) => ({
        id: ++_id, line: l.line, level: (l.level as LogLevel) ?? 'INFO',
      }))
      bufferRef.current = lines.slice(-200)
      if (!pausedRef.current) setLogs([...bufferRef.current])
    } catch { setError('Network error') }
  }

  useEffect(() => {
    fetchLogs(service)
    const interval = setInterval(() => { if (!pausedRef.current) fetchLogs(service) }, 5000)
    return () => clearInterval(interval)
  }, [service])

  useEffect(() => { if (!paused) bottomRef.current?.scrollIntoView({ behavior: 'smooth' }) }, [logs, paused])

  function toggleLevel(level: LogLevel) {
    setLevels(prev => { const n = new Set(prev); n.has(level) ? n.delete(level) : n.add(level); return n })
  }
  function clearLogs() { bufferRef.current = []; setLogs([]) }
  function downloadLogs() {
    const blob = new Blob([bufferRef.current.map(l => l.line).join('\n')], { type: 'text/plain' })
    const a = Object.assign(document.createElement('a'), { href: URL.createObjectURL(blob), download: 'fetchium-logs.txt' })
    a.click()
  }

  const visible = logs.filter(l => levels.has(l.level))

  return (
    <>
      <TopBar title="Log Stream" subtitle="Live service logs via journald" />
      <div className="p-6 space-y-4 max-w-6xl">
        <div className="flex gap-1 bg-zinc-900 border border-zinc-800 rounded-lg p-1 w-fit">
          {(['fetchium-api', 'fetchium-admin'] as const).map(svc => (
            <button key={svc} onClick={() => setService(svc)}
              className={"px-4 py-1.5 text-xs font-medium rounded-md transition-colors " + (service === svc ? 'bg-zinc-700 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300')}>
              {svc}
            </button>
          ))}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {(['ERROR', 'WARN', 'INFO', 'DEBUG'] as LogLevel[]).map(level => (
            <button key={level} onClick={() => toggleLevel(level)}
              className={"text-xs font-medium px-2 py-0.5 rounded-full border transition-colors " + (levels.has(level) ? LEVEL_BTN_ACTIVE[level] : 'bg-transparent text-zinc-600 border-zinc-800')}>
              {level}
            </button>
          ))}
          <div className="flex-1" />
          {[['Refresh', () => fetchLogs(service)], [paused ? 'Resume' : 'Pause', () => paused ? (setPaused(false), setLogs([...bufferRef.current])) : setPaused(true)], ['Clear', clearLogs], ['Download', downloadLogs]].map(([label, fn]) => (
            <button key={label as string} onClick={fn as () => void}
              className="px-3 py-1.5 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300">
              {label as string}
            </button>
          ))}
        </div>
        {error && <div className="bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2 text-xs text-red-400">{error}</div>}
        {paused && <div className="bg-amber-500/10 border border-amber-500/20 rounded-lg px-3 py-2 text-xs text-amber-400">Paused — auto-refresh suspended.</div>}
        <div className="bg-zinc-950 border border-zinc-800 rounded-xl p-4 h-[calc(100vh-340px)] overflow-y-auto font-mono text-xs">
          {visible.length === 0 && !error && <span className="text-zinc-700">No log lines match current filters…</span>}
          {visible.map(l => <div key={l.id} className={"leading-5 " + LEVEL_COLORS[l.level]}>{l.line}</div>)}
          <div ref={bottomRef} />
        </div>
        <p className="text-xs text-zinc-700">Polling journald every 5s · last 200 lines · {visible.length} visible</p>
      </div>
    </>
  )
}
