import { useQuery } from "@tanstack/react-query"
import { getEntityProfile } from "@/api/graph"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

export function EntityProfilePanel({
  entityId,
  onSelectEntity,
}: {
  entityId: string
  onSelectEntity: (id: string) => void
}) {
  const profileQuery = useQuery({
    queryKey: ["entity-profile", entityId],
    queryFn: () => getEntityProfile(entityId),
  })

  if (profileQuery.isPending) {
    return (
      <div className="space-y-3 p-4">
        <Skeleton className="h-5 w-2/3" />
        <Skeleton className="h-4 w-1/3" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-40 w-full" />
      </div>
    )
  }

  if (profileQuery.isError) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        Failed to load profile.{" "}
        <Button variant="ghost" size="sm" onClick={() => void profileQuery.refetch()}>
          Retry
        </Button>
      </div>
    )
  }

  const { entity, stats, connections } = profileQuery.data
  const sorted = [...connections].sort((a, b) => b.evidence_count - a.evidence_count)

  return (
    <div className="flex h-full flex-col">
      <div className="border-border border-b p-4">
        <p className="font-mono text-sm font-semibold break-all">{entity.display_name ?? entity.identifier}</p>
        {entity.display_name ? (
          <p className="text-muted-foreground mt-0.5 font-mono text-xs break-all">{entity.identifier}</p>
        ) : null}
        <div className="mt-2 flex flex-wrap items-center gap-1.5">
          <Badge variant="outline" className="font-mono text-[10px] tracking-wider uppercase">
            {entity.type}
          </Badge>
          {entity.tags.map((tag) => (
            <Badge key={tag} variant="secondary" className="text-[10px]">
              {tag}
            </Badge>
          ))}
        </div>
        <dl className="mt-3 grid grid-cols-3 gap-2 font-mono text-xs tabular-nums">
          <div>
            <dt className="text-muted-foreground text-[9px] tracking-wider uppercase">Events</dt>
            <dd>{stats.events.toLocaleString("en-IN")}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground text-[9px] tracking-wider uppercase">First</dt>
            <dd>{fmtDate(stats.first_seen)}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground text-[9px] tracking-wider uppercase">Last</dt>
            <dd>{fmtDate(stats.last_seen)}</dd>
          </div>
        </dl>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto p-3">
        <p className="mb-2 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
          Connections ({sorted.length})
        </p>
        <ul className="space-y-1.5">
          {sorted.map((conn) => (
            <li key={`${conn.other_entity_id}-${conn.link_type}`}>
              <button
                onClick={() => onSelectEntity(conn.other_entity_id)}
                className="hover:bg-accent/60 w-full rounded-sm px-2 py-1.5 text-left"
              >
                <span className="block truncate font-mono text-xs">{conn.other_identifier}</span>
                <span className="text-muted-foreground mt-0.5 flex items-center gap-2 font-mono text-[10px]">
                  <span className="uppercase">{conn.link_type.replace(/_/g, " ")}</span>
                  <span>·</span>
                  <span className="uppercase">{conn.tier}</span>
                  <span>·</span>
                  <span className="tabular-nums">{conn.evidence_count} events</span>
                </span>
              </button>
            </li>
          ))}
          {sorted.length === 0 ? (
            <li className="text-muted-foreground text-xs">No resolved connections.</li>
          ) : null}
        </ul>
      </div>
    </div>
  )
}

function fmtDate(iso: string | null): string {
  if (!iso) return "—"
  return new Date(iso).toLocaleDateString("en-IN", { day: "2-digit", month: "short" })
}
