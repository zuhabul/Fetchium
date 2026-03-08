import type { Metadata } from "next";
import DocsLayoutClient from "@/components/docs/DocsLayoutClient";

export const metadata: Metadata = {
  title: { default: "Documentation", template: "%s — Fetchium Docs" },
  description: "Complete documentation for the Fetchium API — authentication, endpoints, SDKs, and self-hosting guides.",
  metadataBase: new URL("https://docs.fetchium.com"),
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    title: "Fetchium Docs",
    description:
      "Complete documentation for the Fetchium API — authentication, endpoints, SDKs, and self-hosting guides.",
    type: "website",
    url: "https://docs.fetchium.com",
    siteName: "Fetchium Docs",
  },
};

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return <DocsLayoutClient>{children}</DocsLayoutClient>;
}
