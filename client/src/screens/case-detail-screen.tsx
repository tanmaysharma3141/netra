import { useQuery } from "@tanstack/react-query"
import { Link, useParams } from "react-router-dom"
import { AlertTriangle, ArrowLeft, RefreshCw } from "lucide-react"
import { getCase } from "@/api/cases"
import type { Case } from "@/api/types"
import {
  SEVERITY_ORDER,
  SOURCE_LABELS,
  SOURCE_ORDER,
  severityBadgeClass,
} from "@/lib/severity"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { Skeleton } from "@/components/ui/skeleton"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { TimelinePanel } from "@/components/timeline/timeline-panel"

interface TabDef {
  value: string
  label: string
  phase: string
}

const TABS: readonly TabDef[] = [
  { value: "timeline", label: "Timeline", phase: "PHASE 2 · TIMELINE" },
  { value: "graph", label: "Graph", phase: "PHASE 4 · GRAPH COMPONENT" },
  { value: "map", label: "Map", phase: "PHASE 4 · MAP COMPONENT" },
  { value: "alerts", label: "Alerts", phase: "PHASE 3 · ALERTS" },
  { value: "reports", label: "Reports", phase: "PHASE 5 · REPORTS" },
  { value: "chat", label: "Chat", phase: "PHASE 5 · COPILOT CHAT" },
]

export function CaseDetailScreen() {
  const { id = "" } = useParams()
  const caseQuery = useQuery({ queryKey: ["case", id], queryFn: () => getCase(id) })

  if (caseQuery.isPending) {
    return (
      <div className="mx-auto max-w-6xl space-y-4 p-6">
        <Skeleton className="h-8 w-2/3" />
        <Skeleton className="h-4 w-1/3" />
        <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
          {[0, 1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-20" />
          ))}
        </div>
        <Skeleton className="h-72 w-full" />
      </div>
    )
  }

  if (caseQuery.isError) {
    return (
      <div className="mx-auto max-w-6xl p-6">
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
          <AlertTitle>Failed to load case</AlertTitle>
          <AlertDescription>
            {(caseQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => void caseQuery.refetch()}
            >
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      </div>
    )
  }

  const kase = caseQuery.data

  return (
    <div className="mx-auto max-w-6xl p-6">
      <Link
        to="/cases"
        className="mb-3 inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-3.5" aria-hidden />
        All cases
      </Link>

      <header className="mb-5">
        <div className="flex flex-wrap items-center gap-2.5">
          <h1 className="text-lg font-semibold">{kase.title}</h1>
          <Badge variant="outline" className="font-mono text-[10px] tracking-wider uppercase">
            {kase.status}
          </Badge>
          <Badge variant="outline" className="font-mono text-[10px] tracking-wider uppercase">
            {kase.classification}
          </Badge>
        </div>
        <p className="mt-1.5 font-mono text-xs text-muted-foreground">
          {kase.id} · opened {new Date(kase.created_at).toLocaleString("en-IN")}
        </p>
        {kase.tags.length > 0 ? (
          <div className="mt-2 flex flex-wrap gap-1.5">
            {kase.tags.map((tag) => (
              <Badge key={tag} variant="secondary" className="text-[10px]">
                {tag}
              </Badge>
            ))}
          </div>
        ) : null}
      </header>

      <StatsStrip kase={kase} />

      <Tabs defaultValue="timeline" className="mt-6">
        <TabsList className="w-full justify-start overflow-x-auto">
          {TABS.map((tab) => (
            <TabsTrigger key={tab.value} value={tab.value}>
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>
        {TABS.map((tab) => (
          <TabsContent key={tab.value} value={tab.value} className="mt-4">
            {tab.value === "timeline" ? (
              <div className="h-[calc(100vh-22rem)] min-h-96">
                <TimelinePanel caseId={kase.id} />
              </div>
            ) : (
              <Card className="border-dashed">
                <CardContent className="flex flex-col items-center justify-center py-14 text-center">
                  <p className="font-mono text-sm">{tab.phase}</p>
                  <p className="mt-1 max-w-md text-sm text-muted-foreground">
                    This tab ships in a later phase of docs/PLAN_FRONTEND.md.
                  </p>
                </CardContent>
              </Card>
            )}
          </TabsContent>
        ))}
      </Tabs>
    </div>
  )
}

function StatsStrip({ kase }: { kase: Case }) {
  const totalEvents = SOURCE_ORDER.reduce(
    (sum, source) => sum + (kase.stats.events_by_source[source] ?? 0),
    0,
  )

  return (
    <Card>
      <CardContent className="flex flex-wrap items-center gap-x-6 gap-y-3 px-4 py-3">
        <Metric label="Entities" value={kase.stats.entity_count.toLocaleString("en-IN")} />
        <Separator orientation="vertical" className="hidden h-8 sm:block" />
        <Metric label="Events" value={totalEvents.toLocaleString("en-IN")} />
        <Separator orientation="vertical" className="hidden h-8 sm:block" />
        <div className="flex flex-col gap-1">
          <span className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
            Events by source
          </span>
          <div className="flex gap-3 font-mono text-xs tabular-nums">
            {SOURCE_ORDER.map((source) => (
              <span key={source}>
                <span className="text-muted-foreground">{SOURCE_LABELS[source]}</span>{" "}
                {(kase.stats.events_by_source[source] ?? 0).toLocaleString("en-IN")}
              </span>
            ))}
          </div>
        </div>
        <Separator orientation="vertical" className="hidden h-8 md:block" />
        <div className="flex flex-col gap-1">
          <span className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
            Alerts by severity
          </span>
          <div className="flex gap-1.5">
            {SEVERITY_ORDER.map((severity) => {
              const count = kase.stats.alerts_by_severity[severity]
              if (!count) return null
              return (
                <Badge key={severity} variant="outline" className={severityBadgeClass[severity]}>
                  {count} {severity}
                </Badge>
              )
            })}
            {SEVERITY_ORDER.every((s) => !kase.stats.alerts_by_severity[s]) ? (
              <span className="font-mono text-xs text-muted-foreground">no alerts yet</span>
            ) : null}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col">
      <span className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
        {label}
      </span>
      <span className="font-mono text-lg font-semibold tabular-nums">{value}</span>
    </div>
  )
}
