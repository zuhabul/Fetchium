'use client'

import { useState, useEffect, useRef } from 'react'
import TopBar from '@/components/layout/TopBar'

type LogLevel = 'ERROR' | 'WARN' | 'INFO' | 'DEBUG'

interface LogLine {
  id: number
  ts: string
  level: LogLevel
  message: string
}

const LEVEL_COLORS: Record<LogLevel, string> = {
  ERROR: 'text-red-400',
  WARN:  'text-amber-400',
  INFO:  'text-zinc-400',
  DEBUG: 'text-zinc-600',
}

const MOCK_MESSAGES: Array<{ level: LogLevel; message: string }> = [
  { level: 'INFO',  message: 'GET /v1/search 200 245ms' },
  { level: 'INFO',  message: 'POST /v1/research 200 8432ms' },
  { level: 'WARN',  message: 'Rate limit triggered for key fch_xxx (org: Acme)' },
  { level: 'ERROR', message: 'Google backend returned 429, rotating IP' },
  { level: 'INFO',  message: 'Session validated for admin@fetchium.com' },
  { level: 'DEBUG', message: 'Cache hit: query "AI tools comparison" ttl=287s' },
  { level: 'INFO',  message: 'POST /v1/scrape 200 1230ms' },
  { level: 'WARN',  message: 'DDG backend slow (>3s), fallback to lite' },
  { level: 'DEBUG', message: 'BM25 scored 42 candidates in 12ms' },
  { level: 'ERROR', message: 'Ollama unreachable — semantic rerank skipped' },
  { level: 'INFO',  message: 'GET /v1/health 200 2ms' },
  { level: 'DEBUG', message: 'Proxy client reused (country=us pool_size=4)' },
]

const MAX_BUFFER = 200

function pad(n: number) { return String(n).padStart(2, '0') }
function nowTs() {
  const d = new Date()
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

let _id = 0

export default function LogsPage() {
  const [logs, setLogs] = useState<LogLine[]>([])
  const [paused, setPaused] = useState(false)
  const [activeTab] = useState<'api' | 'admin'>('api')
  const [levels, setLevels] = useState<Set<LogLevel>>(new Set(['ERROR', 'WARN', 'INFO', 'DEBUG']))
  const bufferRef = useRef<LogLine[]>([])
  const pausedRef = useRef(false)
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    pausedRef.current = paused
  }, [paused])

  useEffect(() => {
    const interval = setInterval(() => {
      const template = MOCK_MESSAGES[Math.floor(Math.random() * MOCK_MESSAGES.length)]
      const line: LogLine = { id: ++_id, ts: nowTs(), level: template.level, message: template.message }
      bufferRef.current = [...bufferRef.current, line].slice(-MAX_BUFFER)
      if (!pausedRef.current) {
        setLogs([...bufferRef.current])
      }
    }, 2000)
    return () => clearInterval(interval)
  }, [])

  useEffect(() => {
    if (!paused) bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs, paused])

  function toggleLevel(level: LogLevel) {
    setLevels(prev => {
      const next = new Set(prev)
      next.has(level) ? next.delete(level) : next.add(level)
      return next
    })
  }

  function handleResume() {
    setPaused(false)
    setLogs([...bufferRef.current])
  }

  function clearLogs() {
    bufferRef.current = []
    setLogs([])
  }

  function downloadLogs() {
    const text = bufferRef.current.map(l => `[${l.ts}] [${l.level}] ${l.message}`).join('\n')
    const blob = new Blob([text], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `fetchium-logs-${Date.now()}.txt`
    a.click()
    URL.revokeObjectURL(url)
  }

  const visible = logs.filter(l => levels.has(l.level))

  const LEVEL_BTN_ACTIVE: Record<LogLevel, string> = {
    ERROR: 'bg-red-500/20 text-red-400 border-red-500/30',
    WARN:  'bg-amber-500/20 text-amber-400 border-amber-500/30',
    INFO:  'bg-zinc-700 text-zinc-300 border-zinc-600',
    DEBUG: 'bg-zinc-800 text-zinc-500 border-zinc-700',
  }

  return (
    <>
      <TopBar title="Log Stream" subtitle="Live API & admin logs" />
      <div className="p-6 space-y-4 max-w-6xl">

        {/* Tabs */}
        <div className="flex gap-1 bg-zinc-900 border border-zinc-800 rounded-lg p-1 w-fit">
          {(['api', 'admin'] as const).map(t => (
            <button key={t} className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors capitalize ${activeTab === t ? 'bg-zinc-700 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}>
              {t === 'api' ? 'API logs' : 'Admin logs'}
            </button>
          ))}
        </div>

        {/* Controls */}
        <div className="flex flex-wrap items-center gap-2">
          {(['ERROR', 'WARN', 'INFO', 'DEBUG'] as LogLevel[]).map(level => (
            <button key={level} onClick={() => toggleLevel(level)}
              className={`text-xs font-medium px-2 py-0.5 rounded-full border transition-colors ${levels.has(level) ? LEVEL_BTN_ACTIVE[level] : 'bg-transparent text-zinc-600 border-zinc-800'}`}>
              {level}
            </button>
          ))}
          <div className="flex-1" />
          <button onClick={paused ? handleResume : () => setPaused(true)}
            className="px-3 py-1.5 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
            {paused ? 'Resume' : 'Pause'}
          </button>
          <button onClick={clearLogs}
            className="px-3 py-1.5 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
            Clear
          </button>
          <button onClick={downloadLogs}
            className="px-3 py-1.5 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
            Download
          </button>
        </div>

        {paused && (
          <div className="bg-amber-500/10 border border-amber-500/20 rounded-lg px-3 py-2 text-xs text-amber-400">
            Paused — buffering in background. {bufferRef.current.length} lines buffered.
          </div>
        )}

        {/* Log terminal */}
        <div className="bg-zinc-950 border border-zinc-800 rounded-xl p-4 h-[calc(100vh-340px)] overflow-y-auto font-mono text-xs">
          {visible.length === 0 && (
            <span className="text-zinc-700">No log lines match current filters…</span>
          )}
          {visible.map(l => (
            <div key={l.id} className="flex gap-2 leading-5">
              <span className="text-zinc-700 flex-shrink-0">[{l.ts}]</span>
              <span className={`flex-shrink-0 w-12 ${LEVEL_COLORS[l.level]}`}>[{l.level}]</span>
              <span className="text-zinc-400">{l.message}</span>
            </div>
          ))}
          <div ref={bottomRef} />
        </div>
      </div>
    </>
  )
}
