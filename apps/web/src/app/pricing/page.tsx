import type { Metadata } from "next";
import Navbar from "@/components/Navbar";
import Pricing from "@/components/Pricing";
import FAQ from "@/components/FAQ";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Pricing — Cheapest Full-Pipeline Search API From $9/month",
  description:
    "Free tier: 1,000 req/month forever. Starter: $9/mo for 10,000 requests ($0.90/1K). Growth: $29/mo for 50,000 requests ($0.58/1K). Cheaper than Tavily, Exa, and SerpAPI.",
  keywords: [
    "search API pricing", "web scraping API price", "tavily alternative cheaper",
    "exa alternative", "serpapi alternative", "RAG search API cost"
  ],
};

export default function PricingPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />
      <main className="pt-20">
        <Pricing />
        <FAQ />
      </main>
      <Footer />
    </div>
  );
}
