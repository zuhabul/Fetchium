"use client";
import Link from "next/link";
import { useState } from "react";
import { Search, Menu } from "lucide-react";

interface DocsHeaderProps {
  onMenuClick?: () => void;
}

export default function DocsHeader({ onMenuClick }: DocsHeaderProps) {
  const [q, setQ] = useState("");

  return (
    <header className="sticky top-0 z-50 border-b border-white/[0.06] bg-[#06070d]/90 backdrop-blur-xl">
      <div className="max-w-7xl mx-auto flex items-center gap-2 sm:gap-4 px-3 sm:px-6 h-12 sm:h-14">
        {/* Mobile menu button — opens sidebar */}
        <button
          className="sm:hidden p-2 -ml-2 text-slate-400 hover:text-slate-200 rounded-lg hover:bg-white/5 transition-colors"
          onClick={onMenuClick}
          aria-label="Open navigation"
        >
          <Menu className="w-5 h-5" />
        </button>

        {/* Logo */}
        <Link href="/" className="flex items-center gap-2 mr-2 sm:mr-4 shrink-0">
          <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-indigo-500 to-violet-600 flex items-center justify-center">
            <svg className="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <span className="font-semibold text-slate-200 text-sm">Fetchium</span>
          <span className="text-slate-600 hidden sm:inline">/</span>
          <span className="text-slate-400 text-sm hidden sm:inline">docs</span>
        </Link>

        {/* Search */}
        <div className="flex-1 max-w-[180px] sm:max-w-sm">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-600" />
            <input
              value={q}
              onChange={e => setQ(e.target.value)}
              placeholder="Search docs..."
              className="w-full pl-9 pr-3 py-1.5 rounded-lg bg-white/[0.04] border border-white/[0.06] text-sm text-slate-300 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500/40 focus:bg-indigo-500/5 transition-all"
            />
            <kbd className="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-slate-600 px-1.5 py-0.5 rounded bg-white/5 border border-white/10 font-mono hidden md:block">⌘K</kbd>
          </div>
        </div>

        {/* Right nav */}
        <nav className="flex items-center gap-1 sm:gap-2 ml-auto">
          <Link href="/" className="text-xs text-slate-500 hover:text-slate-300 transition-colors px-1 sm:px-2 py-1 hidden xs:block">Home</Link>
          <Link href="/pricing" className="text-xs text-slate-500 hover:text-slate-300 transition-colors px-1 sm:px-2 py-1 hidden sm:block">Pricing</Link>
          <a href="https://github.com/zuhabul/Fetchium" target="_blank" rel="noopener noreferrer"
            className="text-xs text-slate-500 hover:text-slate-300 transition-colors px-1 sm:px-2 py-1 hidden sm:block">GitHub</a>
          <Link href="https://app.fetchium.com"
            className="ml-1 sm:ml-2 px-2.5 sm:px-3 py-2 sm:py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors whitespace-nowrap min-h-[36px] flex items-center">
            <span className="hidden sm:inline">Dashboard →</span>
            <span className="sm:hidden">App →</span>
          </Link>
        </nav>
      </div>
    </header>
  );
}
