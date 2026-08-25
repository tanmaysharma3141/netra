import { Component, type ErrorInfo, type ReactNode } from "react"

interface Props {
  children: ReactNode
}

interface State {
  error: Error | null
}

/** Last-resort boundary: a render crash must never look like a dead black screen. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Unhandled UI error:", error, info.componentStack)
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex min-h-screen items-center justify-center bg-background p-6">
          <div className="max-w-md rounded-sm border border-destructive/40 bg-card p-6">
            <p className="font-mono text-sm font-semibold text-severity-critical">
              CONSOLE FAULT
            </p>
            <p className="mt-2 text-sm text-muted-foreground">
              The interface hit an unexpected error. Reload the console — your session survives.
            </p>
            <pre className="border-border mt-3 max-h-40 overflow-auto rounded-sm border p-2 font-mono text-[11px] text-muted-foreground">
              {this.state.error.message}
            </pre>
            <button
              onClick={() => window.location.reload()}
              className="bg-primary text-primary-foreground mt-4 rounded-sm px-4 py-2 text-sm font-medium hover:opacity-90"
            >
              Reload console
            </button>
          </div>
        </div>
      )
    }
    return this.props.children
  }
}
