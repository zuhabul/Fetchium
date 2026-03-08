'use client'

import { useState, useEffect } from 'react'

type Density = 'compact' | 'normal'

export default function SettingsClient() {
  const [density, setDensity] = useState<Density>('normal')

  useEffect(() => {
    const stored = localStorage.getItem('fetchium_admin_density') as Density | null
    if (stored) setDensity(stored)
  }, [])

  function handleChange(value: Density) {
    setDensity(value)
    localStorage.setItem('fetchium_admin_density', value)
  }

  return (
    <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
      <h2 className="text-sm font-semibold text-zinc-300 mb-4">Display Preferences</h2>
      <div>
        <p className="text-xs text-zinc-500 mb-3">Data density — controls table row height and spacing.</p>
        <div className="flex items-center gap-3">
          {(['compact', 'normal'] as const).map(opt => (
            <button
              key={opt}
              onClick={() => handleChange(opt)}
              className={`px-4 py-1.5 text-xs font-medium rounded-md border transition-colors capitalize ${
                density === opt
                  ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                  : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-750'
              }`}
            >
              {opt}
            </button>
          ))}
        </div>
      </div>
    </section>
  )
}
