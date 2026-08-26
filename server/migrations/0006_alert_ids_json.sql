-- Convert pipe-delimited entity_ids to JSON arrays
-- "uuid1|uuid2|uuid3" → '["uuid1","uuid2","uuid3"]'
UPDATE alerts
SET entity_ids = '["' || REPLACE(entity_ids, '|', '","') || '"]'
WHERE entity_ids != '[]'
  AND entity_ids NOT LIKE '[%';

-- Single value: "uuid" → '["uuid"]'
UPDATE alerts
SET entity_ids = '["' || entity_ids || '"]'
WHERE entity_ids != '[]'
  AND entity_ids NOT LIKE '[%'
  AND entity_ids NOT LIKE '%|%';

-- Convert pipe-delimited evidence_event_ids to JSON arrays
UPDATE alerts
SET evidence_event_ids = '["' || REPLACE(evidence_event_ids, '|', '","') || '"]'
WHERE evidence_event_ids != '[]'
  AND evidence_event_ids NOT LIKE '[%';

UPDATE alerts
SET evidence_event_ids = '["' || evidence_event_ids || '"]'
WHERE evidence_event_ids != '[]'
  AND evidence_event_ids NOT LIKE '[%'
  AND evidence_event_ids NOT LIKE '%|%';
