import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import { AlertTriangle, Fingerprint, RefreshCw } from "lucide-react"
import { apiFetch } from "@/api/client"
import type { Case, Severity, SourceType } from "@/api/types"
import {
  SEVERITY_ORDER,
  SOURCE_LABELS,
  SOURCE_ORDER,
  severityBadgeClass,
} from "@/lib/severity"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"

export function DashboardScreen() {
  const casesQuery = useQuery({
    queryKey: ["cases"],
    queryFn: () => apiFetch<Case[]>("/cases"),
  })

  const totals = useMemo(() => aggregate(casesQuery.data ?? []), [casesQuery.data])

  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6">
        <h1 className="text-lg font-semibold">Dashboard</h1>
        <p className="text-sm text-muted-foreground">
          Signal overview across all cases you are authorized to see.
        </p>
      </header>

      {casesQuery.isPending ? (
        <div className="space-y-4">
          <Skeleton className="h-4 w-40" />
          <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
            {[0, 1, 2, 3].map((i) => (
              <Skeleton key={i} className="h-24" />
            ))}
          </div>
          <Skeleton className="h-4 w-32" />
          <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
            {[0, 1, 2, 3].map((i) => (
              <Skeleton key={i} className="h-24" />
            ))}
          </div>
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
      ) : (casesQuery.data?.length ?? 0) === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-12 text-center">
            <Fingerprint className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">No cases yet</p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Create your first case and ingest data — KPIs will appear here.
            </p>
          </CardContent>
        </Card>
      ) : (
        <>
          <section className="mb-6">
            <h2 className="mb-2 font-mono text-[11px] tracking-[0.18em] text-muted-foreground uppercase">
              Open alerts by severity
            </h2>
            <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
              {SEVERITY_ORDER.map((severity) => (
                <KpiCard
                  key={severity}
                  label={severity}
                  value={totals.alerts_by_severity[severity] ?? 0}
                  badgeClass={severityBadgeClass[severity]}
                />
              ))}
            </div>
          </section>

          <section className="mb-6">
            <h2 className="mb-2 font-mono text-[11px] tracking-[0.18em] text-muted-foreground uppercase">
              Events by source
            </h2>
            <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
              {SOURCE_ORDER.map((source) => (
                <KpiCard
                  key={source}
                  label={SOURCE_LABELS[source]}
                  value={totals.events_by_source[source] ?? 0}
                />
              ))}
            </div>
          </section>

          <section>
            <h2 className="mb-2 font-mono text-[11px] tracking-[0.18em] text-muted-foreground uppercase">
              Cases ({casesQuery.data.length})
            </h2>
            <div className="space-y-2">
              {casesQuery.data.map((c) => (
                <CaseRow key={c.id} kase={c} />
              ))}
            </div>
          </section>
        </>
      )}
    </div>
  )
}

function KpiCard({
  label,
  value,
  badgeClass,
}: {
  label: string
  value: number
  badgeClass?: string
}) {
  return (
    <Card className="py-4">
      <CardHeader className="pb-0">
        <CardDescription
          className={`font-mono text-[11px] tracking-wider uppercase ${badgeClass ?? ""}`}
        >
          {label}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <span className="font-mono text-2xl font-semibold tabular-nums">
          {value.toLocaleString("en-IN")}
        </span>
      </CardContent>
    </Card>
  )
}

function CaseRow({ kase }: { kase: Case }) {
  const critical = kase.stats.alerts_by_severity.critical ?? 0
  const high = kase.stats.alerts_by_severity.high ?? 0
  return (
    <Card className="py-3">
      <CardContent className="flex items-center justify-between gap-4 px-4">
        <div className="flex min-w-0 items-center gap-3">
          <span className="truncate text-sm font-medium">{kase.title}</span>
          <Badge variant="outline" className="shrink-0 font-mono text-[10px] tracking-wider uppercase">
            {kase.classification}
          </Badge>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {critical > 0 ? <Badge className={severityBadgeClass.critical}>{critical} critical</Badge> : null}
          {high > 0 ? <Badge className={severityBadgeClass.high}>{high} high</Badge> : null}
          <span className="font-mono text-xs tabular-nums text-muted-foreground">
            {Object.values(kase.stats.events_by_source).reduce((a, b) => a + b, 0).toLocaleString("en-IN")} events ·{" "}
            {kase.stats.entity_count.toLocaleString("en-IN")} entities
          </span>
          <span className="font-mono text-xs text-muted-foreground">{kase.id.slice(0, 8)}</span>
        </div>
      </CardContent>
    </Card>
  )
}

interface Totals {
  events_by_source: Record<SourceType, number>
  alerts_by_severity: Record<Severity, number>
  entity_count: number
}

function aggregate(cases: Case[]): Totals {
  const totals: Totals = {
    events_by_source: { CDR: 0, IPDR: 0, BANK: 0, SOCIAL: 0 },
    alerts_by_severity: { critical: 0, high: 0, medium: 0, low: 0 },
    entity_count: 0,
  }
  for (const c of cases) {
    for (const source of SOURCE_ORDER) {
      totals.events_by_source[source] += c.stats.events_by_source[source] ?? 0
    }
    for (const severity of SEVERITY_ORDER) {
      totals.alerts_by_severity[severity] += c.stats.alerts_by_severity[severity] ?? 0
    }
    totals.entity_count += c.stats.entity_count
  }
  return totals
}
