import { useEffect, useMemo, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { Map as MapIcon, Pause, Play, RefreshCw, Route } from "lucide-react"
import { getMovements } from "@/api/geo"
import { MovementMap } from "@/components/map/movement-map"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

const PLAYBACK_INTERVAL_MS = 400

export function MapPanel({ caseId }: { caseId: string }) {
  const movementsQuery = useQuery({
    queryKey: ["movements", caseId],
    queryFn: () => getMovements(caseId),
    staleTime: 60_000,
  })
  const [playing, setPlaying] = useState(false)
  const [fraction, setFraction] = useState(1)

  const trails = useMemo(
    () =>
      (movementsQuery.data?.trails ?? [])
        .map((trail) => ({
          entityId: trail.entity_id,
          label: trail.entity_id,
          points: [...trail.points].sort(
            (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
          ),
        }))
        .filter((trail) => trail.points.length > 0),
    [movementsQuery.data],
  )

  // Playback loop: sweep the visible fraction 0 → 1.
  useEffect(() => {
    if (!playing || trails.length === 0) return
    const timer = window.setInterval(() => {
      setFraction((prev) => {
        if (prev >= 1) {
          setPlaying(false)
          return 1
        }
        return Math.min(1, prev + 0.02)
      })
    }, PLAYBACK_INTERVAL_MS)
    return () => window.clearInterval(timer)
  }, [playing, trails.length])

  function restartPlayback() {
    setFraction(0)
    setPlaying(true)
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-border mb-3 flex flex-wrap items-center gap-x-4 gap-y-2 border-b pb-3">
        <span className="text-muted-foreground font-mono text-[11px] tabular-nums">
          {trails.length} trail{trails.length === 1 ? "" : "s"} ·{" "}
          {trails.reduce((sum, t) => sum + t.points.length, 0).toLocaleString("en-IN")} pings
        </span>
        <div className="ml-auto flex items-center gap-2">
          {trails.length > 0 ? (
            <>
              <Button
                size="sm"
                variant={playing ? "secondary" : "outline"}
                className="h-7 font-mono text-xs"
                onClick={() => (fraction >= 1 ? restartPlayback() : setPlaying(!playing))}
              >
                {playing ? (
                  <Pause className="mr-1 size-3" aria-hidden />
                ) : (
                  <Play className="mr-1 size-3" aria-hidden />
                )}
                {playing ? "Pause" : fraction < 1 ? "Resume" : "Replay"}
              </Button>
              <input
                type="range"
                min={0}
                max={100}
                value={Math.round(fraction * 100)}
                onChange={(e) => {
                  setPlaying(false)
                  setFraction(Number(e.target.value) / 100)
                }}
                aria-label="Playback position"
                className="w-44 accent-cyan-400"
              />
            </>
          ) : null}
          <Button
            size="sm"
            variant="ghost"
            className="h-7 font-mono text-xs"
            onClick={() => void movementsQuery.refetch()}
            disabled={movementsQuery.isFetching}
          >
            <RefreshCw
              className={`mr-1.5 size-3 ${movementsQuery.isFetching ? "animate-spin" : ""}`}
              aria-hidden
            />
            Refresh
          </Button>
        </div>
      </div>

      {movementsQuery.isPending ? (
        <div className="bg-muted/30 flex flex-1 items-center justify-center rounded-sm border border-dashed">
          <Route className="size-6 animate-pulse text-muted-foreground" aria-hidden />
        </div>
      ) : movementsQuery.isError ? (
        <Alert variant="destructive">
          <AlertTitle>Failed to load movement trails</AlertTitle>
          <AlertDescription>
            {(movementsQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button variant="outline" size="sm" className="mt-2" onClick={() => void movementsQuery.refetch()}>
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      ) : trails.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center rounded-sm border border-dashed py-16 text-center">
          <MapIcon className="mb-3 size-8 text-muted-foreground" aria-hidden />
          <p className="text-sm font-medium">No movement data</p>
          <p className="text-muted-foreground mt-1 max-w-sm text-sm">
            Trails appear once ingested events carry cell-tower locations for this case.
          </p>
        </div>
      ) : (
        <div className="border-border relative min-h-0 flex-1 overflow-hidden rounded-sm border">
          <MovementMap trails={trails} playbackFraction={fraction} />
        </div>
      )}
    </div>
  )
}
