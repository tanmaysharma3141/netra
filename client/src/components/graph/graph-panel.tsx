import { useMemo, useState } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { Network, RefreshCw, Search, Waypoints } from "lucide-react"
import { getGraph, resolveCase } from "@/api/graph"
import { ForceGraph, ENTITY_COLORS, ENTITY_LABELS, type ForceGraphNode } from "@/components/graph/force-graph"
import { EntityProfilePanel } from "@/components/graph/entity-profile-panel"
import type { EntityType } from "@/api/types"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"

const HOP_LEVELS = [1, 2, 3] as const

export function GraphPanel({ caseId }: { caseId: string }) {
  const queryClient = useQueryClient()
  const [hops, setHops] = useState<(typeof HOP_LEVELS)[number]>(2)
  const [focusDraft, setFocusDraft] = useState("")
  const [focusEntityId, setFocusEntityId] = useState<string | undefined>(undefined)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<"simple" | "full">("full")

  const graphQuery = useQuery({
    queryKey: ["graph", caseId, hops, focusEntityId],
    queryFn: () => getGraph(caseId, { hops, entityId: focusEntityId }),
  })

  const resolveMutation = useMutation({
    mutationFn: () => resolveCase(caseId),
    onSuccess: (result) => {
      toast.success("Re-resolved", {
        description: `${result.entities} entities · ${result.edges} edges`,
      })
      void queryClient.invalidateQueries({ queryKey: ["graph", caseId] })
      void queryClient.invalidateQueries({ queryKey: ["case", caseId] })
    },
    onError: (err) => {
      toast.error("Resolution failed", { description: err instanceof Error ? err.message : undefined })
    },
  })

  const nodes = graphQuery.data?.nodes ?? []
  const allEdges = graphQuery.data?.edges ?? []
  const edges = viewMode === "simple"
    ? allEdges.filter((e) => e.tier === "high")
    : allEdges

  const degreeById = useMemo(() => {
    const map = new Map<string, number>()
    for (const edge of edges) {
      map.set(edge.source, (map.get(edge.source) ?? 0) + 1)
      map.set(edge.target, (map.get(edge.target) ?? 0) + 1)
    }
    return map
  }, [edges])

  const sortedNodes = useMemo<ForceGraphNode[]>(() => {
    // Render hottest nodes last so their labels sit on top.
    return [...nodes].sort((a, b) => (degreeById.get(a.id) ?? 0) - (degreeById.get(b.id) ?? 0))
  }, [nodes, degreeById])

  const nodeTypes = useMemo(() => [...new Set(nodes.map((n) => n.type))], [nodes])

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-border mb-3 flex flex-wrap items-center gap-x-4 gap-y-2 border-b pb-3">
        <div className="flex items-center gap-1" role="group" aria-label="Hop depth">
          <Network className="size-3.5 text-muted-foreground" aria-hidden />
          {HOP_LEVELS.map((level) => (
            <Button
              key={level}
              size="sm"
              variant={hops === level ? "secondary" : "ghost"}
              onClick={() => setHops(level)}
              className="h-7 px-2 font-mono text-xs"
            >
              {level}-hop{level > 1 ? "s" : ""}
            </Button>
          ))}
        </div>
        <form
          onSubmit={(e) => {
            e.preventDefault()
            setFocusEntityId(focusDraft.trim() || undefined)
            setSelectedId(null)
          }}
          className="flex items-center gap-2"
        >
          <Input
            value={focusDraft}
            onChange={(e) => setFocusDraft(e.target.value)}
            placeholder="Focus entity ID (optional)"
            className="h-8 w-64 font-mono text-xs"
            aria-label="Focus entity ID"
          />
          <Button type="submit" size="sm" variant="outline" className="h-8">
            <Search className="mr-1 size-3.5" aria-hidden />
            Focus
          </Button>
        </form>
        <div className="flex items-center gap-1" role="group" aria-label="View mode">
          {(["simple", "full"] as const).map((mode) => (
            <Button
              key={mode}
              size="sm"
              variant={viewMode === mode ? "secondary" : "ghost"}
              onClick={() => setViewMode(mode)}
              className="h-7 px-2 font-mono text-xs"
            >
              {mode === "simple" ? "Simple" : "Full"}
            </Button>
          ))}
        </div>
        <Button
          size="sm"
          variant="ghost"
          className="ml-auto h-7 font-mono text-xs"
          onClick={() => resolveMutation.mutate()}
          disabled={resolveMutation.isPending}
        >
          <RefreshCw className={`mr-1.5 size-3 ${resolveMutation.isPending ? "animate-spin" : ""}`} aria-hidden />
          Re-resolve
        </Button>
      </div>

      <div className="mb-2 flex flex-wrap items-center gap-3">
        <span className="font-mono text-[11px] tabular-nums text-muted-foreground">
          {nodes.length.toLocaleString("en-IN")} nodes · {edges.length.toLocaleString("en-IN")} edges
          {focusEntityId ? " · focused subgraph" : ""}
        </span>
        <span className="ml-auto flex items-center gap-3">
          {nodeTypes.map((type) => (
            <LegendItem key={type} type={type} />
          ))}
          <span className="text-muted-foreground flex items-center gap-1.5 font-mono text-[10px]">
            <svg width="18" height="6" aria-hidden>
              <line x1="0" y1="3" x2="18" y2="3" stroke="var(--color-border)" strokeWidth="2" strokeDasharray="4 3" />
            </svg>
            non-high tier
          </span>
        </span>
      </div>

      {graphQuery.isPending ? (
        <div className="bg-muted/30 flex flex-1 items-center justify-center rounded-sm border border-dashed">
          <Waypoints className="size-6 animate-pulse text-muted-foreground" aria-hidden />
        </div>
      ) : graphQuery.isError ? (
        <Alert variant="destructive">
          <AlertTitle>Failed to load graph</AlertTitle>
          <AlertDescription>
            {(graphQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button variant="outline" size="sm" className="mt-2" onClick={() => void graphQuery.refetch()}>
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      ) : nodes.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center rounded-sm border border-dashed py-16 text-center">
          <p className="text-sm font-medium">No resolved entities yet</p>
          <p className="text-muted-foreground mt-1 max-w-sm text-sm">
            Ingest data first — correlation runs automatically. Use “Re-resolve” to force a fresh pass.
          </p>
        </div>
      ) : (
        <div className="border-border relative min-h-0 flex-1 overflow-hidden rounded-sm border">
          <div className="absolute inset-0">
            <ForceGraph
              nodes={sortedNodes}
              edges={edges}
              selectedId={selectedId}
              onNodeClick={(id) => setSelectedId(id === selectedId ? null : id)}
            />
          </div>
          <aside className="border-border bg-card absolute top-3 right-3 bottom-3 w-80 overflow-hidden rounded-sm border shadow-lg">
            {selectedId ? (
              <div className="flex h-full flex-col">
                <div className="flex justify-end p-1 pb-0">
                  <Button variant="ghost" size="sm" className="h-6 font-mono text-[10px]" onClick={() => setSelectedId(null)}>
                    close
                  </Button>
                </div>
                <div className="min-h-0 flex-1">
                  <EntityProfilePanel entityId={selectedId} onSelectEntity={(id) => setSelectedId(id)} />
                </div>
              </div>
            ) : (
              <div className="flex h-full flex-col items-center justify-center gap-1 p-4 text-center">
                <Network className="size-5 text-muted-foreground" aria-hidden />
                <p className="text-muted-foreground text-xs">Click a node to inspect its profile and connections.</p>
              </div>
            )}
          </aside>
        </div>
      )}
    </div>
  )
}

function LegendItem({ type }: { type: EntityType }) {
  return (
    <span className="flex items-center gap-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
      <span className="inline-block size-2.5 rounded-full" style={{ background: ENTITY_COLORS[type] }} aria-hidden />
      {ENTITY_LABELS[type]}
    </span>
  )
}
