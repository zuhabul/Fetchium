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
    <div className="flex items-center justify-between text-sm text-zinc-500">
      <span>Showing {shown} of {total}</span>
      <div className="flex items-center gap-2">
        <button
          onClick={() => go(page - 1)}
          disabled={!hasPrev}
          className="bg-zinc-800 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md disabled:opacity-40 disabled:cursor-not-allowed hover:bg-zinc-700 transition-colors"
        >
          Previous
        </button>
        <span className="text-zinc-600 text-xs">Page {page}</span>
        <button
          onClick={() => go(page + 1)}
          disabled={!hasNext}
          className="bg-zinc-800 border border-zinc-700 text-zinc-300 text-sm px-3 py-1.5 rounded-md disabled:opacity-40 disabled:cursor-not-allowed hover:bg-zinc-700 transition-colors"
        >
          Next
        </button>
      </div>
    </div>
  )
}
