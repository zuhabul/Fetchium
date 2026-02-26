import type { Metadata } from "next";
import DocsSidebar from "@/components/docs/DocsSidebar";
import DocsHeader from "@/components/docs/DocsHeader";

export const metadata: Metadata = {
  title: { default: "Documentation", template: "%s — HyperSearchX Docs" },
  description: "Complete documentation for the HyperSearchX API — authentication, endpoints, SDKs, and self-hosting guides.",
};

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-[#06070d]">
      <DocsHeader />
      <div className="max-w-7xl mx-auto flex gap-0 px-4 sm:px-6">
        <DocsSidebar />
        <main className="flex-1 min-w-0 py-10 pl-10">
          {children}
        </main>
        {/* Table of contents placeholder — could add per-page */}
      </div>
    </div>
  );
}
