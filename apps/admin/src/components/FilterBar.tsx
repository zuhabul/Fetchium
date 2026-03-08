'use client'

import { useRouter, usePathname, useSearchParams } from 'next/navigation'
import { useCallback, useTransition } from 'react'

interface SelectFilter {
  key: string
  type: 'select'
  options: { value: string; label: string }[]
}

interface SearchFilter {
  key: string
  type: 'search'
  placeholder?: string
}

type FilterDef = SelectFilter | SearchFilter

interface FilterBarProps {
  filters: FilterDef[]
  current: Record<string, string>
  children?: React.ReactNode
}

export default function FilterBar({ filters, current, children }: FilterBarProps) {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const [, startTransition] = useTransition()

  const update = useCallback((key: string, value: string) => {
    const params = new URLSearchParams(searchParams.toString())
    if (value) params.set(key, value)
    else params.delete(key)
    params.delete('page')
    startTransition(() => router.push(`${pathname}?${params.toString()}`))
  }, [router, pathname, searchParams])

  return (
    <div className="flex items-center gap-3 flex-wrap">
      {filters.map(f =>
        f.type === 'select' ? (
          <select
            key={f.key}
            defaultValue={current[f.key] ?? ''}
            onChange={e => update(f.key, e.target.value)}
            className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-300 focus:outline-none focus:border-zinc-500"
          >
            {(f as SelectFilter).options.map(o => (
              <option key={o.value} value={o.value}>{o.label}</option>
            ))}
          </select>
        ) : (
          <input
            key={f.key}
            type="search"
            placeholder={(f as SearchFilter).placeholder ?? 'Search...'}
            defaultValue={current[f.key] ?? ''}
            onChange={e => update(f.key, e.target.value)}
            className="bg-zinc-800 border border-zinc-700 rounded-md px-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 w-56"
          />
        )
      )}
      {children}
    </div>
  )
}
