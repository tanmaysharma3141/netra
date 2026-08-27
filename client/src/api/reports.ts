import { apiFetch } from "./client"
import type { Report } from "./types"

/** POST /cases/:id/reports — generate a new report (async). Returns { report_id }. */
export function generateReport(caseId: string): Promise<{ report_id: string }> {
  return apiFetch(`/cases/${encodeURIComponent(caseId)}/reports`, {
    method: "POST",
  })
}

/** GET /cases/:id/reports — list reports for a case. */
export function listReports(caseId: string): Promise<Report[]> {
  return apiFetch<Report[]>(`/cases/${encodeURIComponent(caseId)}/reports`)
}

/** GET /reports/:id — single report content (JSON with summary_md). */
export function getReport(reportId: string): Promise<Report> {
  return apiFetch<Report>(`/reports/${encodeURIComponent(reportId)}`)
}

/** GET /reports/:id/export — PDF download link (returns the URL for anchor href). */
export function getReportExportUrl(reportId: string): string {
  // This is used as an href — the actual download goes through the fetch wrapper
  // with auth headers, so we provide a direct apiFetch-based download instead.
  return `/reports/${encodeURIComponent(reportId)}/export`
}

/** PATCH /reports/:id/approve — Supervisor/Admin only. */
export function approveReport(reportId: string): Promise<Report> {
  return apiFetch<Report>(`/reports/${encodeURIComponent(reportId)}/approve`, {
    method: "PATCH",
  })
}
