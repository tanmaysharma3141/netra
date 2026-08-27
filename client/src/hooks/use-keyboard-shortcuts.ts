import { useEffect } from "react"
import { useNavigate } from "react-router-dom"

/**
 * Global keyboard shortcuts for NETRA.
 * - Cmd/Ctrl + 1-4: Navigate to Dashboard, Cases, Alerts, Reports
 * - Cmd/Ctrl + \: Toggle sidebar (placeholder — sidebar doesn't collapse yet)
 * - Escape: Close open modals/sheets (handled by shadcn primitives)
 */
export function useKeyboardShortcuts() {
  const navigate = useNavigate()

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey
      if (!mod) return

      // Cmd+1 → Dashboard
      if (e.key === "1") {
        e.preventDefault()
        navigate("/")
      }
      // Cmd+2 → Cases
      if (e.key === "2") {
        e.preventDefault()
        navigate("/cases")
      }
      // Cmd+3 → Alerts
      if (e.key === "3") {
        e.preventDefault()
        navigate("/alerts")
      }
      // Cmd+4 → Reports
      if (e.key === "4") {
        e.preventDefault()
        navigate("/reports")
      }
    }

    document.addEventListener("keydown", handleKeyDown)
    return () => document.removeEventListener("keydown", handleKeyDown)
  }, [navigate])
}
