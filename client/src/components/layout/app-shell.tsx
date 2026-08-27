import { useState } from "react"
import { NavLink, Outlet, useNavigate } from "react-router-dom"
import {
  ChevronDown,
  ChevronRight,
  Eye,
  FileText,
  FolderOpen,
  LayoutDashboard,
  LogOut,
  ScrollText,
  Search,
  Settings,
  Shield,
  Siren,
  type LucideIcon,
} from "lucide-react"
import { useAuth } from "@/auth/AuthContext"
import type { Permission } from "@/lib/rbac"
import { AlertNotifications } from "@/components/alerts/alert-notifications"
import { useKeyboardShortcuts } from "@/hooks/use-keyboard-shortcuts"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"

interface NavItem {
  to: string
  label: string
  icon: LucideIcon
  end?: boolean
  permission?: Permission
}

const MAIN_NAV_ITEMS: readonly NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard, end: true },
  { to: "/cases", label: "Cases", icon: FolderOpen },
  { to: "/alerts", label: "Alert Center", icon: Siren },
  { to: "/search", label: "Search", icon: Search },
  { to: "/reports", label: "Reports", icon: FileText },
]

const ADMIN_NAV_ITEMS: readonly NavItem[] = [
  { to: "/settings", label: "Settings", icon: Settings, permission: "users.manage" },
  { to: "/audit", label: "Audit Log", icon: ScrollText, permission: "audit.view" },
]

export function AppShell() {
  const { user, can, signOut } = useAuth()
  const navigate = useNavigate()
  useKeyboardShortcuts()

  async function handleSignOut() {
    await signOut()
    navigate("/login", { replace: true })
  }

  return (
    <div className="flex h-screen overflow-hidden">
      <AlertNotifications />
      <aside className="flex w-56 shrink-0 flex-col border-r border-sidebar-border bg-sidebar">
        <div className="flex items-center gap-2 px-4 py-4">
          <Eye className="size-5 text-primary" aria-hidden />
          <div className="flex flex-col">
            <span className="font-mono text-sm font-semibold tracking-widest">NETRA</span>
            <span className="text-[10px] tracking-[0.18em] text-muted-foreground uppercase">
              Forensic Console
            </span>
          </div>
        </div>

        <Separator className="bg-sidebar-border" />

        <nav className="flex-1 space-y-0.5 px-2 py-3">
          {MAIN_NAV_ITEMS.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.end}
              className={({ isActive }) =>
                `flex items-center gap-2.5 rounded-sm px-3 py-2 text-sm transition-colors ${
                  isActive
                    ? "bg-sidebar-accent font-medium text-sidebar-primary"
                    : "text-muted-foreground hover:bg-sidebar-accent/60 hover:text-foreground"
                }`
              }
            >
              <item.icon className="size-4 shrink-0" aria-hidden />
              {item.label}
            </NavLink>
          ))}

          {/* Admin section — collapsible, only for Admin/Supervisor */}
          {ADMIN_NAV_ITEMS.some((item) => !item.permission || can(item.permission)) && (
            <AdminNavGroup items={ADMIN_NAV_ITEMS} can={can} />
          )}
        </nav>

        <Separator className="bg-sidebar-border" />

        <div className="flex items-center justify-between gap-2 px-4 py-3">
          <div className="flex min-w-0 flex-col">
            <span className="truncate font-mono text-xs">{user?.username ?? "—"}</span>
            <Badge variant="outline" className="mt-1 w-fit font-mono text-[10px] tracking-wider uppercase">
              {user?.role ?? ""}
            </Badge>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => void handleSignOut()}
            aria-label="Sign out"
            title="Sign out"
          >
            <LogOut className="size-4" aria-hidden />
          </Button>
        </div>
      </aside>

      <main className="min-w-0 flex-1 overflow-y-auto">
        <Outlet />
      </main>
    </div>
  )
}

function AdminNavGroup({
  items,
  can,
}: {
  items: readonly NavItem[]
  can: (p: Permission) => boolean
}) {
  const [expanded, setExpanded] = useState(false)
  const visibleItems = items.filter((item) => !item.permission || can(item.permission))

  return (
    <div className="mt-2">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center gap-2.5 rounded-sm px-3 py-2 text-sm text-muted-foreground transition-colors hover:bg-sidebar-accent/60 hover:text-foreground"
        aria-expanded={expanded}
      >
        <Shield className="size-4 shrink-0" aria-hidden />
        <span className="flex-1 text-left">Admin</span>
        {expanded ? (
          <ChevronDown className="size-3.5 shrink-0" aria-hidden />
        ) : (
          <ChevronRight className="size-3.5 shrink-0" aria-hidden />
        )}
      </button>
      {expanded && (
        <div className="ml-2 space-y-0.5 border-l border-sidebar-border pl-2">
          {visibleItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                `flex items-center gap-2.5 rounded-sm px-3 py-2 text-sm transition-colors ${
                  isActive
                    ? "bg-sidebar-accent font-medium text-sidebar-primary"
                    : "text-muted-foreground hover:bg-sidebar-accent/60 hover:text-foreground"
                }`
              }
            >
              <item.icon className="size-4 shrink-0" aria-hidden />
              {item.label}
            </NavLink>
          ))}
        </div>
      )}
    </div>
  )
}