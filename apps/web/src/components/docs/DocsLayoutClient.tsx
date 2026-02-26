"use client";
import { useState } from "react";
import DocsSidebar from "@/components/docs/DocsSidebar";
import DocsHeader from "@/components/docs/DocsHeader";

export default function DocsLayoutClient({ children }: { children: React.ReactNode }) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="min-h-screen bg-[#06070d]">
      <DocsHeader onMenuClick={() => setSidebarOpen(true)} />
      <div className="max-w-7xl mx-auto sm:flex px-0 sm:px-6">
        <DocsSidebar isOpen={sidebarOpen} onClose={() => setSidebarOpen(false)} />
        <main className="flex-1 min-w-0 py-8 sm:py-10 px-4 sm:pl-10 sm:pr-0">
          {children}
        </main>
      </div>
    </div>
  );
}
