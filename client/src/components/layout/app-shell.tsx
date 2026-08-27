import { NavLink, Outlet, useNavigate } from "react-router-dom"
import {
  Eye,
  FileText,
  FolderOpen,
  LayoutDashboard,
  LogOut,
  ScrollText,
  Search,
  Settings,
  Siren,
  type LucideIcon,
} from "lucide-react"
import { useAuth } from "@/auth/AuthContext"
import type { Permission } from "@/lib/rbac"
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

const NAV_ITEMS: readonly NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard, end: true },
  { to: "/cases", label: "Cases", icon: FolderOpen },
  { to: "/alerts", label: "Alert Center", icon: Siren },
  { to: "/search", label: "Search", icon: Search },
  { to: "/reports", label: "Reports", icon: FileText },
  { to: "/settings", label: "Settings", icon: Settings, permission: "users.manage" },
  { to: "/audit", label: "Audit Log", icon: ScrollText, permission: "audit.view" },
]

export function AppShell() {
  const { user, can, signOut } = useAuth()
  const navigate = useNavigate()

  async function handleSignOut() {
    await signOut()
    navigate("/login", { replace: true })
  }

  return (
    <div className="flex h-screen overflow-hidden">
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
          {NAV_ITEMS.filter((item) => !item.permission || can(item.permission)).map((item) => (
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
