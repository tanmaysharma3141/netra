import { useState } from "react"
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
import { listCases } from "@/api/cases"
import { ApiClientError } from "@/api/client"
import { API_BASE_URL } from "@/lib/env"
import { getToken } from "@/lib/secureStore"
import {
  approveReport,
  generateReport,
  getReportExportUrl,
  getReport,
  listReports,
} from "@/api/reports"
import { useAuth } from "@/auth/AuthContext"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { Skeleton } from "@/components/ui/skeleton"

export function ReportsScreen() {
  const [selectedCaseId, setSelectedCaseId] = useState<string | null>(null)
  const [selectedReportId, setSelectedReportId] = useState<string | null>(null)

  const casesQuery = useQuery({ queryKey: ["cases"], queryFn: listCases })

  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6">
        <h1 className="text-lg font-semibold">Reports</h1>
        <p className="text-sm text-muted-foreground">
          Generated intelligence reports — view, approve, and export.
        </p>
      </header>

      {casesQuery.isPending ? (
        <div className="space-y-2">
          {[0, 1, 2].map((i) => (
            <Skeleton key={i} className="h-12 w-full" />
          ))}
        </div>
      ) : casesQuery.isError ? (
        <Alert variant="destructive">
          <AlertTriangle className="size-4" aria-hidden />
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
      ) : (casesQuery.data?.length ?? 0) === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-14 text-center">
            <FileText className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">No cases yet</p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Create a case and generate a report to see it here.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="flex min-h-[60vh] gap-4">
          {/* Case list sidebar */}
          <div className="w-64 shrink-0 overflow-y-auto">
            <p className="mb-2 font-mono text-[10px] tracking-[0.18em] text-muted-foreground uppercase">
              Cases
            </p>
            <div className="space-y-1">
              {casesQuery.data!.map((c) => (
                <button
                  key={c.id}
                  onClick={() => {
                    setSelectedCaseId(c.id)
                    setSelectedReportId(null)
                  }}
                  className={`w-full rounded-sm px-3 py-2 text-left text-sm transition-colors ${
                    selectedCaseId === c.id
                      ? "bg-secondary font-medium"
                      : "text-muted-foreground hover:bg-secondary/60 hover:text-foreground"
                  }`}
                >
                  <span className="block truncate">{c.title}</span>
                  <span className="font-mono text-[10px] text-muted-foreground">
                    {c.id.slice(0, 8)}
                  </span>
                </button>
              ))}
            </div>
          </div>

          <Separator orientation="vertical" className="h-auto" />

          {/* Reports area */}
          <div className="min-w-0 flex-1">
            {selectedCaseId ? (
              <CaseReports
                caseId={selectedCaseId}
                selectedReportId={selectedReportId}
                onSelectReport={setSelectedReportId}
              />
            ) : (
              <div className="flex flex-col items-center justify-center py-20 text-center">
                <FileText className="mb-3 size-8 text-muted-foreground" aria-hidden />
                <p className="text-sm font-medium">Select a case</p>
                <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                  Choose a case from the sidebar to view its reports.
                </p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

function CaseReports({
  caseId,
  selectedReportId,
  onSelectReport,
}: {
  caseId: string
  selectedReportId: string | null
  onSelectReport: (id: string | null) => void
}) {
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
          reports.map((report) => (
            <button
              key={report.id}
              onClick={() => onSelectReport(report.id)}
              className={`w-full rounded-sm border p-3 text-left transition-colors ${
                selectedReportId === report.id
                  ? "border-primary/40 bg-secondary"
                  : "border-border hover:border-primary/20 hover:bg-secondary/50"
              }`}
            >
              <div className="flex items-center gap-2">
                <span className="font-mono text-xs font-medium">
                  v{report.version}
                </span>
                <span className="text-muted-foreground font-mono text-[10px]">
                  {report.generated_by}
                </span>
                {report.approved_by ? (
                  <Badge
                    variant="outline"
                    className="ml-auto border-emerald-500/40 bg-emerald-500/10 text-emerald-500 text-[9px]"
                  >
                    <Check className="mr-0.5 size-2.5" aria-hidden />
                    Approved
                  </Badge>
                ) : (
                  <Badge variant="outline" className="ml-auto text-[9px]">
                    Pending
                  </Badge>
                )}
              </div>
              <p className="text-muted-foreground mt-1 font-mono text-[10px]">
                {new Date(report.created_at).toLocaleString("en-IN")}
              </p>
            </button>
          ))
        )}
      </div>

      <Separator orientation="vertical" className="h-auto" />

      {/* Report detail / markdown viewer */}
      <div className="min-w-0 flex-1">
        {selectedReportId ? (
          <ReportDetail
            reportId={selectedReportId}
            caseId={caseId}
          />
        ) : (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <FileText className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">Select a report</p>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Choose a report from the list to view its summary.
            </p>
          </div>
        )}
      </div>
    </div>
  )
}

function ReportDetail({
  reportId,
  caseId,
}: {
  reportId: string
  caseId: string
}) {
  const queryClient = useQueryClient()
  const { can } = useAuth()

  const reportQuery = useQuery({
    queryKey: ["report", reportId],
    queryFn: () => getReport(reportId),
  })

  const approveMutation = useMutation({
    mutationFn: () => approveReport(reportId),
    onSuccess: () => {
      toast.success("Report approved")
      void queryClient.invalidateQueries({ queryKey: ["report", reportId] })
      void queryClient.invalidateQueries({ queryKey: ["reports", caseId] })
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
        `${API_BASE_URL}${getReportExportUrl(reportId)}`,
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
      a.download = `report-${reportId.slice(0, 8)}.pdf`
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

  if (reportQuery.isPending) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-64 w-full" />
      </div>
    )
  }

  if (reportQuery.isError) {
    return (
      <Alert variant="destructive">
        <AlertTriangle className="size-4" aria-hidden />
        <AlertTitle>Failed to load report</AlertTitle>
        <AlertDescription>
          {(reportQuery.error as { message?: string }).message ?? "Unknown error."}
          <Button
            variant="outline"
            size="sm"
            className="mt-2"
            onClick={() => void reportQuery.refetch()}
          >
            <RefreshCw className="mr-1.5 size-3.5" aria-hidden />
            Retry
          </Button>
        </AlertDescription>
      </Alert>
    )
  }

  const report = reportQuery.data

  return (
    <div className="space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-sm font-semibold">Report v{report.version}</h2>
            <Badge variant="outline" className="font-mono text-[10px] tracking-wider uppercase">
              {report.generated_by}
            </Badge>
            {report.approved_by ? (
              <Badge
                variant="outline"
                className="border-emerald-500/40 bg-emerald-500/10 text-emerald-500 text-[10px]"
              >
                <CheckCircle2 className="mr-1 size-3" aria-hidden />
                Approved
              </Badge>
            ) : (
              <Badge variant="outline" className="text-[10px]">
                Pending approval
              </Badge>
            )}
          </div>
          <p className="mt-1 font-mono text-[10px] text-muted-foreground">
            {report.id} · created{" "}
            {new Date(report.created_at).toLocaleString("en-IN")}
            {report.approved_by ? ` · approved by ${report.approved_by}` : ""}
          </p>
        </div>
        <div className="flex shrink-0 gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => void handleExport()}
          >
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
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="font-mono text-xs tracking-wider uppercase">
            Executive Summary
          </CardTitle>
          <CardDescription>
            {report.summary_md.length.toLocaleString("en-IN")} characters
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="prose prose-invert prose-sm max-w-none font-mono text-xs leading-relaxed">
            <MarkdownContent content={report.summary_md} />
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

/** Simple markdown renderer — splits into paragraphs, renders bold/italic/code.
 *  No external dependencies. Falls back to <pre> for code blocks. */
function MarkdownContent({ content }: { content: string }) {
  if (!content) {
    return <p className="text-muted-foreground italic">No summary generated.</p>
  }

  const blocks = content.split(/\n{2,}/)

  return (
    <>
      {blocks.map((block, i) => {
        const trimmed = block.trim()
        if (!trimmed) return null

        // Code block
        if (trimmed.startsWith("```")) {
          const code = trimmed.replace(/^```[\w]*\n?/, "").replace(/\n?```$/, "")
          return (
            <pre
              key={i}
              className="bg-card overflow-x-auto rounded-sm border p-3 text-xs"
            >
              <code>{code}</code>
            </pre>
          )
        }

        // Heading
        if (trimmed.startsWith("# ")) {
          return (
            <h1 key={i} className="mt-4 mb-2 text-base font-semibold">
              {trimmed.slice(2)}
            </h1>
          )
        }
        if (trimmed.startsWith("## ")) {
          return (
            <h2 key={i} className="mt-3 mb-1.5 text-sm font-semibold">
              {trimmed.slice(3)}
            </h2>
          )
        }
        if (trimmed.startsWith("### ")) {
          return (
            <h3 key={i} className="mt-2 mb-1 text-xs font-semibold">
              {trimmed.slice(4)}
            </h3>
          )
        }

        // List items
        if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
          const items = trimmed.split("\n").filter((l) => l.trim())
          return (
            <ul key={i} className="my-1 list-disc pl-5">
              {items.map((item, j) => (
                <li key={j}>{inlineFormat(item.replace(/^[-*]\s+/, ""))}</li>
              ))}
            </ul>
          )
        }

        // Numbered list
        if (/^\d+\.\s/.test(trimmed)) {
          const items = trimmed.split("\n").filter((l) => l.trim())
          return (
            <ol key={i} className="my-1 list-decimal pl-5">
              {items.map((item, j) => (
                <li key={j}>{inlineFormat(item.replace(/^\d+\.\s+/, ""))}</li>
              ))}
            </ol>
          )
        }

        // Regular paragraph
        return (
          <p key={i} className="my-1.5">
            {inlineFormat(trimmed)}
          </p>
        )
      })}
    </>
  )
}

function inlineFormat(text: string): React.ReactNode {
  // Simple inline formatting: **bold**, *italic*, `code`
  const parts: React.ReactNode[] = []
  const regex = /(\*\*(.+?)\*\*|\*(.+?)\*|`(.+?)`)/g
  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index))
    }
    if (match[2]) {
      parts.push(<strong key={match.index}>{match[2]}</strong>)
    } else if (match[3]) {
      parts.push(<em key={match.index}>{match[3]}</em>)
    } else if (match[4]) {
      parts.push(
        <code
          key={match.index}
          className="bg-card rounded-sm px-1 py-0.5 text-[11px]"
        >
          {match[4]}
        </code>,
      )
    }
    lastIndex = match.index + match[0].length
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex))
  }

  return parts.length > 0 ? parts : text
}
