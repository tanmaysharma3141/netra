import type { Role } from "@/api/types"

/**
 * RBAC matrix from docs/NETRA_PRD.md §5.2.
 * The UI only hides what a role cannot do — the server remains the authority.
 */
export type Permission =
  | "case.create"
  | "data.upload"
  | "cases.viewAll"
  | "analysis.run"
  | "report.generate"
  | "report.approve"
  | "users.manage"
  | "training.trigger"
  | "webhooks.configure"
  | "audit.view"

const ADMIN: readonly Permission[] = [
  "case.create",
  "data.upload",
  "cases.viewAll",
  "analysis.run",
  "report.generate",
  "report.approve",
  "users.manage",
  "training.trigger",
  "webhooks.configure",
  "audit.view",
]

const SUPERVISOR: readonly Permission[] = ["cases.viewAll", "report.approve", "audit.view"]

const INVESTIGATOR: readonly Permission[] = [
  "case.create",
  "data.upload",
  "analysis.run",
  "report.generate",
]

const ANALYST: readonly Permission[] = ["analysis.run"]

export const ROLE_PERMISSIONS: Record<Role, readonly Permission[]> = {
  admin: ADMIN,
  supervisor: SUPERVISOR,
  investigator: INVESTIGATOR,
  analyst: ANALYST,
}

export function canRole(role: Role, permission: Permission): boolean {
  return ROLE_PERMISSIONS[role].includes(permission)
}
