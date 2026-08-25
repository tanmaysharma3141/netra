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
