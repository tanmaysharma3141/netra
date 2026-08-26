import { apiFetch } from "./client"

/* ── Webhooks ── */

export interface WebhookConfig {
  discord_url: string | null
  telegram_bot_token: string | null
  telegram_chat_id: string | null
}

/** GET /settings/webhooks — current webhook config (Admin only). */
export function getWebhooks(): Promise<WebhookConfig> {
  return apiFetch<WebhookConfig>("/settings/webhooks")
}

/** PATCH /settings/webhooks — update webhook config (Admin only). */
export function updateWebhooks(config: Partial<WebhookConfig>): Promise<WebhookConfig> {
  return apiFetch<WebhookConfig>("/settings/webhooks", {
    method: "PATCH",
    body: config,
  })
}

/* ── Models ── */

export interface ModelVersion {
  version: string
  status: "active" | "inactive" | "training"
  created_at: string
  feedback_count?: number
}

/** GET /models — list model versions (Admin only). */
export function listModels(): Promise<ModelVersion[]> {
  return apiFetch<ModelVersion[]>("/models")
}

/** POST /models/promote — promote a model version (Admin only). */
export function promoteModel(version: string): Promise<{ promoted: string }> {
  return apiFetch<{ promoted: string }>("/models/promote", {
    method: "POST",
    body: { version },
  })
}

/* ── Training ── */

export interface TrainingQueue {
  queue_size: number
  min_batch: number
  last_run: string | null
  last_loss: number | null
}

/** GET /training/queue — training queue info (Admin only). */
export function getTrainingQueue(): Promise<TrainingQueue> {
  return apiFetch<TrainingQueue>("/training/queue")
}

/** POST /training/trigger — manual retraining run (Admin only). */
export function triggerTraining(): Promise<{ started: boolean }> {
  return apiFetch<{ started: boolean }>("/training/trigger", {
    method: "POST",
  })
}
