import type { Metadata } from "next";
import Navbar from "@/components/Navbar";
import Pricing from "@/components/Pricing";
import FAQ from "@/components/FAQ";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Plans — Current API Tier Limits",
  description:
    "Current Fetchium API tier limits synced to the auth configuration: Free, Starter, Pro, and Enterprise.",
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
