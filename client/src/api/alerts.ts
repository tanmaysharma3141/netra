import { apiFetch } from "./client"
import type { Alert, AlertStatus } from "./types"

/** GET /alerts — cross-case alert list. */
export function listAlerts(
  params: {
    case_id?: string
    severity?: "critical" | "high" | "medium" | "low"
    status?: AlertStatus
    limit?: number
  } = {},
): Promise<Alert[]> {
  const qs = new URLSearchParams()
  if (params.case_id) qs.set("case_id", params.case_id)
  if (params.severity) qs.set("severity", params.severity)
  if (params.status) qs.set("status", params.status)
  if (params.limit) qs.set("limit", String(params.limit))
  const query = qs.toString()
  return apiFetch<Alert[]>(`/alerts${query ? `?${query}` : ""}`)
}

/** GET /alerts/:id — single alert detail. */
export function getAlert(id: string): Promise<Alert> {
  return apiFetch<Alert>(`/alerts/${encodeURIComponent(id)}`)
}

/** PATCH /alerts/:id/status — triage an alert. */
export function triageAlert(
  id: string,
  payload: { status: AlertStatus; note?: string },
): Promise<Alert> {
  return apiFetch<Alert>(`/alerts/${encodeURIComponent(id)}/status`, {
    method: "PATCH",
    body: JSON.stringify(payload),
  })
}

/** POST /cases/:id/analyze — manual re-run. */
export function analyzeCase(caseId: string): Promise<{ alerts_raised: number; by_rule: Record<string, number> }> {
  return apiFetch(`/cases/${encodeURIComponent(caseId)}/analyze`, {
    method: "POST",
  })
}
