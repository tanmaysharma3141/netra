# NETRA — Frontend Plan (Tanmay)

Timeline: 26 Aug → 7 Sep pre-hack, then hackathon day 8 Sep.
Contract source of truth: `docs/API.md` + `contracts/api-types.ts`. Import types directly from `contracts/` — never redefine shapes locally.
You develop against Chirag's stub server until each phase gate flips to real data.

---

## Phase 0 — App Shell + Login (26–27 Aug)
- [x] Tauri v2 + Vite + React + TS scaffold in `client/` (React 18.3, strict mode, `@/` + `@contracts/` aliases)
- [x] Tailwind CSS + shadcn/ui initialized (dark forensic-console theme: near-black palette, severity tokens critical/high/medium/low, Geist Mono for IDs)
- [x] React Router: layout with sidebar nav (Dashboard, Cases, Alerts, Settings, Audit) — HashRouter, RBAC-gated per PRD §5.2
- [x] API client module: fetch wrapper with JWT header injection + 401 redirect to login (`src/api/client.ts`, ApiError parsing, no `any`)
- [x] Login screen against stub `/auth/login`; store token in Tauri secure store (`tauri-plugin-store`; distinct 401/423/network states; localStorage fallback only in plain-browser dev)
- [x] Dashboard shell with KPI card grid — wired LIVE to stub `GET /cases` (severity + source cards, case rows; skeleton/error+retry/empty states)
- [x] **Gate:** login flow works end-to-end against stubs — verified vs IMAAN's server on :8420 (login 200 / bad-creds 401 / `/cases` shaped ✅); strict tsc + vite build clean

## Phase 1 — Cases + Dashboard Real Data (28–29 Aug)
- [x] Cases list: table with search, status filter, role-scoped visibility (`src/screens/cases-screen.tsx`; server-side scoping via IMAAN's RBAC guards, client search over title/tags/ID)
- [x] Create-case modal (Investigator/Admin only — hide for other roles) — gated by `can("case.create")`, toast feedback, invalidates cache
- [x] Case detail page with tab frame: Timeline | Graph | Map | Alerts | Reports | Chat (`case-detail-screen.tsx`; tabs placeholdered per plan phases, real header + stats strip from `/cases/:id`)
- [x] Dashboard wired to real `/cases` stats: KPI cards, recent cases (anomaly trend chart deferred to Phase 2+ when events exist; severity/source KPIs live now)
- [x] Empty states designed (no cases / no matches / no alerts variants)
- [x] **Gate:** create a case from UI and see it in the list — verified vs real backend (`POST /cases` → `93f7ab75…` visible in `GET /cases`, detail loads; manual mouse-through pending on dev machine, all HTTP paths proven)

## Phase 2 — Timeline (30 Aug – 1 Sep)
- [x] Unified timeline view from `/cases/:id/events`: virtualized list (`@tanstack/react-virtual`, dynamic row measurement; components/timeline/timeline-panel.tsx)
- [x] Filters bar: source type, event type, date range, entity search — all four contract params verified live vs server
- [x] Collapsible event groups (cluster events within configurable window) — flat/5m/15m/1h/24h toggles, per-cluster collapse
- [ ] Event detail drawer: full metadata + raw record viewer ✅; **annotation input blocked** — no `PATCH /events/:id` in contract and `PATCH /entities/:id` body shape unspecified (flagged to IMAAN); drawer shows existing notes read-only meanwhile
- [x] Pagination/infinite scroll honoring limit/offset contract (200/page, auto-fetch near viewport end)
- [ ] Side-by-side suspect comparison mode (two filtered timelines) — next slice
- [ ] **Gate:** scroll through a 100k-event synthetic case without jank — awaiting IMAAN's synthetic data generator

## Phase 3 — Ingest + Alerts (2–3 Sep)
- [x] Ingest UI: drag-and-drop upload → `POST /cases/:id/ingest` (`components/ingest/ingest-panel.tsx`; RBAC-gated "Ingest" tab on case detail, hidden without `data.upload`)
- [x] Job progress UI from WS `ingest.progress` + poll fallback via `/ingest/jobs/:id` (`src/api/ws.ts` manager: `?token=` auth per contract, auto-reconnect w/ backoff, re-subscribe; WS handshake + dotted `alert.created` frame verified live)
- [x] Parse errors display (row-level error list, capped view with expandable details)
- [ ] Alert Center: cross-case list, severity badges, filter by status/severity — blocked on IMAAN's anomaly engine (Phase 4 backend)
- [ ] Alert detail: evidence events inline-linked to timeline drawer
- [ ] Triage workflow buttons: Confirmed / False Positive / Needs Review (+ optional note)
- [ ] Toast + native desktop notification on WS `alert.created` (respect severity config) — WS path proven; wiring lands with Alert Center
- [ ] **Gate:** upload file from UI, watch progress, triage resulting alerts — upload→progress→events half verified end-to-end (100k CSV through real API); triage half awaits alerts engine

## Phase 4 — Graph + Map Components (4–6 Sep)
- [ ] D3 force-directed graph component on fixture data first (pure component, no API coupling)
- [ ] Node styling by EntityType, edge thickness by evidence_count, dashed edges for medium tier
- [ ] Interactions: click node → profile side panel, hover highlight neighbors, hop depth control
- [ ] Subgraph focus: start from selected entity via `/cases/:id/graph?entity_id=&hops=`
- [ ] Leaflet map: offline tile layer setup (no internet at demo!)
- [ ] Movement trails from `/cases/:id/movements`, animated playback slider over time range
- [ ] Heatmap layer + alert markers overlay
- [ ] Wire both into case detail tabs
- [ ] **Gate:** graph explores real correlation output; map shows a suspect trail offline

## Phase 5 — Chat + Reports + Polish (7 Sep)
- [ ] Copilot chat panel: message list, streaming render from SSE frames
- [ ] Sources display: cited event IDs rendered as clickable links to timeline drawer
- [ ] Report viewer: markdown summary, approve button (Supervisor/Admin), export PDF download link
- [ ] Settings screens: user management table, webhook config form, model version list + promote
- [ ] Audit log viewer (Admin/Supervisor only)
- [ ] Full RBAC sweep: every screen hides/blocks actions per role matrix
- [ ] Loading skeletons + error states everywhere
- [ ] **Gate:** full investigator flow clickable start-to-finish on real backend

---

## Hackathon Day (8 Sep) — frontend lane
- [ ] H12–16: wire any remaining screens to final endpoints as Chirag lands them
- [ ] H16–20: copilot chat polish (streaming UX, typing indicator, citations)
- [ ] H20–22: notification UX polish
- [ ] H22–24: demo walkthrough rehearsal — upload → alerts → graph → map → chat → report
- [ ] Prepare demo laptop: verify offline tiles load, no console errors, fresh seeded DB

## Standing rules
- All data shapes come from `contracts/api-types.ts`
- Every async action has loading + error + empty state before marking done
- No screen marked done until it works against real (not stubbed) data where the gate says so
