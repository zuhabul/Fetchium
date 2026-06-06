'use client'

import { useState, useTransition } from 'react'

interface Flag {
  id: string
  name: string
  description: string
  enabled: boolean
  is_dangerous: boolean
  rollout_percent?: number
}

interface Props {
  flags: Flag[]
}

export default function ConfigEditorClient({ flags }: Props) {
  const [states, setStates] = useState<Record<string, { enabled: boolean; rollout: number }>>(
    Object.fromEntries(
      flags.map(f => [f.id, { enabled: f.enabled, rollout: f.rollout_percent ?? 100 }])
    )
  )
  const [pending, startTransition] = useTransition()
  const [saved, setSaved] = useState<string | null>(null)
  const [confirmId, setConfirmId] = useState<string | null>(null)

  async function saveFlag(id: string) {
    const s = states[id]
    const res = await fetch(`/api/admin/flags/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enabled: s.enabled, rollout_percent: s.rollout }),
    })
    if (res.ok) {
      setSaved(id)
      setTimeout(() => setSaved(null), 2000)
    }
  }

  function toggle(id: string, isDangerous: boolean) {
    if (isDangerous && !states[id].enabled === true) {
      setConfirmId(id)
      return
    }
    setStates(prev => ({ ...prev, [id]: { ...prev[id], enabled: !prev[id].enabled } }))
    startTransition(() => { saveFlag(id) })
  }

  function confirmToggle() {
    if (!confirmId) return
    setStates(prev => ({ ...prev, [confirmId]: { ...prev[confirmId], enabled: true } }))
    startTransition(() => { saveFlag(confirmId!) })
    setConfirmId(null)
  }

  const killSwitches = flags.filter(f => f.is_dangerous)
  const featureFlags = flags.filter(f => !f.is_dangerous)

  return (
    <div className="space-y-8">
      {/* Confirm modal */}
      {confirmId && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
          <div className="bg-zinc-900 border border-red-500/40 rounded-xl p-6 max-w-sm w-full mx-4">
            <p className="text-sm font-semibold text-red-400 mb-2">Enable kill switch?</p>
            <p className="text-xs text-zinc-400 mb-4">
              This is a dangerous flag. Enabling it may affect live traffic. Are you sure?
            </p>
            <div className="flex gap-3">
              <button onClick={confirmToggle} className="flex-1 py-2 text-xs font-medium bg-red-600 hover:bg-red-500 rounded-md text-white transition-colors">
                Yes, enable
              </button>
              <button onClick={() => setConfirmId(null)} className="flex-1 py-2 text-xs font-medium bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-zinc-300 transition-colors">
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Notice */}
      <div className="flex items-center justify-between bg-zinc-800/50 border border-zinc-700 rounded-lg px-4 py-2.5 text-xs text-zinc-400">
        <span>All changes are logged to the audit trail.</span>
        <span className="text-emerald-400">Hot-reloaded — no restart required</span>
      </div>

      {/* Kill Switches */}
      {killSwitches.length > 0 && (
        <section>
          <p className="text-xs font-medium text-red-400 uppercase tracking-wider mb-3">Kill Switches</p>
          <div className="space-y-3">
            {killSwitches.map(f => {
              const s = states[f.id]
              return (
                <div key={f.id} className={`bg-zinc-900 border rounded-xl p-4 flex items-center gap-4 ${s.enabled ? 'border-red-500/40' : 'border-zinc-800'}`}>
                  <button
                    onClick={() => toggle(f.id, true)}
                    disabled={pending}
                    className={`relative w-12 h-6 rounded-full transition-colors flex-shrink-0 ${s.enabled ? 'bg-red-600' : 'bg-zinc-700'}`}
                  >
                    <span className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${s.enabled ? 'translate-x-7' : 'translate-x-1'}`} />
                  </button>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-zinc-200">{f.name}</p>
                    <p className="text-xs text-zinc-500 mt-0.5">{f.description}</p>
                  </div>
                  <div className="flex items-center gap-2">
                    {saved === f.id && <span className="text-xs text-emerald-400">Saved</span>}
                    <span className={`text-xs font-medium px-2 py-0.5 rounded-full border ${s.enabled ? 'bg-red-500/20 text-red-400 border-red-500/30' : 'bg-zinc-800 text-zinc-500 border-zinc-700'}`}>
                      {s.enabled ? 'ON' : 'OFF'}
                    </span>
                  </div>
                </div>
              )
            })}
          </div>
        </section>
      )}

      {/* Feature Flags */}
      {featureFlags.length > 0 && (
        <section>
          <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Feature Flags</p>
          <div className="space-y-3">
            {featureFlags.map(f => {
              const s = states[f.id]
              return (
                <div key={f.id} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4">
                  <div className="flex items-center gap-4">
                    <button
                      onClick={() => toggle(f.id, false)}
                      disabled={pending}
                      className={`relative w-10 h-5 rounded-full transition-colors flex-shrink-0 ${s.enabled ? 'bg-emerald-600' : 'bg-zinc-700'}`}
                    >
                      <span className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${s.enabled ? 'translate-x-5' : 'translate-x-0.5'}`} />
                    </button>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-zinc-200">{f.name}</p>
                      <p className="text-xs text-zinc-500 mt-0.5">{f.description}</p>
                    </div>
                    {saved === f.id && <span className="text-xs text-emerald-400">Saved</span>}
                  </div>
                  {s.enabled && f.rollout_percent !== undefined && (
                    <div className="mt-3 pl-14">
                      <div className="flex items-center gap-3">
                        <input
                          type="range" min={0} max={100} step={5}
                          value={s.rollout}
                          onChange={e => setStates(prev => ({ ...prev, [f.id]: { ...prev[f.id], rollout: Number(e.target.value) } }))}
                          onMouseUp={() => startTransition(() => saveFlag(f.id))}
                          className="flex-1 accent-emerald-500"
                        />
                        <span className="text-xs text-zinc-400 w-10 text-right">{s.rollout}%</span>
                      </div>
                      <p className="text-[10px] text-zinc-600 mt-1">Rollout percentage</p>
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        </section>
      )}

      {flags.length === 0 && (
        <div className="text-center py-12 text-zinc-600 text-sm">No flags configured yet.</div>
      )}
    </div>
  )
}
