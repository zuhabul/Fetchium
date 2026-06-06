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
    <section className="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h2 className="text-sm font-semibold text-zinc-300">Display Preferences</h2>
          <p className="mt-2 text-xs text-zinc-500">
            Data density controls table row height and spacing across the admin console.
          </p>
        </div>
        <div className="flex flex-wrap gap-2 sm:justify-end">
          {(['compact', 'normal'] as const).map(opt => (
            <button
              key={opt}
              onClick={() => handleChange(opt)}
              className={`inline-flex min-h-11 items-center justify-center rounded-md border px-4 py-2 text-sm font-medium capitalize transition-colors sm:min-h-9 sm:py-1.5 ${
                density === opt
                  ? 'bg-zinc-700 border-zinc-600 text-zinc-100'
                  : 'bg-zinc-800 border-zinc-700 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-200'
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
