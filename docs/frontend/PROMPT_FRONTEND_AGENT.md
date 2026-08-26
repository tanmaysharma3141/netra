# Agent Prompt — NETRA Frontend Development (Phase 5+)

> **Version:** 2.0 (updated 26 Aug 2026)
> **For:** New agent taking over frontend from MIMI
> **Status:** Phases 0-4 complete. Picking up from Phase 5.

---

Copy everything below this line into your coding agent as the kickoff prompt.

---

You are the frontend agent for **NETRA** — an air-gapped forensic intelligence platform for Indian law enforcement. The previous agent (MIMI) shipped Phases 0-4. You're picking up from Phase 5.

## Read these files FIRST (in this order)

1. **`docs/backend/AGENT_HANDOFF.md`** — quick-start guide, what's built, what's left, known gotchas
2. **`docs/backend/COMPREHENSIVE_PROJECT_STATUS.md`** — everything that's been done, every error, every decision
3. **`docs/comms/TEAM_PROGRESS.md`** — chat log between agents (bottom = oldest, top = newest)
4. **`contracts/api-types.ts`** — shared TypeScript types (single source of truth)
5. **`docs/API.md`** — frozen REST/WebSocket/SSE contract
6. **`docs/frontend/PLAN_FRONTEND.md`** — your phased plan (Phases 0-4 are done, start from Phase 5)
7. **`docs/NETRA_PRD.md`** — product requirements (screens §7, RBAC §5.2)

## What's already built (DO NOT rebuild)

### Screens (4 real, 5 placeholder)
- **Login** ✅ — POST /auth/login, handles 401/403/423, secure store, RBAC
- **Dashboard** ✅ — KPI cards from GET /cases, severity/source breakdown
- **Cases** ✅ — Search, filter, create modal, RBAC-gated
- **Case Detail** ✅ — 7 tabs, 4 working:
  - Timeline ✅ — Virtualized 100k events, filters, clustering, A/B comparison, annotations
  - Graph ✅ — D3 force-directed, hop control, BFS focus, entity profile panel
  - Map ✅ — Leaflet, playback slider, offline tile support
  - Ingest ✅ — Drag-drop, WS progress + poll fallback
  - Alerts ⚠️ — Placeholder card (Alert Center shipped separately)
  - Reports ❌ — Placeholder
  - Chat ❌ — Placeholder
- **Alerts** ✅ — Cross-case list, severity badges, triage workflow

### API Layer
- `src/api/client.ts` — Fetch wrapper with JWT injection, 401 redirect, error parsing
- `src/api/ws.ts` — WebSocket client with auto-reconnect, topic pub/sub
- `src/api/auth.ts`, `cases.ts`, `events.ts`, `entities.ts`, `graph.ts`, `geo.ts`, `ingest.ts` — all real
- `src/api/types.ts` — Re-exports from `@contracts/api-types`

### Auth & RBAC
- `src/auth/AuthContext.tsx` — restoring → authenticated / unauthenticated
- `src/lib/rbac.ts` — 4-role permission matrix
- `src/lib/secureStore.ts` — Tauri encrypted + localStorage fallback

### Components
- `components/timeline/` — timeline-panel, compare-pane
- `components/graph/` — graph-panel, force-graph, entity-profile-panel
- `components/map/` — map-panel, movement-map
- `components/ingest/` — ingest-panel
- `components/ui/` — 14 shadcn components
- `components/layout/app-shell.tsx` — sidebar + outlet

## What you need to build (Phase 5)

### 1. Reports Screen (`src/screens/reports-screen.tsx`)
- Report viewer: markdown summary display
- Approve button (Supervisor/Admin only via `can("report.approve")`)
- Export PDF download link
- API: `POST /cases/:id/reports`, `GET /cases/:id/reports`, `GET /reports/:id`, `GET /reports/:id/export`, `PATCH /reports/:id/approve`

### 2. Settings Screen (`src/screens/settings-screen.tsx`)
- User management table (Admin only): list users, create, edit role, deactivate
- Webhook config form (Discord/Telegram)
- Model version list + promote button
- Training queue info + manual trigger
- API: `GET/PATCH /settings/webhooks`, `GET /models`, `POST /models/promote`, `POST /training/trigger`, `GET /training/queue`

### 3. Audit Screen (`src/screens/audit-screen.tsx`)
- Audit log viewer (Admin/Supervisor only)
- Display: timestamp, user, action, detail
- API: `GET /cases/:id/audit`

### 4. Chat Tab in Case Detail
- SSE streaming copilot chat panel
- Message list with streaming render
- Sources display: cited event IDs as clickable links to timeline drawer
- API: `POST /cases/:id/chat` (SSE: `{"delta":...}` → `{"sources":[...]}` → `{"done":true}`)

### 5. Alerts Tab in Case Detail
- Wire the existing Alert Center into the case detail tab
- Or: inline alert list filtered to this case

### 6. Full RBAC Sweep
- Every screen hides/blocks actions per role matrix
- Verify: analysts can't create cases, supervisors can't ingest, etc.

### 7. Polish
- Loading skeletons + error states on every screen
- Empty states with guidance
- No console errors or warnings

## Tech stack (do not substitute)

- Tauri v2 + Vite 7 + React 18 + TypeScript 5.8 (strict mode)
- Tailwind CSS v4 + shadcn/ui v4
- React Router v6 (HashRouter — mandatory for Tauri file:// protocol)
- @tanstack/react-query v5 for server state
- D3 v7 (graph), Leaflet 1.9 (map), @tanstack/react-virtual (timeline)
- Sonner for toasts
- Geist + Geist Mono fonts

## Architecture rules

1. **Import types from `@contracts/api-types`** — never redefine shapes locally
2. **All HTTP through `src/api/client.ts`** — JWT injection, 401 handling
3. **WebSocket via `src/api/ws.ts`** — auto-reconnect, topic pub/sub
4. **No `any`** — TypeScript strict mode is on
5. **Every async view:** loading skeleton + error state (with retry) + empty state
6. **RBAC:** hide/disable actions per role; use `can()` from AuthContext
7. **Components** in `src/components/`, **screens** in `src/screens/`, **hooks** in `src/hooks/`
8. **Data shapes** come from `contracts/api-types.ts`

## Branching & communication

- **Your branch:** `agent/frontend`
- **Docs go on:** `main` (update `docs/comms/TEAM_PROGRESS.md` with your section)
- **Never push code to `main`** — work on `agent/frontend`, let Chirag merge
- **Chat log:** `docs/comms/TEAM_PROGRESS.md` — sign messages with `[MIMI]` (or your agent name), newest at top
- **Pull before writing** on shared files

## Design direction

Dark "forensic console" aesthetic: near-black backgrounds, high-contrast monospace accents for IDs/timestamps, severity color coding (critical=red, high=orange, medium=amber, low=slate). Dense information layout for long investigative sessions. Professional software for police investigators — clarity, scanability, keyboard navigation.

## Known gotchas

1. **HashRouter required** — Tauri uses file:// protocol, History API doesn't work
2. **Leaflet CSS** — must import `leaflet/dist/leaflet.css` or tiles break
3. **WS auth** — browsers can't set headers on WebSocket; use `?token=` query param
4. **`leaflet` package** — already in package.json, just run `npm install` if missing
5. **Server runs on port 8420** — `http://localhost:8420/api/v1`
6. **Login creds:** `admin` / `netra-admin`

## Definition of done (every screen)

- Works against real (not stubbed) backend data
- Loading/error/empty states present
- RBAC-aware (correct permissions enforced)
- No console errors or warnings
- Keyboard accessible (tab order, focus states, Enter submits forms)

## Your first task

1. Read all 7 reference files listed above
2. Update `docs/comms/TEAM_PROGRESS.md` — add your section under "Frontend (YOUR NAME)" with current status
3. Report your understanding of Phase 5 scope in three bullets
4. Start with the Reports screen (it's the most self-contained)

Begin.
