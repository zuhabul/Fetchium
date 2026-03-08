"use client";

import { useState, useEffect, useRef } from "react";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import { Menu, X, Zap, ExternalLink, ChevronDown, Search, FileText, BookOpen, Code2, ShieldCheck, GitCompare } from "lucide-react";

const productLinks = [
  { href: "/product/search", label: "Search API", desc: "11-backend federated search", icon: Search },
  { href: "/product/extract", label: "Extract API", desc: "5-layer CEP content extraction", icon: FileText },
  { href: "/product/research", label: "Research API", desc: "Multi-agent deep research (AMRS)", icon: BookOpen },
  { href: "/product/mcp", label: "MCP Tools", desc: "Model Context Protocol server", icon: Code2 },
];

const compareLinks = [
  { href: "/compare/tavily", label: "vs Tavily" },
  { href: "/compare/exa", label: "vs Exa" },
  { href: "/compare/serpapi", label: "vs SerpAPI" },
  { href: "/compare/firecrawl", label: "vs Firecrawl" },
  { href: "/compare/perplexity", label: "vs Perplexity" },
];

function DropdownMenu({ items, wide = false }: { items: typeof productLinks | typeof compareLinks; wide?: boolean }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 8, scale: 0.97 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: 8, scale: 0.97 }}
      transition={{ duration: 0.18, ease: [0.22, 1, 0.36, 1] }}
      className={`absolute top-full left-0 mt-2 rounded-xl border border-slate-800 bg-[rgba(8,10,18,0.99)] backdrop-blur-2xl shadow-[0_16px_56px_rgba(0,0,0,0.7)] overflow-hidden z-50 ${wide ? "w-64" : "w-44"}`}
    >
      <div className="p-1.5">
        {items.map((item) => (
          <Link
            key={item.href}
            href={item.href}
            className="flex items-start gap-3 rounded-lg px-3 py-2.5 text-sm transition-all hover:bg-white/5 group"
          >
            {"icon" in item && item.icon && (
              <item.icon className="h-4 w-4 text-indigo-400 shrink-0 mt-0.5" strokeWidth={1.75} />
            )}
            <div className="min-w-0">
              <div className="text-[13px] font-medium text-slate-200 group-hover:text-white">
                {item.label}
              </div>
              {"desc" in item && item.desc && (
                <div className="text-[11px] text-slate-600 mt-0.5 truncate">{item.desc}</div>
              )}
            </div>
          </Link>
        ))}
      </div>
    </motion.div>
  );
}

export default function Navbar() {
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);
  const [activeDropdown, setActiveDropdown] = useState<string | null>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 12);
    handleScroll();
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setActiveDropdown(null);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  return (
    <motion.nav
      initial={{ opacity: 0, y: -16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? "border-b border-[rgba(99,102,241,0.12)] bg-[rgba(6,7,13,0.92)] backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.4)]"
          : "border-b border-transparent bg-transparent"
      }`}
    >
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          {/* Logo */}
          <Link href="/" className="group flex items-center gap-2.5 font-semibold text-white shrink-0">
            <div className="relative flex h-8 w-8 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 shadow-[0_0_16px_rgba(99,102,241,0.4)] transition-shadow group-hover:shadow-[0_0_24px_rgba(99,102,241,0.6)]">
              <Zap className="h-4 w-4 text-white" strokeWidth={2.5} />
            </div>
            <span className="text-[15px] tracking-tight text-slate-100">
              Fetchi<span className="text-indigo-400">um</span>
            </span>
          </Link>

          {/* Desktop nav */}
          <div ref={dropdownRef} className="hidden items-center gap-0.5 md:flex">
            {/* Product dropdown */}
            <div className="relative">
              <button
                onClick={() => setActiveDropdown(activeDropdown === "product" ? null : "product")}
                className="flex items-center gap-1 rounded-lg px-3.5 py-2 text-sm font-medium text-slate-400 transition-all duration-200 hover:bg-white/5 hover:text-slate-100"
              >
                Product
                <ChevronDown className={`h-3.5 w-3.5 transition-transform duration-200 ${activeDropdown === "product" ? "rotate-180" : ""}`} />
              </button>
              <AnimatePresence>
                {activeDropdown === "product" && (
                  <DropdownMenu items={productLinks} wide />
                )}
              </AnimatePresence>
            </div>

            <Link href="/pricing" className="rounded-lg px-3.5 py-2 text-sm font-medium text-slate-400 transition-all duration-200 hover:bg-white/5 hover:text-slate-100">
              Pricing
            </Link>

            <Link href="https://docs.fetchium.com" className="rounded-lg px-3.5 py-2 text-sm font-medium text-slate-400 transition-all duration-200 hover:bg-white/5 hover:text-slate-100">
              Docs
            </Link>

            {/* Compare dropdown */}
            <div className="relative">
              <button
                onClick={() => setActiveDropdown(activeDropdown === "compare" ? null : "compare")}
                className="flex items-center gap-1 rounded-lg px-3.5 py-2 text-sm font-medium text-slate-400 transition-all duration-200 hover:bg-white/5 hover:text-slate-100"
              >
                Compare
                <ChevronDown className={`h-3.5 w-3.5 transition-transform duration-200 ${activeDropdown === "compare" ? "rotate-180" : ""}`} />
              </button>
              <AnimatePresence>
                {activeDropdown === "compare" && (
                  <DropdownMenu items={compareLinks} />
                )}
              </AnimatePresence>
            </div>

            <Link href="/blog" className="rounded-lg px-3.5 py-2 text-sm font-medium text-slate-400 transition-all duration-200 hover:bg-white/5 hover:text-slate-100">
              Blog
            </Link>

            <Link href="/security" className="flex items-center gap-1.5 rounded-lg px-3.5 py-2 text-sm font-medium text-slate-400 transition-all duration-200 hover:bg-white/5 hover:text-slate-100">
              <ShieldCheck className="h-3.5 w-3.5" />
              Security
            </Link>
          </div>

          {/* Desktop right actions */}
          <motion.div
            className="hidden items-center gap-3 md:flex"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.35, duration: 0.5 }}
          >
            <Link
              href="https://app.fetchium.com"
              target="_blank"
              rel="noopener noreferrer"
              className="group relative flex items-center gap-1.5 overflow-hidden rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-4 py-2 text-sm font-semibold text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] transition-all duration-300 hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]"
            >
              <span className="absolute inset-0 bg-gradient-to-r from-indigo-400 to-violet-500 opacity-0 transition-opacity group-hover:opacity-100" />
              <span className="relative">Get API Key</span>
              <ExternalLink className="relative h-3.5 w-3.5" />
            </Link>
          </motion.div>

          {/* Mobile hamburger */}
          <button
            className="flex h-11 w-11 items-center justify-center rounded-lg border border-slate-700/50 bg-slate-900/40 text-slate-300 transition-all hover:bg-slate-800 hover:text-white md:hidden"
            onClick={() => setOpen(!open)}
            aria-label="Toggle menu"
          >
            <AnimatePresence mode="wait" initial={false}>
              {open ? (
                <motion.span key="close" initial={{ rotate: -90, opacity: 0 }} animate={{ rotate: 0, opacity: 1 }} exit={{ rotate: 90, opacity: 0 }} transition={{ duration: 0.15 }}>
                  <X className="h-4 w-4" />
                </motion.span>
              ) : (
                <motion.span key="menu" initial={{ rotate: 90, opacity: 0 }} animate={{ rotate: 0, opacity: 1 }} exit={{ rotate: -90, opacity: 0 }} transition={{ duration: 0.15 }}>
                  <Menu className="h-4 w-4" />
                </motion.span>
              )}
            </AnimatePresence>
          </button>
        </div>
      </div>

      {/* Mobile drawer */}
      <AnimatePresence>
        {open && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.25, ease: [0.22, 1, 0.36, 1] }}
            className="overflow-hidden border-t border-[rgba(99,102,241,0.1)] bg-[rgba(6,7,13,0.98)] backdrop-blur-2xl md:hidden max-h-[80vh] overflow-y-auto"
          >
            <div className="px-4 py-4 space-y-0.5">
              <p className="px-4 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-slate-600">Product</p>
              {productLinks.map((link) => (
                <Link key={link.href} href={link.href} className="flex items-center gap-3 rounded-xl px-4 py-2.5 text-sm font-medium text-slate-400 transition-all hover:bg-white/5 hover:text-white" onClick={() => setOpen(false)}>
                  <link.icon className="h-4 w-4 text-indigo-400 shrink-0" strokeWidth={1.75} />
                  <div>
                    <div className="text-[13px] font-medium">{link.label}</div>
                    <div className="text-[11px] text-slate-600">{link.desc}</div>
                  </div>
                </Link>
              ))}

              <div className="my-2 h-px bg-white/5" />

              {[
                { href: "/pricing", label: "Pricing" },
                { href: "https://docs.fetchium.com", label: "Documentation" },
                { href: "/blog", label: "Blog" },
                { href: "/security", label: "Security" },
              ].map((link) => (
                <Link key={link.href} href={link.href} className="flex items-center rounded-xl px-4 py-2.5 text-sm font-medium text-slate-400 transition-all hover:bg-white/5 hover:text-white" onClick={() => setOpen(false)}>
                  {link.label}
                </Link>
              ))}

              <div className="my-2 h-px bg-white/5" />
              <p className="px-4 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-slate-600">Compare</p>
              {compareLinks.map((link) => (
                <Link key={link.href} href={link.href} className="flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm font-medium text-slate-400 transition-all hover:bg-white/5 hover:text-white" onClick={() => setOpen(false)}>
                  <GitCompare className="h-3.5 w-3.5 text-slate-600" />
                  {link.label}
                </Link>
              ))}

              <div className="my-3 h-px bg-white/5" />

              <Link
                href="https://app.fetchium.com"
                target="_blank"
                rel="noopener noreferrer"
                className="mt-2 flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-4 py-3 text-sm font-semibold text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] min-h-[44px]"
                onClick={() => setOpen(false)}
              >
                Get API Key — Free
                <ExternalLink className="h-3.5 w-3.5" />
              </Link>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.nav>
  );
}
