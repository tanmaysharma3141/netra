import { useEffect, useMemo, useRef } from "react"
import { useInfiniteQuery } from "@tanstack/react-query"
import { useVirtualizer } from "@tanstack/react-virtual"
import { listEvents } from "@/api/events"
import type { Event, EventType, SourceType } from "@/api/types"
import { SOURCE_LABELS, sourceBadgeClass } from "@/lib/severity"

const PAGE_LIMIT = 200
const FETCH_THRESHOLD_ROWS = 8

export interface CompareFilters {
  sourceType: "all" | SourceType
  eventType: "all" | EventType
  from: string
  to: string
}

function filtersToQuery(filters: CompareFilters, entityId: string | undefined) {
  const query: Parameters<typeof listEvents>[1] = { limit: PAGE_LIMIT }
  if (filters.sourceType !== "all") query.source_type = filters.sourceType
  if (filters.eventType !== "all") query.event_type = filters.eventType
  if (filters.from) query.from = new Date(filters.from).toISOString()
  if (filters.to) query.to = new Date(filters.to).toISOString()
  if (entityId) query.entity_id = entityId
  return query
}

function fmtCompact(iso: string): string {
  return new Date(iso).toLocaleString("en-IN", {
    day: "2-digit",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  })
}

export function ComparePane({
  caseId,
  side,
  entities,
  entityId,
  onEntityChange,
  filters,
}: {
  caseId: string
  side: "A" | "B"
  entities: { id: string; label: string }[]
  entityId: string | undefined
  onEntityChange: (id: string | undefined) => void
  filters: CompareFilters
}) {
  const eventsQuery = useInfiniteQuery({
    queryKey: ["events", caseId, "compare", side, entityId, filters],
    queryFn: ({ pageParam }) =>
      listEvents(caseId, { ...filtersToQuery(filters, entityId), offset: pageParam }),
    initialPageParam: 0,
    getNextPageParam: (lastPage, pages) =>
      lastPage.length === PAGE_LIMIT ? pages.length * PAGE_LIMIT : undefined,
  })
  const events = useMemo(() => eventsQuery.data?.pages.flat() ?? [], [eventsQuery.data])

  const scrollRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: events.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 40,
    overscan: 8,
  })

  useEffect(() => {
    if (!eventsQuery.hasNextPage || eventsQuery.isFetchingNextPage) return
    const items = virtualizer.getVirtualItems()
    const last = items[items.length - 1]
    if (last && last.index >= events.length - FETCH_THRESHOLD_ROWS) void eventsQuery.fetchNextPage()
  }, [virtualizer, events.length, eventsQuery])

  const byType = useMemo(() => {
    const counts = new Map<EventType, number>()
    for (const e of events) counts.set(e.event_type, (counts.get(e.event_type) ?? 0) + 1)
    return [...counts.entries()].sort((a, b) => b[1] - a[1])
  }, [events])

  const selectedLabel =
    entityId === undefined ? "—" : (entities.find((e) => e.id === entityId)?.label ?? entityId)

  return (
    <section className="border-border flex min-h-0 flex-1 flex-col rounded-sm border">
      <header className="border-border space-y-2 border-b p-2.5">
        <div className="flex items-center gap-2">
          <span className="bg-primary text-primary-foreground rounded-sm px-1.5 py-0.5 font-mono text-[10px] font-semibold">
            {side}
          </span>
          <select
            value={entityId ?? ""}
            onChange={(e) => onEntityChange(e.target.value || undefined)}
            aria-label={`Suspect ${side} entity`}
            className="border-input bg-background h-7 min-w-0 flex-1 rounded-sm border px-1.5 font-mono text-xs"
          >
            <option value="">Select suspect…</option>
            {entities.map((entity) => (
              <option key={entity.id} value={entity.id}>
                {entity.label}
              </option>
            ))}
          </select>
        </div>
        <div className="text-muted-foreground flex flex-wrap items-center gap-x-3 font-mono text-[10px] tabular-nums">
          <span className="truncate">{selectedLabel}</span>
          <span>{events.length.toLocaleString("en-IN")} events</span>
          {byType.slice(0, 3).map(([type, count]) => (
            <span key={type}>
              · {type} {count}
            </span>
          ))}
        </div>
      </header>

      {entityId === undefined ? (
        <div className="text-muted-foreground flex flex-1 items-center justify-center p-4 text-center text-xs">
          Pick a suspect to load their timeline.
        </div>
      ) : eventsQuery.isPending ? (
        <div className="flex-1 space-y-1.5 p-2.5">
          {[0, 1, 2, 3].map((i) => (
            <div key={i} className="bg-muted/40 h-8 animate-pulse rounded-sm" />
          ))}
        </div>
      ) : events.length === 0 ? (
        <div className="text-muted-foreground flex flex-1 items-center justify-center p-4 text-center text-xs">
          No events for this suspect under current filters.
        </div>
      ) : (
        <div ref={scrollRef} className="min-h-0 flex-1 overflow-auto">
          <div style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}>
            {virtualizer.getVirtualItems().map((vItem) => {
              const event: Event = events[vItem.index]
              return (
                <div
                  key={event.id}
                  ref={virtualizer.measureElement}
                  data-index={vItem.index}
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    transform: `translateY(${vItem.start}px)`,
                  }}
                >
                  <div className="border-border flex items-center gap-2 border-b px-2.5 py-1.5">
                    <span className="w-24 shrink-0 font-mono text-[11px] whitespace-nowrap text-muted-foreground">
                      {fmtCompact(event.timestamp)}
                    </span>
                    <span
                      className={`inline-block w-14 shrink-0 rounded-sm border px-1 text-center font-mono text-[9px] leading-4 ${sourceBadgeClass[event.source_type]}`}
                    >
                      {SOURCE_LABELS[event.source_type]}
                    </span>
                    <span className="shrink-0 font-mono text-[10px] tracking-wider">
                      {event.event_type}
                    </span>
                    {event.value !== null ? (
                      <span className="ml-auto shrink-0 font-mono text-[10px] tabular-nums">
                        {event.value.toLocaleString("en-IN")}
                      </span>
                    ) : null}
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}
    </section>
  )
}
