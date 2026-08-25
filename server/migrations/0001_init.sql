CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin','supervisor','investigator','analyst')),
    active INTEGER NOT NULL DEFAULT 1,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cases (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','archived','closed')),
    classification TEXT NOT NULL DEFAULT 'UNCLASSIFIED',
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '[]',
    assignees TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_cases_status ON cases(status);

CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    ts TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('cdr','ipdr','bank','social')),
    entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('phone','imei','bank_acc','ip','handle')),
    event_type TEXT NOT NULL CHECK (event_type IN ('call','sms','data','txn','post','login','other')),
    value REAL,
    lat REAL,
    lng REAL,
    raw TEXT NOT NULL DEFAULT '{}',
    ingested_at TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_events_case_ts ON events(case_id, ts DESC);
CREATE INDEX IF NOT EXISTS idx_events_case_source ON events(case_id, source_type);
CREATE INDEX IF NOT EXISTS idx_events_entity ON events(case_id, entity_id);

CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    type TEXT NOT NULL CHECK (type IN ('phone','imei','bank_acc','ip','handle')),
    identifier TEXT NOT NULL,
    display_name TEXT,
    link_tier TEXT CHECK (link_tier IS NULL OR link_tier IN ('high','medium','low')),
    tags TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    UNIQUE(case_id, type, identifier)
);
CREATE INDEX IF NOT EXISTS idx_entities_case ON entities(case_id);

CREATE TABLE IF NOT EXISTS entity_edges (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    source_entity_id TEXT NOT NULL REFERENCES entities(id),
    target_entity_id TEXT NOT NULL REFERENCES entities(id),
    link_type TEXT NOT NULL,
    tier TEXT NOT NULL CHECK (tier IN ('high','medium','low')),
    confidence REAL NOT NULL,
    evidence_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_edges_case ON entity_edges(case_id);
CREATE INDEX IF NOT EXISTS idx_edges_src ON entity_edges(source_entity_id);
CREATE INDEX IF NOT EXISTS idx_edges_dst ON entity_edges(target_entity_id);

CREATE TABLE IF NOT EXISTS alerts (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    pattern TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('low','medium','high','critical')),
    score INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','reviewing','confirmed','false_positive')),
    entity_ids TEXT NOT NULL DEFAULT '[]',
    evidence_event_ids TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_alerts_case_sev ON alerts(case_id, severity);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    case_id TEXT,
    action TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '{}',
    at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_case ON audit_log(case_id, at DESC);

CREATE TABLE IF NOT EXISTS ingest_jobs (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued','running','done','failed')),
    file_name TEXT,
    sha256 TEXT,
    records_parsed INTEGER NOT NULL DEFAULT 0,
    total_est INTEGER NOT NULL DEFAULT 0,
    errors TEXT NOT NULL DEFAULT '[]',
    started_at TEXT NOT NULL,
    finished_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_jobs_case ON ingest_jobs(case_id);

CREATE TABLE IF NOT EXISTS feedback_queue (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('alert_feedback','report_feedback','approval')),
    alert_id TEXT,
    report_id TEXT,
    label TEXT NOT NULL,
    note TEXT,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    consumed INTEGER NOT NULL DEFAULT 0
);
