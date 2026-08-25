# NETRA — Backend Plan (Chirag)

Timeline: 26 Aug → 7 Sep pre-hack, then hackathon day 8 Sep.
Contract source of truth: `docs/API.md` + `contracts/api-types.ts`. Never deviate without a contract PR.
Hardware note: GPU lives on this machine; everything runs server-side.

---

## Phase 0 — Scaffold + Contract Stubs (26–27 Aug)
- [x] Create repo structure: `server/` (Rust), `client/` (Tanmay's Tauri app), `contracts/`, `docs/`
- [x] `cargo new server` with Axum + tokio + sqlx (SQLite) + tower-http (CORS for dev) *(sqlx deferred to Phase 1)*
- [x] Serve **stub routes** for every endpoint in `docs/API.md` returning hardcoded JSON shaped by `contracts/api-types.ts`
- [x] Wire WebSocket `/ws` endpoint that accepts subscribe frames and echoes a fake `alert.created` every 30s *(plus ingest.progress + training.progress emitters)*
- [x] Health check route + basic request logging middleware
- [ ] **Gate:** Tanmay can build Login/Dashboard against your stubs *(stubs live on port 8420, awaiting his confirmation in TEAM_PROGRESS.md)*

## Phase 1 — Data Layer + Auth (28–29 Aug)
- [x] SQLCipher enabled; migration system set up *(deviation: sqlx has no SQLCipher support — shipped plain SQLite + sqlx migrations now; SQLCipher needs a custom libsqlite3 build, tracked as a hardening-pass item)*
- [x] Migrations: `users`, `cases`, `events`, `entities`, `entity_edges`, `alerts`, `audit_log`, `ingest_jobs`, `feedback_queue`
- [x] Seed script: admin user + demo case *(runs at startup; admin password via `NETRA_ADMIN_PASSWORD` env)*
- [x] Auth service: bcrypt hashing, JWT issue/verify, lockout after 5 failures *(15-min lock, 8h tokens)*
- [x] Auth middleware: role guard implementing the RBAC matrix (PRD §5.2) *(`Authed` extractor + `require(&[Role])` helper)*
- [x] Real `/auth/login` + `/auth/logout`; audit log entries on login/logout
- [x] Real CRUD: `/users`, `/cases` *(cases role-scoped: admins/supervisors see all, others see owned/assigned only; case stats computed from events/alerts/entities tables)*
- [ ] **Gate:** Tanmay ships real login screen

**Smoke-tested:** wrong-pw→401, good login→JWT, no-token→401, admin create user, analyst blocked from /users (403), 5-fail lockout→423.

## Phase 2 — Ingestion Engine (30 Aug – 1 Sep)
- [ ] Universal CSV parser: delimiter sniffing, encoding detection, column-order agnostic
- [ ] Operator detection: Jio / Airtel / BSNL / Vi / MTNL CDR schema fingerprints
- [ ] Bank statement schema detection (major Indian banks)
- [ ] Normalize to Unified Event Schema; preserve raw record verbatim in `raw` column
- [ ] Async ingest jobs: `POST /cases/:id/ingest` → job queue → progress via WS `ingest.progress`
- [ ] Ingestion speed target: 100k records/min (benchmark with synthetic CSV)
- [ ] Audit log entry per file (name, SHA-256 hash, timestamp, user)
- [ ] Synthetic data generator script (offline): realistic multi-domain CDR+bank+social dataset
- [ ] **Gate:** upload a CSV via API, see normalized events in DB

## Phase 3 — Correlation Engine (2–3 Sep)
- [ ] Deterministic entity resolution: exact IMEI/IMSI/account/IP/handle matches
- [ ] Blocking strategy: partition by operator/circle × date bucket before pairwise compare
- [ ] Probabilistic scoring skeleton: Jaro-Winkler names, shared addresses, temporal proximity, txn references
- [ ] Confidence tiers high/medium/low written to `entity_edges`
- [ ] Graph endpoints: `/cases/:id/entities`, `/cases/:id/graph`, `/entities/:id/profile`
- [ ] **Gate:** D3 graph renders real edges from two overlapping synthetic suspects

## Phase 4 — Anomaly Engine + Alerts (4–5 Sep)
- [ ] Rule engine framework: configurable thresholds per pattern
- [ ] Implement rules: IMEI reuse, rapid fund transfer, round-tripping, coordinated silence, hawala signature, tower jump (needs tower DB), unusual call cluster, bot-like posting
- [ ] Anomaly score 0–100 per alert; supporting evidence event IDs attached
- [ ] Alert status transitions via `PATCH /alerts/:id/status` → append to feedback queue
- [ ] WS push `alert.created` to subscribed clients
- [ ] Isolation Forest scoring ONLY if rules are done and time remains
- [ ] **Gate:** synthetic case produces ≥ 4 distinct alert types end-to-end

## Phase 5 — Geospatial + Timeline APIs (6–7 Sep)
- [ ] Bundle OpenCelliD India dataset (pre-filtered if too large); tower ID → lat/lng resolver
- [ ] `/cases/:id/movements` trail assembly from CDR tower pings + ATM locations
- [ ] Timeline query optimization: indexed filters for `/cases/:id/events` (all param combos)
- [ ] Load test: 1M events case, all queries < 500ms

---

## Hackathon Day (8 Sep) — backend lane
- [ ] H0–4: PDF ingestion path (tesseract CLI shell-out fallback)
- [ ] H4–8: finish probabilistic entity resolution tuning
- [ ] H8–12: alert polish + severity calibration
- [ ] H12–16: support Tanmay wiring UI to real endpoints (your priority is unblocking him)
- [ ] H16–20: LLM runtime (llama.cpp sidecar, 4-bit GGUF) + RAG (sqlite-vec) + SSE chat
- [ ] H20–22: Discord/Telegram webhooks + desktop notification triggers
- [ ] H22–24: report generation (template fallback ready if LLM is flaky)

## Standing rules
- Every mutation writes an audit_log row
- No endpoint ships without matching the contract types
- Commit at every gate; tag `phase-N-done`
