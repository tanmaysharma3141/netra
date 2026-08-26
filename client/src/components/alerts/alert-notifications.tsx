import { useEffect } from "react"
import { toast } from "sonner"
import { Siren } from "lucide-react"
import { wsClient, type WsEnvelope } from "@/api/ws"
import type { Alert } from "@/api/types"
import { severityBadgeClass } from "@/lib/severity"

/**
 * Subscribes to WS alert.created events and shows toast notifications.
 * Mount this in the app shell — it runs in the background.
 */
export function AlertNotifications() {
  useEffect(() => {
    wsClient.start()

    const unsubscribe = wsClient.subscribe(["global"], (envelope: WsEnvelope) => {
      if (envelope.event !== "alert.created") return

      const alert = envelope.payload as Alert
      const severity = alert.severity ?? "medium"

      toast(`${severity.toUpperCase()} alert`, {
        description: alert.summary ?? alert.pattern ?? "New alert detected",
        icon: <Siren className={`size-4 ${severityBadgeClass[severity] ?? ""}`} />,
        duration: 8000,
      })
    })

    return () => {
      unsubscribe()
    }
  }, [])

  return null
}
