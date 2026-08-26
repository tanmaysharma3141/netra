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
  return apiFetch<Alert[]>("/alerts", { params })
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
