import { apiFetch } from "./client"
import type { User } from "./types"

export interface LoginResponse {
  token: string
  expires_at: string
  user: User
}

/** POST /auth/login — 401 bad credentials, 423 locked out after 5 failures (contract). */
export function login(username: string, password: string): Promise<LoginResponse> {
  return apiFetch<LoginResponse>("/auth/login", {
    method: "POST",
    body: { username, password },
    authenticate: false,
  })
}

/** POST /auth/logout — 204, server invalidates the token. */
export function logout(tokenPresent: boolean): Promise<void> {
  return apiFetch<void>("/auth/logout", { method: "POST", authenticate: tokenPresent })
}
