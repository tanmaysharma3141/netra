CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti TEXT PRIMARY KEY,
    revoked_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_revoked_jti ON revoked_tokens(jti);

-- Cleanup trigger: delete entries older than 9 hours (token lifetime + 1h buffer)
-- Handled in application code via periodic cleanup task
