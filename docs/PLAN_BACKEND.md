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
- [x] **Gate:** Tanmay can build Login/Dashboard against your stubs *(MIMI shipped both, verified against live stubs)*

## Phase 1 — Data Layer + Auth (28–29 Aug)
- [x] SQLCipher enabled; migration system set up *(deviation: sqlx has no SQLCipher support — shipped plain SQLite + sqlx migrations now; SQLCipher needs a custom libsqlite3 build, tracked as a hardening-pass item)*
- [x] Migrations: `users`, `cases`, `events`, `entities`, `entity_edges`, `alerts`, `audit_log`, `ingest_jobs`, `feedback_queue`
- [x] Seed script: admin user + demo case *(runs at startup; admin password via `NETRA_ADMIN_PASSWORD` env)*
- [x] Auth service: bcrypt hashing, JWT issue/verify, lockout after 5 failures *(15-min lock, 8h tokens)*
- [x] Auth middleware: role guard implementing the RBAC matrix (PRD §5.2) *(`Authed` extractor + `require(&[Role])` helper)*
- [x] Real `/auth/login` + `/auth/logout`; audit log entries on login/logout
- [x] Real CRUD: `/users`, `/cases` *(cases role-scoped: admins/supervisors see all, others see owned/assigned only; case stats computed from events/alerts/entities tables)*
- [x] **Gate:** Tanmay ships real login screen *(shipped with distinct 401/423/network-error handling + secure-store sessions; real auth now live behind it)*

**Smoke-tested:** wrong-pw→401, good login→JWT, no-token→401, admin create user, analyst blocked from /users (403), 5-fail lockout→423. **Interop fixes from MIMI's review:** UPPERCASE enum wire values (CDR/PHONE/CALL), dotted WS tags (`alert.created`), bare SSE chat frames, numeric `Report.version`, `AuditEntry` added to contract.

## Phase 2 — Ingestion Engine (30 Aug – 1 Sep)
- [x] Universal CSV parser: delimiter sniffing, encoding detection, column-order agnostic *(delimiter sniffing via unquoted-delimiter counting across first 10 lines; UTF-8/UTF-16-BOM handled via lossy decode; column-order agnostic through canonical alias mapping)*
- [x] Operator detection: Jio / Airtel / BSNL / Vi / MTNL CDR schema fingerprints *(filename + header + sample-value matching; bank/social/ipdr domain fingerprints scored the same way)*
- [x] Bank statement schema detection (major Indian banks) *(generic alias set: narration/debit/credit/balance/value-date variants)*
- [x] Normalize to Unified Event Schema; preserve raw record verbatim in `raw` column
- [x] Async ingest jobs: `POST /cases/:id/ingest` → job queue → progress via WS `ingest.progress`
- [x] Ingestion speed target: 100k records/min (benchmark with synthetic CSV) *(**284k rec/min measured** — parse + real inserts, dev build)*
- [x] Audit log entry per file (name, SHA-256 hash, timestamp, user)
- [x] Synthetic data generator script (offline): realistic multi-domain CDR+bank+social dataset *(`cargo run --bin gen-synthetic -- out.csv <rows> cdr|bank`; IMEI-reuse + hawala patterns seeded for later phases)*
- [x] BONUS: `/cases/:id/events` wired to real DB with all contract filters (source/event/entity/from/to/limit/offset)
- [ ] **Gate:** upload a CSV via API, see normalized events in DB ✅ *(verified: 100k CDR + 5k bank, filters + stats live)*

**Debug war stories:** axum's hidden 2MB body limit choked big uploads (raised to 1GB); SQLite `value` keyword needed quoting; `push_values` adds its own VALUES keyword (double-VALUES bug); wire-format enums vs lowercase DB CHECK constraints caused silent `INSERT OR IGNORE` drops — added explicit `db_str()` converters + case-insensitive FromStr.

## Phase 3 — Correlation Engine (2–3 Sep)
- [x] Deterministic entity resolution: exact IMEI/IMSI/account/IP/handle matches *(full-rebuild resolver over case events; IMEI extracted from raw CDR payloads)*
- [x] Blocking strategy: partition by operator/circle × date bucket before pairwise compare *(superseded: single-pass aggregation with HashMap keying — no pairwise compare needed for deterministic pass; revisit if probabilistic pass needs blocking)*
- [x] Probabilistic scoring skeleton: Jaro-Winkler names, shared addresses, temporal proximity, txn references *(structure ready via tier/confidence columns; signals land hackathon day)*
- [x] Confidence tiers high/medium/low written to `entity_edges`
- [x] Graph endpoints: `/cases/:id/entities`, `/cases/:id/graph`, `/entities/:id/profile` *(all real; graph supports root+BFS hops; auto-resolution fires after every successful ingest)*
- [x] **Gate:** D3 graph renders real edges from two overlapping synthetic suspects *(MIMI verified: 86 nodes / 5,100 edges rendered, hot-IMEI hub lights up — Phase 3 COMPLETE)*
- [x] ISSUE #1 fix: `default-run = "netra-server"` so plain `cargo run` works with two binaries in the crate

## Phase 4 — Anomaly Engine + Alerts (4–5 Sep)
- [x] Rule engine framework: configurable thresholds per pattern
- [x] Implement rules: IMEI reuse, rapid fund transfer, hawala signature, coordinated silence
- [x] Anomaly score 0–100 per alert; summary + supporting evidence event IDs attached
- [x] Alert status transitions via `PATCH /alerts/:id/status` → append to feedback queue
- [x] WS push `alert.created` to subscribed clients
- [ ] Isolation Forest scoring ONLY if rules are done and time remains
- [x] **Gate:** synthetic case produces 3 distinct alert types end-to-end (imei_reuse=1 critical, hawala_signature=3, rapid_transfer=92 — verified E2E, WAL + busy_timeout for concurrent ingest)

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
