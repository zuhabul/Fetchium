"use client";
import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion, AnimatePresence } from "framer-motion";

interface NavItem {
  title: string;
  href?: string;
  items?: { title: string; href: string; badge?: string }[];
}

const NAV: NavItem[] = [
  {
    title: "Getting Started",
    items: [
      { title: "Introduction", href: "/docs" },
      { title: "Quick Start", href: "/docs/quickstart" },
      { title: "Authentication", href: "/docs/authentication" },
      { title: "Rate Limits", href: "/docs/rate-limits" },
      { title: "Error Handling", href: "/docs/errors" },
    ],
  },
  {
    title: "API Reference",
    items: [
      { title: "Search", href: "/docs/api/search", badge: "POST" },
      { title: "Scrape", href: "/docs/api/scrape", badge: "POST" },
      { title: "Research", href: "/docs/api/research", badge: "POST" },
      { title: "YouTube Search", href: "/docs/api/youtube", badge: "POST" },
      { title: "Social Research", href: "/docs/api/social", badge: "POST" },
      { title: "Usage Stats", href: "/docs/api/usage", badge: "GET" },
      { title: "Health Check", href: "/docs/api/health", badge: "GET" },
    ],
  },
  {
    title: "SDKs & Integrations",
    items: [
      { title: "TypeScript / Node.js", href: "/docs/sdk/typescript" },
      { title: "Python", href: "/docs/sdk/python" },
      { title: "curl Examples", href: "/docs/sdk/curl" },
      { title: "MCP Protocol", href: "/docs/sdk/mcp", badge: "NEW" },
    ],
  },
  {
    title: "Algorithms",
    items: [
      { title: "HyperFusion Ranking", href: "/docs/algorithms/hyperfusion" },
      { title: "CEP Extraction", href: "/docs/algorithms/cep" },
      { title: "QATBE Token Budget", href: "/docs/algorithms/qatbe" },
      { title: "SPRE Pre-ranking", href: "/docs/algorithms/spre" },
    ],
  },
  {
    title: "Self-Hosting",
    items: [
      { title: "Docker Setup", href: "/docs/self-hosting/docker" },
      { title: "Configuration", href: "/docs/self-hosting/config" },
      { title: "SearXNG Integration", href: "/docs/self-hosting/searxng" },
    ],
  },
];

function SectionGroup({ item }: { item: NavItem }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(
    item.items?.some(i => pathname === i.href || pathname.startsWith(i.href + "/")) ?? true
  );

  return (
    <div className="mb-1">
      <button
        onClick={() => setOpen(o => !o)}
        className="flex w-full items-center justify-between px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-slate-500 hover:text-slate-400 transition-colors"
      >
        {item.title}
        <motion.svg
          animate={{ rotate: open ? 180 : 0 }}
          transition={{ duration: 0.2 }}
          className="w-3 h-3"
          fill="none" stroke="currentColor" viewBox="0 0 24 24"
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
            {item.items?.map(link => {
              const active = pathname === link.href;
              return (
                <Link
                  key={link.href}
                  href={link.href}
                  className={`flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-all duration-150 my-0.5 ${
                    active
                      ? "bg-indigo-500/15 text-indigo-300 font-medium border-l-2 border-indigo-500"
                      : "text-slate-400 hover:text-slate-200 hover:bg-white/[0.04]"
                  }`}
                >
                  <span className={active ? "" : "ml-0"}>{link.title}</span>
                  {link.badge && (
                    <span className={`text-[10px] font-bold px-1.5 py-0.5 rounded font-mono ${
                      link.badge === "NEW" ? "bg-emerald-500/15 text-emerald-400"
                      : link.badge === "POST" ? "bg-indigo-500/15 text-indigo-400"
                      : "bg-sky-500/15 text-sky-400"
                    }`}>
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

export default function DocsSidebar() {
  return (
    <nav className="w-60 shrink-0 sticky top-16 h-[calc(100vh-4rem)] overflow-y-auto py-6 pr-4 border-r border-white/[0.06]">
      <div className="text-[11px] font-semibold uppercase tracking-widest text-slate-600 mb-4 px-3">
        Documentation
      </div>
      {NAV.map((item, i) => (
        <SectionGroup key={i} item={item} />
      ))}
      <div className="mt-6 mx-3 p-3 rounded-xl border border-indigo-500/20 bg-indigo-500/5">
        <div className="text-xs font-medium text-indigo-300 mb-1">Need help?</div>
        <div className="text-[11px] text-slate-500 mb-2">Join our Discord or open an issue on GitHub.</div>
        <a href="https://github.com/zuhabul/hypersearchx" target="_blank" rel="noopener noreferrer"
          className="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition-colors">
          GitHub →
        </a>
      </div>
    </nav>
  );
}
