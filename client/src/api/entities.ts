import { apiFetch } from "./client"
import type { Entity } from "./types"

/** GET /cases/:id/entities — all resolved entities + link tiers. */
export function getCaseEntities(caseId: string): Promise<Entity[]> {
  return apiFetch<Entity[]>(`/cases/${encodeURIComponent(caseId)}/entities`)
}
