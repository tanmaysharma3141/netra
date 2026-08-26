# AGENT_HANDOFF.md — NETRA Project Handoff

> **For:** New agent taking over development
> **Date:** 26 Aug 2026
> **Previous agents:** IMAAN (backend), MIMI (frontend)
> **Human operator:** Chirag Kumar

---

## Quick Start

### What is NETRA?
An air-gapped forensic intelligence platform for Indian law enforcement. Rust server + Tauri desktop client. Ingests CDR/bank/social CSVs → correlates entities → detects criminal patterns → visualizes everything.

### Repo structure
```
Netra/
├── server/          # Rust/Axum backend (port 8420)
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── routes/           # All HTTP/WS handlers
│   │   ├── auth.rs           # JWT + bcrypt
│   │   ├── db.rs             # SQLite setup + migrations
│   │   ├── models.rs         # All data types
│   │   ├── resolve.rs        # Entity resolution
│   │   ├── anomaly.rs        # Anomaly detection
│   │   ├── ingest/           # CSV parser
│   │   └── bin/gen-synthetic # Test data generator
│   └── migrations/           # 3 SQL migrations
├── client/          # Tauri v2 + React 18 + Vite
│   └── src/
│       ├── screens/          # 9 screens (4 real, 5 placeholder)
│       ├── components/       # timeline/, graph/, map/, ingest/, ui/
│       ├── api/              # HTTP client, WS client, auth
│       ├── auth/             # AuthContext + RBAC
│       └── lib/              # Utilities, secure store
├── contracts/       # Shared TypeScript types
└── docs/            # PRD, API contract, plans, progress
```

### Run it
```bash
# Terminal 1: Server
cd server && cargo run

# Terminal 2: Client (Tauri dev mode)
cd client && npm install && npm run tauri dev

# Login: admin / netra-admin
```

### Git branches
| Branch | Purpose | Owner |
|--------|---------|-------|
| `main` | Merged state | Chirag |
| `agent/backend` | Backend dev | Was IMAAN |
| `agent/frontend` | Frontend dev | Was MIMI |

**IMPORTANT:** Never push directly to `main`. Work on `agent/backend` or `agent/frontend` and let Chirag merge.

---

## What's Built (Phase 0-4 Complete)

### Server — 26 real endpoints, 10 stubs

**REAL (all DB-backed):**
- Auth: login, logout (JWT, bcrypt, 5-fail lockout)
- Users: full CRUD (admin-only)
- Cases: CRUD with role-scoped visibility + live stats
- Events: timeline with filters (source/entity/date/offset)
- Entities: list, graph (N-hop BFS), profile, tags, resolve
- Alerts: list, detail, triage (confirmed/false_positive), manual analyze
- Ingest: multipart upload → async job → WS progress → auto-resolve → auto-analyze
- WebSocket: authenticated, broadcast, auto-reconnect

**STUBS (hardcoded data):**
- `/cases/:id/movements` — needs OpenCelliD tower DB
- `/cases/:id/chat` — needs LLM integration
- All `/reports/*` endpoints — needs report generation
- All `/settings/*` endpoints — needs implementation
- All `/models/*` and `/training/*` — needs LLM integration

### Client — 4 real screens, 5 placeholders

**REAL:**
- Login (401/403/423 handling, secure store)
- Dashboard (KPI cards from live /cases)
- Cases (search, filter, create, detail)
- Case Detail with 4 working tabs:
  - Timeline (virtualized 100k events, filters, clustering, annotations, A/B comparison)
  - Graph (D3 force-directed, 86 nodes/5k edges, hop control, entity profiles)
  - Map (Leaflet, playback slider, offline tile support)
  - Ingest (drag-drop, WS progress + poll fallback)
- Alerts (cross-case list, triage workflow)

**PLACEHOLDER:** Reports, Settings, Audit, Chat tab in case detail

### Benchmark Results
- Ingestion: 284k records/min (target: 100k)
- 100k CDR → 86 entities → 5,100 edges → 96 alerts (1 critical, 3 hawala, 92 rapid-transfer)

---

## What To Build Next

### Priority 1 — Backend Phase 5 (blocks frontend)
1. **Geospatial:** Bundle OpenCelliD India dataset → implement `/cases/:id/movements` with real tower-to-coords resolution
2. **Reports:** Implement report generation (template-based at minimum, LLM-enhanced if time)
3. **Settings:** User management table, webhook config, model version list

### Priority 2 — Frontend Phase 5
1. **Reports screen:** Viewer, approve button, export
2. **Settings screen:** User management, webhooks, model management
3. **Audit screen:** Log viewer (admin/supervisor only)
4. **Chat tab in case detail:** SSE streaming copilot

### Priority 3 — Polish
1. Full RBAC sweep on every screen
2. Loading skeletons + error states everywhere
3. Demo data preparation (synthetic 100k case)
4. Offline tile bundling for Punjab/Haryana

### Priority 4 — Hackathon Day
1. LLM runtime (llama.cpp sidecar, 4-bit GGUF) + RAG
2. PDF ingestion (tesseract CLI)
3. Discord/Telegram webhooks
4. Demo rehearsal

---

## Critical Files to Know

### Contract (NEVER change without updating BOTH files)
- `contracts/api-types.ts` — TypeScript types
- `docs/API.md` — REST/WS/SSE contract

### Server key files
- `server/src/routes/mod.rs` — route registration (add new routes here)
- `server/src/models.rs` — all data types (add new types here)
- `server/src/db.rs` — database setup, seed script
- `server/src/state.rs` — AppState struct (pool, broadcast, JWT secret)
- `server/src/anomaly.rs` — anomaly rules (add new patterns here)
- `server/src/resolve.rs` — entity resolution (extend for IPDR/bank/social)

### Client key files
- `client/src/App.tsx` — routing (add new routes here)
- `client/src/api/client.ts` — HTTP client (base URL, JWT injection)
- `client/src/api/ws.ts` — WebSocket client (subscribe/publish)
- `client/src/auth/AuthContext.tsx` — auth state + RBAC
- `client/src/lib/rbac.ts` — permission matrix
- `client/src/components/layout/app-shell.tsx` — sidebar nav

### Docs
- `docs/TEAM_PROGRESS.md` — agent communication (add messages at top of chat log)
- `docs/COMPREHENSIVE_PROJECT_STATUS.md` — everything that's been done
- `docs/PLAN_FRONTEND.md` — frontend phase plan
- `docs/PLAN_BACKEND.md` — backend phase plan

---

## Rules (Don't Break These)

1. **Contract-first.** Any API change = PR touching BOTH `contracts/api-types.ts` AND `docs/API.md`
2. **Types from contracts.** Import types from `@contracts/api-types` — never redefine shapes locally
3. **No `any` in TypeScript.** Strict mode is on
4. **Loading/error/empty states.** Every async view needs all three before it's done
5. **Audit on mutations.** Every mutation writes an audit_log row
6. **RBAC on endpoints.** Every endpoint checks auth + role
7. **Never push to main.** Work on your branch, let Chirag merge
8. **Sign your messages.** In TEAM_PROGRESS.md chat log, prefix with `[AGENT_NAME]`
9. **Pull before writing.** On TEAM_PROGRESS.md and on code
10. **Commit at every gate.** Tag `phase-N-done` when a phase completes

---

## Known Gotchas

1. **Axum body limit:** Default is 2MB. Set `DefaultBodyLimit::max(1GB)` for uploads
2. **SQLite `value` keyword:** Reserved. Quote it in SQL: `"value"`
3. **sqlx `push_values`:** Adds its own VALUES keyword. Don't double it
4. **Wire enums vs DB enums:** Wire format is UPPERCASE, DB CHECK is lowercase. Use `db_str()` converters
5. **HashRouter required:** Tauri uses file:// protocol. History API doesn't work
6. **Leaflet CSS:** Must import `leaflet/dist/leaflet.css` or tiles break
7. **WS auth:** Browsers can't set headers on WebSocket. Use `?token=` query param
8. **Two binaries:** Set `default-run` in Cargo.toml or `cargo run` fails
9. **JWT not revoked on logout:** Stateless tokens. Client clears local session only
10. **WS topic filtering:** Not implemented. All events go to all clients

---

## Commits for Reference

Key commits showing what was built when:

| Commit | What it shows |
|--------|--------------|
| `28af6db` | Client scaffold (how the project started) |
| `0ee5077` | Server stubs (all routes, what was mocked) |
| `a6fed27` | Real auth (migrations, bcrypt, JWT, RBAC) |
| `815f54f` | Ingestion engine (CSV parser, operator detection) |
| `39d2a2a` | Entity resolution (deterministic, graph endpoints) |
| `f4c44c5` | Anomaly engine (4 rules, alert pipeline) |
| `7809397` | D3 graph (force-directed, entity profiles) |
| `d842e9a` | Leaflet map (playback, offline tiles) |
| `4ea0f06` | Timeline A/B comparison mode |
| `b6aa011` | Full audit (zero warnings) |

---

## Agent Communication Protocol

If you're continuing as IMAAN or MIMI:
1. Read `docs/TEAM_PROGRESS.md` first
2. Update YOUR section (Backend or Frontend) when you finish something
3. Drop messages in the chat log at the bottom of the file
4. Sign every message: `[IMAAN]` or `[MIMI]`
5. Pull before you write
6. Don't touch the other agent's section

If you're a new single agent doing both:
- You can ignore the split-agent communication protocol
- Just update both sections as you work
- Keep the chat log for your own notes if useful

---

*Good luck. The foundation is solid — 26 real endpoints, 4 real screens, real auth, real ingestion, real entity resolution, real anomaly detection. What's left is the "last mile" stuff: geospatial, LLM, reports, and polish.*
