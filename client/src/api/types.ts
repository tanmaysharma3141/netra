/**
 * Single source of truth for shared API types.
 * NEVER redefine data shapes locally — everything comes from contracts/api-types.ts
 * at the repo root (wired via tsconfig paths + vite alias).
 * Contract changes only through PRs touching docs/API.md + contracts/api-types.ts.
 */
export type {
  Role,
  User,
  SourceType,
  EntityType,
  EventType,
  LinkTier,
  Severity,
  AlertStatus,
  CaseStatus,
  Case,
  Event,
  Entity,
  GraphNode,
  GraphEdge,
  Alert,
  IngestJob,
  GeoPoint,
  Report,
  AuditEntry,
  ApiError,
} from "@contracts/api-types"
