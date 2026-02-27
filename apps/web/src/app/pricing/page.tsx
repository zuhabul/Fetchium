import Navbar from "@/components/Navbar";
import Pricing from "@/components/Pricing";
import Footer from "@/components/Footer";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Pricing — Fetchium",
  description: "Simple, transparent pricing. Start free, upgrade when ready. Self-host for unlimited usage.",
};

export default function PricingPage() {
  return (
    <main className="min-h-screen bg-surface">
      <Navbar />
      <div className="pt-24">
        <Pricing />
      </div>
      <Footer />
    </main>
  );
}
