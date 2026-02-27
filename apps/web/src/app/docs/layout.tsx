import type { Metadata } from "next";
import DocsLayoutClient from "@/components/docs/DocsLayoutClient";

export const metadata: Metadata = {
  title: { default: "Documentation", template: "%s — Fetchium Docs" },
  description: "Complete documentation for the Fetchium API — authentication, endpoints, SDKs, and self-hosting guides.",
};

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return <DocsLayoutClient>{children}</DocsLayoutClient>;
}
