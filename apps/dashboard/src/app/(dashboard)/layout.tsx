import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import Sidebar from "@/components/Sidebar";
import DashHeader from "@/components/DashHeader";

export default async function DashboardLayout({ children }: { children: React.ReactNode }) {
  const cookieStore = await cookies();
  const hasSession =
    cookieStore.has("__Secure-authjs.session-token") ||
    cookieStore.has("authjs.session-token") ||
    cookieStore.has("__Secure-next-auth.session-token") ||
    cookieStore.has("next-auth.session-token");

  if (!hasSession) {
    redirect("/login");
  }

  return (
    <div className="flex h-screen overflow-hidden bg-surface">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <DashHeader />
        <main className="flex-1 overflow-y-auto p-6">{children}</main>
      </div>
    </div>
  );
}
