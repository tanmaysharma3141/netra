import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  AlertTriangle,
  RefreshCw,
  ScrollText,
  Shield,
} from "lucide-react"
import { listCases } from "@/api/cases"
import { listAuditEntries } from "@/api/audit"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
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

export function AuditScreen() {
  const [selectedCaseId, setSelectedCaseId] = useState<string | null>(null)

  const casesQuery = useQuery({ queryKey: ["cases"], queryFn: listCases })

  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6">
        <div className="flex items-center gap-2">
          <h1 className="text-lg font-semibold">Audit Log</h1>
          <Badge variant="outline" className="font-mono text-[10px] tracking-wider uppercase">
            Admin / Supervisor
          </Badge>
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          Immutable system event log — every action, timestamped and attributed.
        </p>
      </header>

      {casesQuery.isPending ? (
        <div className="space-y-2">
          {[0, 1, 2].map((i) => (
            <Skeleton key={i} className="h-12 w-full" />
          ))}
        </div>
      ) : casesQuery.isError ? (
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load cases</AlertTitle>
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
      ) : (casesQuery.data?.length ?? 0) === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-14 text-center">
            <ScrollText className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">No cases yet</p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Create a case and perform actions to see audit entries here.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="flex min-h-[60vh] gap-4">
          {/* Case list sidebar */}
          <div className="w-64 shrink-0 overflow-y-auto">
            <p className="mb-2 font-mono text-[10px] tracking-[0.18em] text-muted-foreground uppercase">
              Cases
            </p>
            <div className="space-y-1">
              {casesQuery.data!.map((c) => (
                <button
                  key={c.id}
                  onClick={() => setSelectedCaseId(c.id)}
                  className={`w-full rounded-sm px-3 py-2 text-left text-sm transition-colors ${
                    selectedCaseId === c.id
                      ? "bg-secondary font-medium"
                      : "text-muted-foreground hover:bg-secondary/60 hover:text-foreground"
                  }`}
                >
                  <span className="block truncate">{c.title}</span>
                  <span className="font-mono text-[10px] text-muted-foreground">
                    {c.id.slice(0, 8)}
                  </span>
                </button>
              ))}
            </div>
          </div>

          <Separator orientation="vertical" className="h-auto" />

          {/* Audit entries */}
          <div className="min-w-0 flex-1">
            {selectedCaseId ? (
              <AuditEntries caseId={selectedCaseId} />
            ) : (
              <div className="flex flex-col items-center justify-center py-20 text-center">
                <ScrollText className="mb-3 size-8 text-muted-foreground" aria-hidden />
                <p className="text-sm font-medium">Select a case</p>
                <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                  Choose a case from the sidebar to view its audit trail.
                </p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

function AuditEntries({ caseId }: { caseId: string }) {
  const [actionFilter, setActionFilter] = useState<string>("all")

  const auditQuery = useQuery({
    queryKey: ["audit", caseId],
    queryFn: () => listAuditEntries(caseId),
  })

  const entries = auditQuery.data ?? []

  // Derive available action types from entries
  const actionTypes = [...new Set(entries.map((e) => e.action))].sort()

  const filtered = actionFilter === "all"
    ? entries
    : entries.filter((e) => e.action === actionFilter)

  if (auditQuery.isPending) {
    return (
      <div className="space-y-2">
        {[0, 1, 2, 3, 4].map((i) => (
          <Skeleton key={i} className="h-10 w-full" />
        ))}
      </div>
    )
  }

  if (auditQuery.isError) {
    return (
      <Alert variant="destructive">
        <AlertTriangle className="size-4" aria-hidden />
        <AlertTitle>Failed to load audit entries</AlertTitle>
        <AlertDescription>
          {(auditQuery.error as { message?: string }).message ?? "Unknown error."}
          <Button
            variant="outline"
            size="sm"
            className="mt-2"
            onClick={() => void auditQuery.refetch()}
          >
            <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
            Retry
          </Button>
        </AlertDescription>
      </Alert>
    )
  }

  if (entries.length === 0) {
    return (
      <Card className="border-dashed">
        <CardContent className="flex flex-col items-center justify-center py-14 text-center">
          <ScrollText className="mb-3 size-8 text-muted-foreground" aria-hidden />
          <p className="text-sm font-medium">No audit entries</p>
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">
            Actions on this case will appear here as an immutable log.
          </p>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-3">
      {/* Action filter */}
      {actionTypes.length > 1 && (
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
            Filter:
          </span>
          <select
            value={actionFilter}
            onChange={(e) => setActionFilter(e.target.value)}
            className="border-input bg-background h-7 rounded-sm border px-2 font-mono text-xs"
          >
            <option value="all">All actions</option>
            {actionTypes.map((action) => (
              <option key={action} value={action}>
                {action}
              </option>
            ))}
          </select>
          <span className="font-mono text-[10px] text-muted-foreground">
            {filtered.length} of {entries.length} entries
          </span>
        </div>
      )}
      <p className="font-mono text-[10px] tracking-[0.18em] text-muted-foreground uppercase">
        {filtered.length} entries (newest first)
      </p>
      <Card className="py-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-44">Timestamp</TableHead>
              <TableHead className="w-32">User</TableHead>
              <TableHead className="w-40">Action</TableHead>
              <TableHead>Detail</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filtered.map((entry) => (
              <TableRow key={entry.id}>
                <TableCell className="font-mono text-xs whitespace-nowrap text-muted-foreground">
                  {formatTimestamp(entry.at)}
                </TableCell>
                <TableCell>
                  <div className="flex items-center gap-1.5">
                    <Shield className="size-3 text-muted-foreground" aria-hidden />
                    <span className="font-mono text-xs">{entry.user_id.slice(0, 8)}</span>
                  </div>
                </TableCell>
                <TableCell>
                  <ActionBadge action={entry.action} />
                </TableCell>
                <TableCell>
                  <DetailDisplay detail={entry.detail} />
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Card>
    </div>
  )
}

function ActionBadge({ action }: { action: string }) {
  const colorMap: Record<string, string> = {
    "auth.login": "border-chart-1/40 bg-chart-1/10 text-chart-1",
    "auth.logout": "border-muted-foreground/40 bg-muted/10 text-muted-foreground",
    "case.created": "border-emerald-500/40 bg-emerald-500/10 text-emerald-500",
    "case.updated": "border-chart-2/40 bg-chart-2/10 text-chart-2",
    "ingest.completed": "border-chart-3/40 bg-chart-3/10 text-chart-3",
    "alert.confirmed": "border-severity-critical/40 bg-severity-critical/10 text-severity-critical",
    "alert.false_positive": "border-muted-foreground/40 bg-muted/10 text-muted-foreground",
    "report.generated": "border-chart-4/40 bg-chart-4/10 text-chart-4",
    "report.approved": "border-emerald-500/40 bg-emerald-500/10 text-emerald-500",
  }

  const cls = colorMap[action] ?? "border-border bg-muted/10 text-muted-foreground"

  return (
    <Badge variant="outline" className={`font-mono text-[10px] ${cls}`}>
      {action}
    </Badge>
  )
}

function DetailDisplay({ detail }: { detail: Record<string, unknown> }) {
  const keys = Object.keys(detail)
  if (keys.length === 0) {
    return <span className="text-muted-foreground font-mono text-[10px]">—</span>
  }

  // Show first 2 key-value pairs, then a count if more
  const shown = keys.slice(0, 2)
  const remaining = keys.length - shown.length

  return (
    <div className="flex flex-wrap gap-x-3 gap-y-0.5">
      {shown.map((key) => (
        <span key={key} className="font-mono text-[10px]">
          <span className="text-muted-foreground">{key}:</span>{" "}
          {formatDetailValue(detail[key])}
        </span>
      ))}
      {remaining > 0 ? (
        <span className="text-muted-foreground font-mono text-[10px]">
          +{remaining} more
        </span>
      ) : null}
    </div>
  )
}

function formatDetailValue(value: unknown): string {
  if (value === null || value === undefined) return "—"
  if (typeof value === "string") return value
  if (typeof value === "number" || typeof value === "boolean") return String(value)
  return JSON.stringify(value)
}

function formatTimestamp(iso: string): string {
  return new Date(iso).toLocaleString("en-IN", {
    day: "2-digit",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  })
}
