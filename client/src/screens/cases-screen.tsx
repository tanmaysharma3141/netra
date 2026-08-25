import { useMemo, useState, type FormEvent } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useNavigate } from "react-router-dom"
import { toast } from "sonner"
import { FolderPlus, Plus, RefreshCw, Search } from "lucide-react"
import { createCase, listCases } from "@/api/cases"
import { ApiClientError } from "@/api/client"
import type { Case, CaseStatus } from "@/api/types"
import { severityBadgeClass } from "@/lib/severity"
import { useAuth } from "@/auth/AuthContext"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Textarea } from "@/components/ui/textarea"

const STATUS_FILTERS: readonly ("all" | CaseStatus)[] = ["all", "active", "archived", "closed"]

export function CasesScreen() {
  const { can } = useAuth()
  const navigate = useNavigate()
  const casesQuery = useQuery({ queryKey: ["cases"], queryFn: listCases })
  const [search, setSearch] = useState("")
  const [statusFilter, setStatusFilter] = useState<(typeof STATUS_FILTERS)[number]>("all")
  const [createOpen, setCreateOpen] = useState(false)

  const filtered = useMemo(() => filterCases(casesQuery.data ?? [], search, statusFilter), [
    casesQuery.data,
    search,
    statusFilter,
  ])

  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold">Cases</h1>
          <p className="text-sm text-muted-foreground">
            Investigations visible to your role — scoped server-side.
          </p>
        </div>
        {can("case.create") ? (
          <Button onClick={() => setCreateOpen(true)}>
            <Plus className="mr-1.5 size-4" aria-hidden />
            New case
          </Button>
        ) : null}
      </header>

      <div className="mb-4 flex flex-wrap items-center gap-3">
        <div className="relative min-w-64 flex-1">
          <Search
            className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search title, tag, or case ID…"
            className="pl-8"
            aria-label="Search cases"
          />
        </div>
        <div className="flex items-center gap-1" role="group" aria-label="Filter by status">
          {STATUS_FILTERS.map((status) => (
            <Button
              key={status}
              size="sm"
              variant={statusFilter === status ? "secondary" : "ghost"}
              onClick={() => setStatusFilter(status)}
              className="font-mono text-xs uppercase"
            >
              {status}
            </Button>
          ))}
        </div>
      </div>

      {casesQuery.isPending ? (
        <div className="space-y-2">
          {[0, 1, 2, 3, 4].map((i) => (
            <Skeleton key={i} className="h-12 w-full" />
          ))}
        </div>
      ) : casesQuery.isError ? (
        <Alert variant="destructive">
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
      ) : filtered.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-14 text-center">
            <FolderPlus className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">
              {(casesQuery.data?.length ?? 0) === 0 ? "No cases yet" : "No cases match"}
            </p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              {(casesQuery.data?.length ?? 0) === 0
                ? can("case.create")
                  ? "Create your first case to start ingesting and correlating data."
                  : "No cases have been assigned to you yet. Ask an investigator to add you."
                : "Try a different search term or clear the status filter."}
            </p>
          </CardContent>
        </Card>
      ) : (
        <Card className="py-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Title</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Classification</TableHead>
                <TableHead className="text-right">Events</TableHead>
                <TableHead className="text-right">Entities</TableHead>
                <TableHead>Alerts</TableHead>
                <TableHead>Created</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filtered.map((kase) => (
                <TableRow
                  key={kase.id}
                  onClick={() => void navigate(`/cases/${kase.id}`)}
                  className="cursor-pointer"
                >
                  <TableCell>
                    <div className="flex flex-col">
                      <span className="font-medium">{kase.title}</span>
                      <span className="font-mono text-[10px] text-muted-foreground">{kase.id}</span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" className="font-mono text-[10px] tracking-wider uppercase">
                      {kase.status}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <span className="font-mono text-xs tracking-wider">{kase.classification}</span>
                  </TableCell>
                  <TableCell className="text-right font-mono text-xs tabular-nums">
                    {Object.values(kase.stats.events_by_source)
                      .reduce((a, b) => a + b, 0)
                      .toLocaleString("en-IN")}
                  </TableCell>
                  <TableCell className="text-right font-mono text-xs tabular-nums">
                    {kase.stats.entity_count.toLocaleString("en-IN")}
                  </TableCell>
                  <TableCell>
                    <div className="flex gap-1.5">
                      {(["critical", "high"] as const).map((severity) => {
                        const count = kase.stats.alerts_by_severity[severity]
                        if (!count) return null
                        return (
                          <Badge key={severity} variant="outline" className={severityBadgeClass[severity]}>
                            {count} {severity}
                          </Badge>
                        )
                      })}
                      {!(kase.stats.alerts_by_severity.critical ?? 0) &&
                      !(kase.stats.alerts_by_severity.high ?? 0) ? (
                        <span className="font-mono text-xs text-muted-foreground">—</span>
                      ) : null}
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-xs whitespace-nowrap text-muted-foreground">
                    {new Date(kase.created_at).toLocaleDateString("en-IN", {
                      day: "2-digit",
                      month: "short",
                      year: "numeric",
                    })}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}

      <CreateCaseDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  )
}

function filterCases(cases: Case[], search: string, status: (typeof STATUS_FILTERS)[number]): Case[] {
  const needle = search.trim().toLowerCase()
  return cases.filter((kase) => {
    if (status !== "all" && kase.status !== status) return false
    if (!needle) return true
    return (
      kase.title.toLowerCase().includes(needle) ||
      kase.id.toLowerCase().includes(needle) ||
      kase.tags.some((tag) => tag.toLowerCase().includes(needle))
    )
  })
}

function CreateCaseDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()
  const [title, setTitle] = useState("")
  const [classification, setClassification] = useState("")
  const [tags, setTags] = useState("")

  const mutation = useMutation({
    mutationFn: () =>
      createCase({
        title: title.trim(),
        classification: classification.trim() || undefined,
        tags: tags
          .split(",")
          .map((t) => t.trim().toLowerCase())
          .filter(Boolean),
      }),
    onSuccess: (created) => {
      toast.success("Case created", { description: created.title })
      void queryClient.invalidateQueries({ queryKey: ["cases"] })
      onOpenChange(false)
      setTitle("")
      setClassification("")
      setTags("")
    },
    onError: (err) => {
      toast.error("Could not create case", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!title.trim() || mutation.isPending) return
    mutation.mutate()
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New case</DialogTitle>
          <DialogDescription>
            Open a fresh investigation. You can ingest data once the case exists.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="case-title">Title</Label>
            <Input
              id="case-title"
              required
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="OP-2026-042: …"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="case-classification">Classification</Label>
            <Input
              id="case-classification"
              value={classification}
              onChange={(e) => setClassification(e.target.value)}
              placeholder="UNCLASSIFIED / RESTRICTED / CONFIDENTIAL"
              className="font-mono"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="case-tags">Tags</Label>
            <Textarea
              id="case-tags"
              value={tags}
              onChange={(e) => setTags(e.target.value)}
              placeholder="hawala, financial-fraud (comma-separated)"
              rows={2}
            />
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="ghost"
              onClick={() => onOpenChange(false)}
              disabled={mutation.isPending}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!title.trim() || mutation.isPending}>
              {mutation.isPending ? "Creating…" : "Create case"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
