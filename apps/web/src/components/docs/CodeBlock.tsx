"use client";
import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

interface Props {
  code: string;
  language?: string;
  filename?: string;
}

export default function CodeBlock({ code, language = "bash", filename }: Props) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(code).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="group relative rounded-2xl overflow-hidden border border-white/[0.07] bg-[#0a0c14] mb-4">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/[0.05] bg-white/[0.015]">
        <div className="flex items-center gap-3">
          <div className="flex gap-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-red-500/40" />
            <div className="w-2.5 h-2.5 rounded-full bg-yellow-500/40" />
            <div className="w-2.5 h-2.5 rounded-full bg-green-500/40" />
          </div>
          {filename && <span className="text-[11px] text-slate-500 font-mono">{filename}</span>}
          {!filename && <span className="text-[11px] text-slate-600 uppercase tracking-wider font-semibold">{language}</span>}
        </div>
        <button onClick={copy}
          className="flex items-center gap-1.5 text-[11px] text-slate-500 hover:text-slate-300 transition-colors bg-white/5 hover:bg-white/8 rounded-md px-2.5 py-1.5 min-h-[32px]">
          <AnimatePresence mode="wait" initial={false}>
            {copied
              ? <motion.span key="check" initial={{ opacity:0, scale:0.8 }} animate={{ opacity:1, scale:1 }} exit={{ opacity:0 }} className="flex items-center gap-1 text-emerald-400">
                  <Check className="w-3 h-3" /> Copied!
                </motion.span>
              : <motion.span key="copy" initial={{ opacity:0 }} animate={{ opacity:1 }} exit={{ opacity:0 }} className="flex items-center gap-1">
                  <Copy className="w-3 h-3" /> Copy
                </motion.span>
            }
          </AnimatePresence>
        </button>
      </div>
      <pre className="p-3 sm:p-4 overflow-x-auto text-[12px] sm:text-[13px] leading-6 font-mono text-slate-300">
        <code>{code}</code>
      </pre>
    </div>
  );
}
