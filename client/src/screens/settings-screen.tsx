import { useAuth } from "@/auth/AuthContext"
import { PlaceholderScreen } from "./placeholder-screen"

export function SettingsScreen() {
  const { can } = useAuth()
  const scope = [
    can("users.manage") ? "user management" : null,
    can("webhooks.configure") ? "webhook configuration" : null,
    can("training.trigger") ? "model management & training scheduler" : null,
  ]
    .filter(Boolean)
    .join(", ")

  return (
    <PlaceholderScreen
      title="Settings"
      description={scope ? `Admin console: ${scope}.` : "Application settings."}
      phase="PHASE 5 · CHAT + REPORTS + POLISH"
    />
  )
}
