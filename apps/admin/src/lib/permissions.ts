import type { AdminRole } from '@/types/admin'
export type { AdminRole }

export type Permission =
  | 'keys.read' | 'keys.revoke' | 'keys.create'
  | 'orgs.read' | 'orgs.suspend' | 'orgs.plan_change' | 'orgs.quota_override' | 'orgs.delete'
  | 'users.read' | 'users.suspend'
  | 'billing.read' | 'billing.refund' | 'billing.credit'
  | 'support.read' | 'support.reply' | 'support.close'
  | 'crm.read' | 'crm.write'
  | 'incidents.read' | 'incidents.manage'
  | 'audit.read'
  | 'flags.read' | 'flags.write'
  | 'proxy.read' | 'proxy.reset'
  | 'campaigns.read'
  | 'admin.staff_manage'

const MATRIX: Record<AdminRole, Permission[]> = {
  owner: [
    'keys.read','keys.revoke','keys.create',
    'orgs.read','orgs.suspend','orgs.plan_change','orgs.quota_override','orgs.delete',
    'users.read','users.suspend',
    'billing.read','billing.refund','billing.credit',
    'support.read','support.reply','support.close',
    'crm.read','crm.write',
    'incidents.read','incidents.manage',
    'audit.read',
    'flags.read','flags.write',
    'proxy.read','proxy.reset',
    'campaigns.read',
    'admin.staff_manage',
  ],
  ops: [
    'keys.read','keys.revoke','keys.create',
    'orgs.read','orgs.suspend','orgs.quota_override',
    'users.read','users.suspend',
    'support.read','support.reply','support.close',
    'incidents.read','incidents.manage',
    'audit.read',
    'flags.read','flags.write',
    'proxy.read','proxy.reset',
  ],
  support: [
    'keys.read','keys.create',
    'orgs.read','users.read',
    'support.read','support.reply','support.close',
    'crm.read','crm.write',
    'incidents.read','audit.read',
  ],
  finance: [
    'keys.read','orgs.read','orgs.plan_change','users.read',
    'billing.read','billing.refund','billing.credit',
    'crm.read','campaigns.read',
  ],
  growth: ['orgs.read','users.read','crm.read','crm.write','campaigns.read'],
  readonly: [
    'keys.read','orgs.read','users.read',
    'support.read','incidents.read','audit.read',
    'flags.read','crm.read','campaigns.read',
  ],
}

export function can(role: AdminRole, perm: Permission): boolean {
  return MATRIX[role]?.includes(perm) ?? false
}
