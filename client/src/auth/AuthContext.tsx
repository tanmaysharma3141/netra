import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react"
import type { User } from "@/api/types"
import * as authApi from "@/api/auth"
import { canRole, type Permission } from "@/lib/rbac"
import { clearSession, getToken, getUser, setSession } from "@/lib/secureStore"

type AuthStatus = "restoring" | "authenticated" | "unauthenticated"

interface AuthContextValue {
  user: User | null
  status: AuthStatus
  signIn: (username: string, password: string) => Promise<void>
  signOut: () => Promise<void>
  can: (permission: Permission) => boolean
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [status, setStatus] = useState<AuthStatus>("restoring")

  useEffect(() => {
    let cancelled = false
    void (async () => {
      const [token, storedUser] = await Promise.all([getToken(), getUser()])
      if (cancelled) return
      if (token && storedUser) {
        setUser(storedUser)
        setStatus("authenticated")
      } else {
        await clearSession()
        setStatus("unauthenticated")
      }
    })()
    return () => {
      cancelled = true
    }
  }, [])

  const signIn = useCallback(async (username: string, password: string) => {
    const res = await authApi.login(username, password)
    await setSession(res.token, res.user)
    setUser(res.user)
    setStatus("authenticated")
  }, [])

  const signOut = useCallback(async () => {
    try {
      await authApi.logout(true)
    } catch {
      // Server-side invalidation is best-effort; local session is cleared regardless.
    }
    await clearSession()
    setUser(null)
    setStatus("unauthenticated")
  }, [])

  const can = useCallback(
    (permission: Permission) => (user ? canRole(user.role, permission) : false),
    [user],
  )

  const value = useMemo<AuthContextValue>(
    () => ({ user, status, signIn, signOut, can }),
    [user, status, signIn, signOut, can],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error("useAuth must be used within AuthProvider")
  return ctx
}
