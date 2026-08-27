import { useCallback, useEffect, useRef, useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { CheckCircle2, CloudUpload, FileWarning, Loader2, X } from "lucide-react"
import { getIngestJob, uploadFile } from "@/api/ingest"
import type { IngestJob } from "@/api/types"
import { wsClient } from "@/api/ws"
import { Button } from "@/components/ui/button"

interface TrackedJob {
  jobId: string
  fileName: string
  status: IngestJob["status"]
  parsed: number
  totalEst: number | null
  errors: string[]
}

const POLL_INTERVAL_MS = 1_500

function humanizeError(raw: string): string {
  if (raw.includes("column") && raw.includes("not found")) {
    const col = raw.match(/column\s+['"]?([^'"\s]+)['"]?/i)?.[1]
    return col ? `Column "${col}" not recognized — file may use a non-standard header name` : raw
  }
  if (raw.includes("failed to parse")) {
    const field = raw.match(/field\s+(\d+)/i)?.[1]
    return field ? `Row has too few columns (expected field ${field} is missing)` : raw
  }
  if (raw.includes("empty file")) return "File is empty — no data rows found"
  if (raw.includes("encoding")) return "File encoding not supported — save as UTF-8 CSV"
  return raw
}

export function IngestPanel({ caseId }: { caseId: string }) {
  const queryClient = useQueryClient()
  const [jobs, setJobs] = useState<TrackedJob[]>([])
  const [dragging, setDragging] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)
  const jobsRef = useRef<TrackedJob[]>([])
  jobsRef.current = jobs

  // Live progress via WS `ingest.progress` frames on this case's topic.
  useEffect(() => {
    wsClient.start()
    const unsubscribe = wsClient.subscribe([`case:${caseId}`], (envelope) => {
      if (envelope.event !== "ingest.progress") return
      const payload = envelope.payload as { job_id?: string; parsed?: number; total_est?: number }
      if (typeof payload?.job_id !== "string") return
      setJobs((prev) =>
        prev.map((job) =>
          job.jobId === payload.job_id
            ? {
                ...job,
                parsed: typeof payload.parsed === "number" ? payload.parsed : job.parsed,
                totalEst: typeof payload.total_est === "number" ? payload.total_est : job.totalEst,
              }
            : job,
        ),
      )
    })
    return unsubscribe
  }, [caseId])

  // Poll fallback: the reliable source of truth for completion + errors.
  useEffect(() => {
    const active = jobs.filter((j) => j.status === "queued" || j.status === "running")
    if (active.length === 0) return
    const timer = window.setInterval(() => {
      void (async () => {
        for (const tracked of active) {
          try {
            const fresh = await getIngestJob(tracked.jobId)
            setJobs((prev) =>
              prev.map((job) =>
                job.jobId === fresh.id
                  ? {
                      ...job,
                      status: fresh.status,
                      parsed: Math.max(job.parsed, fresh.records_parsed),
                      errors: fresh.errors,
                    }
                  : job,
              ),
            )
            if (fresh.status === "done" && tracked.status !== "done") {
              toast.success(`Ingested ${fresh.records_parsed.toLocaleString("en-IN")} records`, {
                description: tracked.fileName,
              })
              void queryClient.invalidateQueries({ queryKey: ["cases"] })
              void queryClient.invalidateQueries({ queryKey: ["case", caseId] })
              void queryClient.invalidateQueries({ queryKey: ["events", caseId] })
            }
            if (fresh.status === "failed" && tracked.status !== "failed") {
              toast.error("Ingest failed", { description: fresh.errors[0] ?? tracked.fileName })
            }
          } catch {
            // transient poll error — next tick retries
          }
        }
      })()
    }, POLL_INTERVAL_MS)
    return () => window.clearInterval(timer)
  }, [jobs, caseId, queryClient])

  const handleFiles = useCallback(
    async (fileList: FileList | null) => {
      if (!fileList || fileList.length === 0) return
      for (const file of [...fileList]) {
        try {
          const ref = await uploadFile(caseId, file)
          setJobs((prev) => [
            {
              jobId: ref.job_id,
              fileName: file.name,
              status: "queued",
              parsed: 0,
              totalEst: file.size,
              errors: [],
            },
            ...prev,
          ])
        } catch (err) {
          toast.error("Upload rejected", {
            description: err instanceof Error ? err.message : file.name,
          })
        }
      }
    },
    [caseId],
  )

  function removeJob(jobId: string) {
    setJobs((prev) => prev.filter((j) => j.jobId !== jobId))
  }

  return (
    <div className="space-y-4">
      <div
        role="button"
        tabIndex={0}
        aria-label="Upload data files"
        onClick={() => inputRef.current?.click()}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") inputRef.current?.click()
        }}
        onDragOver={(e) => {
          e.preventDefault()
          setDragging(true)
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={(e) => {
          e.preventDefault()
          setDragging(false)
          void handleFiles(e.dataTransfer.files)
        }}
        className={`flex cursor-pointer flex-col items-center justify-center rounded-sm border border-dashed py-12 text-center transition-colors ${
          dragging ? "border-primary bg-primary/5" : "border-border hover:bg-accent/40"
        }`}
      >
        <CloudUpload className="mb-2 size-8 text-muted-foreground" aria-hidden />
        <p className="text-sm font-medium">Drop CDR / IPDR / bank / social exports here</p>
        <p className="text-muted-foreground mt-1 text-xs">
          Any format — CSV, PDF, XLSX, DOCX, JSON, XML, SQL dumps. Auto-detected server-side.
        </p>
        <input
          ref={inputRef}
          type="file"
          multiple
          className="hidden"
          onChange={(e) => {
            void handleFiles(e.target.files)
            e.target.value = ""
          }}
        />
      </div>

      {jobs.length > 0 ? (
        <ul className="space-y-2">
          {jobs.map((job) => (
            <JobRow key={job.jobId} job={job} caseId={caseId} onDismiss={() => removeJob(job.jobId)} />
          ))}
        </ul>
      ) : null}
    </div>
  )
}

function JobRow({
  job,
  caseId,
  onDismiss,
}: {
  job: TrackedJob
  caseId: string
  onDismiss: () => void
}) {
  void caseId
  const pct =
    job.status === "done"
      ? 100
      : job.totalEst && job.totalEst > 0
        ? Math.min(99, Math.round((job.parsed / job.totalEst) * 100))
        : null

  return (
    <li className="border-border rounded-sm border p-3">
      <div className="flex items-center gap-2.5">
        {job.status === "done" ? (
          <CheckCircle2 className="size-4 shrink-0 text-chart-3" aria-hidden />
        ) : job.status === "failed" ? (
          <FileWarning className="text-severity-critical size-4 shrink-0" aria-hidden />
        ) : (
          <Loader2 className="size-4 shrink-0 animate-spin text-muted-foreground" aria-hidden />
        )}
        <span className="min-w-0 flex-1 truncate font-mono text-xs">{job.fileName}</span>
        {pct !== null ? (
          <span className="shrink-0 font-mono text-xs tabular-nums text-muted-foreground">{pct}%</span>
        ) : (
          <span className="text-muted-foreground shrink-0 font-mono text-xs">
            {job.parsed.toLocaleString("en-IN")} recs
          </span>
        )}
        {job.status === "done" || job.status === "failed" ? (
          <Button variant="ghost" size="icon" className="size-6" onClick={onDismiss} aria-label="Dismiss">
            <X className="size-3.5" aria-hidden />
          </Button>
        ) : null}
      </div>
      {job.status !== "failed" && pct !== null ? (
        <div className="bg-muted mt-2 h-1 overflow-hidden rounded-full">
          <div
            className="bg-primary h-full transition-all duration-500 ease-out"
            style={{ width: `${pct}%` }}
          />
        </div>
      ) : null}
      {job.errors.length > 0 ? (
        <details className="mt-2">
          <summary className="text-severity-high cursor-pointer font-mono text-[11px]">
            {job.errors.length} parse error{job.errors.length === 1 ? "" : "s"}
          </summary>
          <ul className="border-border mt-1.5 space-y-1 border-l pl-3">
            {job.errors.slice(0, 20).map((error, i) => (
              <li key={i} className="font-mono text-[11px] text-muted-foreground">
                {humanizeError(error)}
              </li>
            ))}
            {job.errors.length > 20 ? (
              <li className="font-mono text-[11px] text-muted-foreground">
                …and {job.errors.length - 20} more
              </li>
            ) : null}
          </ul>
        </details>
      ) : null}
    </li>
  )
}
