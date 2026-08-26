# NETRA API Contract — v0.1 (FROZEN for parallel dev)

Base URL: `http://<server>:8420/api/v1`
Auth: `Authorization: Bearer <JWT>` on every route except `/auth/login`.
Errors: `{ "error": { "code": string, "message": string } }` with proper HTTP status.
Types referenced below live in `contracts/api-types.ts` (single source of truth).

**Change process:** any change to this file MUST be a PR touching both this file and `api-types.ts`. No verbal agreements.

---

## Auth

### POST /auth/login
```json
// req
{ "username": "string", "password": "string" }
// res 200
{ "token": "jwt", "expires_at": "ISO8601", "user": User }
```
401 on bad credentials. 423 after 5 failed attempts (lockout).

### POST /auth/logout
204. Server invalidates token.

---

## Users (Admin only)

| Method | Path | Notes |
|--------|------|-------|
| GET | /users | List all users |
| POST | /users | Create user `{ username, password, role }` |
| PATCH | /users/:id | Update role / reset password |
| DELETE | /users/:id | Deactivate (soft delete) |

---

## Cases

| Method | Path | Notes |
|--------|------|-------|
| GET | /cases | List cases visible to caller (role-scoped) |
| POST | /cases | Investigator/Admin only |
| GET | /cases/:id | Case detail + stats |
| PATCH | /cases/:id | Update title/status/tags/assignees |
| GET | /cases/:id/audit | Audit log entries for case → `AuditEntry[]` (max 200, newest first) |

Case object includes computed stats: event counts per source type, alert counts by severity, entity count.

**AuditEntry** (also in `contracts/api-types.ts`):
```json
{ "id": "uuid", "user_id": "uuid", "case_id": "uuid|null", "action": "case.created", "detail": {}, "at": "ISO8601" }
```

## Events & Timeline

| Method | Path | Notes |
|--------|------|-------|
| GET | /cases/:id/events | Unified timeline. Query params: `source_type`, `event_type`, `entity_id`, `from`, `to`, `limit`, `offset` |
| GET | /events/:id | Single event incl. raw original record |
| POST | /events/:id/notes | Append investigator note. Body `{ "note": "string" }`. Investigator/Admin only → returns updated Event |

Events are paginated (`limit` default 200, max 1000), ordered by timestamp desc.

## Entities

| Method | Path | Notes |
|--------|------|-------|
| GET | /cases/:id/entities | All resolved entities + link tiers |
| GET | /cases/:id/graph | Adjacency list for D3: `{ nodes: GraphNode[], edges: GraphEdge[] }`. Query: `entity_id`, `hops` (default 2, BFS from root when entity_id given) |
| POST | /cases/:id/resolve | Re-run entity resolution for case (Admin/Investigator) → `{ entities, edges, device_links, communication_links }`. Runs automatically after each successful ingest |
| GET | /entities/:id/profile | Full profile: `{ entity, stats: { events, first_seen, last_seen }, connections: [{ link_type, tier, confidence, evidence_count, other_entity_id, other_identifier, other_type }] }` |
| PATCH | /entities/:id | Tags / annotations. Body `{ "tags": ["suspect"] }`. Investigator/Admin only |

## Alerts

| Method | Path | Notes |
|--------|------|-------|
| GET | /alerts | Cross-case list. Query: `case_id`, `severity`, `status` |
| GET | /alerts/:id | Detail incl. supporting evidence event IDs |
| POST | /cases/:id/analyze | Manual anomaly analysis trigger (Admin/Investigator) → `{ alerts_raised, by_rule }` |
| PATCH | /alerts/:id/status | `{ status: "open"|"reviewing"|"confirmed"|"false_positive", note?: string }` → queues feedback for retraining |

## Ingestion

### POST /cases/:id/ingest
Multipart upload of one or more files. Server auto-detects format/type.
```json
// res 202 Accepted (async job)
{ "job_id": "uuid" }
```
Progress via WS `ingest.progress`.

### GET /ingest/jobs/:id
```json
{ "status": "queued"|"running"|"done"|"failed", "records_parsed": 12345, "errors": [ "line 402: unparseable date" ] }
```

## Geospatial

### GET /cases/:id/movements?entity_id=&from=&to=
```json
{ "trails": [ { "entity_id": "...", "points": [ GeoPoint ] } ] }
```

## Copilot

### POST /cases/:id/chat  (streaming)
```json
// req
{ "question": "string" }
// res: text/event-stream, SSE frames:
data: {"delta": "text chunk"}
data: {"sources": ["event_id1", ...]}   // final frame before done
data: {"done": true}
```

## Reports

| Method | Path | Notes |
|--------|------|-------|
| POST | /cases/:id/reports | Generate report (async) → `{ report_id }` |
| GET | /cases/:id/reports | List reports |
| GET | /reports/:id | Report content (JSON) |
| GET | /reports/:id/export | PDF download |
| PATCH | /reports/:id/approve | Supervisor/Admin only |

## Settings (Admin only)

| Method | Path | Notes |
|--------|------|-------|
| GET/PATCH | /settings/webhooks | Discord/Telegram config |
| GET | /models | Model versions + status |
| POST | /models/promote | `{ version: "v1.1" }` |
| POST | /training/trigger | Manual retraining run |
| GET | /training/queue | Queue size + last run info |

---

## WebSocket

Connect: `ws://<server>:8420/ws` with `Authorization` header (or `?token=` fallback).
Client sends subscribe frames; server pushes events:

```json
// client → server
{ "type": "subscribe", "topics": ["case:UUID", "global"] }

// server → client
{ "topic": "case:UUID", "event": "alert.created",   "payload": Alert }
{ "topic": "case:UUID", "event": "ingest.progress", "payload": { "job_id": "...", "parsed": 5000, "total_est": 12000 } }
{ "topic": "global",     "event": "training.progress", "payload": { "epoch": 2, "loss": 0.41, "stage": "training" } }
{ "topic": "global",     "event": "model.updated",  "payload": { "version": "v1.1" } }
```
