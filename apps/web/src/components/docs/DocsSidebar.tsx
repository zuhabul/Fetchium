"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface NavItem {
  title: string;
  href?: string;
  items?: { title: string; href: string; badge?: string }[];
}

function hrefPath(href: string): string {
  try {
    return new URL(href).pathname;
  } catch {
    return href;
  }
}

const NAV: NavItem[] = [
  {
    title: "Getting Started",
    items: [
      { title: "Introduction", href: "https://docs.fetchium.com" },
      { title: "Quick Start", href: "https://docs.fetchium.com/quickstart" },
      { title: "Authentication", href: "https://docs.fetchium.com/authentication" },
      { title: "Rate Limits", href: "https://docs.fetchium.com/rate-limits" },
      { title: "Error Handling", href: "https://docs.fetchium.com/errors" },
    ],
  },
  {
    title: "API Reference",
    items: [
      { title: "Search", href: "https://docs.fetchium.com/api/search", badge: "POST" },
      { title: "Scrape / Fetch", href: "https://docs.fetchium.com/api/scrape", badge: "POST" },
      { title: "Research", href: "https://docs.fetchium.com/api/research", badge: "POST" },
      { title: "Async Jobs", href: "https://docs.fetchium.com/api/async-jobs", badge: "ASYNC" },
      { title: "Estimate", href: "https://docs.fetchium.com/api/estimate", badge: "POST" },
      { title: "YouTube", href: "https://docs.fetchium.com/api/youtube", badge: "POST" },
      { title: "Social", href: "https://docs.fetchium.com/api/social", badge: "POST" },
      { title: "Usage Stats", href: "https://docs.fetchium.com/api/usage", badge: "GET" },
      { title: "Health Check", href: "https://docs.fetchium.com/api/health", badge: "GET" },
      { title: "Admin Keys", href: "https://docs.fetchium.com/api/admin-keys", badge: "ADMIN" },
      { title: "Proxy Admin", href: "https://docs.fetchium.com/api/proxy-admin", badge: "ADMIN" },
    ],
  },
  {
    title: "SDKs & Integrations",
    items: [
      { title: "TypeScript / Node.js", href: "https://docs.fetchium.com/sdk/typescript" },
      { title: "Python", href: "https://docs.fetchium.com/sdk/python" },
      { title: "curl Examples", href: "https://docs.fetchium.com/sdk/curl" },
      { title: "MCP Protocol", href: "https://docs.fetchium.com/sdk/mcp", badge: "NEW" },
    ],
  },
  {
    title: "Algorithms",
    items: [
      { title: "HyperFusion Ranking", href: "https://docs.fetchium.com/algorithms/hyperfusion" },
      { title: "CEP Extraction", href: "https://docs.fetchium.com/algorithms/cep" },
      { title: "QATBE Token Budget", href: "https://docs.fetchium.com/algorithms/qatbe" },
      { title: "SPRE Pre-ranking", href: "https://docs.fetchium.com/algorithms/spre" },
    ],
  },
];

function SectionGroup({ item, onLinkClick }: { item: NavItem; onLinkClick?: () => void }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(
    item.items?.some((i) => {
      const path = hrefPath(i.href);
      return pathname === path || pathname.startsWith(path + "/");
    }) ?? true,
  );

  return (
    <div className="mb-1">
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex w-full items-center justify-between px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-slate-500 hover:text-slate-400 transition-colors"
      >
        {item.title}
        <motion.svg
          animate={{ rotate: open ? 180 : 0 }}
          transition={{ duration: 0.2 }}
          className="w-3 h-3 shrink-0"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M19 9l-7 7-7-7" />
        </motion.svg>
      </button>
      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.25, ease: "easeInOut" }}
            style={{ overflow: "hidden" }}
          >
            {item.items?.map((link) => {
              const path = hrefPath(link.href);
              const active = pathname === path;
              return (
                <Link
                  key={link.href}
                  href={link.href}
                  onClick={onLinkClick}
                  className={`flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-all duration-150 my-0.5 ${
                    active
                      ? "bg-indigo-500/15 text-indigo-300 font-medium border-l-2 border-indigo-500"
                      : "text-slate-400 hover:text-slate-200 hover:bg-white/[0.04]"
                  }`}
                >
                  <span>{link.title}</span>
                  {link.badge && (
                    <span
                      className={`text-[10px] font-bold px-1.5 py-0.5 rounded font-mono ${
                        link.badge === "NEW"
                          ? "bg-emerald-500/15 text-emerald-400"
                        : link.badge === "POST"
                            ? "bg-indigo-500/15 text-indigo-400"
                        : link.badge === "GET"
                              ? "bg-sky-500/15 text-sky-400"
                              : link.badge === "ASYNC"
                                ? "bg-fuchsia-500/15 text-fuchsia-400"
                              : "bg-amber-500/15 text-amber-400"
                      }`}
                    >
                      {link.badge}
                    </span>
                  )}
                </Link>
              );
            })}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function SidebarContent({ onLinkClick }: { onLinkClick?: () => void }) {
  return (
    <>
      <div className="text-[11px] font-semibold uppercase tracking-widest text-slate-600 mb-4 px-3">
        Documentation
      </div>
      {NAV.map((item, i) => (
        <SectionGroup key={i} item={item} onLinkClick={onLinkClick} />
      ))}
      <div className="mt-6 mx-3 p-3 rounded-xl border border-indigo-500/20 bg-indigo-500/5">
        <div className="text-xs font-medium text-indigo-300 mb-1">Need help?</div>
        <div className="text-[11px] text-slate-500 mb-2">Join our Discord or open an issue on GitHub.</div>
        <a
          href="https://github.com/zuhabul/Fetchium"
          target="_blank"
          rel="noopener noreferrer"
          className="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition-colors"
        >
          GitHub →
        </a>
      </div>
    </>
  );
}

interface DocsSidebarProps {
  isOpen?: boolean;
  onClose?: () => void;
}

export default function DocsSidebar({ isOpen = false, onClose }: DocsSidebarProps) {
  return (
    <>
      <nav className="hidden sm:block w-60 shrink-0 sticky top-14 self-start h-[calc(100vh-3.5rem)] overflow-y-auto py-6 pr-4 border-r border-white/[0.06]">
        <SidebarContent />
      </nav>

      <AnimatePresence>
        {isOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm sm:hidden"
              onClick={onClose}
            />
            <motion.nav
              initial={{ x: "-100%" }}
              animate={{ x: 0 }}
              exit={{ x: "-100%" }}
              transition={{ type: "spring", damping: 28, stiffness: 220 }}
              className="fixed top-12 left-0 bottom-0 w-72 z-50 bg-[#06070d] border-r border-white/[0.08] overflow-y-auto py-4 px-2 sm:hidden"
            >
              <SidebarContent onLinkClick={onClose} />
            </motion.nav>
          </>
        )}
      </AnimatePresence>
    </>
  );
}
