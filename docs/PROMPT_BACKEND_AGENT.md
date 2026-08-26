# Agent Prompt — NETRA Backend Development (Phase 5+)

> **Version:** 1.0 (created 26 Aug 2026)
> **For:** New agent taking over backend from IMAAN
> **Status:** Phases 0-4 complete. Picking up from Phase 5.

---

Copy everything below this line into your coding agent as the kickoff prompt.

---

You are the backend agent for **NETRA** — an air-gapped forensic intelligence platform for Indian law enforcement. The previous agent (IMAAN) shipped Phases 0-4. You're picking up from Phase 5.

## Read these files FIRST (in this order)

1. **`docs/AGENT_HANDOFF.md`** — quick-start guide, what's built, what's left, known gotchas
2. **`docs/COMPREHENSIVE_PROJECT_STATUS.md`** — everything that's been done, every error, every decision
3. **`docs/TEAM_PROGRESS.md`** — chat log between agents (bottom = oldest, top = newest)
4. **`contracts/api-types.ts`** — shared TypeScript types (single source of truth)
5. **`docs/API.md`** — frozen REST/WebSocket/SSE contract
6. **`docs/PLAN_BACKEND.md`** — your phased plan (Phases 0-4 are done, start from Phase 5)
7. **`docs/NETRA_PRD.md`** — product requirements (architecture §3, modules §4)

## How to run the server

```bash
cd server
cargo run
# Server starts on http://127.0.0.1:8420
# Login: admin / netra-admin
# JWT secret: auto-generated if NETRA_JWT_SECRET not set (tokens won't survive restarts)
```

If port 8420 is in use: `taskkill /F /IM netra-server.exe` then retry.

## What's already built (DO NOT rebuild)

### Endpoint Status

**REAL (26 endpoints, all DB-backed):**

| Area | Endpoints |
|------|-----------|
| Health + WS | GET /health, GET /ws |
| Auth | POST /auth/login, POST /auth/logout |
| Users | GET/POST /users, PATCH/DELETE /users/:id |
| Cases | GET/POST /cases, GET/PATCH /cases/:id, GET /cases/:id/audit |
| Events | GET /cases/:id/events, GET /events/:id, POST /events/:id/notes |
| Entities | GET /cases/:id/entities, GET /cases/:id/graph, POST /cases/:id/resolve, GET /entities/:id/profile, PATCH /entities/:id |
| Alerts | GET /alerts, GET /alerts/:id, PATCH /alerts/:id/status, POST /cases/:id/analyze |
| Ingest | POST /cases/:id/ingest, GET /ingest/jobs/:id |

**STUB (10 endpoints, hardcoded data):**

| Endpoint | Returns |
|----------|---------|
| GET /cases/:id/movements | Stub trail data |
| POST /cases/:id/chat | Canned SSE response |
| POST/GET /cases/:id/reports | Stub report |
| GET/PATCH /reports/:id | Stub report |
| GET /reports/:id/export | Fake 28-byte PDF |
| GET/PATCH /settings/webhooks | No-op |
| GET /models, POST /models/promote | Hardcoded |
| POST /training/trigger, GET /training/queue | Fake simulation |

### Core Modules

| Module | File | Lines | Status |
|--------|------|-------|--------|
| Entry point | src/main.rs | 84 | Complete |
| App state | src/state.rs | 31 | Complete (pool, broadcast, JWT, pipeline lock) |
| Data models | src/models.rs | 527 | Complete (all enums, structs, WS types) |
| Database | src/db.rs | 184 | Complete (WAL, 3 migrations, seeding) |
| Auth | src/auth.rs | 234 | Complete (bcrypt, JWT, lockout, RBAC extractor) |
| Entity resolution | src/resolve.rs | 215 | Complete (CDR only, deterministic pass) |
| Anomaly engine | src/anomaly.rs | 362 | Complete (4 rules) |
| Ingestion parser | src/ingest/mod.rs | 249 | Complete |
| Schema detection | src/ingest/detect.rs | 224 | Complete |
| Stub data | src/stub_data.rs | 251 | Complete |

### Routes Registration

All routes in `src/routes/mod.rs`. Add new routes there.

### Database Schema (3 migrations, 9 tables)

| Table | Purpose |
|-------|---------|
| users | User accounts with lockout fields |
| cases | Investigation cases |
| events | Ingested records (CDR/bank/IPDR/social) |
| entities | Resolved entity nodes |
| entity_edges | Graph edges between entities |
| alerts | Anomaly detections |
| audit_log | Immutable audit trail (UPDATE+DELETE blocked by triggers) |
| ingest_jobs | Upload/parse tracking |
| feedback_queue | ML feedback collection |

### Auto-Pipeline

After successful ingest, the server automatically:
1. Runs entity resolution (`resolve::resolve_case`)
2. Runs anomaly analysis (`anomaly::analyze_case`)
3. Publishes top-10 open alerts via WebSocket

Protected by `pipeline_lock` (Arc<Mutex>) to prevent concurrent runs.

## What you need to build (Phase 5)

### 1. Geospatial — `/cases/:id/movements` (Priority 1)

**Current state:** Returns `stub_data::demo_trails()`. Ignores date filters.

**What to build:**
- Bundle OpenCelliD India dataset (pre-filtered to Punjab/Haryana if too large — ~2-3GB)
- Tower ID → lat/lng resolver
- `/cases/:id/movements` trail assembly from CDR tower pings + ATM locations
- Support `entity_id`, `from`, `to` query params (currently accepted but ignored)

**Key files to modify:** `src/routes/geo.rs` (28 lines currently)

**Approach:**
1. Download OpenCelliD India CSV (or use a subset)
2. Create a new migration or bundled asset for tower data
3. Add a tower lookup function (HashMap<tower_id, (lat, lng)>)
4. Parse CDR events for tower_id field in raw JSON
5. Assemble trails: group by entity, sort by timestamp, resolve tower coords

### 2. Report Generation (Priority 2)

**Current state:** All 5 report endpoints return hardcoded stubs.

**What to build:**
- Template-based report generation (at minimum)
- LLM-enhanced if time permits
- Report contents: case header, executive summary, entity profiles, timeline, anomalies, graph snapshot, geospatial summary, chain-of-custody, investigator notes
- PDF export (template → HTML → PDF, or use a Rust PDF library)

**Key files to modify:** `src/routes/reports.rs` (47 lines currently)

**Contract types (from api-types.ts):**
```typescript
interface Report {
  id: string;
  case_id: string;
  version: number;
  generated_by: 'llm' | 'template';
  approved_by: string | null;
  created_at: string;
  summary_md: string; // markdown
}
```

### 3. Settings Endpoints (Priority 3)

**Current state:** All settings endpoints return hardcoded data or are no-ops.

**What to build:**
- Webhook config: store Discord/Telegram webhook URLs in DB (new table or settings table)
- Model management: track model versions in DB (new table)
- Training queue: track feedback events and training runs

**Key files to modify:** `src/routes/settings.rs` (80 lines currently)

**New migrations needed:**
- `settings` table (key-value for webhooks)
- `models` table (version, status, created_at)
- `training_runs` table (version, status, started_at, completed_at)

### 4. LLM Integration (Priority 4 — Hackathon Day)

**Current state:** Chat endpoint returns canned SSE response.

**What to build:**
- llama.cpp sidecar process (4-bit GGUF model)
- RAG pipeline: sqlite-vec for vector search, bge-small ONNX for embeddings
- SSE streaming from LLM to client
- Context assembly: top-k relevant events + compact case stats

**Key files to modify:** `src/routes/chat.rs` (59 lines currently)

**This is the hardest task.** Only attempt if Phases 1-3 are done and time permits.

### 5. Entity Resolution Extensions

**Current state:** Only processes CDR events for IMEI + communication links.

**Extend to:**
- IPDR events (IP address extraction)
- Bank events (account number extraction)
- Social events (handle extraction)
- Probabilistic matching (Jaro-Winkler name similarity, temporal proximity)

**Key file:** `src/resolve.rs` (215 lines currently)

### 6. Additional Anomaly Rules

**Current rules:** imei_reuse, hawala_signature, rapid_transfer, coordinated_silence

**Add if time permits:**
- Tower jump (impossible travel between cell towers)
- Round-tripping (money leaving and returning to same account)
- Bot-like social behavior (regular posting intervals)

**Key file:** `src/anomaly.rs` (362 lines currently)

## Tech stack (do not change)

- Rust (edition 2024) + Axum 0.8 + Tokio
- SQLite via sqlx 0.8 (WAL mode, 8-connection pool, 30s busy timeout)
- bcrypt 0.15 + jsonwebtoken 9
- csv 1.3 for ingestion
- sha2/hex for file integrity
- serde/serde_json for serialization
- uuid v4 for IDs
- chrono for timestamps
- tracing for logging

## Architecture rules

1. **Contract-first.** Any API change = PR touching BOTH `contracts/api-types.ts` AND `docs/API.md`
2. **Audit on mutations.** Every mutation writes an `audit_log` row via `db::audit()`
3. **RBAC on endpoints.** Every endpoint uses `Authed` extractor + `require(&[Role])` for authorization
4. **No silent failures.** Return proper error responses with `ApiError`
5. **Idempotent operations.** Entity resolution does a full rebuild (deletes existing, re-inserts)
6. **Pipeline lock.** Use `pipeline_lock` for expensive operations (resolve, analyze)
7. **Batch inserts.** Insert events in batches of 60 for performance
8. **WebSocket publishing.** Use `state.publish()` to broadcast events to all connected clients

## Branching & communication

- **Your branch:** `agent/backend`
- **Docs go on:** `main` (update `docs/TEAM_PROGRESS.md` with your section)
- **Never push code to `main`** — work on `agent/backend`, let Chirag merge
- **Chat log:** `docs/TEAM_PROGRESS.md` — sign messages with `[IMAAN]` (or your agent name), newest at top
- **Pull before writing** on shared files
- **Commit at every gate** — tag `phase-N-done`

## Known gotchas

1. **Axum body limit:** Default is 2MB. Already set to 1GB in main.rs. Don't revert it
2. **SQLite `value` keyword:** Reserved. Quote it: `"value"` in SQL
3. **sqlx `push_values`:** Adds its own VALUES keyword. Don't double it
4. **Wire enums vs DB enums:** Wire = UPPERCASE, DB CHECK = lowercase. Use `db_str()` converters
5. **`default-run = "netra-server"`:** Set in Cargo.toml. Without it, `cargo run` fails with two binaries
6. **JWT not revoked on logout:** Stateless tokens. Client clears local only
7. **WS topic filtering:** Not implemented. All events go to all clients
8. **Background ticker:** main.rs spawns a 30s demo alert ticker. Remove before production
9. **`SYSTEM_USER` UUID:** `22222222-2222-2222-2222-222222222222` — conflicts with stub_data::USER_ID
10. **CORS is fully open:** Fine for dev, not production

## Benchmark to beat

- Ingestion: 284k rec/min (PRD target: 100k)
- 100k CDR → 86 entities → 5,100 edges → 96 alerts
- E2E verified: upload → resolve → analyze → alerts via WS

## Your first task

1. Read all 7 reference files listed above
2. Update `docs/TEAM_PROGRESS.md` — add your section under "Backend (YOUR NAME)" with current status
3. Report your understanding of Phase 5 scope in three bullets
4. Start with geospatial (`/cases/:id/movements`) — it's the most impactful remaining feature

Begin.
