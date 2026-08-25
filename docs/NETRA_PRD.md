# NETRA — Product Requirements Document
**Team:** BinaryBros  
**Hackathon:** Chandigarh Police National Hackathon 2026 — Track 6 (DFAP)  
**Version:** 1.0  
**Date:** August 2026  

---

## 1. Overview

### 1.1 Product Summary
NETRA (नेत्र — "The Eye") is a fully air-gapped forensic intelligence platform: a Rust server deployed on the police station LAN, accessed through lightweight Tauri v2 desktop clients. It ingests Telecom CDR/IPDR records, bank statements, and social media activity data, correlates them across a unified event timeline, detects anomalies and criminal patterns, visualizes entity relationship networks, and generates court-ready investigation reports — all on-premises with zero data leaving the premises.

NETRA ships with a locally-running fine-tuned open-weight LLM that acts as an AI investigator copilot: answering case questions, suggesting investigative leads, and generating structured intelligence reports. The model continuously self-improves through an on-premises feedback loop, getting smarter on each department's specific case patterns over time.

### 1.2 Problem Statement
Modern criminal investigations rely on evidence scattered across three siloed data domains:
- **Telecom CDR/IPDR** — analyzed by one team using one tool
- **Bank statements** — analyzed by a separate team using another tool
- **Social media activity** — analyzed by yet another team

Investigators must manually cross-reference timestamps, identities, locations, and financial flows across all three. This is time-consuming, error-prone, and does not scale. Hidden connections between suspects — shared IMEI numbers, common bank accounts, coordinated online activity — go undetected for weeks.

NETRA eliminates this fragmentation.

### 1.3 Target Users
| Role | Description |
|------|-------------|
| Investigator | Creates and manages cases, uploads data, runs analysis, generates reports |
| Analyst | Analyzes data and views alerts; cannot create cases |
| Supervisor | Read-only access across all cases; approves and signs off on reports |
| Admin | Manages users, roles, system config, model retraining |

### 1.4 Core Design Principles
1. **Air-gapped first** — zero internet dependency; all traffic stays on the department LAN
2. **Evidence integrity** — every action logged with chain-of-custody trail
3. **Investigator-first UX** — built for non-technical law enforcement users
4. **Universal ingestion** — no format is unsupported
5. **Self-improving** — gets smarter on real case data over time

---

## 2. Goals & Non-Goals

### 2.1 Goals
- Ingest CDR/IPDR, bank statements, and social media data in any format
- Resolve and link entities (phone numbers, IMEIs, bank accounts, IPs, social handles) across all three data domains
- Detect anomalies and suspicious cross-domain patterns automatically
- Reconstruct chronological timelines and multi-hop entity relationship graphs
- Map suspect movement using cell tower + transaction geolocation data
- Generate court-ready intelligence reports with chain-of-custody logs
- Provide an AI copilot for case Q&A, pattern suggestions, and summaries
- Support multi-user, multi-case workflows with role-based access control
- Send alerts via in-app notifications, native desktop notifications, Discord, and Telegram
- Self-fine-tune the local LLM on investigator feedback continuously

### 2.2 Non-Goals (v1)
- Cloud sync or remote access (intentionally excluded — air-gapped by design)
- Real-time social media monitoring (ingestion is file/export based)
- Mobile application
- Integration with external government databases (future roadmap)
- Automated arrest recommendations (NETRA surfaces patterns; humans decide)

---

## 3. System Architecture

### 3.1 High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│               NETRA — Air-Gapped LAN Deployment                  │
│                                                                  │
│   ┌──────────────────┐   ┌──────────────────┐                    │
│   │  Tauri Client A  │   │  Tauri Client B  │  ... up to 20     │
│   │  React + TS UI   │   │  React + TS UI   │  concurrent LAN    │
│   │  (thin client)   │   │  (thin client)   │  clients           │
│   └────────┬─────────┘   └────────┬─────────┘                    │
│            │   REST + WebSocket (LAN only, no internet)         │
│   ┌────────▼──────────────────────▼─────────────┐                 │
│   │        NETRA Server (Rust / Axum)           │                 │
│   │                                             │                 │
│   │  ┌───────────────────────────────────────┐  │                 │
│   │  │ API Layer (REST + WebSocket channel)  │  │                 │
│   │  ├───────────────────────────────────────┤  │                 │
│   │  │ Auth Service (bcrypt + JWT issuance)  │  │                 │
│   │  ├───────────────────────────────────────┤  │                 │
│   │  │ Ingestion Engine (Universal Parser)   │  │                 │
│   │  ├───────────────────────────────────────┤  │                 │
│   │  │ Correlation Engine (Entity Resolution)│  │                 │
│   │  ├───────────────────────────────────────┤  │                 │
│   │  │ Anomaly Engine (ML Scoring)           │  │                 │
│   │  ├───────────────────────────────────────┤  │                 │
│   │  │ LLM Runtime (candle / llama.cpp)      │  │                 │
│   │  │ + RAG Pipeline                        │  │                 │
│   │  ├───────────────────────────────────────┤  │                 │
│   │  │ Fine-tune Engine (Training Queue)     │  │                 │
│   │  ├───────────────────────────────────────┤  │                 │
│   │  │ Notification Svc (Desktop/Discord/    │  │                 │
│   │  │ Telegram)                             │  │                 │
│   │  └───────────────────────────────────────┘  │                 │
│   └──────────────────────┬──────────────────────┘                 │
│                          │                                        │
│   ┌──────────────────────▼───────────────────────────────────┐   │
│   │                   Local Storage Layer                     │   │
│   │   SQLite/SQLCipher │ Vector Index │ Cell Tower DB          │   │
│   │   Model Weights                                            │   │
│   └───────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### 3.2 Technology Stack

| Layer | Technology | Justification |
|-------|-----------|---------------|
| Desktop Shell | Tauri v2 (thin client) | Lightweight, secure, native OS integration; stores session tokens in secure store |
| Server | Rust + Axum (tokio) | Async HTTP/WebSocket server for LAN clients; hosts all engines and storage |
| Transport | REST + WebSocket | REST for CRUD/commands; WebSocket for live alerts and training progress |
| Frontend | React + TypeScript + Vite | Fast iteration, rich ecosystem |
| UI Components | shadcn/ui + Tailwind CSS | Consistent, accessible, customizable |
| Graph Visualization | D3.js | Full control over multi-hop network rendering |
| Map Visualization | Leaflet.js | Offline tile support, lightweight |
| Charts | Recharts | Composable, React-native |
| Local Database | SQLite via sqlx (server-side, SQLCipher-encrypted) | Embedded, zero-config, encrypted at rest |
| RAG Store | sqlite-vec + local embedding model (bge-small ONNX) | Retrieval-augmented context for the copilot without a separate vector DB |
| LLM Runtime | candle (HuggingFace Rust) or llama.cpp sidecar | Native Rust ML inference, GPU support (GPU required on server machine only) |
| LLM Model | Mistral 7B or Llama 3.1 8B (fine-tuned) | Best perf/quality tradeoff for local GPU |
| Cell Tower DB | Bundled static DB (OpenCelliD India dataset) | ~2-3GB, covers all Indian operators |
| Notifications | Tauri notification API + reqwest webhooks | Discord + Telegram webhook support |

---

## 4. Modules

### Module 1: Ingestion Engine (Drishti — दृष्टि)
**Purpose:** Parse and normalize any file format into NETRA's unified event schema.

**Supported Formats:**
- CSV (all delimiters, encodings, column orderings)
- PDF (text-based and scanned via OCR)
- Excel/XLSX
- Word/DOCX
- SQL dump files
- JSON / XML
- Plain text

**Supported Data Types:**
- Telecom CDR (all Indian operators: Jio, Airtel, BSNL, Vi, MTNL — auto-detected)
- IPDR (internet packet detail records)
- Bank statements (all major Indian banks — auto-detected schema)
- Social media activity exports (Facebook, Instagram, Twitter/X, WhatsApp)

**Output:** Every event normalized to the Unified Event Schema:
```
{
  event_id: UUID,
  timestamp: DateTime<UTC>,
  source_type: CDR | IPDR | BANK | SOCIAL,
  entity_id: String,           // phone/account/handle
  entity_type: PHONE | IMEI | BANK_ACC | IP | HANDLE,
  event_type: CALL | DATA | TXN | POST | LOGIN | ...,
  value: f64,                  // amount, duration, etc.
  location: Option<LatLng>,    // resolved from tower DB
  raw: JSON,                   // original record preserved
  case_id: UUID,
  ingested_at: DateTime<UTC>,
}
```

### Module 2: Correlation Engine (Sambandh — संबंध)
**Purpose:** Resolve entities across data domains and build the relationship graph.

**Entity Resolution:**

*Deterministic pass (confidence = 1.0, auto-linked):*
- Exact-match links on IMEI, IMSI, bank account numbers, IP addresses, and social handles

*Probabilistic pass (for ambiguous links):*
Each candidate link is scored as a weighted sum of signals into a single 0–1 confidence score:
- Name similarity — Jaro-Winkler ≥ 0.85 between bank holder names and SIM registration names
- Shared address fields across telecom and bank KYC records
- Temporal proximity of events across domains (< 15 min sliding window)
- Transaction reference/narration strings containing phone or handle fragments

*Confidence tiers:*
- **High** (> 0.9) — auto-linked
- **Medium** (0.7–0.9) — surfaced for investigator review
- **Low** (< 0.7) — discarded but logged for audit
- Thresholds configurable per deployment in Settings

*Blocking strategy (scalability):*
- Records partitioned into blocks (operator/circle × date bucket) before pairwise comparison, keeping resolution near-linear instead of O(n²)

Existing links:
- Links phone numbers ↔ IMEI/IMSI numbers
- Links phone numbers ↔ bank accounts (via shared names, addresses, timestamps)
- Links IP addresses ↔ social media handles
- Links bank accounts ↔ social media handles (via transaction references)
- Each edge labeled with link type, confidence tier, and supporting evidence count

**Multi-hop Graph Construction:**
- Suspect A → shared IMEI → Device → Suspect B → shared bank account → Suspect C
- No depth limit on hops
- Each edge labeled with: link type, confidence score, supporting evidence count
- Graph stored in SQLite as adjacency list with edge metadata

**Temporal Correlation:**
- Sliding time window analysis (configurable: 15min, 30min, 1hr, 6hr, 24hr)
- Cross-domain event clustering within windows
- Pattern templates: call → withdrawal, transfer → silence, coordinated posting

### Module 3: Anomaly Engine (Sanket — संकेत)
**Purpose:** Score and flag suspicious patterns across all three data domains.

**Detection Methods:**
- Isolation Forest for statistical outlier detection on numeric features
- Rule-based engine for domain-specific patterns (configurable thresholds)
- Cross-domain correlation scoring (weighted sum of domain-level anomalies)

**Flagged Patterns:**
| Pattern | Description |
|---------|-------------|
| Unusual call cluster | Spike in calls to new/unknown numbers in short window |
| IMEI reuse | Same IMEI appearing on multiple SIMs |
| Rapid fund transfer | Large amounts moved quickly across accounts |
| Round-tripping | Money leaving and returning to same account |
| Tower jump | Impossible travel between cell towers |
| Coordinated silence | All linked phones going offline simultaneously |
| Hawala signature | Multiple small transfers aggregating to large amount |
| Bot-like social behavior | Regular posting intervals, scripted content patterns |

**Output per Alert:**
- Alert ID, timestamp, type, severity (Low/Medium/High/Critical)
- Entities involved
- Supporting evidence (events that triggered it)
- Anomaly score (0–100)
- Status (Open/Reviewing/Confirmed/False Positive)

### Module 4: Geospatial Engine (Bhoomi — भूमि)
**Purpose:** Map suspect movement using cell tower pings and transaction geolocations.

**Cell Tower Resolution:**
- Bundled OpenCelliD India dataset (~2-3GB)
- Tower ID → lat/lng resolution at query time
- Coverage: all Indian operators and circles

**Features:**
- Movement trail visualization per suspect over time range
- Location overlap detection between suspects
- Geofencing alerts (configurable zones)
- Transaction ATM/branch location overlay
- Heatmap of activity concentration

### Module 5: Timeline Engine (Kaal — काल)
**Purpose:** Reconstruct a chronological cross-source view of any person of interest.

**Features:**
- Unified timeline across CDR, IPDR, bank, and social events
- Filterable by source type, event type, date range, entity
- Collapsible event groups (cluster nearby events)
- Side-by-side timeline comparison between two suspects
- Annotatable — investigators can add case notes to any event

### Module 6: LLM Copilot (Vivek — विवेक)
**Purpose:** On-device AI investigator assistant for report generation, Q&A, and suggestions.

**Base Model:** Mistral 7B Instruct or Llama 3.1 8B (fine-tuned on synthetic forensic data)

**Runtime:** candle (Rust-native HuggingFace inference) with GPU acceleration via CUDA/Metal — GPU lives on the server machine only; clients need no special hardware

**Context Pipeline (RAG):**
- Every ingested event is embedded at ingestion time via a local embedding model (bge-small ONNX through candle) and stored in a sqlite-vec vector index on the server
- On each query, top-k relevant event chunks are retrieved and assembled into a token-budgeted context window alongside compact structured case stats (entity counts, alert summaries, key relationships)
- Full-case serialization never happens — context always fits the model's window regardless of case size

**Capabilities:**
- **Report generation** — structured court-ready intelligence reports from case data
- **Case Q&A** — "What is the most suspicious transaction in this case?" / "Who is Suspect A most connected to?"
- **Pattern suggestions** — "Based on this CDR, this looks like a drug delivery pattern. Consider checking..."
- **Entity summarization** — full profile of any entity on demand
- **Cross-case insights** — "This phone number appeared in Case #2026-041 last month"

**Context Pipeline:** See RAG pipeline above — retrieved event chunks + compact case stats, always within the model's token budget

### Module 7: Self Fine-Tuning Engine (Guru — गुरु)
**Purpose:** Continuously improve the local LLM on real case feedback.

**Feedback Collection:**
- Investigator marks alert: Confirmed / False Positive
- Investigator rates generated report: Accurate / Inaccurate
- Supervisor approves/rejects case conclusions
- Every feedback event appended to local training queue (SQLite)

**Training Triggers:**
- **Scheduled** — configurable timer (default: 2AM daily if system is idle)
- **Manual** — Admin triggers retraining from Settings
- Minimum queue size before training: configurable (default: 50 feedback events)

**Training Process:**
- LoRA fine-tuning on queued feedback examples (parameter-efficient)
- New model version saved alongside previous (rollback supported)
- Admin can promote / rollback model versions
- Training progress shown in real-time in Settings panel

**Data Privacy:** Training data never leaves the device. Model weights stored locally. No telemetry.

### Module 8: Case Management (Pramaana — प्रमाण)
**Purpose:** Organize, search, and manage investigation cases.

**Features:**
- Create / archive / close cases
- Assign cases to investigators
- Case timeline (audit log of all actions)
- Search by: phone number, IMEI, bank account, IP address, name, alias, case ID
- Tag and annotate entities
- Case collaboration between investigators
- Evidence export (zip of all case data + report)

### Module 9: Report Engine (Vivaran — विवरण)
**Purpose:** Generate structured, court-ready intelligence reports.

**Report Contents:**
- Case header (ID, dates, investigators, classification)
- Executive summary (LLM-generated)
- Entity profile section (all identified suspects and their links)
- Timeline of key events
- Anomalies and their evidence
- Relationship graph snapshot
- Geospatial movement summary
- Chain-of-custody log (every data ingestion and analysis action)
- Investigator notes and annotations

**Output Formats:** PDF (primary), structured JSON (for integration with other systems)

### Module 10: Notification Service (Suuchna — सूचना)
**Purpose:** Alert investigators to new high-priority findings.

**Channels:**
- In-app alert center (always active)
- Native OS desktop notification (Tauri notification API)
- Discord webhook (configurable per case or global)
- Telegram bot webhook (configurable per case or global)

**Notification Triggers:** Configurable per severity level (High, Critical always notify; Low/Medium optional)

---

## 5. Authentication & Access Control

### 5.1 Auth
- Local username/password auth — bcrypt hashed, stored server-side in encrypted SQLite
- JWT session tokens issued by the server; stored client-side in the Tauri secure store
- Session timeout: configurable (default 8 hours)
- Failed login lockout after 5 attempts
- All credential verification and role checks happen on the server; clients never touch the database directly

### 5.2 RBAC Matrix

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

## 6. Data & Privacy

### 6.1 Storage
- All data stored in local SQLite database (encrypted at rest via SQLCipher)
- Cell tower DB: read-only bundled asset
- LLM model weights: local filesystem
- No cloud storage, no telemetry, no external API calls (except webhook notifications if configured)

### 6.2 Legal Compliance
- **IT Act 2000** — NETRA operates within lawful interception and digital evidence provisions
- **DPDP Act 2023** — all personal data processed only within the authorized investigative context; no data shared externally
- **CrPC Section 65B** — reports generated by NETRA are structured to meet electronic evidence admissibility requirements
- **Chain of custody** — every ingestion, analysis, export, and access event logged with timestamp and user ID

### 6.3 Audit Trail
Every action in NETRA is immutably logged:
- User login/logout
- Data ingestion (file name, hash, timestamp, user)
- Analysis runs
- Alert status changes
- Report generation and export
- Model retraining events
- Configuration changes

---

## 7. Screens & User Flows

### 7.1 Screen List
1. **Login** — Officer ID + password
2. **Dashboard** — KPI summary, active alerts, anomaly trend chart, recent cases
3. **Cases** — List of all cases with search and filters
4. **Case Detail** — Overview of a single case with tabs: Timeline, Graph, Map, Alerts, Reports, Chat
5. **Ingest** — Drag-and-drop upload with format auto-detection and preview
6. **Timeline** — Unified chronological event view with filters
7. **Relationship Graph** — Interactive D3 multi-hop network visualization
8. **Geospatial Map** — Leaflet map with movement trails and alert markers
9. **Alert Center** — All alerts across cases with triage workflow
10. **Report Viewer** — Generated report with approve/export controls
11. **LLM Chat** — Case-aware AI copilot interface
12. **Settings** — User management, webhook config, model management, training scheduler
13. **Audit Log** — Immutable system event log (Admin/Supervisor only)

### 7.2 Primary User Flow (Investigator)
```
Login → Dashboard → Create Case → Upload Data → 
Auto-analysis runs → Review Alerts → Explore Timeline → 
Explore Relationship Graph → Chat with Copilot → 
Generate Report → Submit for Supervisor Approval
```

### 7.3 Alert Triage Flow
```
Alert fires → In-app notification + Desktop notification + Webhook →
Investigator opens alert → Reviews supporting evidence →
Marks: Confirmed | False Positive | Needs Review →
Feedback queued for LLM retraining →
Alert status updated in case log
```

---

## 8. Non-Functional Requirements

| Requirement | Target |
|------------|--------|
| Startup time | < 5 seconds on target hardware |
| CDR ingestion speed | 100,000 records/minute |
| Entity resolution | < 30 seconds for 1M records |
| Anomaly detection | < 60 seconds for full case analysis |
| LLM inference | < 10 seconds per response on GPU |
| LLM fine-tuning | Completes overnight on target hardware |
| Database encryption | AES-256 via SQLCipher |
| Concurrent users | Up to 20 concurrent LAN clients (role-separated) |
| LAN round-trip latency | < 100 ms for UI actions (100 Mbps wired LAN recommended) |
| Offline operation | 100% — no internet dependency; server + clients coexist on isolated LAN |

### 8.1 Target Server Hardware (Recommended)
The GPU is required only on the machine hosting the NETRA server. Client machines need no special hardware.
- CPU: 8-core modern processor
- RAM: 32GB
- GPU: NVIDIA RTX 3090 or equivalent (24GB VRAM for 7B model)
- Storage: 1TB SSD (OS + app + cell tower DB + case data + model weights)

### 8.2 Minimum Server Hardware
- CPU: 4-core
- RAM: 16GB
- GPU: NVIDIA RTX 3060 12GB (quantized 4-bit model)
- Storage: 512GB SSD

### 8.3 Client Hardware
- Any modern laptop/desktop capable of running Tauri v2 (4GB RAM, integrated graphics)

---

## 9. LLM Fine-Tuning Strategy

### 9.1 Initial Model
- Base: Mistral 7B Instruct v0.3 or Llama 3.1 8B
- Fine-tuned on synthetic forensic dataset (generated offline, not shipped)
- Training format: instruction-following (input: case context + question, output: investigator-quality response)

### 9.2 Synthetic Training Data Categories
- CDR anomaly → investigation report pairs
- Bank statement fraud patterns → analysis summaries
- Multi-hop entity graphs → relationship explanations
- Cross-domain correlation examples (call + transfer + silence)
- Indian criminal typologies: hawala, drug distribution, cyber fraud, SIM swap
- CrPC-compliant court report templates

### 9.3 Continuous Fine-Tuning
- Method: LoRA (Low-Rank Adaptation) — parameter efficient, fast, reversible
- Training data: investigator feedback events accumulated in local queue
- Minimum batch: 50 feedback events before a training run
- Versioning: each training run produces a named checkpoint (v1.0, v1.1, etc.)
- Rollback: Admin can revert to any previous checkpoint

---

## 10. Milestones

### Round 1 (Submitted: 25 Aug 2026) ✅
- [x] Pitch deck (NETRA branding)
- [x] Frontend mockup (React, hardcoded synthetic data) *(shipped as live Tauri app — login/dashboard/cases on real stubs)*
- [x] Screen recording video *(2:23 final cut + segment clips)*

### Pre-Hack Prep (26 Aug – 7 Sep 2026)
- [ ] Rust project scaffold (Axum server binary + Tauri v2 thin client + sqlx + entity schema)
- [ ] REST API skeleton + WebSocket channel for live alerts/training progress
- [ ] Universal CSV/PDF parser skeleton
- [ ] React screens: Login, Dashboard, Timeline, Graph, Map, Chat
- [ ] D3 relationship graph component
- [ ] Leaflet map component
- [ ] LLM runtime integration (candle or llama.cpp sidecar)
- [ ] Synthetic data generator (offline script)
- [ ] Initial model fine-tune run

### 24-Hour Hackathon (8 Sep 2026)
**Hour 0-4:** Ingestion engine (CSV + PDF parsing, normalization)  
**Hour 4-8:** Entity resolution + correlation engine  
**Hour 8-12:** Anomaly detection + alert generation  
**Hour 12-16:** Timeline + graph + map UI wired to real backend  
**Hour 16-20:** LLM copilot integration + report generation  
**Hour 20-22:** Notifications (desktop + Discord/Telegram)  
**Hour 22-24:** Polish, demo data, bug fixes  

### Grand Finale (9 Sep 2026)
- Live demo with synthetic case walkthrough
- Show: upload → correlation → alerts → graph → report → copilot Q&A

---

## 11. Risks & Mitigations

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| LLM inference too slow without GPU | Medium | Quantize to 4-bit (GGUF); fallback to rule-based report template |
| Cell tower DB too large for demo | Low | Pre-filter to Punjab/Haryana towers for demo |
| PDF parsing fails on scanned docs | Medium | OCR fallback by shelling out to the tesseract CLI (more mature than tesseract-rs) |
| Fine-tuning crashes on limited VRAM | Medium | LoRA + 4-bit quantized base model reduces VRAM to ~8GB |
| LAN deployment complicates demo setup | Medium | Demo runs server + client on a single laptop over loopback/local hotspot with a scripted setup; identical code paths as real LAN mode |
| 24-hour scope too large | High | Core MVP: ingest + correlation + alerts + graph + basic report (LLM + fine-tuning are demo-layer) |
