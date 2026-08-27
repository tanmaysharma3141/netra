import { useState, type FormEvent } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import {
  AlertTriangle,
  Bot,
  Check,
  RefreshCw,
  Settings,
  Train,
  Trash2,
  UserPlus,
  Webhook,
} from "lucide-react"
import { listUsers, createUser, deleteUser, type CreateUserInput } from "@/api/users"
import {
  getWebhooks,
  updateWebhooks,
  listModels,
  promoteModel,
  getTrainingQueue,
  triggerTraining,
  type WebhookConfig,
} from "@/api/settings"
import {
  getAlertThresholds,
  updateAlertThresholds,
  type AlertThresholds,
} from "@/api/alerts-config"
import { ApiClientError } from "@/api/client"
import type { Role, User } from "@/api/types"
import { useAuth } from "@/auth/AuthContext"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Separator } from "@/components/ui/separator"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"

const ROLES: readonly Role[] = ["admin", "supervisor", "investigator", "analyst"]

type SettingsTab = "users" | "webhooks" | "models" | "training" | "alerts"

const TABS: readonly { id: SettingsTab; label: string; icon: typeof Settings }[] = [
  { id: "users", label: "Users", icon: UserPlus },
  { id: "webhooks", label: "Webhooks", icon: Webhook },
  { id: "alerts", label: "Alert Rules", icon: AlertTriangle },
  { id: "models", label: "AI Models", icon: Bot },
  { id: "training", label: "Training", icon: Train },
]

export function SettingsScreen() {
  const { can } = useAuth()
  const [activeTab, setActiveTab] = useState<SettingsTab>("users")

  if (!can("users.manage")) {
    return (
      <div className="mx-auto max-w-6xl p-6">
        <header className="mb-6">
          <h1 className="text-lg font-semibold">Settings</h1>
        </header>
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-14 text-center">
            <Settings className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">Access restricted</p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Settings are only accessible to administrators.
            </p>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6">
        <h1 className="text-lg font-semibold">Settings</h1>
        <p className="text-sm text-muted-foreground">
          System administration — users, webhooks, models, and training.
        </p>
      </header>

      <div className="flex gap-6">
        {/* Vertical tab sidebar */}
        <nav className="w-48 shrink-0 space-y-1" aria-label="Settings sections">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex w-full items-center gap-2.5 rounded-sm px-3 py-2 text-sm transition-colors ${
                activeTab === tab.id
                  ? "bg-sidebar-accent font-medium text-sidebar-primary"
                  : "text-muted-foreground hover:bg-sidebar-accent/60 hover:text-foreground"
              }`}
            >
              <tab.icon className="size-4 shrink-0" aria-hidden />
              {tab.label}
            </button>
          ))}
        </nav>

        <Separator orientation="vertical" className="h-auto self-stretch" />

        {/* Tab content */}
        <div className="min-w-0 flex-1">
          {activeTab === "users" && <UserManagementSection />}
          {activeTab === "webhooks" && <WebhookSection />}
          {activeTab === "alerts" && <AlertThresholdsSection />}
          {activeTab === "models" && <ModelSection />}
          {activeTab === "training" && <TrainingSection />}
        </div>
      </div>
    </div>
  )
}

/* ── User Management ── */

function UserManagementSection() {
  const [createOpen, setCreateOpen] = useState(false)

  const usersQuery = useQuery({
    queryKey: ["users"],
    queryFn: listUsers,
  })

  return (
    <section>
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h2 className="text-sm font-semibold">User Management</h2>
          <p className="mt-0.5 text-xs text-muted-foreground">
            Create, edit, and deactivate user accounts.
          </p>
        </div>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <UserPlus className="mr-1 size-3.5" aria-hidden />
          Add user
        </Button>
      </div>

      {usersQuery.isPending ? (
        <div className="space-y-2">
          {[0, 1, 2].map((i) => (
            <Skeleton key={i} className="h-12 w-full" />
          ))}
        </div>
      ) : usersQuery.isError ? (
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load users</AlertTitle>
          <AlertDescription>
            {(usersQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => void usersQuery.refetch()}
            >
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      ) : (
        <Card className="py-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Username</TableHead>
                <TableHead>Role</TableHead>
                <TableHead className="text-right">Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {(usersQuery.data ?? []).map((user) => (
                <UserRow key={user.id} user={user} />
              ))}
            </TableBody>
          </Table>
        </Card>
      )}

      <CreateUserDialog open={createOpen} onOpenChange={setCreateOpen} />
    </section>
  )
}

function UserRow({ user }: { user: User }) {
  const queryClient = useQueryClient()

  const deactivateMutation = useMutation({
    mutationFn: () => deleteUser(user.id),
    onSuccess: () => {
      toast.success(`User ${user.username} deactivated`)
      void queryClient.invalidateQueries({ queryKey: ["users"] })
    },
    onError: (err) => {
      toast.error("Could not deactivate user", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  const roleBadgeClass: Record<Role, string> = {
    admin: "border-severity-critical/40 bg-severity-critical/10 text-severity-critical",
    supervisor: "border-severity-high/40 bg-severity-high/10 text-severity-high",
    investigator: "border-chart-1/40 bg-chart-1/10 text-chart-1",
    analyst: "border-chart-2/40 bg-chart-2/10 text-chart-2",
  }

  return (
    <TableRow className={!user.active ? "opacity-50" : ""}>
      <TableCell>
        <span className="font-mono text-xs font-medium">{user.username}</span>
        <span className="text-muted-foreground ml-2 font-mono text-[10px]">
          {user.id.slice(0, 8)}
        </span>
      </TableCell>
      <TableCell>
        <Badge variant="outline" className={`font-mono text-[10px] tracking-wider uppercase ${roleBadgeClass[user.role]}`}>
          {user.role}
        </Badge>
      </TableCell>
      <TableCell className="text-right">
        <Badge variant="outline" className={`font-mono text-[10px] ${user.active ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-500" : "text-muted-foreground"}`}>
          {user.active ? "Active" : "Inactive"}
        </Badge>
      </TableCell>
      <TableCell className="text-right">
        <Button
          size="sm"
          variant="ghost"
          onClick={() => void deactivateMutation.mutate()}
          disabled={!user.active || deactivateMutation.isPending}
          title={user.active ? "Deactivate user" : "Already inactive"}
        >
          <Trash2 className="size-3.5" aria-hidden />
        </Button>
      </TableCell>
    </TableRow>
  )
}

function CreateUserDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [role, setRole] = useState<Role>("analyst")

  const mutation = useMutation({
    mutationFn: (input: CreateUserInput) => createUser(input),
    onSuccess: (created) => {
      toast.success("User created", { description: created.username })
      void queryClient.invalidateQueries({ queryKey: ["users"] })
      onOpenChange(false)
      setUsername("")
      setPassword("")
      setRole("analyst")
    },
    onError: (err) => {
      toast.error("Could not create user", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!username.trim() || !password.trim() || mutation.isPending) return
    mutation.mutate({ username: username.trim(), password: password.trim(), role })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add user</DialogTitle>
          <DialogDescription>
            Create a new user account with the specified role.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="new-username">Username</Label>
            <Input
              id="new-username"
              required
              autoFocus
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="officer-name"
              className="font-mono"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="new-password">Password</Label>
            <Input
              id="new-password"
              type="password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Minimum 8 characters"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="new-role">Role</Label>
            <select
              id="new-role"
              value={role}
              onChange={(e) => setRole(e.target.value as Role)}
              className="border-input bg-background h-9 w-full rounded-sm border px-2 font-mono text-xs"
            >
              {ROLES.map((r) => (
                <option key={r} value={r}>
                  {r.charAt(0).toUpperCase() + r.slice(1)}
                </option>
              ))}
            </select>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="ghost"
              onClick={() => onOpenChange(false)}
              disabled={mutation.isPending}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!username.trim() || !password.trim() || mutation.isPending}>
              {mutation.isPending ? "Creating…" : "Create user"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

/* ── Webhook Configuration ── */

function WebhookSection() {
  const queryClient = useQueryClient()

  const webhooksQuery = useQuery({
    queryKey: ["webhooks"],
    queryFn: getWebhooks,
  })

  const [discordUrl, setDiscordUrl] = useState("")
  const [telegramToken, setTelegramToken] = useState("")
  const [telegramChatId, setTelegramChatId] = useState("")
  const [initialized, setInitialized] = useState(false)

  // Initialize form state from query data
  if (webhooksQuery.data && !initialized) {
    setDiscordUrl(webhooksQuery.data.discord_url ?? "")
    setTelegramToken(webhooksQuery.data.telegram_bot_token ?? "")
    setTelegramChatId(webhooksQuery.data.telegram_chat_id ?? "")
    setInitialized(true)
  }

  const updateMutation = useMutation({
    mutationFn: (config: Partial<WebhookConfig>) => updateWebhooks(config),
    onSuccess: () => {
      toast.success("Webhook config updated")
      void queryClient.invalidateQueries({ queryKey: ["webhooks"] })
    },
    onError: (err) => {
      toast.error("Could not update webhooks", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (updateMutation.isPending) return
    updateMutation.mutate({
      discord_url: discordUrl.trim() || null,
      telegram_bot_token: telegramToken.trim() || null,
      telegram_chat_id: telegramChatId.trim() || null,
    })
  }

  return (
    <section>
      <div className="mb-4">
        <h2 className="text-sm font-semibold">Webhook Configuration</h2>
        <p className="mt-0.5 text-xs text-muted-foreground">
          Configure Discord and Telegram notification webhooks.
        </p>
      </div>

      {webhooksQuery.isPending ? (
        <Skeleton className="h-48 w-full" />
      ) : webhooksQuery.isError ? (
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load webhook config</AlertTitle>
          <AlertDescription>
            {(webhooksQuery.error as { message?: string }).message ?? "Unknown error."}
          </AlertDescription>
        </Alert>
      ) : (
        <Card>
          <CardContent className="p-4">
            <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
              <div className="space-y-1.5">
                <Label htmlFor="discord-url" className="font-mono text-xs">
                  Discord Webhook URL
                </Label>
                <Input
                  id="discord-url"
                  value={discordUrl}
                  onChange={(e) => setDiscordUrl(e.target.value)}
                  placeholder="https://discord.com/api/webhooks/..."
                  className="font-mono text-xs"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1.5">
                  <Label htmlFor="tg-token" className="font-mono text-xs">
                    Telegram Bot Token
                  </Label>
                  <Input
                    id="tg-token"
                    value={telegramToken}
                    onChange={(e) => setTelegramToken(e.target.value)}
                    placeholder="123456:ABC-..."
                    className="font-mono text-xs"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="tg-chat" className="font-mono text-xs">
                    Telegram Chat ID
                  </Label>
                  <Input
                    id="tg-chat"
                    value={telegramChatId}
                    onChange={(e) => setTelegramChatId(e.target.value)}
                    placeholder="-1001234567890"
                    className="font-mono text-xs"
                  />
                </div>
              </div>
              <Button type="submit" size="sm" disabled={updateMutation.isPending}>
                {updateMutation.isPending ? "Saving…" : "Save configuration"}
              </Button>
            </form>
          </CardContent>
        </Card>
      )}
    </section>
  )
}

/* ── Model Versions ── */

function ModelSection() {
  const queryClient = useQueryClient()

  const modelsQuery = useQuery({
    queryKey: ["models"],
    queryFn: listModels,
  })

  const promoteMutation = useMutation({
    mutationFn: (version: string) => promoteModel(version),
    onSuccess: (result) => {
      toast.success(`Model ${result.promoted} promoted to active`)
      void queryClient.invalidateQueries({ queryKey: ["models"] })
    },
    onError: (err) => {
      toast.error("Could not promote model", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  return (
    <section>
      <div className="mb-4">
        <h2 className="text-sm font-semibold">Model Versions</h2>
        <p className="mt-0.5 text-xs text-muted-foreground">
          Manage LLM model versions and promote active models.
        </p>
      </div>

      {modelsQuery.isPending ? (
        <div className="space-y-2">
          {[0, 1].map((i) => (
            <Skeleton key={i} className="h-16 w-full" />
          ))}
        </div>
      ) : modelsQuery.isError ? (
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load models</AlertTitle>
          <AlertDescription>
            {(modelsQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => void modelsQuery.refetch()}
            >
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      ) : (modelsQuery.data?.length ?? 0) === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-10 text-center">
            <Bot className="mb-2 size-6 text-muted-foreground" aria-hidden />
            <p className="text-xs font-medium">No model versions</p>
            <p className="mt-1 max-w-xs text-xs text-muted-foreground">
              Models will appear here once training completes.
            </p>
          </CardContent>
        </Card>
      ) : (
        <Card className="py-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Version</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {modelsQuery.data!.map((model) => (
                <TableRow key={model.version}>
                  <TableCell>
                    <span className="font-mono text-xs font-medium">{model.version}</span>
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant="outline"
                      className={`font-mono text-[10px] tracking-wider uppercase ${
                        model.active
                          ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-500"
                          : "text-muted-foreground"
                      }`}
                    >
                      {model.active ? "active" : "inactive"}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-mono text-xs whitespace-nowrap text-muted-foreground">
                    {model.trained_at ? new Date(model.trained_at).toLocaleString("en-IN") : "—"}
                  </TableCell>
                  <TableCell className="text-right">
                    {!model.active ? (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => void promoteMutation.mutate(model.version)}
                        disabled={promoteMutation.isPending}
                      >
                        <Check className="mr-1 size-3.5" aria-hidden />
                        Promote
                      </Button>
                    ) : (
                      <Badge variant="outline" className="border-emerald-500/40 bg-emerald-500/10 text-emerald-500 text-[10px]">
                        Active
                      </Badge>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </section>
  )
}

/* ── Alert Thresholds ── */

function AlertThresholdsSection() {
  const queryClient = useQueryClient()
  const thresholdsQuery = useQuery({
    queryKey: ["alert-thresholds"],
    queryFn: getAlertThresholds,
  })

  const updateMutation = useMutation({
    mutationFn: (t: Partial<AlertThresholds>) => updateAlertThresholds(t),
    onSuccess: () => {
      toast.success("Alert thresholds updated")
      void queryClient.invalidateQueries({ queryKey: ["alert-thresholds"] })
    },
    onError: (err) => {
      toast.error("Could not update thresholds", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  if (thresholdsQuery.isPending) return <Skeleton className="h-64 w-full" />
  if (thresholdsQuery.isError) return (
    <Alert variant="destructive">
      <AlertTriangle className="size-4" aria-hidden />
      <AlertTitle>Failed to load thresholds</AlertTitle>
      <AlertDescription>{(thresholdsQuery.error as { message?: string }).message ?? "Unknown error."}</AlertDescription>
    </Alert>
  )

  const t = thresholdsQuery.data!

  return (
    <section>
      <div className="mb-4">
        <h2 className="text-sm font-semibold">Alert Detection Thresholds</h2>
        <p className="mt-0.5 text-xs text-muted-foreground">
          Tune sensitivity for each anomaly detection rule. Changes apply to the next analysis run.
        </p>
      </div>
      <Card>
        <CardContent className="space-y-6 p-4">
          <ThresholdGroup title="IMEI Reuse" fields={[
            { key: "imei_min_subscribers", label: "Min subscribers per IMEI", value: t.imei_min_subscribers, min: 2, max: 20 },
            { key: "imei_min_evidence", label: "Min evidence events", value: t.imei_min_evidence, min: 10, max: 200 },
          ]} onChange={(k, v) => updateMutation.mutate({ [k]: v })} />
          <ThresholdGroup title="Hawala Signature" fields={[
            { key: "hawala_window_hours", label: "Window (hours)", value: t.hawala_window_hours, min: 12, max: 168 },
            { key: "hawala_min_txns", label: "Min transactions", value: t.hawala_min_txns, min: 2, max: 20 },
            { key: "hawala_min_total", label: "Min total amount", value: t.hawala_min_total, min: 10000, max: 500000 },
            { key: "hawala_max_total", label: "Max total amount", value: t.hawala_max_total, min: 50000, max: 1000000 },
          ]} onChange={(k, v) => updateMutation.mutate({ [k]: v })} />
          <ThresholdGroup title="Rapid Transfer" fields={[
            { key: "rapid_window_minutes", label: "Window (minutes)", value: t.rapid_window_minutes, min: 15, max: 240 },
            { key: "rapid_min_txns", label: "Min transactions", value: t.rapid_min_txns, min: 2, max: 20 },
            { key: "rapid_min_flow", label: "Min total flow", value: t.rapid_min_flow, min: 50000, max: 2000000 },
          ]} onChange={(k, v) => updateMutation.mutate({ [k]: v })} />
          <ThresholdGroup title="Coordinated Silence" fields={[
            { key: "silence_min_parties", label: "Min parties silent", value: t.silence_min_parties, min: 2, max: 10 },
          ]} onChange={(k, v) => updateMutation.mutate({ [k]: v })} />
          <ThresholdGroup title="Bot Social Activity" fields={[
            { key: "bot_min_posts", label: "Min posts", value: t.bot_min_posts, min: 3, max: 100 },
            { key: "bot_max_interval_secs", label: "Max interval (secs)", value: t.bot_max_interval_secs, min: 60, max: 3600 },
          ]} onChange={(k, v) => updateMutation.mutate({ [k]: v })} />
          <ThresholdGroup title="Round-Trip Transfer" fields={[
            { key: "round_trip_window_hours", label: "Window (hours)", value: t.round_trip_window_hours, min: 12, max: 168 },
          ]} onChange={(k, v) => updateMutation.mutate({ [k]: v })} />
          <ThresholdGroup title="Tower Jump" fields={[
            { key: "tower_jump_max_minutes", label: "Max time (minutes)", value: t.tower_jump_max_minutes, min: 5, max: 120 },
            { key: "tower_jump_min_km", label: "Min distance (km)", value: t.tower_jump_min_km, min: 10, max: 200 },
          ]} onChange={(k, v) => updateMutation.mutate({ [k]: v })} />
        </CardContent>
      </Card>
    </section>
  )
}

function ThresholdGroup({
  title,
  fields,
  onChange,
}: {
  title: string
  fields: { key: string; label: string; value: number; min: number; max: number }[]
  onChange: (key: string, value: number) => void
}) {
  return (
    <div>
      <h3 className="mb-2 font-mono text-[11px] tracking-wider text-muted-foreground uppercase">{title}</h3>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {fields.map((f) => (
          <div key={f.key} className="space-y-1">
            <Label className="text-xs text-muted-foreground">{f.label}</Label>
            <Input
              type="number"
              defaultValue={f.value}
              min={f.min}
              max={f.max}
              className="font-mono text-xs"
              onBlur={(e) => {
                const v = parseFloat(e.target.value)
                if (!isNaN(v)) onChange(f.key, v)
              }}
            />
          </div>
        ))}
      </div>
    </div>
  )
}

/* ── Training Queue ── */

function TrainingSection() {
  const queryClient = useQueryClient()

  const queueQuery = useQuery({
    queryKey: ["training-queue"],
    queryFn: getTrainingQueue,
  })

  const triggerMutation = useMutation({
    mutationFn: triggerTraining,
    onSuccess: () => {
      toast.success("Training triggered")
      void queryClient.invalidateQueries({ queryKey: ["training-queue"] })
      void queryClient.invalidateQueries({ queryKey: ["models"] })
    },
    onError: (err) => {
      toast.error("Could not trigger training", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  return (
    <section>
      <div className="mb-4">
        <h2 className="text-sm font-semibold">Training Queue</h2>
        <p className="mt-0.5 text-xs text-muted-foreground">
          Monitor the feedback queue and trigger manual retraining.
        </p>
      </div>

      {queueQuery.isPending ? (
        <Skeleton className="h-24 w-full" />
      ) : queueQuery.isError ? (
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load training queue</AlertTitle>
          <AlertDescription>
            {(queueQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => void queueQuery.refetch()}
            >
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      ) : (
        <Card>
          <CardContent className="flex flex-wrap items-center gap-6 p-4">
            <div className="flex flex-col">
              <span className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                Queue Size
              </span>
              <span className="font-mono text-lg font-semibold tabular-nums">
                {queueQuery.data!.queued_events.toLocaleString("en-IN")}
              </span>
            </div>
            <Separator orientation="vertical" className="hidden h-8 sm:block" />
            <div className="flex flex-col">
              <span className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                Min Batch
              </span>
              <span className="font-mono text-lg font-semibold tabular-nums">
                {queueQuery.data!.minimum_batch}
              </span>
            </div>
            <Separator orientation="vertical" className="hidden h-8 sm:block" />
            <div className="flex flex-col">
              <span className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                Last Run
              </span>
              <span className="font-mono text-xs text-muted-foreground">
                {queueQuery.data!.last_run
                  ? new Date(queueQuery.data!.last_run).toLocaleString("en-IN")
                  : "Never"}
              </span>
            </div>

            <div className="ml-auto">
              <Button
                size="sm"
                onClick={() => void triggerMutation.mutate()}
                disabled={triggerMutation.isPending}
              >
                <Train className="mr-1 size-3.5" aria-hidden />
                {triggerMutation.isPending ? "Triggering…" : "Trigger training"}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </section>
  )
}
