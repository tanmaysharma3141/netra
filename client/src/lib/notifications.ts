/**
 * Native OS notification helpers for NETRA.
 *
 * Uses tauri-plugin-notification when running inside the Tauri shell,
 * falls back to the browser Notification API otherwise.
 */

import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification"

export type NotificationOptions = {
  title: string
  body: string
  /** Optional severity icon for the body text. */
  severity?: "info" | "warning" | "critical"
}

let permissionGranted: boolean | null = null

/** Ensure we have notification permission (cached after first call). */
export async function ensurePermission(): Promise<boolean> {
  if (permissionGranted !== null) return permissionGranted
  try {
    permissionGranted = await isPermissionGranted()
    if (!permissionGranted) {
      const result = await requestPermission()
      permissionGranted = result === "granted"
    }
  } catch {
    // Not in Tauri — fall back to browser API
    if ("Notification" in window && Notification.permission === "default") {
      await Notification.requestPermission()
    }
    permissionGranted = true // will attempt browser fallback
  }
  return permissionGranted
}

/** Send a native notification. Tauri → browser fallback. */
export async function notify(opts: NotificationOptions): Promise<void> {
  await ensurePermission()

  const prefix =
    opts.severity === "critical"
      ? "🚨 CRITICAL: "
      : opts.severity === "warning"
        ? "⚠️ WARNING: "
        : ""

  try {
    // Try Tauri plugin first
    await sendNotification({
      title: opts.title,
      body: `${prefix}${opts.body}`,
    })
  } catch {
    // Fallback to browser Notification API
    if ("Notification" in window && Notification.permission === "granted") {
      new Notification(opts.title, { body: `${prefix}${opts.body}` })
    }
  }
}

/**
 * Subscribe to live alerts via the server WebSocket and fire native
 * notifications for each incoming alert. Returns an unsubscribe function.
 */
export function subscribeAlerts(
  caseId: string,
  wsUrl: string,
  onAlert?: (alert: Record<string, unknown>) => void,
): () => void {
  const ws = new WebSocket(`${wsUrl}/ws/alerts/${caseId}`)

  ws.onmessage = (event) => {
    try {
      const alert = JSON.parse(event.data)
      onAlert?.(alert)

      // Fire native notification for alert
      const severity = (alert.severity as string)?.toLowerCase() ?? "info"
      notify({
        title: `NETRA Alert — ${alert.rule ?? "Unknown Rule"}`,
        body: `${alert.entity_id ?? "Unknown"} • ${alert.summary ?? "No details"}`,
        severity: severity === "critical" ? "critical" : severity === "high" ? "warning" : "info",
      })
    } catch {
      // Ignore malformed messages
    }
  }

  return () => {
    ws.close()
  }
}
