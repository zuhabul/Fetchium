import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({ variable: "--font-geist-sans", subsets: ["latin"] });
const geistMono = Geist_Mono({ variable: "--font-geist-mono", subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Fetchium — The Search API That Thinks",
  description:
    "11 federated sources, 8-signal ranking, evidence graphs, cross-session learning. The most advanced search and scraping API for AI applications.",
  metadataBase: new URL("https://fetchium.com"),
  keywords: ["search api", "web scraping api", "firecrawl alternative", "ai search", "rust search"],
  authors: [{ name: "Fetchium" }],
  alternates: {
    canonical: "/",
  },
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    title: "Fetchium — The Search API That Thinks",
    description:
      "11 federated sources, 8-signal ranking, evidence graphs, cross-session learning.",
    type: "website",
    url: "https://fetchium.com",
  },
  twitter: {
    card: "summary_large_image",
    title: "Fetchium — The Search API That Thinks",
    description: "The most advanced search and scraping API for AI applications.",
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="scroll-smooth">
      <body className={`${geistSans.variable} ${geistMono.variable} antialiased`}>
        {children}
      </body>
    </html>
  );
}
