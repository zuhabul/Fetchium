import Navbar from "@/components/Navbar";
import Comparison from "@/components/Comparison";
import Pricing from "@/components/Pricing";
import Footer from "@/components/Footer";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Fetchium vs Firecrawl — Full Comparison",
  description:
    "Fetchium vs Firecrawl: 11-backend federation, 8-signal ranking, evidence graphs, cross-session learning — all missing from Firecrawl. Same price, 10x more features.",
};

export default function VsFirecrawl() {
  return (
    <main className="min-h-screen bg-surface">
      <Navbar />
      <div className="pt-24 pb-12 px-4 text-center">
        <h1 className="text-4xl font-bold text-white">
          Fetchium vs{" "}
          <span className="text-white/40">Firecrawl</span>
        </h1>
        <p className="mt-4 text-white/50 max-w-2xl mx-auto">
          Firecrawl is a great scraper. Fetchium is a complete intelligence layer —
          search federation, ranking, learning, monitoring, evidence graphs, and more.
          At 87% lower cost than Firecrawl Pro.
        </p>
      </div>
      <Comparison />
      <Pricing />
      <Footer />
    </main>
  );
}
