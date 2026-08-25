import { apiFetch } from "./client"
import type { Event, EventType, SourceType } from "./types"

export interface EventQuery {
  source_type?: SourceType
  event_type?: EventType
  entity_id?: string
  from?: string
  to?: string
  limit?: number
  offset?: number
}

/**
 * GET /cases/:id/events — unified timeline, timestamp desc,
 * paginated via limit/offset per docs/API.md.
 */
export function listEvents(caseId: string, query: EventQuery = {}): Promise<Event[]> {
  const params = new URLSearchParams()
  if (query.source_type) params.set("source_type", query.source_type)
  if (query.event_type) params.set("event_type", query.event_type)
  if (query.entity_id) params.set("entity_id", query.entity_id)
  if (query.from) params.set("from", query.from)
  if (query.to) params.set("to", query.to)
  if (query.limit !== undefined) params.set("limit", String(query.limit))
  if (query.offset !== undefined) params.set("offset", String(query.offset))
  const qs = params.toString()
  return apiFetch<Event[]>(`/cases/${encodeURIComponent(caseId)}/events${qs ? `?${qs}` : ""}`)
}
