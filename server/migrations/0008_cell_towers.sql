CREATE TABLE IF NOT EXISTS cell_towers (
    id INTEGER PRIMARY KEY,
    lat REAL NOT NULL,
    lng REAL NOT NULL,
    range_m INTEGER,
    operator TEXT,
    mcc TEXT,
    mnc TEXT,
    lac INTEGER,
    cid INTEGER,
    samples INTEGER
);

CREATE INDEX IF NOT EXISTS idx_towers_lac_cid ON cell_towers(lac, cid);
CREATE INDEX IF NOT EXISTS idx_towers_operator ON cell_towers(operator);
