import { apiFetch } from "./client"
import type { GeoPoint } from "./types"

export interface Trail {
  entity_id: string
  points: GeoPoint[]
}

export interface TrailsResponse {
  trails: Trail[]
}

/** GET /cases/:id/movements — suspect movement trails from tower pings + txns. */
export function getMovements(
  caseId: string,
  opts: { entityId?: string; from?: string; to?: string } = {},
): Promise<TrailsResponse> {
  const params = new URLSearchParams()
  if (opts.entityId) params.set("entity_id", opts.entityId)
  if (opts.from) params.set("from", opts.from)
  if (opts.to) params.set("to", opts.to)
  const qs = params.toString()
  return apiFetch<TrailsResponse>(`/cases/${encodeURIComponent(caseId)}/movements${qs ? `?${qs}` : ""}`)
}
