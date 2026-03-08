import type { Metadata } from "next";
import DocsLayoutClient from "@/components/docs/DocsLayoutClient";

export const metadata: Metadata = {
  title: { default: "Documentation", template: "%s — Fetchium Docs" },
  description: "Complete documentation for the Fetchium API — authentication, endpoints, SDKs, and integration guides.",
  metadataBase: new URL("https://docs.fetchium.com"),
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    title: "Fetchium Docs",
    description:
      "Complete documentation for the Fetchium API — authentication, endpoints, SDKs, and integration guides.",
    type: "website",
    url: "https://docs.fetchium.com",
    siteName: "Fetchium Docs",
  },
};

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return <DocsLayoutClient>{children}</DocsLayoutClient>;
}
