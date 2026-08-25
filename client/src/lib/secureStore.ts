import { load, type Store } from "@tauri-apps/plugin-store"
import type { User } from "@/api/types"

/**
 * Session persistence via the Tauri secure store plugin (tauri-plugin-store).
 * In a plain browser dev session (no Tauri runtime) we fall back to localStorage
 * so the login flow is testable without launching the desktop shell.
 * The packaged app always uses the Tauri store — never raw localStorage in prod paths.
 */

const TOKEN_KEY = "netra.auth.token"
const USER_KEY = "netra.auth.user"

let storePromise: Promise<Store> | null = null

function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
}

async function tauriStore(): Promise<Store> {
  storePromise ??= load("netra-session.dat", { autoSave: true })
  return storePromise
}

export async function getToken(): Promise<string | null> {
  if (inTauri()) {
    const value = await (await tauriStore()).get<string>(TOKEN_KEY)
    return value ?? null
  }
  return localStorage.getItem(TOKEN_KEY)
}

export async function getUser(): Promise<User | null> {
  let raw: string | null
  if (inTauri()) {
    const value = await (await tauriStore()).get<string>(USER_KEY)
    raw = value ?? null
  } else {
    raw = localStorage.getItem(USER_KEY)
  }
  if (!raw) return null
  return parseStoredUser(raw)
}

function parseStoredUser(raw: string): User | null {
  try {
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== "object" || parsed === null) return null
    const record = parsed as { id?: unknown; username?: unknown; role?: unknown; active?: unknown }
    if (
      typeof record.id === "string" &&
      typeof record.username === "string" &&
      typeof record.role === "string" &&
      typeof record.active === "boolean"
    ) {
      return { id: record.id, username: record.username, role: record.role, active: record.active } as User
    }
    return null
  } catch {
    return null
  }
}

export async function setSession(token: string, user: User): Promise<void> {
  if (inTauri()) {
    const store = await tauriStore()
    await store.set(TOKEN_KEY, token)
    await store.set(USER_KEY, JSON.stringify(user))
    return
  }
  localStorage.setItem(TOKEN_KEY, token)
  localStorage.setItem(USER_KEY, JSON.stringify(user))
}

export async function clearSession(): Promise<void> {
  if (inTauri()) {
    const store = await tauriStore()
    await store.delete(TOKEN_KEY)
    await store.delete(USER_KEY)
    return
  }
  localStorage.removeItem(TOKEN_KEY)
  localStorage.removeItem(USER_KEY)
}
