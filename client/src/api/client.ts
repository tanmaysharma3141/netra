import { API_BASE_URL } from "@/lib/env"
import { clearSession, getToken } from "@/lib/secureStore"
import type { ApiError } from "./types"

export class ApiClientError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
  ) {
    super(message)
    this.name = "ApiClientError"
  }

  get isNetworkError(): boolean {
    return this.status === 0
  }
}

type Method = "GET" | "POST" | "PATCH" | "PUT" | "DELETE"

interface RequestOptions {
  method?: Method
  body?: unknown
  /** Skip Authorization header + 401 redirect (login itself, health checks). */
  authenticate?: boolean
}

async function toClientError(res: Response): Promise<ApiClientError> {
  let parsed: unknown = null
  try {
    parsed = await res.json()
  } catch {
    // Some error responses carry no body (e.g. stub 401) — fall through.
  }
  const body = parsed as ApiError | null
  return new ApiClientError(
    res.status,
    body?.error?.code ?? "unknown_error",
    body?.error?.message ?? `Request failed (${res.status} ${res.statusText})`,
  )
}

async function handleUnauthorized(): Promise<void> {
  await clearSession()
  // Hash routing keeps this a soft in-app redirect with no full reload.
  window.location.hash = "#/login"
}

export async function apiFetch<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { method = "GET", body, authenticate = true } = options

  const headers = new Headers({ Accept: "application/json" })
  if (body !== undefined) headers.set("Content-Type", "application/json")
  if (authenticate) {
    const token = await getToken()
    if (token) headers.set("Authorization", `Bearer ${token}`)
  }

  let res: Response
  try {
    res = await fetch(`${API_BASE_URL}${path}`, {
      method,
      headers,
      ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    })
  } catch {
    throw new ApiClientError(0, "network_error", "NETRA server unreachable.")
  }

  if (res.status === 204) {
    return undefined as T
  }

  if (!res.ok) {
    const err = await toClientError(res)
    if (res.status === 401 && authenticate) await handleUnauthorized()
    throw err
  }

  return (await res.json()) as T
}
