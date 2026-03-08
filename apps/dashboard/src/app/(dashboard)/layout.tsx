import Sidebar from "@/components/Sidebar";
import DashHeader from "@/components/DashHeader";
import DashboardAccessGate from "@/components/DashboardAccessGate";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen overflow-hidden bg-surface">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <DashHeader />
        <main className="flex-1 overflow-y-auto p-6">
          <DashboardAccessGate>{children}</DashboardAccessGate>
        </main>
      </div>
    </div>
  );
}
