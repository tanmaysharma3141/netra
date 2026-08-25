import { Navigate, Outlet, Route, Routes } from "react-router-dom"
import { Loader2 } from "lucide-react"
import { useAuth } from "@/auth/AuthContext"
import { AppShell } from "@/components/layout/app-shell"
import { LoginScreen } from "@/screens/login-screen"
import { DashboardScreen } from "@/screens/dashboard-screen"
import { CasesScreen } from "@/screens/cases-screen"
import { CaseDetailScreen } from "@/screens/case-detail-screen"
import { AlertsScreen } from "@/screens/alerts-screen"
import { ReportsScreen } from "@/screens/reports-screen"
import { SettingsScreen } from "@/screens/settings-screen"
import { AuditScreen } from "@/screens/audit-screen"

function RequireAuth() {
  const { status } = useAuth()
  if (status === "restoring") {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <Loader2 className="size-6 animate-spin text-muted-foreground" aria-label="Restoring session" />
      </div>
    )
  }
  if (status === "unauthenticated") {
    return <Navigate to="/login" replace />
  }
  return <Outlet />
}

function LoginRoute() {
  const { status } = useAuth()
  if (status === "authenticated") {
    return <Navigate to="/" replace />
  }
  return <LoginScreen />
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginRoute />} />
      <Route element={<RequireAuth />}>
        <Route element={<AppShell />}>
          <Route index element={<DashboardScreen />} />
          <Route path="cases" element={<CasesScreen />} />
          <Route path="cases/:id" element={<CaseDetailScreen />} />
          <Route path="alerts" element={<AlertsScreen />} />
          <Route path="reports" element={<ReportsScreen />} />
          <Route path="settings" element={<SettingsScreen />} />
          <Route path="audit" element={<AuditScreen />} />
        </Route>
      </Route>
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  )
}
