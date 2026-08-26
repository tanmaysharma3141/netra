import { apiFetch } from "./client"
import type { Entity, EntityType, GraphEdge, GraphNode, LinkTier } from "./types"

export interface GraphResponse {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

/** GET /cases/:id/graph — adjacency list for D3. hops default 2 per contract. */
export function getGraph(caseId: string, opts: { entityId?: string; hops?: number } = {}): Promise<GraphResponse> {
  const params = new URLSearchParams()
  if (opts.entityId) params.set("entity_id", opts.entityId)
  if (opts.hops !== undefined) params.set("hops", String(opts.hops))
  const qs = params.toString()
  return apiFetch<GraphResponse>(`/cases/${encodeURIComponent(caseId)}/graph${qs ? `?${qs}` : ""}`)
}

export interface EntityProfile {
  entity: Entity
  stats: { events: number; first_seen: string | null; last_seen: string | null }
  connections: {
    link_type: string
    tier: LinkTier
    confidence: number
    evidence_count: number
    other_entity_id: string
    other_identifier: string
    other_type: EntityType
  }[]
}

/** GET /entities/:id/profile — full cross-domain entity profile. */
export function getEntityProfile(entityId: string): Promise<EntityProfile> {
  return apiFetch<EntityProfile>(`/entities/${encodeURIComponent(entityId)}/profile`)
}

/** POST /cases/:id/resolve — force re-resolution of the correlation engine. */
export function resolveCase(caseId: string): Promise<{ entities: number; edges: number }> {
  return apiFetch<{ entities: number; edges: number }>(`/cases/${encodeURIComponent(caseId)}/resolve`, {
    method: "POST",
    body: {},
  })
}
