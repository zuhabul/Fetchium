import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Fetchium Status — API & Service Health",
  description: "Health endpoints and service surfaces for Fetchium.",
};

const services = [
  { name: "Health endpoint", endpoint: "/v1/health", status: "available" },
  { name: "Search API", endpoint: "/v1/search", status: "available" },
  { name: "Extract API", endpoint: "/v1/scrape", status: "available" },
  { name: "Research API", endpoint: "/v1/research", status: "available" },
  { name: "MCP Server", endpoint: "stdio / http transport", status: "available" },
  { name: "Documentation", endpoint: "docs.fetchium.com", status: "available" },
];

function StatusBadge({ status }: { status: string }) {
  const styles: Record<string, string> = {
    available: "bg-indigo-500/12 text-indigo-300 border-indigo-500/20",
  };
  return (
    <span className={`rounded-full border px-2.5 py-0.5 text-[11px] font-semibold ${styles[status] || styles.available}`}>
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}

export default function StatusPage() {
  return (
    <div className="min-h-screen bg-[#06070d] text-slate-100">
      <Navbar />
      <main className="pt-24 pb-16 px-4">
        <div className="mx-auto max-w-3xl">
          <nav className="mb-6 flex items-center gap-2 text-xs text-slate-600">
            <Link href="/" className="hover:text-slate-400">Home</Link>
            <span>/</span>
            <span className="text-slate-400">Status</span>
          </nav>

          <div className="mb-10 flex items-center gap-4">
            <div>
              <h1 className="text-3xl font-bold">System Status</h1>
              <p className="text-slate-500 mt-1 text-sm">Reference page for health endpoints and public service surfaces.</p>
            </div>
            <div className="ml-auto flex items-center gap-2 rounded-full border border-indigo-500/20 bg-indigo-500/8 px-3 py-1.5 text-sm font-medium text-indigo-300">
              <span className="h-2 w-2 rounded-full bg-indigo-400 animate-pulse" />
              Service Reference
            </div>
          </div>

          {/* Services */}
          <div className="mb-10 space-y-2">
            {services.map((s) => (
              <div key={s.name} className="flex items-center gap-4 rounded-xl border border-white/6 bg-white/[0.02] px-5 py-4">
                <div className="flex-1">
                  <div className="text-sm font-semibold text-slate-200">{s.name}</div>
                  <div className="text-[11px] text-slate-600 font-mono mt-0.5">{s.endpoint}</div>
                </div>
                <StatusBadge status={s.status} />
              </div>
            ))}
          </div>

          {/* Incident history */}
          <div className="rounded-2xl border border-white/6 bg-white/[0.02] p-6">
            <h2 className="text-base font-semibold mb-4">Recent Incidents</h2>
            <div className="text-center py-6 text-slate-600">
              <p className="text-sm">This page does not publish incident history or uptime guarantees.</p>
            </div>
          </div>

          <p className="mt-6 text-center text-[12px] text-slate-600">
            Subscribe to status updates:{" "}
            <a href="mailto:status@fetchium.com" className="text-indigo-400 hover:text-indigo-300">status@fetchium.com</a>
          </p>
        </div>
      </main>
      <Footer />
    </div>
  );
}
