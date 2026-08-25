import type { Severity, SourceType } from "@/api/types"

export const SEVERITY_ORDER: readonly Severity[] = ["critical", "high", "medium", "low"]

export const SOURCE_ORDER: readonly SourceType[] = ["CDR", "IPDR", "BANK", "SOCIAL"]

export const SOURCE_LABELS: Record<SourceType, string> = {
  CDR: "Telecom CDR",
  IPDR: "IPDR",
  BANK: "Banking",
  SOCIAL: "Social Media",
}

export const severityTextClass: Record<Severity, string> = {
  critical: "text-severity-critical",
  high: "text-severity-high",
  medium: "text-severity-medium",
  low: "text-severity-low",
}

export const severityBadgeClass: Record<Severity, string> = {
  critical: "border-severity-critical/40 bg-severity-critical/10 text-severity-critical",
  high: "border-severity-high/40 bg-severity-high/10 text-severity-high",
  medium: "border-severity-medium/40 bg-severity-medium/10 text-severity-medium",
  low: "border-severity-low/40 bg-severity-low/10 text-severity-low",
}

/** Domain colors aligned with --chart-1..4 tokens. */
export const sourceBadgeClass: Record<SourceType, string> = {
  CDR: "border-chart-1/40 bg-chart-1/10 text-chart-1",
  IPDR: "border-chart-2/40 bg-chart-2/10 text-chart-2",
  BANK: "border-chart-3/40 bg-chart-3/10 text-chart-3",
  SOCIAL: "border-chart-4/40 bg-chart-4/10 text-chart-4",
}
