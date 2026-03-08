import type { Metadata } from "next";
import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import TrustBar from "@/components/TrustBar";
import WorkflowSection from "@/components/WorkflowSection";
import Features from "@/components/Features";
import UseCases from "@/components/UseCases";
import HowItWorks from "@/components/HowItWorks";
import CodeDemo from "@/components/CodeDemo";
import Comparison from "@/components/Comparison";
import Pricing from "@/components/Pricing";
import FAQ from "@/components/FAQ";
import CTABand from "@/components/CTABand";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium — The Search API That Thinks | Cheapest Full-Pipeline AI Search",
  description:
    "One API call for search + extraction + citations across 11+ backends. Built for RAG pipelines, AI agents, and research workflows. From $0.58/1K queries. 1,000 free requests/month, no credit card.",
  alternates: {
    canonical: "/",
  },
  keywords: [
    "search API", "web search API for AI", "RAG retrieval API", "AI search API",
    "firecrawl alternative", "tavily alternative", "serpapi alternative", "web scraping for LLMs",
    "MCP search tools", "token budgeted extraction", "federated search API"
  ],
  openGraph: {
    title: "Fetchium — The Search API That Thinks",
    description: "11+ federated backends, 8-signal ranking, 5-layer extraction, token-budgeted context. From $0.58/1K queries.",
    type: "website",
    url: "https://fetchium.com",
  },
  twitter: {
    card: "summary_large_image",
    title: "Fetchium — The Search API That Thinks",
    description: "Cheapest full-pipeline search API on the market. From $0.58/1K. Free tier forever.",
  },
};

export default function Home() {
  return (
    <main className="min-h-screen bg-surface">
      <Navbar />
      <Hero />
      <TrustBar />
      <WorkflowSection />
      <Features />
      <UseCases />
      <HowItWorks />
      <CodeDemo />
      <Comparison />
      <Pricing />
      <FAQ />
      <CTABand />
      <Footer />
    </main>
  );
}
