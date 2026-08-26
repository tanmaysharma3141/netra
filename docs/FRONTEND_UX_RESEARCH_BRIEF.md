# NETRA Frontend — UX Research Brief for Chirag

> **Goal:** Make NETRA as simple as possible for non-technical police investigators.
> Every screen should feel obvious — zero confusion, zero training needed.
> Chirag is researching the internet for the best UX patterns. Return actionable
> recommendations that prioritize simplicity above all else.

---

## 1. What Is NETRA

NETRA (नेत्र — "The Eye") is an **air-gapped forensic intelligence platform** for Indian law enforcement.
It runs entirely on a police station LAN — zero internet. Officers use it to:

- Ingest telecom CDR/IPDR, bank statements, and social media data
- Correlate entities across domains (phone ↔ IMEI ↔ bank account ↔ IP ↔ social handle)
- Detect anomalies (IMEI reuse, hawala patterns, rapid transfers, coordinated silence)
- Visualize relationships on an interactive graph
- Map suspect movements via cell tower data
- Chat with an AI copilot about case data
- Generate court-ready intelligence reports

**Target users:** Police investigators, analysts, supervisors, and administrators in Indian law enforcement.
They are **not software engineers** — they need clarity, scanability, and minimal cognitive load.
Sessions can last hours during active investigations.

---

## 2. Tech Stack (Immutable)

- **Desktop shell:** Tauri v2
- **Frontend:** React 18 + TypeScript 5.8
- **Styling:** Tailwind CSS v4 + shadcn/ui v4
- **Routing:** React Router v6 (HashRouter)
- **Server state:** @tanstack/react-query v5
- **Graph:** D3 v7
- **Map:** Leaflet 1.9
- **Fonts:** Geist + Geist Mono

---

## 3. Design Direction

**Dark forensic console:** near-black backgrounds, monospace accents for IDs, severity colors (critical=red, high=orange, medium=amber, low=slate). Dense but clear. **Simplicity is the priority over looking cool.**

---

## 4. All Screens — Current State

### 4.1 Login Screen
- Username/password form
- Handles 401 (bad creds), 403 (wrong role), 423 (locked out with countdown timer)
- JWT stored in Tauri secure store (localStorage fallback in browser dev)
- **Current implementation:** Basic form with error states

### 4.2 Dashboard
- KPI card grid: alerts by severity, events by source, recent cases
- Case rows with severity badges, event/entity counts
- **Current implementation:** Cards + table, no charts/visualizations

### 4.3 Cases List
- Table with search (title/tag/ID), status filter (active/archived/closed)
- Create case modal (RBAC-gated: Investigator/Admin only)
- Click row → case detail
- **Current implementation:** Standard table + modal

### 4.4 Case Detail (7 tabs)
The main investigative view. Tab layout:

#### 4.4.1 Timeline Tab
- Virtualized infinite-scroll list of 100k+ events
- Filters: source type, event type, date range, entity ID
- Temporal clustering: flat/5m/15m/1h/24h toggles
- Event drawer: full metadata + raw JSON + annotation input
- Side-by-side A/B suspect comparison mode
- **Current implementation:** Dense virtualized list, functional but information-dense

#### 4.4.2 Graph Tab
- D3 force-directed network visualization
- Type-colored nodes, log-scaled edge widths
- Hop selector (1-3), BFS subgraph focus
- Click node → entity profile side panel
- Drag/zoom/pan, hover-neighbor dimming
- **Current implementation:** Interactive SVG graph

#### 4.4.3 Map Tab
- Leaflet map with per-entity colored polylines
- Playback slider for movement timeline
- Offline tile support (VITE_TILE_URL env var)
- **Current implementation:** Map with trails + playback

#### 4.4.4 Alerts Tab
- Case-scoped alert list with severity/status filters
- Severity-colored cards, expand for detail
- Triage buttons: Confirm / False Positive / Needs Review
- Run Analysis button (RBAC-gated)
- **Current implementation:** Card list with filter bar + triage sheet

#### 4.4.5 Ingest Tab
- Drag-and-drop file upload
- WS progress + poll fallback
- Row-level parse error display
- **Current implementation:** Upload zone + progress card

#### 4.4.6 Reports Tab
- Case-scoped report list with version/status
- Markdown summary viewer (custom inline renderer)
- Approve button (RBAC-gated), PDF export
- **Current implementation:** Report cards + markdown display

#### 4.4.7 Chat Tab
- SSE streaming copilot from POST /cases/:id/chat
- Real-time delta rendering as tokens arrive
- Source event IDs as badges
- Message history with user/assistant avatars
- **Current implementation:** Chat interface with streaming

### 4.5 Alert Center (Standalone)
- Cross-case alert list
- Severity/status filter bar
- Same card UI as case detail alerts tab
- **Current implementation:** Case sidebar + alert cards

### 4.6 Reports (Standalone)
- Case sidebar → report list → markdown viewer
- Approve + export PDF
- **Current implementation:** Three-panel layout

### 4.7 Settings (Admin only)
- User management table (create/deactivate)
- Webhook config form (Discord/Telegram)
- Model version list + promote
- Training queue stats + manual trigger
- **Current implementation:** Sectioned admin console

### 4.8 Audit Log (Admin/Supervisor only)
- Case sidebar → audit entries table
- Timestamp, user, action badge, detail display
- **Current implementation:** Case sidebar + table

---

## 5. RBAC Matrix (4 roles)

| Permission | Admin | Supervisor | Investigator | Analyst |
|-----------|-------|------------|--------------|---------|
| Create case | ✅ | ❌ | ✅ | ❌ |
| Upload data | ✅ | ❌ | ✅ | ❌ |
| View all cases | ✅ | ✅ | Own only | Own only |
| Run analysis | ✅ | ❌ | ✅ | ✅ |
| Generate report | ✅ | ❌ | ✅ | ❌ |
| Approve report | ✅ | ✅ | ❌ | ❌ |
| Manage users | ✅ | ❌ | ❌ | ❌ |
| Trigger retraining | ✅ | ❌ | ❌ | ❌ |
| Configure webhooks | ✅ | ❌ | ❌ | ❌ |
| View audit log | ✅ | ✅ | ❌ | ❌ |

---

## 6. Navigation Structure

Sidebar nav (RBAC-gated):
- Dashboard (`/`)
- Cases (`/cases`)
- Alert Center (`/alerts`)
- Reports (`/reports`)
- Settings (`/settings`) — Admin only
- Audit Log (`/audit`) — Admin/Supervisor only

Case detail tabs: Timeline | Graph | Map | Alerts | Ingest | Reports | Chat

---

## 7. Key Data Types

```
Case: { id, title, status, classification, created_by, created_at, assignees, tags, stats }
Event: { id, case_id, timestamp, source_type, entity_id, entity_type, event_type, value, location, raw, notes }
Entity: { id, case_id, type, identifier, display_name, link_tier, tags }
Alert: { id, case_id, pattern, severity, score, summary, status, entity_ids, evidence_event_ids, created_at }
Report: { id, case_id, version, generated_by, approved_by, created_at, summary_md }
AuditEntry: { id, user_id, case_id, action, detail, at }
GeoPoint: { entity_id, lat, lng, tower_id, timestamp }
```

---

## 8. What Needs UX Research — FOCUS ON SIMPLICITY

**The #1 rule: if a police investigator needs to ask "what do I do next?", the UX has failed.**

Research each area below. For each, answer: "What is the simplest possible way to do this?"

### 8.1 Login
- Is the error messaging clear enough for non-technical users?
- Lockout countdown — is it obvious what happened and what to do?
- Should there be a "forgot password" or is that unnecessary for LAN-deployed tool?

### 8.2 Dashboard
- KPI cards — are they showing the right info at the right level of detail?
- Is it immediately obvious what the user should do next from the dashboard?
- Should there be a prominent "Recent Activity" or "Needs Attention" section?
- How do simple admin dashboards (not enterprise BI) present overview data?

### 8.3 Cases List
- Is search + filter the simplest way to find a case?
- Should cases show more or less info in the list?
- Create case flow — how many fields are truly necessary?
- How do simple task/case management tools (Trello, Notion, Linear) present lists?

### 8.4 Case Detail — Timeline
- 100k events is a LOT. Is a scrollable list the simplest way?
- Should we group events more aggressively (by day? by entity?) instead of flat list?
- Filters — are 5 filter fields too many? What's the minimum needed?
- Event drawer — is opening a side panel the simplest way to see details?
- Comparison mode — is this too complex? Should it be hidden by default?
- How do tools like Google Timeline, Apple Maps timeline, or simple log viewers show events?

### 8.5 Case Detail — Graph
- Force-directed graph is visually impressive but is it simple to USE?
- Can a non-technical user understand what nodes and edges mean?
- Hop selector (1-3) — is this intuitive? Should it be simpler (e.g., "Show me connections")?
- Click-to-inspect — is the side panel the right pattern?
- Should the graph have a "simplified view" that shows fewer connections?
- How does Maltego make network graphs accessible to non-data-scientists?

### 8.6 Case Detail — Map
- Playback slider — is this the simplest way to show movement over time?
- Should the map auto-play instead of requiring manual scrubbing?
- Multiple entity trails on one map — is this confusing?
- How does Google Maps Timeline show a single person's movement simply?

### 8.7 Case Detail — Alerts
- Alert cards with severity colors — is this immediately understandable?
- Triage buttons (Confirm/False Positive/Needs Review) — is "Needs Review" necessary?
- Should alerts auto-sort by severity (critical first)?
- Is the expand-for-detail pattern simple enough?
- How do simple notification systems present alerts?

### 8.8 Case Detail — Ingest
- Drag-and-drop upload — is this obvious enough?
- Progress display — is WS progress + poll fallback clear?
- Error display — are parse errors shown in a way non-technical users understand?
- Should there be a "What happens next?" explanation after upload?

### 8.9 Case Detail — Reports
- Markdown viewer — is this readable for non-technical users?
- Approve button — is the approval flow obvious?
- PDF export — download vs. open in new tab?
- Should reports have a "simple" view vs. "full" view?

### 8.10 Case Detail — Chat
- Is it obvious you can ask questions about the case?
- Streaming response — does the typing effect help or distract?
- Source citations — are they useful or just noise?
- Should suggested prompts be shown when the chat is empty?
- How does ChatGPT/Copilot make the chat interface obvious to first-time users?

### 8.11 Alert Center (Standalone)
- Case sidebar + alert list — is this the simplest navigation?
- Should there be a "all alerts across all cases" default view?
- Filter bar — too many options? What's the minimum?

### 8.12 Settings
- Four sections on one page — is this overwhelming?
- Should settings be tabbed instead?
- User management — is the table the simplest way?
- Webhook config — should this be a wizard instead of a form?

### 8.13 Audit Log
- Case sidebar + table — is this simple enough?
- Should audit entries be filterable by action type?
- Is a table the right format, or would a timeline view be simpler?

### 8.14 Global Navigation
- Sidebar with 6 items — is this the right number?
- Should some items be grouped (e.g., Settings + Audit under "Admin")?
- Is the sidebar always visible, or should it collapse on case detail?
- How do simple desktop apps handle navigation?

### 8.15 Error & Empty States
- Are error messages human-readable (not technical jargon)?
- Empty states — do they tell the user what to do next?
- Loading states — are skeletons better than spinners for this UI?
- How should partial failures be shown (e.g., timeline loads but graph fails)?

### 8.16 Keyboard & Accessibility
- What keyboard shortcuts would save the most time for investigators?
- Is tab order logical across dense screens?
- Are focus states visible on the dark theme?

---

## 9. Constraints

- **No internet at runtime** — the app is air-gapped. All assets must be bundled.
- **Tauri desktop app** — not a web app. Behaves like native desktop software.
- **Non-technical users** — police investigators, not developers. Minimize jargon.
- **Long sessions** — investigations can last hours. Eye strain matters.
- **Dark theme only** — no light mode. Optimize for dark backgrounds.
- **Indian locale** — numbers formatted en-IN, dates in Indian format.
- **Hackathon deadline** — 8 Sep 2026. Only suggest changes that are implementable in time.

---

## 10. Deliverable

For each area in Section 8, provide:
1. **Is it simple enough?** — Yes/No, and if no, what's confusing
2. **Simplest fix** — the one change that would make the biggest difference
3. **Reference** — how a well-known simple tool handles this (Google, Apple, Notion, Linear, etc.)
4. **Effort** — quick (< 1 hour), medium (1-3 hours), or large (> 3 hours)

**Skip anything that's already simple enough.** Don't over-engineer — we want fewer features done well, not more features done poorly.
