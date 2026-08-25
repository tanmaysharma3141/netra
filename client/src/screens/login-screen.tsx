import { useState, type FormEvent } from "react"
import { Eye, LockKeyhole, OctagonX, ServerOff, ShieldX } from "lucide-react"
import { useAuth } from "@/auth/AuthContext"
import { ApiClientError } from "@/api/client"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

type LoginError =
  | { kind: "invalid"; message: string }
  | { kind: "locked"; message: string }
  | { kind: "network"; message: string }
  | { kind: "unknown"; message: string }

const ERROR_TITLES: Record<LoginError["kind"], string> = {
  invalid: "Invalid credentials",
  locked: "Account locked",
  network: "Server unreachable",
  unknown: "Login failed",
}

const ERROR_ICONS: Record<LoginError["kind"], typeof ShieldX> = {
  invalid: ShieldX,
  locked: LockKeyhole,
  network: ServerOff,
  unknown: OctagonX,
}

export function LoginScreen() {
  const { signIn } = useAuth()
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<LoginError | null>(null)

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (pending) return
    setPending(true)
    setError(null)
    try {
      await signIn(username.trim(), password)
    } catch (err) {
      setError(toLoginError(err))
    } finally {
      setPending(false)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <div className="w-full max-w-sm">
        <div className="mb-6 flex items-center justify-center gap-2">
          <Eye className="size-7 text-primary" aria-hidden />
          <h1 className="font-mono text-xl font-semibold tracking-[0.3em]">NETRA</h1>
        </div>
        <p className="mb-8 text-center text-xs tracking-[0.18em] text-muted-foreground uppercase">
          Forensic Intelligence Console
        </p>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Officer Sign-in</CardTitle>
            <CardDescription>Use your department credentials to access the console.</CardDescription>
          </CardHeader>
          <CardContent>
            {error ? (
              <Alert variant="destructive" className="mb-4">
                {(() => {
                  const Icon = ERROR_ICONS[error.kind]
                  return <Icon className="size-4" aria-hidden />
                })()}
                <AlertTitle>{ERROR_TITLES[error.kind]}</AlertTitle>
                <AlertDescription>{error.message}</AlertDescription>
              </Alert>
            ) : null}

            <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4" noValidate>
              <div className="space-y-1.5">
                <Label htmlFor="username">Officer ID</Label>
                <Input
                  id="username"
                  name="username"
                  autoComplete="username"
                  autoFocus
                  required
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="e.g. inv001"
                  className="font-mono"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  name="password"
                  type="password"
                  autoComplete="current-password"
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                />
              </div>
              <Button type="submit" className="w-full" disabled={pending || !username || !password}>
                {pending ? "Authenticating…" : "Sign in"}
              </Button>
            </form>
          </CardContent>
        </Card>

        <p className="mt-6 text-center font-mono text-[10px] tracking-wider text-muted-foreground">
          AIR-GAPPED DEPLOYMENT · NO DATA LEAVES PREMISES
        </p>
      </div>
    </div>
  )
}

function toLoginError(err: unknown): LoginError {
  if (err instanceof ApiClientError) {
    switch (err.status) {
      case 401:
        return {
          kind: "invalid",
          message: "The officer ID or password is incorrect. Check your credentials and try again.",
        }
      case 423: {
        const match = /retry in (\d+)s/.exec(err.message)
        return {
          kind: "locked",
          message: match
            ? `Too many failed attempts. This account is locked — try again in ${match[1]}s or contact your system administrator.`
            : "Too many failed attempts. This account is locked — contact your system administrator.",
        }
      }
      case 0:
        return {
          kind: "network",
          message: "Cannot reach the NETRA server on the LAN. Verify the server address in Settings.",
        }
      default:
        return { kind: "unknown", message: err.message }
    }
  }
  return { kind: "unknown", message: "An unexpected error occurred." }
}
