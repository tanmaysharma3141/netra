import { API_BASE_URL } from "@/lib/env"
import { getToken } from "@/lib/secureStore"
import type { IngestJob } from "./types"
import { ApiClientError } from "./client"

/** POST /cases/:id/ingest — multipart upload, returns async job (202). */
export async function uploadFile(caseId: string, file: File): Promise<IngestJobRef> {
  const form = new FormData()
  form.append("files", file)

  const token = await getToken()
  let res: Response
  try {
    res = await fetch(`${API_BASE_URL}/cases/${encodeURIComponent(caseId)}/ingest`, {
      method: "POST",
      headers: token ? { Authorization: `Bearer ${token}` } : undefined,
      body: form,
    })
  } catch {
    throw new ApiClientError(0, "network_error", "NETRA server unreachable.")
  }
  if (!res.ok) {
    let body: { error?: { code?: string; message?: string } } | null = null
    try {
      body = await res.json()
    } catch {
      // empty error body
    }
    throw new ApiClientError(
      res.status,
      body?.error?.code ?? "unknown_error",
      body?.error?.message ?? `Upload failed (${res.status})`,
    )
  }
  return (await res.json()) as IngestJobRef
}

export interface IngestJobRef {
  job_id: string
}

/** GET /ingest/jobs/:id — status, records parsed, row-level errors. */
export function getIngestJob(jobId: string): Promise<IngestJob> {
  return apiJson<IngestJob>(`/ingest/jobs/${encodeURIComponent(jobId)}`)
}

async function apiJson<T>(path: string): Promise<T> {
  const token = await getToken()
  const res = await fetch(`${API_BASE_URL}${path}`, {
    headers: token ? { Authorization: `Bearer ${token}` } : undefined,
  })
  if (!res.ok) throw new ApiClientError(res.status, "request_failed", `Request failed (${res.status})`)
  return (await res.json()) as T
}
