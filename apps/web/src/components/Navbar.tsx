"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import { Menu, X, Zap, Github, ExternalLink } from "lucide-react";

const navLinks = [
  { href: "#features", label: "Features" },
  { href: "#pricing", label: "Pricing" },
  { href: "/docs", label: "Docs" },
  { href: "#compare", label: "Compare" },
];

export default function Navbar() {
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 12);
    handleScroll();
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <motion.nav
      initial={{ opacity: 0, y: -16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? "border-b border-[rgba(99,102,241,0.12)] bg-[rgba(6,7,13,0.88)] backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.4)]"
          : "border-b border-transparent bg-transparent"
      }`}
    >
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          {/* Logo */}
          <Link
            href="/"
            className="group flex items-center gap-2.5 font-semibold text-white"
          >
            <div className="relative flex h-8 w-8 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 shadow-[0_0_16px_rgba(99,102,241,0.4)] transition-shadow group-hover:shadow-[0_0_24px_rgba(99,102,241,0.6)]">
              <Zap className="h-4 w-4 text-white" strokeWidth={2.5} />
            </div>
            <span className="text-[15px] tracking-tight text-slate-100">
              HyperSearch
              <span className="text-indigo-400">X</span>
            </span>
          </Link>

          {/* Desktop nav links */}
          <div className="hidden items-center gap-1 md:flex">
            {navLinks.map((link, i) => (
              <motion.div
                key={link.href}
                initial={{ opacity: 0, y: -8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.1 + i * 0.05, duration: 0.4 }}
              >
                <Link
                  href={link.href}
                  className="rounded-lg px-3.5 py-2 text-sm font-medium text-slate-400 transition-all duration-200 hover:bg-white/5 hover:text-slate-100"
                >
                  {link.label}
                </Link>
              </motion.div>
            ))}
          </div>

          {/* Desktop right actions */}
          <motion.div
            className="hidden items-center gap-3 md:flex"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.35, duration: 0.5 }}
          >
            <Link
              href="https://github.com/hypersearchx/hypersearchx"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 rounded-lg px-3 py-2 text-sm text-slate-400 transition-all hover:bg-white/5 hover:text-slate-100"
            >
              <Github className="h-4 w-4" />
              <span>GitHub</span>
            </Link>
            <div className="h-4 w-px bg-white/10" />
            <Link
              href="https://app.hypersearchx.zuhabul.com"
              target="_blank"
              rel="noopener noreferrer"
              className="group relative flex items-center gap-1.5 overflow-hidden rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-4 py-2 text-sm font-semibold text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] transition-all duration-300 hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]"
            >
              <span className="absolute inset-0 bg-gradient-to-r from-indigo-400 to-violet-500 opacity-0 transition-opacity group-hover:opacity-100" />
              <span className="relative">Get Started</span>
              <ExternalLink className="relative h-3.5 w-3.5" />
            </Link>
          </motion.div>

          {/* Mobile hamburger */}
          <button
            className="flex h-11 w-11 items-center justify-center rounded-lg border border-white/10 text-slate-400 transition-all hover:bg-white/5 hover:text-white md:hidden"
            onClick={() => setOpen(!open)}
            aria-label="Toggle menu"
          >
            <AnimatePresence mode="wait" initial={false}>
              {open ? (
                <motion.span
                  key="close"
                  initial={{ rotate: -90, opacity: 0 }}
                  animate={{ rotate: 0, opacity: 1 }}
                  exit={{ rotate: 90, opacity: 0 }}
                  transition={{ duration: 0.15 }}
                >
                  <X className="h-4 w-4" />
                </motion.span>
              ) : (
                <motion.span
                  key="menu"
                  initial={{ rotate: 90, opacity: 0 }}
                  animate={{ rotate: 0, opacity: 1 }}
                  exit={{ rotate: -90, opacity: 0 }}
                  transition={{ duration: 0.15 }}
                >
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
            className="overflow-hidden border-t border-[rgba(99,102,241,0.1)] bg-[rgba(6,7,13,0.96)] backdrop-blur-2xl md:hidden"
          >
            <div className="space-y-1 px-4 py-4">
              {navLinks.map((link, i) => (
                <motion.div
                  key={link.href}
                  initial={{ opacity: 0, x: -12 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: i * 0.05, duration: 0.2 }}
                >
                  <Link
                    href={link.href}
                    className="flex items-center rounded-xl px-4 py-2.5 text-sm font-medium text-slate-400 transition-all hover:bg-white/5 hover:text-white"
                    onClick={() => setOpen(false)}
                  >
                    {link.label}
                  </Link>
                </motion.div>
              ))}

              <div className="my-3 h-px bg-white/5" />

              <Link
                href="https://github.com/hypersearchx/hypersearchx"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm font-medium text-slate-400 transition-all hover:bg-white/5 hover:text-white"
                onClick={() => setOpen(false)}
              >
                <Github className="h-4 w-4" />
                Star on GitHub
              </Link>

              <Link
                href="https://app.hypersearchx.zuhabul.com"
                target="_blank"
                rel="noopener noreferrer"
                className="mt-2 flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 px-4 py-3 text-sm font-semibold text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] min-h-[44px]"
                onClick={() => setOpen(false)}
              >
                Get Started Free
                <ExternalLink className="h-3.5 w-3.5" />
              </Link>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.nav>
  );
}
