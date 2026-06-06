'use client'

import { useRouter, usePathname, useSearchParams } from 'next/navigation'
import { useCallback, useEffect, useMemo, useState, useTransition } from 'react'
import { Check, ChevronDown, X } from 'lucide-react'

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

function MobileSelect({
  filter,
  value,
  onSelect,
}: {
  filter: SelectFilter
  value: string
  onSelect: (value: string) => void
}) {
  const [open, setOpen] = useState(false)

  const activeOption = useMemo(
    () => filter.options.find((option) => option.value === value) ?? filter.options[0],
    [filter.options, value]
  )

  useEffect(() => {
    if (!open) return

    const previousOverflow = document.body.style.overflow
    document.body.style.overflow = 'hidden'

    function handleEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setOpen(false)
      }
    }

    window.addEventListener('keydown', handleEscape)

    return () => {
      document.body.style.overflow = previousOverflow
      window.removeEventListener('keydown', handleEscape)
    }
  }, [open])

  return (
    <>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="flex min-h-11 w-full items-center justify-between rounded-md border border-zinc-700 bg-zinc-800 px-3 text-left text-sm text-zinc-200 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] transition-colors hover:border-zinc-600"
        aria-haspopup="dialog"
        aria-expanded={open}
      >
        <span className="truncate">{activeOption?.label ?? 'Select'}</span>
        <ChevronDown className="h-4 w-4 flex-shrink-0 text-zinc-500" />
      </button>

      {open && (
        <div className="fixed inset-0 z-50 sm:hidden">
          <button
            type="button"
            className="absolute inset-0 bg-zinc-950/80 backdrop-blur-sm"
            aria-label="Close filter picker"
            onClick={() => setOpen(false)}
          />
          <div className="absolute inset-x-0 bottom-0 rounded-t-2xl border-t border-zinc-800 bg-zinc-900 px-4 pb-[calc(1rem+env(safe-area-inset-bottom,0px))] pt-4 shadow-2xl">
            <div className="mb-4 flex items-center justify-between gap-3">
              <div>
                <p className="text-[11px] uppercase tracking-[0.18em] text-zinc-600">Filter</p>
                <h3 className="text-sm font-semibold text-zinc-100">
                  {filter.options[0]?.label ?? 'Choose option'}
                </h3>
              </div>
              <button
                type="button"
                onClick={() => setOpen(false)}
                className="flex h-10 w-10 items-center justify-center rounded-full border border-zinc-800 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200"
                aria-label="Close filter picker"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="space-y-2">
              {filter.options.map((option) => {
                const selected = option.value === value
                return (
                  <button
                    key={option.value || '__all__'}
                    type="button"
                    onClick={() => {
                      onSelect(option.value)
                      setOpen(false)
                    }}
                    className={`flex w-full items-center justify-between rounded-xl border px-4 py-3 text-left text-sm transition-colors ${
                      selected
                        ? 'border-zinc-600 bg-zinc-800 text-zinc-100'
                        : 'border-zinc-800 bg-zinc-950/70 text-zinc-400 hover:border-zinc-700 hover:text-zinc-200'
                    }`}
                  >
                    <span>{option.label}</span>
                    {selected && <Check className="h-4 w-4 flex-shrink-0 text-zinc-300" />}
                  </button>
                )
              })}
            </div>
          </div>
        </div>
      )}
    </>
  )
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
    <div className="grid grid-cols-1 gap-3 sm:flex sm:flex-wrap sm:items-center">
      {filters.map(f =>
        f.type === 'select' ? (
          <div key={f.key} className="w-full sm:w-auto">
            <div className="sm:hidden">
              <MobileSelect
                filter={f}
                value={current[f.key] ?? ''}
                onSelect={(value) => update(f.key, value)}
              />
            </div>

            <div className="relative hidden sm:block">
              <select
                defaultValue={current[f.key] ?? ''}
                onChange={e => update(f.key, e.target.value)}
                className="min-h-9 w-full appearance-none rounded-md border border-zinc-700 bg-zinc-800 px-3 pr-10 text-sm text-zinc-300 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] focus:outline-none focus:border-zinc-500 sm:w-auto sm:py-1.5"
              >
                {(f as SelectFilter).options.map(o => (
                  <option key={o.value} value={o.value}>{o.label}</option>
                ))}
              </select>
              <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500" />
            </div>
          </div>
        ) : (
          <input
            key={f.key}
            type="search"
            placeholder={(f as SearchFilter).placeholder ?? 'Search...'}
            defaultValue={current[f.key] ?? ''}
            onChange={e => update(f.key, e.target.value)}
            className="min-h-11 w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 text-sm text-zinc-100 placeholder-zinc-600 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] focus:outline-none focus:border-zinc-500 sm:min-h-9 sm:w-56 sm:py-1.5"
          />
        )
      )}
      {children}
    </div>
  )
}
