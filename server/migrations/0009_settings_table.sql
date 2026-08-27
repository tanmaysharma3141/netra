CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Seed default alert thresholds
INSERT OR IGNORE INTO settings (key, value) VALUES ('alert_thresholds', '{"imei_min_subscribers":3,"imei_min_evidence":40,"hawala_window_hours":48,"hawala_min_txns":4,"hawala_min_total":40000.0,"hawala_max_total":150000.0,"rapid_window_minutes":60,"rapid_min_txns":3,"rapid_min_flow":300000.0,"silence_min_parties":3,"bot_min_posts":10,"bot_max_interval_secs":300,"round_trip_window_hours":48,"tower_jump_max_minutes":30,"tower_jump_min_km":50.0}');

-- Seed default retention config
INSERT OR IGNORE INTO settings (key, value) VALUES ('retention', '{"archive_after_days":365,"delete_after_days":730,"enabled":false}');
