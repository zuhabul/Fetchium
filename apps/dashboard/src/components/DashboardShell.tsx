"use client";

import { useEffect, useState } from "react";
import Sidebar from "@/components/Sidebar";
import DashHeader from "@/components/DashHeader";
import { DASHBOARD_SHELL_MAIN } from "@/lib/dashboard-layout";

export default function DashboardShell({ children }: { children: React.ReactNode }) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  useEffect(() => {
    function handleSidebarToggle() {
      setSidebarOpen((value) => !value);
    }

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setSidebarOpen(false);
      }
    }

    window.addEventListener("fetchium:toggle-dashboard-sidebar", handleSidebarToggle);
    window.addEventListener("keydown", handleKeydown);

    return () => {
      window.removeEventListener("fetchium:toggle-dashboard-sidebar", handleSidebarToggle);
      window.removeEventListener("keydown", handleKeydown);
    };
  }, []);

  useEffect(() => {
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = sidebarOpen ? "hidden" : previousOverflow;

    return () => {
      document.body.style.overflow = previousOverflow;
    };
  }, [sidebarOpen]);

  return (
    <div className="flex h-screen overflow-hidden bg-[var(--app-bg)] text-[var(--text-primary)]">
      <div
        aria-hidden={!sidebarOpen}
        className={`fixed inset-0 z-30 bg-black/50 backdrop-blur-sm transition-opacity lg:hidden ${
          sidebarOpen ? "opacity-100" : "pointer-events-none opacity-0"
        }`}
        onClick={() => setSidebarOpen(false)}
      />

      <Sidebar mobileOpen={sidebarOpen} onClose={() => setSidebarOpen(false)} />

      <div className="flex flex-1 flex-col overflow-hidden">
        <DashHeader />
        <main className={DASHBOARD_SHELL_MAIN}>{children}</main>
      </div>
    </div>
  );
}
