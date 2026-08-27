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
  summary: string; // human-readable alert summary
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

// ── Dashboard ──────────────────────────────────────────────────────
export interface DashboardStats {
  total_cases: number;
  active_cases: number;
  alerts_by_severity: {
    critical: number;
    high: number;
    medium: number;
    low: number;
  };
  recent_alerts: Alert[];
  events_this_week: number;
  entities_count: number;
}

// ── Cross-case search ──────────────────────────────────────────────
export interface SearchResults {
  results: SearchResult[];
  total: number;
}

export interface SearchResult {
  result_type: 'entity' | 'alert' | 'case';
  case_id: string;
  case_title: string;
  identifier: string;
  detail: Record<string, unknown>;
}

// ── Ingest preview ─────────────────────────────────────────────────
export interface IngestPreview {
  headers: string[];
  rows: string[][];
  total_rows: number;
  detected_domain: string | null;
  detected_operator: string | null;
}

// ── Settings: alert thresholds ─────────────────────────────────────
export interface AlertThresholds {
  imei_reuse: { min_subscribers: number; severity: Severity };
  hawala_signature: { min_deposits: number; max_amount: number; window_hours: number; severity: Severity };
  rapid_transfer: { min_transfers: number; min_total: number; window_minutes: number; severity: Severity };
  coordinated_silence: { quiet_hours: number; min_parties: number; severity: Severity };
  bot_social: { min_std_ratio: number; severity: Severity };
  round_trip: { window_hours: number; severity: Severity };
  tower_jump: { max_minutes: number; min_distance_km: number; severity: Severity };
}

// ── Settings: data retention ───────────────────────────────────────
export interface RetentionConfig {
  archive_after_days: number;
  delete_after_days: number;
  enabled: boolean;
}

// ── Health ─────────────────────────────────────────────────────────
export interface HealthStatus {
  status: string;
  version: string;
  uptime_seconds: number;
  db_size_bytes: number;
  event_count: number;
  entity_count: number;
  alert_count: number;
  case_count: number;
}
