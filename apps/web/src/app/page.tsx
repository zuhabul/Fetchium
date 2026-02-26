import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import TrustBar from "@/components/TrustBar";
import Features from "@/components/Features";
import HowItWorks from "@/components/HowItWorks";
import CodeDemo from "@/components/CodeDemo";
import Comparison from "@/components/Comparison";
import Pricing from "@/components/Pricing";
import Footer from "@/components/Footer";

export default function Home() {
  return (
    <main className="min-h-screen bg-surface">
      <Navbar />
      <Hero />
      <TrustBar />
      <Features />
      <HowItWorks />
      <CodeDemo />
      <Comparison />
      <Pricing />
      <Footer />
    </main>
  );
}
