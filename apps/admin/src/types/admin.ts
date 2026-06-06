export interface AdminSession {
  sessionToken: string
  id: string
  email: string
  name: string
  role: AdminRole
  exp: number
}

export type AdminRole = 'owner' | 'ops' | 'support' | 'finance' | 'growth' | 'readonly'

export const ROLE_LABELS: Record<AdminRole, string> = {
  owner: 'Owner',
  ops: 'Ops',
  support: 'Support',
  finance: 'Finance',
  growth: 'Growth',
  readonly: 'Read-only',
}

export const ROLE_COLORS: Record<AdminRole, string> = {
  owner: 'bg-red-500/20 text-red-400 border-red-500/30',
  ops: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
  support: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  finance: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
  growth: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
  readonly: 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30',
}
