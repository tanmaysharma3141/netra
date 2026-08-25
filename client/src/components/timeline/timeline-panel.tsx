import { useEffect, useMemo, useRef, useState, type FormEvent } from "react"
import { useInfiniteQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useVirtualizer } from "@tanstack/react-virtual"
import { toast } from "sonner"
import { ChevronDown, ChevronRight, ListTree, MapPin, RefreshCw, Search } from "lucide-react"
import { listEvents, addEventNote, type EventQuery } from "@/api/events"
import type { Event, EventType, SourceType } from "@/api/types"
import { SOURCE_LABELS, sourceBadgeClass } from "@/lib/severity"
import { EVENT_TYPES, SOURCE_TYPES } from "@/lib/timeline-constants"
import { useAuth } from "@/auth/AuthContext"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"

const PAGE_LIMIT = 200
const FETCH_THRESHOLD_ROWS = 10

const GROUP_WINDOWS = [
  { label: "5m", ms: 5 * 60_000 },
  { label: "15m", ms: 15 * 60_000 },
  { label: "1h", ms: 3_600_000 },
  { label: "24h", ms: 86_400_000 },
] as const

type GroupingMs = (typeof GROUP_WINDOWS)[number]["ms"]
type Grouping = "off" | GroupingMs

interface TimelineFilters {
  sourceType: "all" | SourceType
  eventType: "all" | EventType
  from: string
  to: string
  entityId: string
}

const EMPTY_FILTERS: TimelineFilters = {
  sourceType: "all",
  eventType: "all",
  from: "",
  to: "",
  entityId: "",
}

function filtersToQuery(filters: TimelineFilters): EventQuery {
  const query: EventQuery = {}
  if (filters.sourceType !== "all") query.source_type = filters.sourceType
  if (filters.eventType !== "all") query.event_type = filters.eventType
  const entityId = filters.entityId.trim()
  if (entityId) query.entity_id = entityId
  if (filters.from) query.from = new Date(filters.from).toISOString()
  if (filters.to) query.to = new Date(filters.to).toISOString()
  return query
}

interface EventRowData {
  kind: "event"
  key: string
  event: Event
}

interface GroupRowData {
  kind: "group"
  key: string
  head: Event
  cluster: Event[]
}

type Row = EventRowData | GroupRowData

export function TimelinePanel({ caseId }: { caseId: string }) {
  const [applied, setApplied] = useState<TimelineFilters>(EMPTY_FILTERS)
  const [draft, setDraft] = useState<TimelineFilters>(EMPTY_FILTERS)
  const [grouping, setGrouping] = useState<Grouping>("off")
  const [collapsedGroups, setCollapsedGroups] = useState<ReadonlySet<string>>(new Set())
  const [selected, setSelected] = useState<Event | null>(null)

  const eventsQuery = useInfiniteQuery({
    queryKey: useMemo(() => ["events", caseId, applied], [caseId, applied]),
    queryFn: ({ pageParam }) =>
      listEvents(caseId, { ...filtersToQuery(applied), limit: PAGE_LIMIT, offset: pageParam }),
    initialPageParam: 0,
    getNextPageParam: (lastPage, pages) =>
      lastPage.length === PAGE_LIMIT ? pages.length * PAGE_LIMIT : undefined,
  })

  const events = useMemo(() => eventsQuery.data?.pages.flat() ?? [], [eventsQuery.data])

  const rows = useMemo<Row[]>(() => {
    const eventRows: EventRowData[] = events.map((event) => ({ kind: "event", key: event.id, event }))
    if (grouping === "off") return eventRows

    const out: Row[] = []
    let i = 0
    while (i < eventRows.length) {
      const head = eventRows[i]
      const cluster: Event[] = [head.event]
      let j = i + 1
      while (
        j < eventRows.length &&
        new Date(eventRows[j - 1].event.timestamp).getTime() -
              new Date(eventRows[j].event.timestamp).getTime() <=
          grouping
      ) {
        cluster.push(eventRows[j].event)
        j++
      }
      const key = `grp-${head.key}`
      out.push({ kind: "group", key, head: head.event, cluster })
      if (!collapsedGroups.has(key)) {
        for (let k = i; k < j; k++) out.push(eventRows[k])
      }
      i = j
    }
    return out
  }, [events, grouping, collapsedGroups])

  const scrollRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 52,
    overscan: 10,
  })

  // Infinite scroll: fetch the next page as the viewport nears the rendered end.
  useEffect(() => {
    if (!eventsQuery.hasNextPage || eventsQuery.isFetchingNextPage) return
    const virtualItems = virtualizer.getVirtualItems()
    const last = virtualItems[virtualItems.length - 1]
    if (last && last.index >= rows.length - FETCH_THRESHOLD_ROWS) void eventsQuery.fetchNextPage()
  }, [virtualizer, rows.length, eventsQuery])

  function applyFilters(formEvent: FormEvent<HTMLFormElement>) {
    formEvent.preventDefault()
    setApplied(draft)
    setCollapsedGroups(new Set())
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <form
        onSubmit={(e) => void applyFilters(e)}
        className="border-border mb-3 flex flex-wrap items-end gap-x-4 gap-y-2 border-b pb-3"
      >
        <div className="space-y-1">
          <Label
            htmlFor="tl-source"
            className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
          >
            Source
          </Label>
          <select
            id="tl-source"
            value={draft.sourceType}
            onChange={(e) =>
              setDraft({ ...draft, sourceType: e.target.value as TimelineFilters["sourceType"] })
            }
            className="border-input bg-background h-8 rounded-sm border px-2 font-mono text-xs"
          >
            <option value="all">ALL</option>
            {SOURCE_TYPES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>
        <div className="space-y-1">
          <Label
            htmlFor="tl-type"
            className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
          >
            Event
          </Label>
          <select
            id="tl-type"
            value={draft.eventType}
            onChange={(e) =>
              setDraft({ ...draft, eventType: e.target.value as TimelineFilters["eventType"] })
            }
            className="border-input bg-background h-8 rounded-sm border px-2 font-mono text-xs"
          >
            <option value="all">ALL</option>
            {EVENT_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div className="space-y-1">
          <Label
            htmlFor="tl-from"
            className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
          >
            From
          </Label>
          <Input
            id="tl-from"
            type="datetime-local"
            value={draft.from}
            onChange={(e) => setDraft({ ...draft, from: e.target.value })}
            className="h-8 w-56 font-mono text-xs"
          />
        </div>
        <div className="space-y-1">
          <Label
            htmlFor="tl-to"
            className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
          >
            To
          </Label>
          <Input
            id="tl-to"
            type="datetime-local"
            value={draft.to}
            onChange={(e) => setDraft({ ...draft, to: e.target.value })}
            className="h-8 w-56 font-mono text-xs"
          />
        </div>
        <div className="min-w-48 flex-1 space-y-1">
          <Label
            htmlFor="tl-entity"
            className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
          >
            Entity ID
          </Label>
          <Input
            id="tl-entity"
            value={draft.entityId}
            onChange={(e) => setDraft({ ...draft, entityId: e.target.value })}
            placeholder="+9198… / IMEI / account / handle"
            className="h-8 font-mono text-xs"
          />
        </div>
        <Button type="submit" size="sm" className="h-8">
          <Search className="mr-1 size-3.5" aria-hidden />
          Apply
        </Button>
        <div className="ml-auto flex items-center gap-1" role="group" aria-label="Cluster window">
          <ListTree className="size-3.5 text-muted-foreground" aria-hidden />
          {(["off", ...GROUP_WINDOWS.map((w) => w.ms)] as Grouping[]).map((g) => (
            <Button
              key={String(g)}
              type="button"
              size="sm"
              variant={grouping === g ? "secondary" : "ghost"}
              onClick={() => {
                setGrouping(g)
                setCollapsedGroups(new Set())
              }}
              className="h-7 px-2 font-mono text-xs"
            >
              {g === "off" ? "flat" : GROUP_WINDOWS.find((w) => w.ms === g)?.label}
            </Button>
          ))}
        </div>
      </form>

      {eventsQuery.isPending ? (
        <div className="space-y-2" aria-label="Loading events">
          {[0, 1, 2, 3, 4, 5, 6].map((i) => (
            <div key={i} className="bg-muted/40 h-11 animate-pulse rounded-sm" />
          ))}
        </div>
      ) : eventsQuery.isError ? (
        <Alert variant="destructive">
          <AlertTitle>Failed to load events</AlertTitle>
          <AlertDescription>
            {(eventsQuery.error as { message?: string }).message ?? "Unknown error."}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => void eventsQuery.refetch()}
            >
              <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      ) : events.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-sm border border-dashed py-16 text-center">
          <p className="text-sm font-medium">No events</p>
          <p className="text-muted-foreground mt-1 max-w-sm text-sm">
            Ingest data into this case, or loosen the filters above.
          </p>
        </div>
      ) : (
        <>
          <div className="text-muted-foreground mb-2 font-mono text-[11px] tabular-nums">
            {events.length.toLocaleString("en-IN")} events loaded
            {eventsQuery.isFetchingNextPage ? " · fetching more…" : ""}
            {eventsQuery.hasNextPage ? " · scroll for more" : ""}
          </div>
          <div ref={scrollRef} className="min-h-0 flex-1 overflow-auto rounded-sm border">
            <div style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}>
              {virtualizer.getVirtualItems().map((vItem) => {
                const row = rows[vItem.index]
                if (!row) return null
                const style = {
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  transform: `translateY(${vItem.start}px)`,
                } as const
                if (row.kind === "group") {
                  const collapsed = collapsedGroups.has(row.key)
                  return (
                    <button
                      key={row.key}
                      ref={virtualizer.measureElement}
                      data-index={vItem.index}
                      style={style}
                      onClick={() =>
                        setCollapsedGroups((prev) => {
                          const next = new Set(prev)
                          if (next.has(row.key)) next.delete(row.key)
                          else next.add(row.key)
                          return next
                        })
                      }
                      className="bg-secondary/50 hover:bg-secondary flex w-full items-center gap-2 border-b border-dashed px-3 py-1.5 text-left"
                    >
                      {collapsed ? (
                        <ChevronRight className="size-3.5 shrink-0" aria-hidden />
                      ) : (
                        <ChevronDown className="size-3.5 shrink-0" aria-hidden />
                      )}
                      <span className="font-mono text-xs">{fmtTime(row.head.timestamp)}</span>
                      <span className="text-muted-foreground font-mono text-[10px] tracking-wider uppercase">
                        cluster of {row.cluster.length} within {grouping === "off" ? "" : labelFor(grouping)}
                      </span>
                      <span className="ml-auto flex gap-1">
                        {[...new Set(row.cluster.map((e) => e.source_type))].map((s) => (
                          <Badge key={s} variant="outline" className={`text-[9px] ${sourceBadgeClass[s]}`}>
                            {s}
                          </Badge>
                        ))}
                      </span>
                    </button>
                  )
                }
                const event = row.event
                return (
                  <button
                    key={row.key}
                    ref={virtualizer.measureElement}
                    data-index={vItem.index}
                    style={style}
                    onClick={() => setSelected(event)}
                    className="hover:bg-accent/50 flex w-full items-center gap-3 border-b px-3 py-2 text-left"
                  >
                    <span className="w-36 shrink-0 font-mono text-xs whitespace-nowrap text-muted-foreground">
                      {fmtTime(event.timestamp)}
                    </span>
                    <Badge
                      variant="outline"
                      className={`w-24 shrink-0 justify-center text-[9px] ${sourceBadgeClass[event.source_type]}`}
                    >
                      {SOURCE_LABELS[event.source_type]}
                    </Badge>
                    <span className="w-14 shrink-0 font-mono text-[11px] tracking-wider">
                      {event.event_type}
                    </span>
                    <span className="min-w-0 flex-1 truncate font-mono text-xs">{event.entity_id}</span>
                    {event.value !== null ? (
                      <span className="shrink-0 text-right font-mono text-xs tabular-nums">
                        {event.value.toLocaleString("en-IN")}
                      </span>
                    ) : null}
                    {event.location ? (
                      <MapPin className="size-3.5 shrink-0 text-chart-1" aria-label="Has location" />
                    ) : null}
                  </button>
                )
              })}
            </div>
          </div>
        </>
      )}

      <EventDrawer
        event={selected}
        onClose={() => setSelected(null)}
        onEventUpdated={(updated) => setSelected(updated)}
      />
    </div>
  )
}

function labelFor(ms: GroupingMs): string {
  return GROUP_WINDOWS.find((w) => w.ms === ms)?.label ?? String(ms)
}

function fmtTime(iso: string): string {
  return new Date(iso).toLocaleString("en-IN", {
    day: "2-digit",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  })
}

function EventDrawer({
  event,
  onClose,
  onEventUpdated,
}: {
  event: Event | null
  onClose: () => void
  onEventUpdated: (updated: Event) => void
}) {
  const queryClient = useQueryClient()
  const { can } = useAuth()
  const [note, setNote] = useState("")

  const noteMutation = useMutation({
    mutationFn: () => addEventNote(event!.id, note.trim()),
    onSuccess: (updated) => {
      toast.success("Note added")
      setNote("")
      onEventUpdated(updated)
      void queryClient.invalidateQueries({ queryKey: ["events"] })
    },
    onError: (err) => {
      toast.error("Could not add note", {
        description: err instanceof Error ? err.message : "Unexpected error.",
      })
    },
  })

  const canAnnotate = can("data.upload") // admin + investigator — server enforces the real check

  function submitNote(formEvent: FormEvent<HTMLFormElement>) {
    formEvent.preventDefault()
    if (!event || !note.trim() || noteMutation.isPending) return
    noteMutation.mutate()
  }

  return (
    <Sheet open={event !== null} onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="w-full overflow-y-auto sm:max-w-lg">
        {event ? (
          <>
            <SheetHeader>
              <SheetTitle className="font-mono text-sm">
                {event.event_type} · {event.entity_type}
              </SheetTitle>
              <SheetDescription className="font-mono text-xs">{event.id}</SheetDescription>
            </SheetHeader>
            <dl className="mt-2 space-y-2">
              <Field
                label="Timestamp"
                value={`${fmtTime(event.timestamp)} (${event.timestamp})`}
                mono
              />
              <Field label="Entity" value={`${event.entity_type} — ${event.entity_id}`} mono />
              <Field label="Source" value={SOURCE_LABELS[event.source_type]} />
              {event.value !== null ? (
                <Field label="Value" value={event.value.toLocaleString("en-IN")} mono />
              ) : null}
              {event.location ? (
                <Field
                  label="Location"
                  value={`${event.location.lat.toFixed(5)}, ${event.location.lng.toFixed(5)}`}
                  mono
                />
              ) : null}
            </dl>

            <div className="mt-4">
              <p className="mb-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                Notes
              </p>
              {event.notes.length > 0 ? (
                <ul className="mb-3 space-y-1.5">
                  {event.notes.map((noteItem, i) => (
                    <li key={i} className="border-border rounded-sm border-l-2 pl-2.5 text-sm">
                      {noteItem}
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="text-muted-foreground mb-3 text-xs">No notes yet.</p>
              )}
              {canAnnotate ? (
                <form onSubmit={(e) => void submitNote(e)} className="space-y-2">
                  <Textarea
                    value={note}
                    onChange={(e) => setNote(e.target.value)}
                    placeholder="Add an investigator note…"
                    rows={2}
                    aria-label="New note"
                  />
                  <Button
                    type="submit"
                    size="sm"
                    variant="secondary"
                    disabled={!note.trim() || noteMutation.isPending}
                  >
                    {noteMutation.isPending ? "Saving…" : "Add note"}
                  </Button>
                </form>
              ) : null}
            </div>

            <div className="mt-4">
              <p className="mb-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                Raw record (verbatim)
              </p>
              <pre className="border-border bg-card max-h-80 overflow-auto rounded-sm border p-3 font-mono text-[11px] leading-relaxed">
                {JSON.stringify(event.raw, null, 2)}
              </pre>
            </div>
          </>
        ) : null}
      </SheetContent>
    </Sheet>
  )
}

function Field({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex gap-3">
      <dt className="text-muted-foreground w-24 shrink-0 text-xs">{label}</dt>
      <dd className={`min-w-0 break-all text-xs ${mono ? "font-mono" : ""}`}>{value}</dd>
    </div>
  )
}
