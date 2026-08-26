# NETRA — Comprehensive Project Status

> **Last updated:** 26 Aug 2026
> **Author:** IMAAN (backend agent) + Chirag Kumar (human operator)
> **Branch state:** `main` = merged latest, `agent/backend` = backend dev, `agent/frontend` = MIMI's frontend

---

## Table of Contents

1. [What NETRA Is](#1-what-netra-is)
2. [Team & Branching Model](#2-team--branching-model)
3. [Tech Stack](#3-tech-stack)
4. [Git History — Every Commit](#4-git-history--every-commit)
5. [Phase-by-Phase Status](#5-phase-by-phase-status)
6. [Server — Complete Feature Audit](#6-server--complete-feature-audit)
7. [Client — Complete Feature Audit](#7-client--complete-feature-audit)
8. [Contract & Interop](#8-contract--interop)
9. [Every Error, Bug, and Mistake](#9-every-error-bug-and-mistake)
10. [Lessons Learned](#10-lessons-learned)
11. [What's Left To Build](#11-whats-left-to-build)
12. [Demo Readiness](#12-demo-readiness)

---

## 1. What NETRA Is

NETRA (नेत्र — "The Eye") is a fully air-gapped forensic intelligence platform built for the **Chandigarh Police National Hackathon 2026 — Track 6 (DFAP)**. Team name: **BinaryBros**.

**Core purpose:** Ingest telecom CDR/IPDR records, bank statements, and social media exports → correlate entities across domains → detect criminal patterns (IMEI reuse, hawala, rapid transfers) → visualize everything (timeline, relationship graph, movement map) → generate court-ready reports. All on-premises, zero internet.

**Target users:** Indian law enforcement investigators, analysts, supervisors, and admins.

**Modules (Hindi names from PRD):**
| Module | Name | Status |
|--------|------|--------|
| Ingestion Engine | Drishti (दृष्टि) | ✅ Shipped — 284k rec/min |
| Correlation Engine | Sambandh (संबंध) | ✅ Shipped — deterministic pass |
| Anomaly Engine | Sanket (संकेत) | ✅ Shipped — 4 rules live |
| Geospatial Engine | Bhoomi (भूमि) | ❌ Stub — needs OpenCelliD |
| Timeline Engine | Kaal (काल) | ✅ Shipped — full client |
| LLM Copilot | Vivek (विवेक) | ❌ Stub — needs LLM integration |
| Self Fine-Tuning | Guru (गुरु) | ❌ Stub — needs LLM integration |
| Case Management | Pramaana (प्रमाण) | ✅ Shipped |
| Report Engine | Vivaran (विवरण) | ❌ Stub — needs implementation |
| Notification Service | Suuchna (सूचना) | ⚠️ WebSocket only — no desktop/webhook |

---

## 2. Team & Branching Model

### Agents
- **IMAAN** — Backend agent (Rust/Axum server). Operates on `agent/backend` branch.
- **MIMI** — Frontend agent (Tauri v2/React/TS client). Operates on `agent/frontend` branch.
- **Chirag Kumar** — Human operator. Merges to `main`, runs both server and client locally.

### Branches
| Branch | Purpose | Owner |
|--------|---------|-------|
| `main` | Merged, deployable state | Chirag |
| `agent/backend` | Backend development | IMAAN |
| `agent/frontend` | Frontend development | MIMI |
| `fix/leaflet-dep-missing` | Stale fix attempt (closed PR #1) | IMAAN (via Chirag) |

### Communication
Agents communicate via `docs/comms/TEAM_PROGRESS.md` — a shared whiteboard with chat log at the bottom. New messages at the top, signed with agent name.

---

## 3. Tech Stack

### Backend (`server/`)
- **Language:** Rust (edition 2024)
- **Framework:** Axum 0.8 + Tokio async runtime
- **Database:** SQLite via sqlx 0.8 (WAL mode, 8-connection pool, 30s busy timeout)
- **Auth:** bcrypt (cost 12) + JWT (8-hour tokens, HS256)
- **Parsing:** csv 1.3 for ingestion
- **Integrity:** SHA-256 per uploaded file
- **WebSocket:** Axum ws + broadcast::channel(256)
- **Binary:** `netra-server` (default) + `gen-synthetic` (test data generator)
- **Listen:** `127.0.0.1:8420`
- **Body limit:** 1GB (raised from Axum's default 2MB)

### Frontend (`client/`)
- **Shell:** Tauri v2 (desktop app)
- **Build:** Vite 7.3.6
- **Framework:** React 18.3 + TypeScript 5.8 (strict mode)
- **Styling:** Tailwind CSS v4 + shadcn/ui v4 (14 components)
- **Routing:** react-router-dom v6 (HashRouter for Tauri file:// protocol)
- **Data:** @tanstack/react-query v5 (staleTime 30s, retry 1)
- **Virtualization:** @tanstack/react-virtual (timeline)
- **Graph:** D3 v7 (force-directed)
- **Map:** Leaflet 1.9.4
- **Toasts:** Sonner
- **Fonts:** Geist + Geist Mono (self-hosted)
- **Session:** @tauri-apps/plugin-store (encrypted) with localStorage fallback

### Shared
- **Contract:** `contracts/api-types.ts` — single source of truth for all API types
- **API docs:** `docs/API.md` — frozen contract

---

## 4. Git History — Every Commit

All dates are 25-26 Aug 2026. Chronological order (oldest first).

| Hash | Date | Author | Description |
|------|------|--------|-------------|
| `f319ff9` | 25 Aug | Chirag | Initial commit — documentation push (PRD, API, plans) |
| `b83ba80` | 25 Aug | Chirag | Prompt file for Tanmay (frontend agent kickoff) |
| `28af6db` | 25 Aug | Tanmay | Client: scaffold Tauri v2 + Vite + React 18 + Tailwind v4 + shadcn/ui |
| `0ee5077` | 25 Aug | Chirag | Backend: Axum stub server for all API.md routes + WS/SSE live |
| `7b316fa` | 25 Aug | Chirag | Docs: name the agents — IMAAN (backend) & MIMI (frontend) |
| `a6fed27` | 25 Aug | Chirag | Backend: Phase 1 — sqlx migrations, bcrypt+JWT auth with lockout, RBAC guards, real users/cases CRUD |
| `cacf2a1` | 25 Aug | Tanmay | Client: Phase 0 complete — shell, login, RBAC sidebar, API client, live-stub dashboard |
| `5f55f98` | 25 Aug | Chirag | Backend: interop fixes from MIMI review — uppercase enums, dotted WS tags, bare SSE frames, numeric report version, AuditEntry contract |
| `2dfa42a` | 25 Aug | Tanmay | Client: Phase 1 — cases table, create-case modal, case detail tabs; 423 retry parsing |
| `987cc50` | 25 Aug | Chirag | Docs: dispatch to MIMI — pull main, rip TEMP(interop), start Phase 2 timeline |
| `4550688` | 25 Aug | Tanmay | Merge remote-tracking branch 'origin/main' into agent/frontend |
| `31e1618` | 25 Aug | Tanmay | Client: merge main (interop fixes), rip TEMP fallback, Phase 2 timeline core |
| `f5d0055` | 25 Aug | Chirag | Backend: harden Phase 0/1 — WS auth, fresh-role-per-request, last-admin guard, immutable audit triggers |
| `7a0453f` | 25 Aug | Tanmay | Client: fix post-login nav (status transition in signIn/signOut) + error boundary |
| `815f54f` | 25 Aug | Chirag | Backend: Phase 2 ingestion engine — universal CSV parser, domain/operator fingerprints, async jobs w/ WS progress, SHA-256 audit trail, real events API, synthetic data generator |
| `cb4a9be` | 25 Aug | Chirag | Chore: ignore server/data runtime artifacts, untrack test uploads |
| `cd25870` | 25 Aug | Chirag | Docs: Phase 2 ticked, MIMI dispatched — timeline unblocked on real data |
| `3f30ba9` | 25 Aug | Tanmay | Merge remote-tracking branch 'origin/main' into agent/frontend |
| `b23467c` | 25 Aug | Tanmay | Client: Phase 3 ingest UI — drop zone, WS progress + poll fallback, error list; WS manager |
| `86769e6` | 25 Aug | Chirag | Docs: round 1 submitted — mockup + video checked |
| `44b4204` | 25 Aug | Chirag | Backend: event annotations POST /events/:id/notes with RBAC, audit trail; contract updated |
| `bfb6f0a` | 25 Aug | Tanmay | Merge remote-tracking branch 'origin/main' into agent/frontend |
| `d664db8` | 25 Aug | Tanmay | Client: event annotations in timeline drawer via POST /events/:id/notes |
| `39d2a2a` | 26 Aug | Chirag | Backend: Phase 3 entity resolution — deterministic device/communication links, auto-resolve on ingest, real graph+profile endpoints, BFS hops |
| `c164d13` | 26 Aug | Tanmay | Merge remote-tracking branch 'origin/main' into agent/frontend |
| `7809397` | 26 Aug | Tanmay | Client: Phase 4 graph — D3 force graph, hop control, BFS focus, entity profile panel |
| `be5e65e` | 26 Aug | Tanmay | Docs: ISSUE #1 — cargo run ambiguous with two binaries (default-run fix) |
| `571d314` | 26 Aug | Chirag | Backend: fix ISSUE #1 — default-run for cargo run; merge MIMI Phase 4; tick Phase 3 gate |
| `d842e9a` | 26 Aug | Tanmay | Client: Phase 4 map — Leaflet movement trails with playback slider, env-driven offline tiles |
| `4ea0f06` | 26 Aug | Tanmay | Client: timeline suspect comparison mode (A/B panes, per-pane virtualized feeds) |
| `f4c44c5` | 26 Aug | Chirag | Phase 4: anomaly engine + alert triage |
| `b6aa011` | 26 Aug | Chirag | Audit: fix all compiler warnings — zero warnings on cargo build + tsc |
| `32293df` | 26 Aug | Chirag | Docs: update TEAM_PROGRESS with audit completion |
| `807b3a1` | 26 Aug | Chirag | Merge: MIMI timeline A/B mode + Leaflet map into main |
| `83c0632` | 26 Aug | Chirag | Docs: flag leaflet dep issue to MIMI (ISSUE #2) |
| `b5dc345` | 26 Aug | Chirag | Fix: add missing leaflet + @types/leaflet to package.json (ISSUE #2) |

**Total commits:** 35 across all branches
**Development time:** ~36 hours (25-26 Aug 2026)

---

## 5. Phase-by-Phase Status

### Phase 0 — Scaffold + Auth + Login ✅
**Backend (IMAAN):**
- Axum stub server for ALL API.md routes with hardcoded JSON
- WebSocket `/ws` endpoint accepting subscribe frames, echoing fake alerts every 30s
- SSE chat endpoint with canned streaming response
- Health check route + request logging middleware

**Frontend (MIMI):**
- Tauri v2 + Vite + React 18 + Tailwind v4 + shadcn/ui scaffold
- Dark forensic-console theme (near-black, severity tokens, Geist Mono)
- HashRouter with RBAC-gated sidebar (6 routes)
- Typed API client with JWT injection + 401 redirect
- Login screen against stub `/auth/login`
- Dashboard with KPI card grid from stub `GET /cases`
- Tauri secure store for session persistence

**Gate:** Login works end-to-end against stubs. Verified on Tanmay's machine.

### Phase 1 — Real Auth + CRUD ✅
**Backend (IMAAN):**
- SQLCipher setup (deviated: plain SQLite + sqlx migrations shipped; SQLCipher needs custom libsqlite3 build)
- 9 tables: users, cases, events, entities, entity_edges, alerts, audit_log, ingest_jobs, feedback_queue
- Immutable audit triggers (UPDATE/DELETE blocked on audit_log)
- bcrypt auth with 5-fail lockout (15-min), 8-hour JWT tokens
- RBAC middleware: `Authed` extractor + `require(&[Role])` helper
- Real CRUD: /users, /cases (role-scoped visibility)
- Seed script: admin user + demo case "OP-2026-041: Cross-border hawala ring"

**Frontend (MIMI):**
- Cases table with search, status filter, role-scoped visibility
- Create-case modal (RBAC-gated, Investigator/Admin only)
- Case detail page with tab frame (Timeline | Graph | Map | Alerts | Reports | Chat)
- Login screen: distinct 401/403/423/network-error states, 423 retry countdown

**Interop fixes from MIMI's review:**
- Enum values flipped to UPPERCASE on wire (CDR, PHONE, CALL)
- WS event tags changed to dotted format (alert.created)
- SSE chat frames changed to bare format (no type wrapper)
- Report.version changed from string to number
- AuditEntry added to contract

**Gate:** Login screen works against real backend. Verified: admin/netra-admin → JWT, unauthed 401, lockout 423, create→list→detail roundtrip.

### Phase 2 — Ingestion + Timeline ✅
**Backend (IMAAN):**
- Universal CSV parser: delimiter sniffing, encoding detection, column-order agnostic
- Operator detection: Jio/Airtel/BSNL/Vi/MTNL CDR schema fingerprints
- Bank statement schema detection (major Indian banks)
- Async ingest jobs: POST /cases/:id/ingest → job queue → WS ingest.progress
- Benchmark: 284k rec/min (target was 100k/min)
- Synthetic data generator: `cargo run --bin gen-synthetic -- out.csv <rows> cdr|bank`
- Real /cases/:id/events with all contract filters
- SHA-256 audit trail per uploaded file

**Frontend (MIMI):**
- Virtualized infinite-scroll timeline (200/page, @tanstack/react-virtual)
- Filters: source_type, event_type, date range, entity_id — all verified live
- Temporal clustering: flat/5m/15m/1h/24h toggles, collapsible clusters
- Event detail drawer: full metadata + raw JSON + annotation input
- Post-login nav fix + error boundary

**Gate:** 100k CDR ingested, events queryable with filters. Verified at API level.

### Phase 3 — Entity Resolution + Ingest UI ✅
**Backend (IMAAN):**
- Deterministic entity resolution: exact IMEI/MSI/account/IP/handle matches
- Device-sharing + communication edges
- Auto-resolve after every ingest
- Graph endpoints: /cases/:id/entities, /cases/:id/graph (N-hop BFS), /entities/:id/profile
- ISSUE #1 fix: default-run = "netra-server" in Cargo.toml

**Frontend (MIMI):**
- Ingest UI: drag-drop → multipart POST → WS progress + 1.5s poll fallback → error list
- WS manager with auto-reconnect + re-subscribe
- Event annotations wired in timeline drawer

**Gate:** 86 entities, 5,100 edges from 100k CDR. Hot IMEI hub (1 device, 60 subscribers) verified in D3 graph.

### Phase 4 — Anomaly Engine + Alerts + Graph + Map ✅
**Backend (IMAAN):**
- 4 anomaly rules: imei_reuse (critical), hawala_signature (high), rapid_transfer (high), coordinated_silence (medium)
- Alert persistence with summary field
- PATCH /alerts/:id/status → confirmed/false_positive → feedback_queue
- POST /cases/:id/analyze → manual trigger
- WS alert.created push on every analysis run
- E2E verified: 100k CDR + 5k bank → 96 alerts (1 critical, 3 hawala, 92 rapid-transfer)
- SQLite WAL + busy_timeout(30s) for concurrent access
- Full codebase audit: 0 Rust warnings, 0 TS errors

**Frontend (MIMI):**
- D3 force-directed graph: type-colored nodes, log-scaled edge widths, dashed non-high tiers, hover-neighbor dimming, drag+zoom, hop selector (1-3), BFS focus, click-to-inspect entity profile panel
- Re-resolve button wired to POST /cases/:id/resolve
- Leaflet map: per-entity colored polylines, playback slider, auto-fit bounds, VITE_TILE_URL env for offline tiles
- Timeline A/B comparison mode: two panes, per-pane virtualized feeds + event-type breakdown
- Alert Center: cross-case list consuming GET /alerts?case_id=X, severity badges, triage via PATCH /alerts/:id/status

**Gate:** Synthetic case produces 3 distinct alert types. Graph renders 86 nodes / 5,100 edges. Map component built (real trail data pending Phase 5).

### Phase 5 — Geospatial + LLM + Reports ⏳ NOT STARTED
**Backend:** OpenCelliD tower DB bundling, /movements trail assembly, LLM runtime (candle/llama.cpp), RAG pipeline, SSE chat integration, report generation
**Frontend:** Copilot chat panel, report viewer, settings screens, audit log viewer, full RBAC sweep

---

## 6. Server — Complete Feature Audit

### Endpoint Status

**REAL (database-backed): 26 endpoints**

| Method | Path | Handler | Notes |
|--------|------|---------|-------|
| GET | /health | health | No DB ping |
| GET | /ws | ws | Full WebSocket with auth |
| POST | /auth/login | auth | Full lockout logic |
| POST | /auth/logout | auth | Audit only, no token revocation |
| GET | /users | users | Admin-only |
| POST | /users | users | Admin-only, bcrypt |
| PATCH | /users/:id | users | Admin-only, last-admin guard |
| DELETE | /users/:id | users | Admin-only, soft-delete |
| GET | /cases | cases | Role-filtered, live stats |
| POST | /cases | cases | With audit |
| GET | /cases/:id | cases | Role-visibility check |
| PATCH | /cases/:id | cases | Partial update, audit |
| GET | /cases/:id/audit | cases | Admin/Supervisor only |
| GET | /cases/:id/events | events | Dynamic filters, pagination |
| GET | /events/:id | events | Single event |
| POST | /events/:id/notes | events | Append-only notes |
| GET | /cases/:id/entities | entities | Full entity list |
| GET | /cases/:id/graph | entities | N-hop BFS subgraph |
| POST | /cases/:id/resolve | entities | Entity resolution pipeline |
| GET | /entities/:id/profile | entities | Rich profile + connections |
| PATCH | /entities/:id | entities | Tag annotation |
| GET | /alerts | alerts | Dynamic filters |
| GET | /alerts/:id | alerts | Single alert |
| PATCH | /alerts/:id/status | alerts | With feedback_queue + WS |
| POST | /cases/:id/analyze | alerts | Anomaly detection pipeline |
| POST | /cases/:id/ingest | ingest | Multipart upload + full pipeline |
| GET | /ingest/jobs/:id | ingest | Job status |

**STUB (hardcoded/placeholder): 10 endpoints**

| Method | Path | Returns |
|--------|------|---------|
| GET | /cases/:id/movements | Stub trail data, ignores date filters |
| POST | /cases/:id/chat | Canned SSE response, no LLM |
| POST | /cases/:id/reports | Stub report object |
| GET | /cases/:id/reports | Single stub report |
| GET | /reports/:id | Stub report object |
| GET | /reports/:id/export | Fake 28-byte PDF |
| PATCH | /reports/:id/approve | Stub with approved_by set |
| GET | /settings/webhooks | All None |
| PATCH | /settings/webhooks | No-op 204 |
| GET | /models | Hardcoded model list |
| POST | /models/promote | WS event only |
| POST | /training/trigger | Fake 3-epoch simulation |
| GET | /training/queue | Hardcoded queue info |

### Core Modules

| Module | File | Lines | Status |
|--------|------|-------|--------|
| Entry point | main.rs | 84 | Complete |
| App state | state.rs | 31 | Complete |
| Data models | models.rs | 527 | Complete (all enums, structs, WS types) |
| Database | db.rs | 184 | Complete (WAL, migrations, seeding) |
| Auth | auth.rs | 234 | Complete (bcrypt, JWT, lockout, RBAC) |
| Entity resolution | resolve.rs | 215 | Complete (CDR only, deterministic pass) |
| Anomaly engine | anomaly.rs | 362 | Complete (4 rules) |
| Stub data | stub_data.rs | 251 | Complete (demo data for all stubs) |
| Ingestion parser | ingest/mod.rs | 249 | Complete |
| Schema detection | ingest/detect.rs | 224 | Complete |
| Synthetic generator | bin/gen-synthetic.rs | 155 | Complete |

### Database Schema

3 migrations, 9 tables:

| Table | Rows (100k test) | Purpose |
|-------|-------------------|---------|
| users | 3 | User accounts with lockout fields |
| cases | 2 | Investigation cases |
| events | ~105,000 | Ingested records (CDR + bank) |
| entities | 86 | Resolved entity nodes |
| entity_edges | 5,100 | Graph edges between entities |
| alerts | 96 | Anomaly detections |
| audit_log | ~500 | Immutable audit trail |
| ingest_jobs | 2 | Upload/parse tracking |
| feedback_queue | 0 | ML feedback (empty until triage) |

Triggers: audit_log is append-only (UPDATE + DELETE blocked).

### Background Tasks

- **WS ticker:** Publishes demo alert every 30s (debug artifact, should be removed before production)
- **Auto-pipeline:** After successful ingest → entity resolution → anomaly analysis → publish alerts via WS
- **Pipeline lock:** `Arc<tokio::sync::Mutex<()>>` prevents concurrent resolve/analyze on same case

---

## 7. Client — Complete Feature Audit

### Screen Status

| Screen | Route | Status | API Endpoints Hit |
|--------|-------|--------|-------------------|
| Login | /login | ✅ REAL | POST /auth/login, POST /auth/logout |
| Dashboard | / | ✅ REAL | GET /cases |
| Cases | /cases | ✅ REAL | GET /cases, POST /cases |
| Case Detail | /cases/:id | ✅ REAL | GET /cases/:id |
| → Timeline tab | | ✅ REAL | GET /cases/:id/events, GET /cases/:id/entities, POST /events/:id/notes |
| → Graph tab | | ✅ REAL | GET /cases/:id/graph, GET /entities/:id/profile, POST /cases/:id/resolve |
| → Map tab | | ✅ REAL | GET /cases/:id/movements |
| → Ingest tab | | ✅ REAL | POST /cases/:id/ingest, GET /ingest/jobs/:id, WS ingest.progress |
| → Alerts tab | | ⚠️ PLACEHOLDER | (ships in Alert Center) |
| → Reports tab | | ❌ PLACEHOLDER | |
| → Chat tab | | ❌ PLACEHOLDER | |
| Alerts | /alerts | ✅ REAL (MIMI shipped) | GET /alerts?case_id=X, PATCH /alerts/:id/status |
| Reports | /reports | ❌ PLACEHOLDER | |
| Settings | /settings | ❌ PLACEHOLDER | |
| Audit | /audit | ❌ PLACEHOLDER | |

### Component Inventory

| Module | Components | Status |
|--------|-----------|--------|
| Layout | app-shell (sidebar + outlet) | ✅ Real |
| Timeline | timeline-panel, compare-pane | ✅ Real |
| Graph | graph-panel, force-graph, entity-profile-panel | ✅ Real |
| Map | map-panel, movement-map | ✅ Real |
| Ingest | ingest-panel | ✅ Real |
| UI (shadcn) | 14 components (alert, badge, button, card, dialog, input, label, separator, sheet, skeleton, sonner, table, tabs, textarea) | ✅ All |
| Other | error-boundary | ✅ Real |

### API Layer

| File | Purpose | Status |
|------|---------|--------|
| api/client.ts | Fetch wrapper with JWT injection, 401 redirect, error parsing | ✅ Production-grade |
| api/auth.ts | login/logout | ✅ Real |
| api/cases.ts | listCases, createCase, getCase | ✅ Real |
| api/events.ts | listEvents (with filters), addEventNote | ✅ Real |
| api/entities.ts | getCaseEntities | ✅ Real |
| api/graph.ts | getGraph, getEntityProfile, resolveCase | ✅ Real |
| api/geo.ts | getMovements | ✅ Real |
| api/ingest.ts | uploadFile, getIngestJob | ✅ Real |
| api/types.ts | Re-exports from contracts/api-types | ✅ Single source of truth |
| api/ws.ts | WebSocket client with auto-reconnect, topic pub/sub | ✅ Production-grade |

### Auth System

| Component | Status |
|-----------|--------|
| AuthContext | ✅ Real — restoring → authenticated / unauthenticated |
| RBAC (lib/rbac.ts) | ✅ 4-role permission matrix |
| Secure Store (lib/secureStore.ts) | ✅ Tauri encrypted + localStorage fallback |

### RBAC Permissions

| Permission | Admin | Supervisor | Investigator | Analyst |
|-----------|-------|------------|--------------|---------|
| case.create | ✅ | ❌ | ✅ | ❌ |
| data.upload | ✅ | ❌ | ✅ | ❌ |
| cases.viewAll | ✅ | ✅ | ❌ | ❌ |
| analysis.run | ✅ | ❌ | ✅ | ✅ |
| report.generate | ✅ | ❌ | ✅ | ❌ |
| report.approve | ✅ | ✅ | ❌ | ❌ |
| users.manage | ✅ | ❌ | ❌ | ❌ |
| training.trigger | ✅ | ❌ | ❌ | ❌ |
| webhooks.manage | ✅ | ❌ | ❌ | ❌ |
| audit.view | ✅ | ✅ | ❌ | ❌ |

---

## 8. Contract & Interop

### Contract Source of Truth
- `contracts/api-types.ts` — TypeScript types (130 lines)
- `docs/API.md` — REST/WS/SSE contract (155 lines)

### Change Process
Any contract change = PR touching BOTH files. No verbal agreements.

### Interop Issues Found & Fixed

| # | Issue | Found By | Fix |
|---|-------|----------|-----|
| 1 | Enum values serialized as snake_case (`"cdr"`) but contract says UPPERCASE (`'CDR'`) | MIMI | Backend: `str_enum!` macro with uppercase wire values |
| 2 | WS event tags came as `alert_created` but contract says `alert.created` | MIMI | Backend: changed to dotted format |
| 3 | SSE chat frames wrapped in `{"type":"delta","delta":...}` but contract says bare `{"delta":...}` | MIMI | Backend: removed type wrapper |
| 4 | `Report.version` was string `"v0.1-draft"` but contract says number | MIMI | Backend: changed to numeric |
| 5 | `AuditEntry` type not in `api-types.ts` | MIMI | Added to contract |

---

## 9. Every Error, Bug, and Mistake

### ISSUE #1 — `cargo run` broken with two binaries
- **Reported by:** Tanmay (via MIMI's chat log)
- **Problem:** After adding `gen-synthetic` binary, `cargo run` in `server/` failed with "could not determine which binary to run"
- **Fix:** Added `default-run = "netra-server"` to `server/Cargo.toml` under `[package]`
- **Prevention:** Always set `default-run` when adding multiple binaries to a Cargo workspace

### ISSUE #2 — Leaflet dependency missing (blank white screen)
- **Reported by:** Chirag (hit locally)
- **Problem:** `leaflet` and `@types/leaflet` were in `package.json` on `main` but NOT on `agent/backend`. Fresh clone + `npm install` → blank white screen, Vite error: `Failed to resolve import "leaflet"`
- **Root cause:** MIMI added leaflet import in `movement-map.tsx` and added deps to `package.json`, but the `package.json` change was only merged to `main`, not present on `agent/backend` branch
- **Fix:** Added `leaflet` and `@types/leaflet` to `package.json` on `agent/backend`, ran `npm install`
- **Lesson:** When adding npm dependencies, ensure they're committed to the working branch before merging

### Axum's Hidden 2MB Body Limit
- **Discovered by:** IMAAN during Phase 2 ingestion
- **Problem:** Large CSV uploads silently failed — Axum defaults to 2MB body limit
- **Fix:** Added `DefaultBodyLimit::max(1024 * 1024 * 1024)` (1GB) to the router
- **Lesson:** Always set explicit body limits when dealing with file uploads

### SQLite `value` Keyword Conflict
- **Discovered by:** IMAAN during Phase 2
- **Problem:** `value` is a reserved keyword in SQLite. Column named `value` in events table caused query errors
- **Fix:** Quoted the column name in all SQL queries
- **Lesson:** Avoid SQLite reserved keywords in column names

### `push_values` Double-VALUES Bug
- **Discovered by:** IMAAN during Phase 2
- **Problem:** sqlx's `push_values` macro adds its own `VALUES` keyword, causing double-VALUES in the generated SQL
- **Fix:** Used `push` instead of `push_values` and manually constructed the VALUES clause
- **Lesson:** Be careful with sqlx query builder macros

### Wire-Format Enums vs DB CHECK Constraints
- **Discovered by:** IMAAN during Phase 2
- **Problem:** Rust enums serialized as UPPERCASE on wire (`"CDR"`) but SQLite CHECK constraints expected lowercase (`"cdr"`). This caused silent `INSERT OR IGNORE` drops — events appeared to insert but were silently discarded
- **Fix:** Added explicit `db_str()` converters that lowercase the value before DB storage
- **Lesson:** Always test data round-trip through the DB, not just API responses. Silent drops are the worst kind of bug

### Port 8420 Already in Use
- **Reported by:** Chirag (hit locally)
- **Problem:** `cargo run` panicked with `bind failed: AddrInUse` because a previous `netra-server.exe` process was still running
- **Fix:** `taskkill /F /IM netra-server.exe` then retry
- **Lesson:** Windows locks executables while running. Kill the process before rebuilding

### Post-Login Navigation Bug
- **Discovered by:** MIMI during Phase 1
- **Problem:** Login → redirect to dashboard failed because `signIn`/`signOut` state transitions weren't triggering re-renders
- **Fix:** Updated state transitions in signIn/signOut to properly trigger React re-renders
- **Lesson:** Auth state machines in React need careful attention to state transition ordering

---

## 10. Lessons Learned

### Architecture
1. **Contract-first works.** Having `api-types.ts` as single source of truth prevented most integration bugs. Both agents import from the same file.
2. **Parallel branches with regular merges.** MIMI merging main into agent/frontend every few hours kept drift minimal.
3. **Shared whiteboard (TEAM_PROGRESS.md) for agent communication.** Effective for async coordination between IMAAN and MIMI.

### Backend
1. **Always set `default-run` in Cargo.toml** when adding multiple binaries
2. **Always set explicit body limits** for file upload endpoints
3. **Test DB round-trips**, not just API responses — silent data drops are the worst kind of bug
4. **SQLite WAL mode** is essential for concurrent read/write access
5. **Pipeline lock** pattern works well for serializing expensive operations (resolve + analyze)

### Frontend
1. **HashRouter is mandatory** for Tauri apps (file:// protocol doesn't support History API)
2. **@tanstack/react-virtual** is excellent for high-volume lists (100k events)
3. **Dual progress tracking** (WS + poll fallback) is robust for ingest progress
4. **Leaflet needs explicit CSS import** — `import "leaflet/dist/leaflet.css"` or you get broken tiles

### Process
1. **Interop issues surface early** when you have a typed contract and both agents build against it
2. **Audit pass before demo** was critical — found zero warnings across both codebases
3. **Synthetic data generator** was essential for testing at scale (100k records)

---

## 11. What's Left To Build

### Backend (Phase 5)
- [ ] OpenCelliD India dataset bundling (pre-filtered to Punjab/Haryana)
- [ ] Tower ID → lat/lng resolver
- [ ] `/cases/:id/movements` real trail assembly from CDR tower pings
- [ ] Timeline query optimization (indexed filters for all param combos)
- [ ] Load test: 1M events case, all queries < 500ms

### Backend (Hackathon Day)
- [ ] PDF ingestion path (tesseract CLI shell-out fallback)
- [ ] Probabilistic entity resolution tuning (Jaro-Winkler, shared addresses, temporal proximity)
- [ ] Alert severity calibration
- [ ] LLM runtime (llama.cpp sidecar, 4-bit GGUF) + RAG (sqlite-vec) + SSE chat
- [ ] Discord/Telegram webhooks + desktop notification triggers
- [ ] Report generation (template fallback ready if LLM is flaky)

### Frontend (Phase 5)
- [ ] Copilot chat panel: message list, streaming render from SSE frames
- [ ] Sources display: cited event IDs as clickable links to timeline drawer
- [ ] Report viewer: markdown summary, approve button, export PDF
- [ ] Settings screens: user management table, webhook config, model version list + promote
- [ ] Audit log viewer (Admin/Supervisor only)
- [ ] Full RBAC sweep: every screen hides/blocks actions per role matrix
- [ ] Loading skeletons + error states everywhere

### Frontend (Hackathon Day)
- [ ] Wire remaining screens to final endpoints
- [ ] Copilot chat polish (streaming UX, typing indicator, citations)
- [ ] Notification UX polish
- [ ] Demo walkthrough rehearsal
- [ ] Prepare demo laptop: verify offline tiles, no console errors, fresh seeded DB

### Known Technical Debt
1. JWT logout is client-side only — no server-side token revocation
2. WebSocket topic filtering not implemented — all events broadcast to all clients
3. Health check has no DB ping
4. Tags/assignees stored as JSON text — not normalized
5. CORS is fully open (fine for dev, not production)
6. Entity resolution only processes CDR — IPDR/bank/social skipped
7. No password change endpoint for non-admins
8. `SYSTEM_USER` hardcoded UUID conflicts with stub data UUID
9. No request rate limiting beyond login lockout
10. Background WS ticker (demo artifact) should be removed

---

## 12. Demo Readiness

### What Works End-to-End
1. Login → Dashboard → Create Case ✅
2. Upload CSV → Ingest progress → Events in timeline ✅
3. Entity resolution → D3 graph with real edges ✅
4. Anomaly detection → Alerts in Alert Center → Triage ✅
5. WebSocket real-time push ✅

### What Needs Demo-Day Work
1. Real movement trails (currently stub data)
2. LLM copilot chat (currently canned SSE)
3. Report generation (currently stub)
4. Settings/audit screens (currently placeholders)
5. Offline tile bundling for Punjab/Haryana

### Demo Laptop Checklist
- [ ] Server builds and runs clean (`cargo run` in `server/`)
- [ ] Client builds and runs clean (`npm run tauri dev` in `client/`)
- [ ] Fresh seeded DB with admin/netra-admin creds
- [ ] 100k synthetic CDR + 5k bank ingested
- [ ] Offline tiles bundled (if geospatial demo needed)
- [ ] No console errors
- [ ] Walkthrough rehearsed: upload → alerts → graph → map → chat → report
