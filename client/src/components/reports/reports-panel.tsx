import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import {
  AlertTriangle,
  Check,
  CheckCircle2,
  Download,
  FileText,
  FilePlus,
  RefreshCw,
} from "lucide-react"
import Markdown from "react-markdown"
import { ApiClientError } from "@/api/client"
import { API_BASE_URL } from "@/lib/env"
import { getToken } from "@/lib/secureStore"
import {
  approveReport,
  generateReport,
  getReportExportUrl,
  listReports,
} from "@/api/reports"
import type { Report } from "@/api/types"
import { useAuth } from "@/auth/AuthContext"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"

export function ReportsPanel({ caseId }: { caseId: string }) {
  return <CaseReports caseId={caseId} />
}

function CaseReports({ caseId }: { caseId: string }) {
  const queryClient = useQueryClient()
  const { can } = useAuth()

  const reportsQuery = useQuery({
    queryKey: ["reports", caseId],
    queryFn: () => listReports(caseId),
  })

  const generateMutation = useMutation({
    mutationFn: () => generateReport(caseId),
    onSuccess: () => {
      toast.success("Report generation started")
      void queryClient.invalidateQueries({ queryKey: ["reports", caseId] })
    },
    onError: (err) => {
      toast.error("Could not generate report", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  if (reportsQuery.isPending) {
    return (
      <div className="space-y-3">
        {[0, 1, 2].map((i) => (
          <Skeleton key={i} className="h-20 w-full" />
        ))}
      </div>
    )
  }

  if (reportsQuery.isError) {
    return (
      <Alert variant="destructive">
        <AlertTriangle className="size-4" aria-hidden />
        <AlertTitle>Failed to load reports</AlertTitle>
        <AlertDescription>
          {(reportsQuery.error as { message?: string }).message ?? "Unknown error."}
          <Button
            variant="outline"
            size="sm"
            className="mt-2"
            onClick={() => void reportsQuery.refetch()}
          >
            <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
            Retry
          </Button>
        </AlertDescription>
      </Alert>
    )
  }

  const reports = reportsQuery.data ?? []

  return (
    <div className="flex min-h-[50vh] gap-4">
      {/* Report list */}
      <div className="w-72 shrink-0 space-y-3">
        <div className="flex items-center justify-between">
          <p className="font-mono text-[10px] tracking-[0.18em] text-muted-foreground uppercase">
            Reports ({reports.length})
          </p>
          {can("report.generate") ? (
            <Button
              size="sm"
              variant="outline"
              onClick={() => void generateMutation.mutate()}
              disabled={generateMutation.isPending}
            >
              <FilePlus className="mr-1 size-3.5" aria-hidden />
              {generateMutation.isPending ? "Generating…" : "Generate"}
            </Button>
          ) : null}
        </div>

        {reports.length === 0 ? (
          <Card className="border-dashed">
            <CardContent className="flex flex-col items-center justify-center py-10 text-center">
              <FileText className="mb-2 size-6 text-muted-foreground" aria-hidden />
              <p className="text-xs font-medium">No reports</p>
              <p className="mt-1 max-w-[160px] text-xs text-muted-foreground">
                Generate a report from case data.
              </p>
            </CardContent>
          </Card>
        ) : (
          <ReportList reports={reports} />
        )}
      </div>
    </div>
  )
}

function ReportList({
  reports,
}: {
  reports: Report[]
}) {
  return (
    <div className="space-y-3">
      {reports.map((report) => (
        <ReportCard key={report.id} report={report} />
      ))}
    </div>
  )
}

function ReportCard({ report }: { report: Report }) {
  const queryClient = useQueryClient()
  const { can } = useAuth()

  const approveMutation = useMutation({
    mutationFn: () => approveReport(report.id),
    onSuccess: () => {
      toast.success("Report approved")
      void queryClient.invalidateQueries({ queryKey: ["reports", report.case_id] })
      void queryClient.invalidateQueries({ queryKey: ["report", report.id] })
    },
    onError: (err) => {
      toast.error("Could not approve report", {
        description: err instanceof ApiClientError ? err.message : "Unexpected error.",
      })
    },
  })

  async function handleExport() {
    try {
      const token = await getToken()
      const res = await fetch(
        `${API_BASE_URL}${getReportExportUrl(report.id)}`,
        {
          headers: token ? { Authorization: `Bearer ${token}` } : {},
        },
      )
      if (!res.ok) {
        toast.error("Export failed", { description: `HTTP ${res.status}` })
        return
      }
      const blob = await res.blob()
      const url = URL.createObjectURL(blob)
      const a = document.createElement("a")
      a.href = url
      a.download = `report-${report.id.slice(0, 8)}.pdf`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
      toast.success("PDF downloaded")
    } catch (err) {
      toast.error("Export failed", {
        description: err instanceof Error ? err.message : "Unexpected error.",
      })
    }
  }

  return (
    <Card>
      <CardContent className="space-y-3 p-4">
        <div className="flex items-center gap-2">
          <span className="font-mono text-xs font-medium">v{report.version}</span>
          <Badge variant="outline" className="font-mono text-[10px] tracking-wider uppercase">
            {report.generated_by}
          </Badge>
          {report.approved_by ? (
            <Badge
              variant="outline"
              className="ml-auto border-emerald-500/40 bg-emerald-500/10 text-emerald-500 text-[9px]"
            >
              <CheckCircle2 className="mr-0.5 size-2.5" aria-hidden />
              Approved
            </Badge>
          ) : (
            <Badge variant="outline" className="ml-auto text-[9px]">
              Pending
            </Badge>
          )}
        </div>
        <p className="text-muted-foreground font-mono text-[10px]">
          {report.id} · {new Date(report.created_at).toLocaleString("en-IN")}
          {report.approved_by ? ` · approved by ${report.approved_by}` : ""}
        </p>

        {/* Summary preview */}
        <div className="border-border rounded-sm border p-3">
          <p className="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
            Summary
          </p>
          {report.summary_md ? (
            <div className="prose prose-invert prose-xs mt-1 max-w-none font-mono text-xs leading-relaxed">
              <Markdown>{report.summary_md}</Markdown>
            </div>
          ) : (
            <p className="mt-1 text-xs text-muted-foreground">No summary generated.</p>
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-2">
          <Button size="sm" variant="outline" onClick={() => void handleExport()}>
            <Download className="mr-1 size-3.5" aria-hidden />
            Export PDF
          </Button>
          {can("report.approve") && !report.approved_by ? (
            <Button
              size="sm"
              onClick={() => void approveMutation.mutate()}
              disabled={approveMutation.isPending}
            >
              <Check className="mr-1 size-3.5" aria-hidden />
              {approveMutation.isPending ? "Approving…" : "Approve"}
            </Button>
          ) : null}
        </div>
      </CardContent>
    </Card>
  )
}
