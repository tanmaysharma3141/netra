// NETRA shared API types — single source of truth (mirrors docs/API.md)
// Backend implements toward these; frontend imports these directly.

export type Role = 'admin' | 'supervisor' | 'investigator' | 'analyst';

export interface User {
  id: string;
  username: string;
  role: Role;
  active: boolean;
}

export type SourceType = 'CDR' | 'IPDR' | 'BANK' | 'SOCIAL';
export type EntityType = 'PHONE' | 'IMEI' | 'BANK_ACC' | 'IP' | 'HANDLE';
export type EventType = 'CALL' | 'SMS' | 'DATA' | 'TXN' | 'POST' | 'LOGIN' | 'OTHER';
export type LinkTier = 'high' | 'medium' | 'low';
export type Severity = 'low' | 'medium' | 'high' | 'critical';
export type AlertStatus = 'open' | 'reviewing' | 'confirmed' | 'false_positive';
export type CaseStatus = 'active' | 'archived' | 'closed';

export interface Case {
  id: string;
  title: string;
  status: CaseStatus;
  classification: string;
  created_by: string;
  created_at: string;
  assignees: string[];
  tags: string[];
  stats: {
    events_by_source: Record<SourceType, number>;
    alerts_by_severity: Record<Severity, number>;
    entity_count: number;
  };
}

export interface Event {
  id: string;
  case_id: string;
  timestamp: string; // ISO8601 UTC
  source_type: SourceType;
  entity_id: string;
  entity_type: EntityType;
  event_type: EventType;
  value: number | null;
  location: { lat: number; lng: number } | null;
  raw: unknown; // original record preserved verbatim
  notes: string[];
}

export interface Entity {
  id: string;
  case_id: string;
  type: EntityType;
  identifier: string; // phone / account no / handle etc.
  display_name: string | null;
  link_tier: LinkTier | null; // confidence tier from resolution
  tags: string[];
}

export interface GraphNode {
  id: string;
  type: EntityType;
  label: string;
}

export interface GraphEdge {
  source: string; // node id
  target: string; // node id
  link_type: string;
  tier: LinkTier;
  confidence: number; // 0-1
  evidence_count: number;
}

export interface Alert {
  id: string;
  case_id: string;
  pattern: string; // e.g. "imei_reuse", "hawala_signature"
  severity: Severity;
  score: number; // 0-100 anomaly score
  status: AlertStatus;
  entity_ids: string[];
  evidence_event_ids: string[];
  created_at: string;
}

export interface IngestJob {
  id: string;
  case_id: string;
  status: 'queued' | 'running' | 'done' | 'failed';
  records_parsed: number;
  errors: string[];
}

export interface GeoPoint {
  entity_id: string;
  lat: number;
  lng: number;
  tower_id: string | null;
  timestamp: string;
}

export interface Report {
  id: string;
  case_id: string;
  version: number;
  generated_by: 'llm' | 'template';
  approved_by: string | null;
  created_at: string;
  summary_md: string; // executive summary as markdown
}

export interface AuditEntry {
  id: string;
  user_id: string;
  case_id: string | null;
  action: string; // e.g. "auth.login", "ingest.completed", "case.created"
  detail: Record<string, unknown>;
  at: string; // ISO8601 UTC
}

export interface ApiError {
  error: { code: string; message: string };
}

export interface AddNoteRequest {
  note: string;
}
