import { API_BASE_URL } from "@/lib/env"
import { getToken } from "@/lib/secureStore"
import { ApiClientError } from "./client"

export interface PreviewResult {
  headers: string[]
  sample_rows: string[][]
  domain: string
  domain_score: number
  estimated_rows: number
  operator: string | null
}

export async function previewFile(
  caseId: string,
  file: File
): Promise<PreviewResult> {
  const form = new FormData()
  form.append("files", file)

  const token = await getToken()
  let res: Response
  try {
    res = await fetch(
      `${API_BASE_URL}/cases/${encodeURIComponent(caseId)}/ingest/preview`,
      {
        method: "POST",
        headers: token ? { Authorization: `Bearer ${token}` } : undefined,
        body: form,
      }
    )
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
      body?.error?.code ?? "preview_failed",
      body?.error?.message ?? `Preview failed (${res.status})`
    )
  }
  return (await res.json()) as PreviewResult
}
