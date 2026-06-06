import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import DashboardShell from "@/components/DashboardShell";

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
    <DashboardShell>{children}</DashboardShell>
  );
}
