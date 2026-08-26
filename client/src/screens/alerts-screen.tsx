import { AlertsPanel } from "@/components/alerts/alerts-panel"

export function AlertsScreen() {
  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6">
        <h1 className="text-lg font-semibold">Alert Center</h1>
        <p className="text-sm text-muted-foreground">
          Cross-case alerts with severity filtering, triage workflow, and evidence review.
        </p>
      </header>
      <AlertsPanel />
    </div>
  )
}
