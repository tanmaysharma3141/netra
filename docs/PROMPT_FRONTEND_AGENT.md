# Agent Prompt — NETRA Frontend Development

Copy everything below this line into Tanmay's coding agent as the kickoff prompt.

---

You are building the frontend for **NETRA** — an air-gapped forensic intelligence platform for Indian law enforcement. A Rust/Axum backend server already exists (or is being built in parallel) on the LAN; you build the Tauri v2 desktop client that talks to it.

## Project context

NETRA ingests telecom CDR/IPDR records, bank statements, and social media exports; correlates entities across those domains; detects criminal-pattern anomalies; and presents everything through a case-management UI with timeline, relationship graph, geospatial map, alert triage, and an LLM copilot chat.

**Read these files first — they are your source of truth:**
1. `docs/NETRA_PRD.md` — product requirements, screens list (§7), RBAC matrix (§5.2)
2. `docs/API.md` — frozen REST/WebSocket/SSE contract
3. `contracts/api-types.ts` — shared TypeScript types. **Import from here directly; never redefine data shapes locally**
4. `docs/PLAN_FRONTEND.md` — your phased plan with per-phase completion gates

## Tech stack (do not substitute)

- Tauri v2 + Vite + React 18 + TypeScript (strict mode)
- Tailwind CSS + shadcn/ui components
- React Router v6+
- Recharts (charts), D3 v7 (relationship graph), Leaflet (map — **offline tiles only**)
- State: lightweight — React Query (TanStack) for server state; Zustand only if genuinely needed

## Architecture rules

- All HTTP calls go through a single typed API client module (`src/api/client.ts`) that injects `Authorization: Bearer <JWT>` and handles 401 → redirect to login. Base URL from env/config (`http://<server>:8420/api/v1`)
- JWT stored via Tauri secure store plugin, not localStorage
- WebSocket connection manager (`src/api/ws.ts`): auto-reconnect, subscribe frames per `docs/API.md`, dispatch to subscribers
- SSE streaming parser for the copilot chat endpoint
- Role-based rendering driven by the RBAC matrix in PRD §5.2 — hide/disable actions per role; never trust the UI alone, but never show what users can't do
- Every async view has three states implemented before done: loading skeleton, error state (with retry), empty state

## Design direction

Dark "forensic console" aesthetic: near-black backgrounds, high-contrast monospace accents for IDs/timestamps, severity color coding (critical=red, high=orange, medium=amber, low=slate) used consistently across alerts, badges, and charts. Dense information layout suited to long investigative sessions — no marketing fluff. This is professional software used by police investigators; prioritize clarity, scanability, and keyboard navigation.

## Working method

1. Work strictly phase-by-phase through `docs/PLAN_FRONTEND.md`. Do not start Phase N+1 until Phase N's gate passes
2. The backend may be stubbed (hardcoded JSON matching the contract) early on — build against it exactly as if real. When a phase gate says "real backend," verify against actual server responses, not mocks
3. If the contract seems wrong or insufficient, STOP and flag it — do not invent endpoints or reshape payloads locally. Contract changes go through `docs/API.md` + `contracts/api-types.ts` PRs
4. TypeScript strict: no `any`, no type assertions to dodge contract types
5. Components live in `src/components/`, screens in `src/screens/`, hooks in `src/hooks/`. Keep D3 graph and Leaflet map as pure, self-contained components decoupled from data fetching

## Your first task (Phase 0)

Scaffold the app and ship the login flow:
1. Initialize Tauri v2 + Vite + React-TS in `client/`
2. Set up Tailwind + shadcn/ui with the dark theme tokens
3. Router + sidebar shell (Dashboard, Cases, Alerts, Reports, Settings, Audit)
4. Typed API client + auth token handling via Tauri secure store
5. Login screen hitting `POST /auth/login`; handle 401 (bad credentials) and 423 (lockout) states distinctly
6. Dashboard shell with KPI card grid using hardcoded data shaped by `Case['stats']` from the contract

Phase 0 gate: a user can log in against the stub server and land on a populated dashboard.

## Definition of done (every screen)

- Works against real or stubbed contract-conformant data
- Loading/error/empty states present
- RBAC-aware
- No console errors or warnings
- Keyboard accessible (tab order, focus states, Enter submits forms)

Begin by reading the four reference files above, then report your understanding of the Phase 0 scope in three bullets before writing code.
