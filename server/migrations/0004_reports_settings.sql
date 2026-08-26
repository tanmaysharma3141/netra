CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    version INTEGER NOT NULL DEFAULT 1,
    generated_by TEXT NOT NULL CHECK (generated_by IN ('llm', 'template')),
    approved_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL,
    summary_md TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_reports_case ON reports(case_id);

CREATE TABLE IF NOT EXISTS webhook_configs (
    id TEXT PRIMARY KEY DEFAULT '00000000-0000-0000-0000-000000000001',
    discord_url TEXT,
    telegram_bot_token TEXT,
    telegram_chat_id TEXT,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    active INTEGER NOT NULL DEFAULT 0,
    trained_at TEXT,
    base_model TEXT NOT NULL DEFAULT 'mistral-7b-instruct-v0.3',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS training_queue (
    id TEXT PRIMARY KEY DEFAULT '00000000-0000-0000-0000-000000000001',
    queued_events INTEGER NOT NULL DEFAULT 0,
    minimum_batch INTEGER NOT NULL DEFAULT 50,
    last_run TEXT
);
