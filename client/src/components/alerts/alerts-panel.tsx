import { useState } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import {
  AlertTriangle,
  Check,
  RefreshCw,
  Siren,
  X,
} from "lucide-react"
import { listAlerts, triageAlert, analyzeCase } from "@/api/alerts"
import { ApiClientError } from "@/api/client"
import type { Alert, AlertStatus, Severity } from "@/api/types"
import { SEVERITY_ORDER, severityBadgeClass } from "@/lib/severity"
import { useAuth } from "@/auth/AuthContext"
import { Alert as AlertUi, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import { Textarea } from "@/components/ui/textarea"

type SeverityFilter = "all" | Severity
type StatusFilter = "all" | AlertStatus

const STATUS_OPTIONS: readonly (StatusFilter)[] = ["all", "open", "confirmed", "false_positive"]

const STATUS_BADGE_CLASS: Record<AlertStatus, string> = {
  open: "border-severity-high/40 bg-severity-high/10 text-severity-high",
  reviewing: "border-chart-1/40 bg-chart-1/10 text-chart-1",
  confirmed: "border-emerald-500/40 bg-emerald-500/10 text-emerald-500",
  false_positive: "border-muted-foreground/40 bg-muted/10 text-muted-foreground",
}

interface AlertsPanelProps {
  /** When provided, filter alerts to this case. Otherwise show cross-case. */
  caseId?: string
}

export function AlertsPanel({ caseId }: AlertsPanelProps) {
  const [severityFilter, setSeverityFilter] = useState<SeverityFilter>("all")
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all")
  const [selectedAlert, setSelectedAlert] = useState<Alert | null>(null)
  const { can } = useAuth()

  const alertsQuery = useQuery({
    queryKey: ["alerts", caseId ?? "all", severityFilter, statusFilter],
    queryFn: () =>
      listAlerts({
        case_id: caseId,
        severity: severityFilter === "all" ? undefined : severityFilter,
        status: statusFilter === "all" ? undefined : statusFilter,
        limit: 500,
      }),
  })

  const analyzeMutation = useMutation({
    mutationFn: () => analyzeCase(caseId!),
    onSuccess: (result) => {
      toast.success(`Analysis complete — ${result.alerts_raised} alerts raised`)
      void alertsQuery.refetch()
    },
    onError: (err) => {
      toast.error("Analysis failed", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  const alerts = alertsQuery.data ?? []

  // Group by severity for the card view
  const grouped = SEVERITY_ORDER.reduce(
    (acc, severity) => {
      acc[severity] = alerts.filter((a) => a.severity === severity)
      return acc
    },
    {} as Record<Severity, Alert[]>,
  )

  return (
    <div className="space-y-4">
      {/* Filters bar */}
      <div className="flex flex-wrap items-end gap-x-4 gap-y-2">
        <div className="space-y-1">
          <Label
            className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
          >
            Severity
          </Label>
          <div className="flex gap-1" role="group" aria-label="Filter by severity">
            {(["all", ...SEVERITY_ORDER] as SeverityFilter[]).map((s) => (
              <Button
                key={s}
                size="sm"
                variant={severityFilter === s ? "secondary" : "ghost"}
                onClick={() => setSeverityFilter(s)}
                className="h-7 font-mono text-xs uppercase"
              >
                {s}
              </Button>
            ))}
          </div>
        </div>
        <div className="space-y-1">
          <Label
            className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
          >
            Status
          </Label>
          <div className="flex gap-1" role="group" aria-label="Filter by status">
            {STATUS_OPTIONS.map((s) => (
              <Button
                key={s}
                size="sm"
                variant={statusFilter === s ? "secondary" : "ghost"}
                onClick={() => setStatusFilter(s)}
                className="h-7 font-mono text-xs"
              >
                {s === "all" ? "ALL" : s.replace("_", " ")}
              </Button>
            ))}
          </div>
        </div>
        {caseId && can("analysis.run") ? (
          <Button
            size="sm"
            variant="outline"
            onClick={() => void analyzeMutation.mutate()}
            disabled={analyzeMutation.isPending}
            className="ml-auto"
          >
            <Siren className="mr-1 size-3.5" aria-hidden />
            {analyzeMutation.isPending ? "Analyzing…" : "Run analysis"}
          </Button>
        ) : null}
      </div>

      {/* Alert cards grouped by severity */}
      {alertsQuery.isPending ? (
        <div className="space-y-3">
          {[0, 1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-24 w-full" />
          ))}
        </div>
      ) : alertsQuery.isError ? (
        <AlertUi variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load alerts</AlertTitle>
          <AlertDescription>
            {(alertsQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => void alertsQuery.refetch()}
            >
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </AlertUi>
      ) : alerts.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-14 text-center">
            <Siren className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">No alerts</p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              {caseId
                ? "No alerts match the current filters for this case."
                : "No alerts across any case."}
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-3">
          {SEVERITY_ORDER.map((severity) => {
            const items = grouped[severity]
            if (items.length === 0) return null
            return (
              <div key={severity}>
                <p className="mb-1.5 font-mono text-[10px] tracking-[0.18em] text-muted-foreground uppercase">
                  {severity} ({items.length})
                </p>
                <div className="space-y-2">
                  {items.map((alert) => (
                    <AlertCard
                      key={alert.id}
                      alert={alert}
                      onClick={() => setSelectedAlert(alert)}
                    />
                  ))}
                </div>
              </div>
            )
          })}
        </div>
      )}

      {/* Detail sheet */}
      <AlertDetailSheet
        alert={selectedAlert}
        onClose={() => setSelectedAlert(null)}
      />
    </div>
  )
}

function AlertCard({
  alert,
  onClick,
}: {
  alert: Alert
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className={`w-full rounded-sm border p-3 text-left transition-colors hover:bg-secondary/50 ${
        alert.severity === "critical"
          ? "border-severity-critical/40"
          : alert.severity === "high"
            ? "border-severity-high/40"
            : "border-border"
      }`}
    >
      <div className="flex items-center gap-2">
        <Badge
          variant="outline"
          className={`font-mono text-[10px] tracking-wider uppercase ${severityBadgeClass[alert.severity]}`}
        >
          {alert.severity}
        </Badge>
        <span className="font-mono text-xs font-medium">{alert.pattern}</span>
        <Badge
          variant="outline"
          className={`ml-auto font-mono text-[9px] ${STATUS_BADGE_CLASS[alert.status]}`}
        >
          {alert.status.replace("_", " ")}
        </Badge>
      </div>
      <p className="mt-1.5 line-clamp-2 text-xs text-muted-foreground">
        {alert.summary}
      </p>
      <div className="mt-1.5 flex items-center gap-3 font-mono text-[10px] text-muted-foreground">
        <span>Score: {alert.score}</span>
        <span>{alert.entity_ids.length} entities</span>
        <span>{alert.evidence_event_ids.length} evidence events</span>
        <span className="ml-auto">{new Date(alert.created_at).toLocaleString("en-IN")}</span>
      </div>
    </button>
  )
}

function AlertDetailSheet({
  alert,
  onClose,
}: {
  alert: Alert | null
  onClose: () => void
}) {
  const queryClient = useQueryClient()
  const [note, setNote] = useState("")

  const triageMutation = useMutation({
    mutationFn: (payload: { status: AlertStatus; note?: string }) =>
      triageAlert(alert!.id, payload),
    onSuccess: () => {
      toast.success("Alert triaged")
      setNote("")
      onClose()
      void queryClient.invalidateQueries({ queryKey: ["alerts"] })
    },
    onError: (err) => {
      toast.error("Triage failed", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  function handleTriage(status: AlertStatus) {
    if (!alert) return
    const payload: { status: AlertStatus; note?: string } = { status }
    if (note.trim()) payload.note = note.trim()
    triageMutation.mutate(payload)
  }

  return (
    <Sheet open={alert !== null} onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="w-full overflow-y-auto sm:max-w-lg">
        {alert ? (
          <>
            <SheetHeader>
              <div className="flex items-center gap-2">
                <Badge
                  variant="outline"
                  className={`font-mono text-[10px] tracking-wider uppercase ${severityBadgeClass[alert.severity]}`}
                >
                  {alert.severity}
                </Badge>
                <SheetTitle className="font-mono text-sm">
                  {alert.pattern}
                </SheetTitle>
              </div>
              <SheetDescription className="font-mono text-xs">
                {alert.id}
              </SheetDescription>
            </SheetHeader>

            <div className="mt-4 space-y-4">
              {/* Summary */}
              <div>
                <p className="mb-1 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                  Summary
                </p>
                <p className="text-sm">{alert.summary}</p>
              </div>

              {/* Metadata */}
              <dl className="grid grid-cols-2 gap-2 text-xs">
                <div>
                  <dt className="text-muted-foreground">Score</dt>
                  <dd className="font-mono font-semibold tabular-nums">{alert.score}/100</dd>
                </div>
                <div>
                  <dt className="text-muted-foreground">Status</dt>
                  <dd>
                    <Badge
                      variant="outline"
                      className={`font-mono text-[10px] ${STATUS_BADGE_CLASS[alert.status]}`}
                    >
                      {alert.status.replace("_", " ")}
                    </Badge>
                  </dd>
                </div>
                <div>
                  <dt className="text-muted-foreground">Case</dt>
                  <dd className="font-mono text-[11px]">{alert.case_id.slice(0, 8)}</dd>
                </div>
                <div>
                  <dt className="text-muted-foreground">Created</dt>
                  <dd className="font-mono text-[11px]">
                    {new Date(alert.created_at).toLocaleString("en-IN")}
                  </dd>
                </div>
              </dl>

              {/* Entities */}
              {alert.entity_ids.length > 0 ? (
                <div>
                  <p className="mb-1 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                    Entities ({alert.entity_ids.length})
                  </p>
                  <div className="flex flex-wrap gap-1.5">
                    {alert.entity_ids.map((eid) => (
                      <Badge key={eid} variant="outline" className="font-mono text-[10px]">
                        {eid.slice(0, 12)}…
                      </Badge>
                    ))}
                  </div>
                </div>
              ) : null}

              {/* Evidence events */}
              {alert.evidence_event_ids.length > 0 ? (
                <div>
                  <p className="mb-1 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                    Evidence events ({alert.evidence_event_ids.length})
                  </p>
                  <div className="flex flex-wrap gap-1.5">
                    {alert.evidence_event_ids.map((eid) => (
                      <Badge key={eid} variant="secondary" className="font-mono text-[10px]">
                        {eid.slice(0, 12)}…
                      </Badge>
                    ))}
                  </div>
                </div>
              ) : null}

              {/* Triage */}
              <div className="border-border border-t pt-4">
                <p className="mb-2 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                  Triage
                </p>
                <div className="space-y-2">
                  <Textarea
                    value={note}
                    onChange={(e) => setNote(e.target.value)}
                    placeholder="Optional note…"
                    rows={2}
                    aria-label="Triage note"
                  />
                  <div className="flex gap-2">
                    <Button
                      size="sm"
                      onClick={() => handleTriage("confirmed")}
                      disabled={triageMutation.isPending}
                    >
                      <Check className="mr-1 size-3.5" aria-hidden />
                      Confirm
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleTriage("false_positive")}
                      disabled={triageMutation.isPending}
                    >
                      <X className="mr-1 size-3.5" aria-hidden />
                      False Positive
                    </Button>

                  </div>
                </div>
              </div>
            </div>
          </>
        ) : null}
      </SheetContent>
    </Sheet>
  )
}
