import type { Metadata } from "next";

export const metadata: Metadata = { title: "Settings — HyperSearchX Dashboard" };

export default function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-sm text-white/40 mt-1">Manage your account preferences.</p>
      </div>

      <div className="rounded-xl border border-white/5 bg-surface-1 divide-y divide-white/5">
        {[
          { label: "Display name", value: "Developer", type: "text" },
          { label: "Email", value: "dev@example.com", type: "email" },
          { label: "Password", value: "••••••••••••", type: "password" },
        ].map(f => (
          <div key={f.label} className="flex items-center justify-between px-5 py-4 gap-4">
            <div className="w-32 shrink-0">
              <label className="text-sm font-medium text-white/60">{f.label}</label>
            </div>
            <input
              type={f.type}
              defaultValue={f.value}
              className="flex-1 rounded-lg border border-white/5 bg-white/5 px-3 py-1.5 text-sm text-white outline-none focus:border-brand-500/50"
            />
            <button className="shrink-0 rounded-lg border border-white/10 px-3 py-1.5 text-xs text-white/50 hover:text-white transition-colors">
              Save
            </button>
          </div>
        ))}
      </div>

      <div className="rounded-xl border border-red-500/10 bg-red-500/5 p-5">
        <h2 className="font-medium text-red-400 mb-1">Danger zone</h2>
        <p className="text-sm text-white/40 mb-4">These actions are permanent and cannot be undone.</p>
        <button className="rounded-lg border border-red-500/20 px-4 py-2 text-sm text-red-400 hover:bg-red-500/10 transition-colors">
          Delete account
        </button>
      </div>
    </div>
  );
}
