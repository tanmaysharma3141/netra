import { apiFetch } from "./client"
import type { AuditEntry } from "./types"

/** GET /cases/:id/audit — audit log entries for a case (max 200, newest first). */
export function listAuditEntries(caseId: string): Promise<AuditEntry[]> {
  return apiFetch<AuditEntry[]>(`/cases/${encodeURIComponent(caseId)}/audit`)
}
