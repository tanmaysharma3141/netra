# NETRA Frontend — Complete Handoff Document

**Author:** MIMI (frontend agent, `big-pickle` model)
**Date:** 26 Aug 2026
**Branch:** `agent/frontend`
**For:** Next agent taking over frontend work

---

## 1. What Is NETRA

NETRA (नेत्र — "The Eye") is an air-gapped forensic intelligence platform for Indian law enforcement. It ingests telecom CDR/IPDR, bank statements, and social media data; correlates entities across domains; detects anomalies; visualizes relationships on an interactive graph; maps suspect movements; and generates court-ready reports. Everything runs on-premises — zero internet.

**Hackathon:** Chandigarh Police National Hackathon 2026, Track 6 (DFAP). Hackathon day is **8 Sep 2026**. Pre-hack prep window: **26 Aug – 7 Sep**.

**Team BinaryBros:** Tanmay (human developer + AI agent operator), IMAAN (backend agent), MIMI (frontend agent — me).

---

## 2. The Agent System

This project uses a two-agent parallel development model:

| Agent | Role | Model | Branch |
|-------|------|-------|--------|
| **IMAAN** | Backend (Rust/Axum server) | Unknown | `agent/backend` |
| **MIMI** | Frontend (Tauri v2 + React) | `big-pickle` | `agent/frontend` |

### Communication Protocol
- **Whiteboard:** `docs/TEAM_PROGRESS.md` — both agents update their own section, newest-at-top chat log at bottom, messages signed with agent name.
- **Contract:** `docs/API.md` + `contracts/api-types.ts` — single source of truth for all data shapes. Changes require a PR touching BOTH files.
- **Issues:** Filed in the chat log as `[ISSUE #N — AUTHOR]`. The responsible party fixes and replies.
- **Dispatches:** When one agent unblocks the other, a `[Phase N DISPATCH]` message is posted with full technical details of what's now available.

### How to Pull IMAAN's Work
```bash
git fetch --all --prune
git log origin/agent/backend --oneline -5   # check for new commits
git merge origin/agent/backend --no-edit    # merge into your branch
# Resolve conflicts in docs/TEAM_PROGRESS.md (chat log) if any
```

---

## 3. Tech Stack & Conventions

### Frontend Stack
- **Tauri v2** (desktop shell, Rust backend for native APIs)
- **Vite** (bundler)
- **React 18** (strict mode, NOT React 19 — template defaults to 19, must downgrade: `npm install react@^18 react-dom@^18`)
- **TypeScript** (strict)
- **Tailwind CSS v4**
- **shadcn/ui** (component library)
- **React Router v6** (HashRouter — required for Tauri file:// protocol)
- **TanStack Query** (server state)
- **TanStack Virtual** (virtualized lists)
- **D3 v7** (force-directed graph)
- **Leaflet** (map — pure component, NO react-leaflet wrapper)
- **Zustand** (if needed for client state — not yet used)

### Design System
- Dark "forensic console" aesthetic: near-black bg (`#09090b`), monospace accents (Geist Mono for IDs/codes)
- Severity palette: `critical=red`, `high=orange`, `medium=amber`, `low=slate`
- Source type colors: CDR=cyan, IPDR=violet, BANK=emerald, SOCIAL=pink
- Entity type colors: PHONE=cyan, IMEI=amber, BANK_ACC=emerald, IP=violet, HANDLE=pink
- Fonts: Geist (body), Geist Mono (IDs, codes, timestamps)

### Key Conventions
- All data shapes come from `contracts/api-types.ts` — import via `@contracts/api-types`, never redefine locally
- `src/api/types.ts` is just a re-export barrel: `export * from "@contracts/api-types"`
- Every async action needs loading + error + empty state
- RBAC gating: use `can("permission")` from `AuthContext`, never hardcode role checks
- API calls go through `src/api/client.ts` (single fetch wrapper with Bearer injection, 401 redirect, ApiError parsing)
- Environment: `VITE_API_URL` (default `http://127.0.0.1:8420/api/v1`), `VITE_WS_URL` (default `ws://127.0.0.1:8420/ws`), `VITE_TILE_URL` (Leaflet tile URL, defaults to OSM)

---

## 4. Branch Structure & Git History

### Branches
| Branch | Purpose | Latest |
|--------|---------|--------|
| `main` | Stable, merged PRs only | `807b3a1` |
| `agent/frontend` | My work (MIMI) | `4ea0f06` + pending merge of IMAAN's Phase 4 |
| `agent/backend` | IMAAN's work | `b5dc345` |
| `fix/leaflet-dep-missing` | IMAAN's dep fix branch | `b5dc345` |

### Commit History (agent/frontend, chronological)
```
28af6db  client: scaffold Tauri v2 + Vite + React 18 + Tailwind v4 + shadcn/ui
cacf2a1  client: Phase 0 complete — shell, login, RBAC sidebar, API client, live-stub dashboard
2dfa42a  client: Phase 1 — cases table, create-case modal, case detail tabs; 423 retry parsing
7a0453f  client: fix post-login nav (status transition in signIn/signOut) + error boundary
31e1618  client: merge main (interop fixes), rip TEMP fallback, Phase 2 timeline core
d664db8  client: event annotations in timeline drawer via POST /events/:id/notes
b23467c  client: Phase 3 ingest UI — drop zone, WS progress + poll fallback, error list; ws manager
7809397  client: Phase 4 graph — D3 force graph, hop control, BFS focus, entity profile panel
be5e65e  docs: ISSUE #1 — cargo run ambiguous with two binaries (default-run fix)
571d314  backend: fix ISSUE #1 — default-run for cargo run; merge mimi phase4; tick phase3 gate
d842e9a  client: Phase 4 map — Leaflet movement trails with playback slider, env-driven offline tiles
4ea0f06  client: timeline suspect comparison mode (A/B panes, per-pane virtualized feeds)
```

### Commit History (agent/backend, key commits)
```
0ee5077  backend: axum stub server for all API.md routes + ws/sse live
a6fed27  backend: phase 1 — sqlx migrations, bcrypt+jwt auth with lockout, rbac guards
5f55f98  backend: interop fixes from mimi review — uppercase enums, dotted ws tags, bare sse frames
f5d0055  backend: harden phase 0/1 — ws auth, fresh-role-per-request, last-admin guard
815f54f  backend: phase 2 ingestion engine — universal csv parser, async jobs, sha256 audit trail
44b4204  backend: event annotations POST /events/:id/notes with rbac, audit trail
39d2a2a  backend: phase 3 entity resolution — deterministic links, auto-resolve, real graph+profile
f4c44c5  Phase 4: anomaly engine + alert triage
83c0632  docs: flag leaflet dep issue to MIMI (ISSUE #2)
b5dc345  fix: add missing leaflet + @types/leaflet to package.json (ISSUE #2)
```

---

## 5. Phase-by-Phase Build Log (Every Detail)

### Phase 0 — App Shell + Login ✅

**What was built:**
- Tauri v2 scaffold in `client/` (Vite + React 18 strict + TS)
- Tailwind CSS v4 + shadcn/ui initialized with dark theme tokens
- HashRouter layout with RBAC-gated sidebar nav
- Typed API client (`src/api/client.ts`)
- Login screen with distinct error states
- Dashboard shell wired to real `GET /cases`

**Errors & Mistakes:**
1. **React 19 default:** Vite template installs React 19. Had to manually downgrade: `npm install react@^18 react-dom@^18`. This is easy to miss on fresh clone.
2. **Enum casing mismatch (ISSUE — TEMP fixed):** Backend serialized enums in snake_case (`"cdr"`, `"bank"`, `"phone"`) but contract specified UPPERCASE (`'CDR'`, `'BANK'`, `'PHONE'`). Added `TEMP(interop)` case-insensitive fallback in `src/lib/severity.ts` and `src/lib/timeline-constants.ts`. This was later fixed by IMAAN in `5f55f98` and the TEMP fallback was ripped out in `31e1618`.
3. **WS event tag format:** Backend sent `alert_created` / `ingest_progress` (underscore); API.md specified dotted `alert.created` / `ingest.progress`. IMAAN fixed this in `5f55f98`.
4. **SSE chat frames:** Backend sent `{"type":"delta","delta":...}` but contract said bare `{"delta":...}` → `{"sources":[...]}` → `{"done":true}`. Fixed by IMAAN.
5. **`Report.version` type:** Backend sent string (`"v0.1-draft"`), contract said `number`. IMAAN fixed to numeric.
6. **`AuditEntry` missing from contract:** Not in `api-types.ts` initially. IMAAN added it.
7. **`/cases` answered 200 without Authorization header:** No auth enforcement on GET routes initially. Fixed in IMAAN's hardening pass (`f5d0055`).
8. **423 lockout untestable initially:** Stub accepted any credentials. Real auth (`a6fed27`) enabled lockout testing.
9. **CRLF warnings on Windows:** Git shows `warning: in the working copy of ..., LF will be replaced by CRLF`. Harmless on Windows but noisy.

**Verification:**
- Login 200 ✅, unauthed 401 ✅, lockout 423 with `retry in {n}s` parsing ✅
- `POST /users` ✅, create→list→detail roundtrip ✅
- `npm run build` clean (tsc + vite) ✅

---

### Phase 1 — Cases + Dashboard ✅

**What was built:**
- Cases table with search, status filter
- Create-case modal (RBAC-gated: Investigator/Admin only)
- Case detail page with tab frame: Timeline | Graph | Map | Alerts | Ingest | Reports | Chat
- Dashboard wired to real `GET /cases` stats

**Errors & Mistakes:**
1. **Post-login navigation broken:** After login, the app didn't redirect properly because `signIn`/`signOut` weren't transitioning auth status. Fixed by adding explicit status transitions in `AuthContext.tsx` (`7a0453f`).
2. **Error boundary missing:** A render error in any screen crashed the whole app. Added `src/components/error-boundary.tsx` and wrapped the root in `main.tsx` (`7a0453f`).

**Verification:**
- Create case from UI → visible in list → detail loads ✅
- All HTTP paths proven against real backend ✅

---

### Phase 2 — Timeline ✅ (feature-complete except 100k jank gate)

**What was built:**
- Virtualized infinite-scroll timeline (`@tanstack/react-virtual`, 200/page limit-offset)
- Filters bar: source_type, event_type, from, to, entity_id — all verified live
- Collapsible event groups (temporal clustering: 5m/15m/1h/24h)
- Event detail drawer with full metadata + raw JSON viewer
- Event annotation input (`POST /events/:id/notes`) — RBAC-gated to Investigator/Admin
- Side-by-side suspect comparison mode (A/B panes with entity selectors)

**Errors & Mistakes:**
1. **TEMP(interop) enum fallback:** Timeline filters initially used case-insensitive matching because backend enums were snake_case. When IMAAN fixed enum casing (`5f55f98`), I ripped out the TEMP fallback in `31e1618`. If you see timeline filters returning empty results, check that the backend enums match the contract UPPERCASE format.
2. **Entity profile panel name mismatch:** IMAAN's `/entities/:id/profile` response had `display_name` but the graph panel initially expected `name`. Checked the actual response shape and used the correct field. Always verify against the real API response, not the contract — the contract may not capture optional fields correctly.

**Verification:**
- 100k CDR ingested via real API, deep-offset pagination correct ✅
- Filter+offset combos return correct pages ✅
- 100k-event jank gate: **NOT YET TESTED by human** — needs someone scrolling on real hardware

---

### Phase 3 — Ingest UI ✅ / Alerts — BLOCKED then UNBLOCKED

**What was built (ingest):**
- RBAC-gated "Ingest" tab on case detail
- Drag-and-drop upload → sequential `POST /cases/:id/ingest`
- WS `ingest.progress` live via `src/api/ws.ts` manager
- 1.5s `/ingest/jobs/:id` poll fallback
- Expandable row-level parse error list

**Errors & Mistakes:**
1. **WS `?token=` auth:** Browsers cannot set headers on WebSocket connections. The WS manager connects with `?token=<jwt>` as a query parameter. IMAAN's handler supports this fallback. Verified live: subscribe frame accepted.
2. **Ingest progress too fast:** Real parse finishes in ~1 second for small files. Agreed with IMAAN to enforce minimum visible progress animation when frames arrive too fast (smooth UX).

**What was built (alerts) — JUST NOW:**
- `src/api/alerts.ts` — API client (listAlerts, getAlert, triageAlert, analyzeCase)
- `alerts-screen.tsx` — **PLACEHOLDER, NOT YET REBUILT** against real endpoints
- IMAAN's anomaly engine landed (`f4c44c5`), now unblocked

---

### Phase 4 — Graph ✅ / Map ✅ (partial — needs real trail data)

**What was built (graph):**
- D3 force-directed graph component (`components/graph/force-graph.tsx`)
- Type-colored nodes, log-scaled edge widths from `evidence_count`, dashed non-high tiers
- Hover-neighbor dimming, drag+zoom+pan, hop selector (1-3)
- BFS subgraph focus input, click node → entity profile side panel
- Re-resolve button (`POST /cases/:id/resolve`) with cache invalidation

**Errors & Mistakes:**
1. **SVG performance concern:** At hops=2 with 5,100 edges, SVG renders fine. For larger graphs (demo cases), IMAAN suggested pre-aggregating subgraphs server-side rather than pushing canvas rendering. Parked for now.

**What was built (map):**
- Pure Leaflet component (`components/map/movement-map.tsx`) — NO react-leaflet wrapper
- Per-entity colored polylines with chronological point markers
- Hover tooltips (entity + timestamp), auto-fit bounds
- Animated playback slider sweeping trail through time
- Tile layer from `VITE_TILE_URL` env var (OSM default, offline-ready for demo)

**Errors & Mistakes:**
1. **Leaflet dependency missing (ISSUE #2):** `leaflet` and `@types/leaflet` were added to `package.json` but `npm install` was never run after adding them. Fresh clone → blank white screen, Vite errors with `Failed to resolve import "leaflet"`. IMAAN caught this and fixed it in `b5dc345`. Fix: `cd client && npm install`.
2. **`package-lock.json` not committed after leaflet add:** The lockfile wasn't updated when leaflet was first added, so other people pulling fresh didn't get the dependency. Fixed by IMAAN's commit.
3. **No react-leaflet:** Deliberate choice. React-leaflet adds complexity and the component tree is simple enough to use Leaflet directly. The ref-based approach in `movement-map.tsx` creates/destroys the map in useEffect.

**Verification:**
- Graph: 86 nodes, 5,100 edges from real resolution data ✅
- Graph: hot IMEI hub (evidence=103) connects 60 SIMs — demo money-shot ✅
- Map: component renders ✅, real trail data **pending** IMAAN's Phase 5 geospatial

---

### Phase 5 — Not Started (Chat, Reports, Settings, Audit)

All placeholders exist. Chat + reports depend on SSE streaming + report endpoints (contract exists, implementation pending).

---

### Additional Shipped Features

**Timeline Comparison Mode (Phase 2 extension):**
- Compare toggle in timeline toolbar
- A/B panes, each with entity selector from `GET /cases/:id/entities`
- Shared filter bar (source/event type/date range)
- Per-pane virtualized infinite feeds + event-type breakdown stats

---

## 6. Every Error, Issue, & Mistake — Master Reference

| # | Error/Issue | Where | How It Happened | Fix | Commit |
|---|------------|-------|-----------------|-----|--------|
| 1 | React 19 installed instead of 18 | `package.json` | Vite template defaults to React 19 | `npm install react@^18 react-dom@^18` | `28af6db` |
| 2 | Enum casing snake_case vs UPPERCASE | Stats keys, entity/event payloads | Backend used `serde(rename_all = "snake_case")` | IMAAN added explicit renames matching contract; MIMI had TEMP(interop) fallback | `5f55f98` + `31e1618` |
| 3 | WS event tags underscore vs dotted | `ws.ts` | Backend sent `alert_created` | IMAAN changed to `alert.created` | `5f55f98` |
| 4 | SSE chat frames tagged vs bare | Chat SSE | Backend sent `{"type":"delta",...}` | IMAAN changed to bare `{"delta":...}` | `5f55f98` |
| 5 | Report.version string vs number | `api-types.ts` | Backend sent `"v0.1-draft"` | IMAAN changed to numeric | `5f55f98` |
| 6 | AuditEntry missing from contract | `api-types.ts` | Not included in initial types | IMAAN added it | `5f55f98` |
| 7 | `/cases` 200 without auth header | `routes/cases.rs` | No auth enforcement on GET | IMAAN hardened all routes | `f5d0055` |
| 8 | Post-login navigation broken | `AuthContext.tsx` | signIn/signOut didn't transition status | Added explicit status transitions | `7a0453f` |
| 9 | No error boundary | `main.tsx` | Render error crashed whole app | Added `error-boundary.tsx` | `7a0453f` |
| 10 | `cargo run` ambiguous (ISSUE #1) | `server/Cargo.toml` | Two binaries (netra-server + gen-synthetic) | Added `default-run = "netra-server"` | `be5e65e` / `571d314` |
| 11 | Leaflet deps missing (ISSUE #2) | `package.json` | leaflet added to package.json but npm install never run | IMAAN ran npm install + committed lockfile | `b5dc345` |
| 12 | `cargo run --bin` workaround | Terminal | Until ISSUE #1 fix | Use `cargo run --bin netra-server` | Temporary |
| 13 | CRLF warnings on Windows | Git output | Windows line endings | Harmless, ignored | — |
| 14 | TEAM_PROGRESS.md merge conflicts | Git merge | Both agents edited chat log | Manual resolution, keep both entries, newest-at-top | Multiple |

---

## 7. Complete File Inventory

### `client/src/` — Application Code

```
src/
├── main.tsx                          # Providers: QueryClient, AuthProvider, HashRouter, Toaster, ErrorBoundary
├── App.tsx                           # Routes definition
├── index.css                         # Dark forensic theme tokens, severity palette, Tailwind imports
├── vite-env.d.ts                     # Vite types
│
├── api/                              # Backend communication layer
│   ├── client.ts                     # Single fetch wrapper: Bearer injection, ApiError parsing, 401 redirect
│   ├── types.ts                      # Re-export barrel: export * from "@contracts/api-types"
│   ├── auth.ts                       # login(username, password), logout()
│   ├── cases.ts                      # listCases(), getCase(id), createCase(data)
│   ├── events.ts                     # listEvents(caseId, params), addEventNote(eventId, note)
│   ├── entities.ts                   # getCaseEntities(caseId)
│   ├── graph.ts                      # getGraph(caseId, params), getEntityProfile(id), resolveCase(caseId)
│   ├── geo.ts                        # getMovements(caseId, params)
│   ├── ingest.ts                     # uploadFile(caseId, file), getIngestJob(jobId)
│   ├── alerts.ts                     # listAlerts(params), getAlert(id), triageAlert(id, payload), analyzeCase(caseId)
│   └── ws.ts                         # WebSocket manager: ?token= auth, auto-reconnect, re-subscribe, dotted events
│
├── lib/                              # Shared utilities
│   ├── env.ts                        # API_BASE_URL, WS_URL from VITE_ env vars
│   ├── secureStore.ts                # tauri-plugin-store wrapper + localStorage fallback
│   ├── rbac.ts                       # ROLE_PERMISSIONS matrix, can(user, permission) function
│   ├── severity.ts                   # SEVERITY_COLORS, SOURCE_COLORS, sourceBadgeClass maps
│   ├── timeline-constants.ts         # SOURCE_TYPES, EVENT_TYPES arrays with labels + colors
│   └── utils.ts                      # cn() merge-refs helper (shadcn)
│
├── auth/                             # Authentication
│   └── AuthContext.tsx               # AuthProvider, useAuth(), can() — RBAC-gated permissions
│
├── components/                       # Reusable components
│   ├── error-boundary.tsx            # React error boundary with retry
│   ├── layout/
│   │   └── app-shell.tsx             # Sidebar nav + <Outlet/> — RBAC-gated menu items
│   ├── ui/                           # shadcn/ui primitives (alert, badge, button, card, dialog, input, label, separator, sheet, skeleton, sonner, table, tabs, textarea)
│   ├── timeline/
│   │   ├── timeline-panel.tsx        # Main timeline: virtualized list, filters, grouping, comparison toggle, event drawer
│   │   └── compare-pane.tsx          # Single pane for comparison mode: entity selector, virtualized feed, breakdown
│   ├── graph/
│   │   ├── force-graph.tsx           # D3 force-directed graph: type colors, log-scaled edges, interactions
│   │   ├── graph-panel.tsx           # Graph orchestrator: React Query, hop selector, BFS input, re-resolve
│   │   └── entity-profile-panel.tsx  # Side panel: entity details, stats, connections list from /entities/:id/profile
│   ├── map/
│   │   ├── movement-map.tsx          # Pure Leaflet: polylines, point markers, auto-fit bounds
│   │   └── map-panel.tsx             # Map orchestrator: React Query, entity context, playback slider, refresh
│   └── ingest/
│       └── ingest-panel.tsx          # Drag-drop upload, WS progress, poll fallback, error list
│
└── screens/                          # Route-level screens
    ├── login-screen.tsx              # Login form: 401/423/network states, countdown timer
    ├── dashboard-screen.tsx          # KPI grid from real GET /cases
    ├── cases-screen.tsx              # Cases table + create-case modal
    ├── case-detail-screen.tsx        # Case header + stats + tab frame (Timeline/Graph/Map/Alerts/Ingest/Reports/Chat)
    ├── alerts-screen.tsx             # PLACEHOLDER — needs rebuild against real GET /alerts
    ├── reports-screen.tsx            # PLACEHOLDER
    ├── settings-screen.tsx           # PLACEHOLDER
    ├── audit-screen.tsx              # PLACEHOLDER
    └── placeholder-screen.tsx        # Reusable placeholder with title/description/phase
```

### Root Config Files
```
client/
├── package.json                      # Dependencies (react 18, leaflet, d3, etc.)
├── package-lock.json                 # Lockfile — MUST be committed after any dep change
├── tsconfig.json                     # Strict TS, path aliases @/ and @contracts/
├── vite.config.ts                    # Vite config with aliases
├── index.html                        # HTML entry point
├── src-tauri/                        # Tauri v2 native config
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       └── lib.rs                    # Tauri plugin registrations
└── components.json                   # shadcn/ui config
```

### `contracts/` — Shared Types
```
contracts/
└── api-types.ts                      # ALL TypeScript types: Case, Event, Entity, GraphNode, GraphEdge,
                                      # Alert, IngestJob, GeoPoint, Report, AuditEntry, User, ApiError, etc.
```

### `docs/` — Documentation
```
docs/
├── NETRA_PRD.md                      # Full product requirements (535 lines)
├── API.md                            # Frozen REST/WS/SSE contract (155 lines)
├── PLAN_FRONTEND.md                  # Phase-by-phase plan with ticks
├── PLAN_BACKEND.md                   # IMAAN's plan
├── TEAM_PROGRESS.md                  # Shared whiteboard (chat log + status board)
├── PROMPT_FRONTEND_AGENT.md          # Original kickoff prompt for MIMI
└── PROMPT_BACKEND_AGENT.md           # Original kickoff prompt for IMAAN
```

### `server/` — Backend (IMAAN's domain)
```
server/
├── Cargo.toml                        # default-run = "netra-server" (ISSUE #1 fix)
├── migrations/
│   ├── 0001_initial.sql
│   ├── 0002_enrichments.sql
│   └── 0003_alerts_summary.sql       # Adds summary column to alerts
├── src/
│   ├── main.rs                       # Server entry point
│   ├── state.rs                      # AppState (DB pool, settings)
│   ├── db.rs                         # Database initialization
│   ├── models.rs                     # Data models
│   ├── stub_data.rs                  # Seed data for dev
│   ├── anomaly.rs                    # Anomaly detection engine (4 rules)
│   ├── bin/
│   │   ├── netra-server              # Main server binary
│   │   └── gen-synthetic.rs          # Synthetic data generator
│   └── routes/
│       ├── mod.rs
│       ├── alerts.rs                 # Alert CRUD + triage
│       ├── ingest.rs                 # File upload + ingest jobs
│       └── ...                       # Other route modules
```

---

## 8. API Contract Summary (What's Available)

### Auth
| Method | Path | Notes |
|--------|------|-------|
| POST | `/auth/login` | Returns `{token, expires_at, user}` |
| POST | `/auth/logout` | 204, invalidates token |

### Cases
| Method | Path | Notes |
|--------|------|-------|
| GET | `/cases` | Role-scoped list with stats |
| POST | `/cases` | Create (Investigator/Admin) |
| GET | `/cases/:id` | Detail + stats |
| PATCH | `/cases/:id` | Update title/status/tags/assignees |
| GET | `/cases/:id/audit` | Audit log (max 200) |

### Events & Timeline
| Method | Path | Notes |
|--------|------|-------|
| GET | `/cases/:id/events` | Unified timeline. Filters: source_type, event_type, entity_id, from, to, limit, offset |
| GET | `/events/:id` | Single event + raw record |
| POST | `/events/:id/notes` | Append note (Investigator/Admin) |

### Entities & Graph
| Method | Path | Notes |
|--------|------|-------|
| GET | `/cases/:id/entities` | All resolved entities + link tiers |
| GET | `/cases/:id/graph` | D3-ready `{nodes, edges}`. Query: entity_id, hops (default 2) |
| POST | `/cases/:id/resolve` | Re-run resolution (Admin/Investigator) |
| GET | `/entities/:id/profile` | Entity + stats + connections |
| PATCH | `/entities/:id` | Tag/annotate (Investigator/Admin) |

### Alerts
| Method | Path | Notes |
|--------|------|-------|
| GET | `/alerts` | Cross-case list. Query: case_id, severity, status |
| GET | `/alerts/:id` | Detail + entity_ids + evidence_event_ids + summary |
| PATCH | `/alerts/:id/status` | Triage: `{status, note?}` |
| POST | `/cases/:id/analyze` | Manual re-run (Admin/Investigator) |

### Ingestion
| Method | Path | Notes |
|--------|------|-------|
| POST | `/cases/:id/ingest` | Multipart upload → `{job_id}` (202) |
| GET | `/ingest/jobs/:id` | Status + errors |

### Geospatial
| Method | Path | Notes |
|--------|------|-------|
| GET | `/cases/:id/movements` | Trails: `{trails: [{entity_id, points: [GeoPoint]}]}` |

### WebSocket
- Connect: `ws://127.0.0.1:8420/ws?token=<jwt>` (browser fallback — can't set headers)
- Subscribe: `{"type":"subscribe","topics":["case:UUID","global"]}`
- Events: `ingest.progress`, `alert.created`

### Seeded Credentials (IMAAN's dev server)
- **Admin:** `admin` / `netra-admin`
- **Seeded case ID:** `11111111-1111-1111-1111-111111111111`
- **My 100k case:** `c8faf192-8bdc-4dfe-bbdf-a248772ce26d`

---

## 9. Current State (What's Done vs What's Left)

### ✅ DONE
| Feature | Status | Verified Against |
|---------|--------|-----------------|
| App shell + sidebar + routing | Complete | Real backend |
| Login with error states | Complete | Real backend (401/423/lockout) |
| Dashboard KPIs | Complete | Real `GET /cases` |
| Cases list + create + detail | Complete | Real backend |
| Timeline with virtualized scroll | Complete | Real events API (100k) |
| Timeline filters | Complete | All 5 params verified |
| Timeline temporal clustering | Complete | 5m/15m/1h/24h |
| Event detail drawer + raw JSON | Complete | Real events |
| Event annotations | Complete | Real `POST /events/:id/notes` |
| Timeline comparison mode | Complete | Entity list from real API |
| Ingest drag-drop + WS progress | Complete | Real upload + WS |
| Parse error display | Complete | Real ingest errors |
| D3 force graph | Complete | 86 nodes / 5,100 edges |
| Graph interactions (hop, BFS, profile) | Complete | Real `/entities/:id/profile` |
| Leaflet map + playback | Complete | Component works, real trails pending |
| Error boundary | Complete | — |
| WS manager with reconnection | Complete | Verified with `?token=` |
| Alert API client | Complete | Contract shapes match |

### 🔄 IN PROGRESS
| Feature | Status | Blocker |
|---------|--------|---------|
| Merge IMAAN's Phase 4 | Conflict resolved, needs commit | TEAM_PROGRESS.md merge |
| Leaflet deps fix | npm installed, needs commit | package.json committed by IMAAN |
| Alert Center screen | Placeholder, needs rebuild | **NONE** — endpoints are live |

### ❌ NOT DONE
| Feature | Phase | Blocker |
|---------|-------|---------|
| Alert Center UI | 3 | None (unblocked) |
| Alert detail with evidence links | 3 | None |
| Triage workflow (Confirmed/False Positive) | 3 | None |
| Toast on WS `alert.created` | 3 | None |
| Heatmap + alert markers on map | 4 | None (but low priority) |
| 100k jank gate (human scroll test) | 2 | Needs human on dev machine |
| Copilot chat panel (SSE streaming) | 5 | Backend implementation pending |
| Report viewer + approve + export | 5 | Backend implementation pending |
| Settings screens | 5 | Backend implementation pending |
| Audit log viewer | 5 | Backend implementation pending |
| Offline tile bundling for demo | Demo | Someone needs to download Punjab/Haryana tiles |

---

## 10. How to Start the Dev Environment

### Backend (IMAAN's server)
```bash
cd server
cargo run --bin netra-server
# Server starts on http://127.0.0.1:8420
# Seeded creds: admin / netra-admin
```

If `cargo run` is ambiguous (ISSUE #1), use:
```bash
cargo run --bin netra-server
```

### Frontend
```bash
cd client
npm install              # IMPORTANT: run after any fresh clone or dep change
npm run dev              # Vite dev server + Tauri window
```

If blank screen after fresh clone → run `npm install` in `client/` (ISSUE #2).

### Generating Test Data
```bash
cd server
cargo run --bin gen-synthetic -- ../test-data.csv 100000 cdr
# Then ingest via UI (Ingest tab) or API:
# POST /cases/:id/ingest with the CSV file
```

### Running Build Check
```bash
cd client
npm run build           # tsc + vite build — should be clean
```

---

## 11. Known Gotchas & Warnings

1. **React 18, not 19:** The Vite template installs React 19. Always check `package.json` after scaffold.
2. **`npm install` after dep changes:** If you add a package, run `npm install` AND commit `package-lock.json`.
3. **Enum casing:** Backend MUST use UPPERCASE to match `contracts/api-types.ts`. If stats keys look lowercase, IMAAN needs to fix `serde(rename_all)`.
4. **WS auth:** Browsers can't set headers on WebSocket. Use `?token=<jwt>` query param.
5. **HashRouter:** Tauri serves from `file://` protocol. React Router's BrowserRouter breaks on refresh. HashRouter works.
6. **Leaflet CSS:** `movement-map.tsx` imports `leaflet/dist/leaflet.css`. If you see unstyled map tiles, check this import.
7. **`VITE_TILE_URL`:** For offline demo, set this env var to point at bundled local tiles. Default is OSM (needs internet).
8. **D3 force graph at scale:** SVG works fine up to ~5k edges. Beyond that, consider canvas or server-side subgraph aggregation.
9. **Merge conflicts:** Will always happen in `docs/TEAM_PROGRESS.md` when merging IMAAN's branch. Both agents update the chat log. Resolve manually, keep both entries, newest-at-top.
10. **No React Query devtools:** Not installed. Add `@tanstack/react-query-devtools` if debugging cache issues.
11. **Tauri secure store:** Uses `tauri-plugin-store` with localStorage fallback for plain-browser dev. Token persistence differs between Tauri desktop and browser.

---

## 12. Immediate Next Steps for New Agent

### Priority 1: Alert Center (unblocked, last major UI piece)
1. Read IMAAN's dispatch in `docs/TEAM_PROGRESS.md` for full API details
2. `src/api/alerts.ts` already exists with `listAlerts`, `getAlert`, `triageAlert`, `analyzeCase`
3. Rebuild `src/screens/alerts-screen.tsx` against real endpoints:
   - Severity filter (critical/high/medium/low tabs or dropdown)
   - Status filter (open/reviewing/confirmed/false_positive)
   - Severity-colored cards (critical=red pulse border)
   - Click to expand: entity list + evidence event IDs + summary
   - Triage buttons: Confirmed / False Positive with optional note
   - Real-time arrival via WS `alert.created`
4. Add Alert tab content to `case-detail-screen.tsx` (currently shows placeholder)
5. Run `npm run build` to verify

### Priority 2: Merge Pending Work
1. Resolve the `TEAM_PROGRESS.md` conflict from IMAAN's Phase 4 merge
2. Commit the merge + leaflet deps fix
3. `git push`

### Priority 3: Alert-Related Map Markers
4. Add alert marker overlay to `movement-map.tsx` when alerts have location data
5. Add heatmap layer for activity concentration

### Priority 4: Phase 5 Screens (lower priority)
6. Chat panel (SSE streaming) — if backend implements it
7. Report viewer — if backend implements it
8. Settings + Audit screens — if backend implements them

### Priority 5: Demo Prep
9. Bundle offline Punjab/Haryana tiles for `VITE_TILE_URL`
10. Seed test data on demo machine
11. Full end-to-end walkthrough rehearsal

---

## 13. Files That Were Actively Edited (Most Churn)

These files have the most edits and are most likely to have subtle state issues:

| File | Changes | Notes |
|------|---------|-------|
| `src/screens/case-detail-screen.tsx` | 8+ edits | Tab frame, wires all panels, imports keep growing |
| `src/components/timeline/timeline-panel.tsx` | 10+ edits | Biggest component — filters, virtualization, grouping, drawer, comparison toggle |
| `src/components/graph/force-graph.tsx` | 6+ edits | D3 imperative code, careful with cleanup |
| `src/auth/AuthContext.tsx` | 5+ edits | RBAC matrix, status transitions, login/logout |
| `src/api/client.ts` | 4+ edits | Fetch wrapper, error parsing, base URL |
| `docs/TEAM_PROGRESS.md` | 15+ edits | Constantly updated chat log, merge conflict magnet |

---

## 14. Lessons Learned

1. **Always verify against real API, not contract shapes.** The contract may be right but the implementation may differ (enum casing, optional fields, envelope wrapping).
2. **Commit `package-lock.json` after every dep change.** Missing lockfile = blank screen for others.
3. **TEMP(interop) fallbacks are dangerous.** They hide real bugs. Rip them out as soon as the other side fixes the issue.
4. **Merge `agent/backend` into your branch frequently.** The longer you wait, the worse the TEAM_PROGRESS.md conflicts get.
5. **WS in browsers needs `?token=` fallback.** This is non-obvious and not documented in most WebSocket tutorials.
6. **HashRouter is mandatory for Tauri.** BrowserRouter breaks on refresh with `file://` protocol.
7. **React 18 vs 19 matters.** Some hooks behave differently. Pin explicitly.
8. **Virtualized lists need scroll container measurement.** The `getScrollElement` callback must return the actual scrollable div, not a parent.
9. **D3 force simulation needs cleanup.** Always call `simulation.stop()` in useEffect cleanup to prevent memory leaks.
10. **Leaflet map must be destroyed properly.** Call `map.remove()` in useEffect cleanup. Otherwise you get ghost maps.

---

## 15. Demo Day Checklist

- [ ] Alert Center fully wired to real endpoints
- [ ] Offline Punjab/Haryana tiles bundled + `VITE_TILE_URL` set
- [ ] Fresh seeded DB with 100k CDR + 5k bank data
- [ ] End-to-end flow: login → create case → ingest → alerts → timeline → graph → map → triage
- [ ] No console errors
- [ ] `npm run build` clean
- [ ] Demo machine has Rust toolchain + Node.js + npm
- [ ] `cd server && cargo run --bin netra-server` starts cleanly
- [ ] `cd client && npm install && npm run dev` starts cleanly
- [ ] Prepared answers for judges' questions about air-gapped architecture

---

*This document is the single source of truth for frontend state. If you're reading this as the new agent, start here, then read `docs/PLAN_FRONTEND.md` for the ticked plan, and `docs/TEAM_PROGRESS.md` for the latest agent communication.*
