import { useMemo } from "react"
import { useNavigate } from "react-router-dom"
import { useQuery } from "@tanstack/react-query"
import {
  AlertTriangle,
  ArrowRight,
  ChevronRight,
  Fingerprint,
  RefreshCw,
  ShieldAlert,
  Siren,
} from "lucide-react"
import { apiFetch } from "@/api/client"
import type { Alert as AlertType, Case, Severity } from "@/api/types"

import {
  SEVERITY_ORDER,
  severityBadgeClass,
} from "@/lib/severity"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"

export function DashboardScreen() {
  const navigate = useNavigate()

  const casesQuery = useQuery({
    queryKey: ["cases"],
    queryFn: () => apiFetch<Case[]>("/cases"),
  })

  const alertsQuery = useQuery({
    queryKey: ["alerts"],
    queryFn: () => apiFetch<AlertType[]>("/alerts"),
  })

  const cases = casesQuery.data ?? []
  const alerts = alertsQuery.data ?? []

  const criticalAlerts = useMemo(
    () => alerts.filter((a) => a.severity === "critical" && a.status === "open"),
    [alerts]
  )

  const highAlerts = useMemo(
    () => alerts.filter((a) => a.severity === "high" && a.status === "open"),
    [alerts]
  )

  const activeCases = useMemo(
    () => cases.filter((c) => c.status === "active"),
    [cases]
  )

  const recentOpenAlerts = useMemo(
    () =>
      alerts
        .filter((a) => a.status === "open")
        .sort((a, b) => {
          const sevDiff =
            SEVERITY_ORDER.indexOf(b.severity) - SEVERITY_ORDER.indexOf(a.severity)
          if (sevDiff !== 0) return sevDiff
          return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
        })
        .slice(0, 8),
    [alerts]
  )

  const isLoading = casesQuery.isPending || alertsQuery.isPending

  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6">
        <h1 className="text-lg font-semibold">Dashboard</h1>
        <p className="text-sm text-muted-foreground">
          What needs your attention right now.
        </p>
      </header>

      {isLoading ? (
        <div className="space-y-4">
          <Skeleton className="h-20 w-full" />
          <div className="grid grid-cols-3 gap-3">
            {[0, 1, 2].map((i) => (
              <Skeleton key={i} className="h-24" />
            ))}
          </div>
          <Skeleton className="h-48 w-full" />
        </div>
      ) : casesQuery.isError ? (
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load dashboard</AlertTitle>
          <AlertDescription>
            {(casesQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => void casesQuery.refetch()}
            >
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      ) : cases.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-12 text-center">
            <Fingerprint className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">No cases yet</p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Create your first case and ingest data — your action center will appear here.
            </p>
          </CardContent>
        </Card>
      ) : (
        <>
          {/* Critical Alert Banner */}
          {criticalAlerts.length > 0 && (
            <button
              onClick={() => navigate("/alerts")}
              className="mb-6 flex w-full items-center gap-3 rounded-lg border border-red-500/30 bg-red-500/10 p-4 text-left transition-colors hover:bg-red-500/15"
            >
              <div className="flex size-10 shrink-0 items-center justify-center rounded-full bg-red-500/20">
                <Siren className="size-5 text-red-400" aria-hidden />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-red-300">
                  {criticalAlerts.length} critical {criticalAlerts.length === 1 ? "alert" : "alerts"} need review
                </p>
                <p className="mt-0.5 truncate text-xs text-red-400/70">
                  {criticalAlerts[0].summary}
                </p>
              </div>
              <ArrowRight className="size-4 shrink-0 text-red-400" aria-hidden />
            </button>
          )}

          {highAlerts.length > 0 && !criticalAlerts.length && (
            <button
              onClick={() => navigate("/alerts")}
              className="mb-6 flex w-full items-center gap-3 rounded-lg border border-orange-500/30 bg-orange-500/10 p-4 text-left transition-colors hover:bg-orange-500/15"
            >
              <div className="flex size-10 shrink-0 items-center justify-center rounded-full bg-orange-500/20">
                <Siren className="size-5 text-orange-400" aria-hidden />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-orange-300">
                  {highAlerts.length} high-severity {highAlerts.length === 1 ? "alert" : "alerts"} open
                </p>
                <p className="mt-0.5 truncate text-xs text-orange-400/70">
                  {highAlerts[0].summary}
                </p>
              </div>
              <ArrowRight className="size-4 shrink-0 text-orange-400" aria-hidden />
            </button>
          )}

          {/* Quick Stats */}
          <div className="mb-6 grid grid-cols-3 gap-3">
            <StatCard
              label="Active Cases"
              value={activeCases.length}
              icon={<FolderIcon />}
              onClick={() => navigate("/cases")}
            />
            <StatCard
              label="Open Alerts"
              value={alerts.filter((a) => a.status === "open").length}
              icon={<Siren className="size-4 text-muted-foreground" aria-hidden />}
              onClick={() => navigate("/alerts")}
              highlight={criticalAlerts.length > 0}
            />
            <StatCard
              label="Total Entities"
              value={cases.reduce((sum, c) => sum + c.stats.entity_count, 0)}
              icon={<ShieldAlert className="size-4 text-muted-foreground" aria-hidden />}
            />
          </div>

          {/* Active Cases */}
          {activeCases.length > 0 && (
            <section className="mb-6">
              <div className="mb-3 flex items-center justify-between">
                <h2 className="font-mono text-[11px] tracking-[0.18em] text-muted-foreground uppercase">
                  Active Cases
                </h2>
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-xs"
                  onClick={() => navigate("/cases")}
                >
                  View all <ChevronRight className="ml-1 size-3" aria-hidden />
                </Button>
              </div>
              <div className="space-y-2">
                {activeCases.slice(0, 5).map((c) => (
                  <CaseActionRow key={c.id} kase={c} />
                ))}
              </div>
            </section>
          )}

          {/* Recent Alerts */}
          {recentOpenAlerts.length > 0 && (
            <section>
              <div className="mb-3 flex items-center justify-between">
                <h2 className="font-mono text-[11px] tracking-[0.18em] text-muted-foreground uppercase">
                  Recent Alerts
                </h2>
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-xs"
                  onClick={() => navigate("/alerts")}
                >
                  View all <ChevronRight className="ml-1 size-3" aria-hidden />
                </Button>
              </div>
              <div className="space-y-2">
                {recentOpenAlerts.map((a) => (
                  <AlertActionRow key={a.id} alert={a} />
                ))}
              </div>
            </section>
          )}
        </>
      )}
    </div>
  )
}

function StatCard({
  label,
  value,
  icon,
  onClick,
  highlight,
}: {
  label: string
  value: number
  icon: React.ReactNode
  onClick?: () => void
  highlight?: boolean
}) {
  const Comp = onClick ? "button" : "div"
  return (
    <Comp
      onClick={onClick}
      className={`flex items-center gap-3 rounded-lg border p-4 text-left transition-colors ${
        highlight
          ? "border-red-500/30 bg-red-500/5"
          : "border-border bg-card"
      } ${onClick ? "cursor-pointer hover:bg-accent/50" : ""}`}
    >
      <div className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted">
        {icon}
      </div>
      <div>
        <p className="font-mono text-2xl font-semibold tabular-nums">
          {value.toLocaleString("en-IN")}
        </p>
        <p className="text-xs text-muted-foreground">{label}</p>
      </div>
    </Comp>
  )
}

function FolderIcon() {
  return (
    <svg
      className="size-4 text-muted-foreground"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" />
    </svg>
  )
}

function CaseActionRow({ kase }: { kase: Case }) {
  const navigate = useNavigate()
  const critical = kase.stats.alerts_by_severity.critical ?? 0
  const high = kase.stats.alerts_by_severity.high ?? 0
  const totalAlerts = critical + high + (kase.stats.alerts_by_severity.medium ?? 0) + (kase.stats.alerts_by_severity.low ?? 0)

  return (
    <button
      onClick={() => void navigate(`/cases/${kase.id}`)}
      className="flex w-full items-center gap-3 rounded-lg border border-border bg-card p-3 text-left transition-colors hover:bg-accent/50"
    >
      <div className="flex min-w-0 flex-1 items-center gap-3">
        <span className="truncate text-sm font-medium">{kase.title}</span>
        <span className="font-mono text-[10px] text-muted-foreground">{kase.id.slice(0, 8)}</span>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        {critical > 0 && (
          <Badge className={severityBadgeClass.critical}>
            {critical} critical
          </Badge>
        )}
        {high > 0 && (
          <Badge className={severityBadgeClass.high}>
            {high} high
          </Badge>
        )}
        {totalAlerts === 0 && (
          <span className="font-mono text-xs text-muted-foreground">No alerts</span>
        )}
        <ChevronRight className="size-4 shrink-0 text-muted-foreground" aria-hidden />
      </div>
    </button>
  )
}

function AlertActionRow({ alert }: { alert: AlertType }) {
  const navigate = useNavigate()
  const severityColors: Record<Severity, string> = {
    critical: "border-l-red-500",
    high: "border-l-orange-500",
    medium: "border-l-amber-500",
    low: "border-l-slate-500",
  }

  return (
    <button
      onClick={() => void navigate(`/cases/${alert.case_id}`)}
      className={`flex w-full items-center gap-3 rounded-lg border border-border border-l-2 bg-card p-3 text-left transition-colors hover:bg-accent/50 ${severityColors[alert.severity]}`}
    >
      <Badge className={severityBadgeClass[alert.severity]}>
        {alert.severity}
      </Badge>
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-sm">{alert.summary}</span>
        <span className="font-mono text-[10px] text-muted-foreground">
          {alert.pattern} · {new Date(alert.created_at).toLocaleDateString("en-IN")}
        </span>
      </div>
      <ChevronRight className="size-4 shrink-0 text-muted-foreground" aria-hidden />
    </button>
  )
}
