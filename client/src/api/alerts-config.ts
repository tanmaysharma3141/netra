import { apiFetch } from "./client"

export interface AlertThresholds {
  imei_min_subscribers: number
  imei_min_evidence: number
  hawala_window_hours: number
  hawala_min_txns: number
  hawala_min_total: number
  hawala_max_total: number
  rapid_window_minutes: number
  rapid_min_txns: number
  rapid_min_flow: number
  silence_min_parties: number
  bot_min_posts: number
  bot_max_interval_secs: number
  round_trip_window_hours: number
  tower_jump_max_minutes: number
  tower_jump_min_km: number
}

export function getAlertThresholds(): Promise<AlertThresholds> {
  return apiFetch<AlertThresholds>("/settings/alerts")
}

export function updateAlertThresholds(
  thresholds: Partial<AlertThresholds>
): Promise<AlertThresholds> {
  return apiFetch<AlertThresholds>("/settings/alerts", {
    method: "PATCH",
    body: thresholds,
  })
}
