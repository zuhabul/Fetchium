'use client'

import { useRouter, usePathname, useSearchParams } from 'next/navigation'
import { useTransition } from 'react'

interface PaginationBarProps {
  page: number
  total: number
  pageSize: number
  shown: number
}

export default function PaginationBar({ page, total, pageSize, shown }: PaginationBarProps) {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const [, startTransition] = useTransition()

  const go = (p: number) => {
    const params = new URLSearchParams(searchParams.toString())
    params.set('page', String(p))
    startTransition(() => router.push(`${pathname}?${params.toString()}`))
  }

  const hasPrev = page > 1
  const hasNext = shown >= pageSize

  return (
    <div className="flex flex-col gap-3 text-sm text-zinc-500 sm:flex-row sm:items-center sm:justify-between">
      <span>Showing {shown} of {total}</span>
      <div className="flex w-full items-center gap-2 sm:w-auto">
        <button
          onClick={() => go(page - 1)}
          disabled={!hasPrev}
          className="flex-1 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-40 sm:flex-none sm:py-1.5"
        >
          Previous
        </button>
        <span className="min-w-14 text-center text-xs text-zinc-600">Page {page}</span>
        <button
          onClick={() => go(page + 1)}
          disabled={!hasNext}
          className="flex-1 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-40 sm:flex-none sm:py-1.5"
        >
          Next
        </button>
      </div>
    </div>
  )
}
