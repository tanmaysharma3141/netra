# LinkedIn Posts — Round 1 Submission

## Post 1 — Chirag

Submitted our entry for the Chandigarh Police National Hackathon 2026 exactly 2 minutes before the deadline.

Cutting it close is an understatement. But it went in, and it works.

We are building NETRA for Track 6 (Digital Forensic Analysis and Prevention). Here is the problem it solves. When police investigate a case, the evidence sits in three separate worlds. Telecom call records with one team, bank statements with another, social media data with a third. Nobody connects them until someone manually cross checks timestamps across hundreds of pages. That is weeks of work and suspects stay connected longer than they should.

NETRA takes all of it in through one system. An investigator uploads whatever files they have, any format, and gets back a unified timeline, a graph showing how suspects link up through shared devices and accounts, automatic alerts on suspicious patterns, and a report structured for court.

I own the backend. Rust on Axum with SQLite. A universal CSV ingestion engine that auto detects which operator format it is looking at, role based access control, JWT auth, and an immutable audit log because evidence handling needs chain of custody. Ingestion is clocking around 280k records a minute right now.

Tanmay built the entire desktop client. Tauri, React, TypeScript. Login with proper session handling, a live dashboard, case management, and a timeline view built to scroll through lakhs of events without dying.

Round 1 was deck, working mockup and a demo video. If we are shortlisted, the real fun starts September 8. Twenty four hours to build the correlation engine, anomaly alerts, relationship graph, geospatial mapping and a fully local LLM copilot, on stage.

Video of the current build attached.

---

## Post 2 — Tanmay

Last night I hit submit on our hackathon entry 2 minutes before the deadline. Genuinely do not recommend this workflow, but here we are.

The event is the Chandigarh Police National Hackathon 2026 and we are building NETRA for Track 6. It is a forensic platform for police investigations. Today, evidence from phone records, bank statements and social media lives in three different tools handled by three different teams. Patterns that connect suspects, like one IMEI used across multiple SIM cards, can go unnoticed for weeks. NETRA ingests everything into one air gapped system and surfaces those connections automatically.

My part was the frontend. A desktop app using Tauri, React and TypeScript with Tailwind and shadcn. Login with real session security including lockout handling, a dashboard fed by live case stats, case creation and management flows, and a timeline view that has to stay smooth while scrolling through very large event volumes, because real CDR dumps are massive.

Everything I built talks to a Rust backend over an API contract we wrote and froze before either of us touched feature code. Best decision we made. Zero integration fights so far. Chirag owns that side, and his ingestion engine was already processing 100k record files cleanly before deadline night.

This was Round 1. Pitch deck, a working product mockup and a walkthrough video, all in.

Shortlisting comes next, and if we make it, we build the heavy stuff live at the finale on September 8. Correlation graphs, anomaly detection, maps and an AI copilot that runs fully offline.

Current build in the video below.
