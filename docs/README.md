# NETRA Documentation

This directory contains all project documentation, organized by team and purpose.

## Directory Structure

```
docs/
├── README.md                              # This file
├── API.md                                 # Frozen REST/WebSocket/SSE contract (shared)
├── NETRA_PRD.md                           # Product requirements document (shared)
├── LINKEDIN_POSTS.md                      # Marketing content
│
├── frontend/                              # Frontend agent documentation
│   ├── PLAN_FRONTEND.md                   # Phased frontend build plan
│   ├── PROMPT_FRONTEND_AGENT.md           # Agent kickoff prompt for frontend
│   ├── AGENT_HANDOFF.md                   # Handoff guide for new frontend agent
│   └── HANDOFF_FRONTEND.md               # MIMI's detailed handoff document
│
├── backend/                               # Backend agent documentation
│   ├── PLAN_BACKEND.md                    # Phased backend build plan
│   ├── PROMPT_BACKEND_AGENT.md            # Agent kickoff prompt for backend
│   ├── AGENT_HANDOFF.md                   # Backend handoff guide
│   └── COMPREHENSIVE_PROJECT_STATUS.md    # Full backend status and decision log
│
└── comms/                                 # Agent-to-agent communication
    └── TEAM_PROGRESS.md                   # Shared whiteboard + chat log (IMAAN ↔ Frontend)
```

## Conventions

- **Contract source of truth:** `docs/API.md` + `contracts/api-types.ts` — changes require a PR touching BOTH files
- **Agent chat log:** `docs/comms/TEAM_PROGRESS.md` — newest messages at top, sign with `[AGENT_NAME]`
- **Shared docs** (API, PRD) stay at the `docs/` root since both teams reference them
- **Agent-specific docs** go in `frontend/` or `backend/`
- **Docs live on `main`** — code lives on `agent/frontend` or `agent/backend`
