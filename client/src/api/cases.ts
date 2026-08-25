import { apiFetch } from "./client"
import type { Case } from "./types"

/** GET /cases — role-scoped visibility enforced server-side. */
export function listCases(): Promise<Case[]> {
  return apiFetch<Case[]>("/cases")
}

export interface CreateCaseInput {
  title: string
  classification?: string
  tags?: string[]
}

/** POST /cases — Investigator/Admin only (RBAC matrix). */
export function createCase(input: CreateCaseInput): Promise<Case> {
  return apiFetch<Case>("/cases", { method: "POST", body: input })
}

/** GET /cases/:id — detail + computed stats. */
export function getCase(id: string): Promise<Case> {
  return apiFetch<Case>(`/cases/${encodeURIComponent(id)}`)
}
