use std::collections::HashMap;

use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Default, serde::Serialize)]
pub struct ResolveStats {
    pub events_scanned: u64,
    pub entities: u64,
    pub edges: u64,
    pub device_links: u64,
    pub communication_links: u64,
    #[serde(skip_serializing_if = "is_zero")]
    pub probabilistic_links: u64,
}

fn is_zero(v: &u64) -> bool { *v == 0 }

type EntityKey = String;

fn key(t: &str, id: &str) -> EntityKey {
    format!("{t}|{}", id.trim().to_lowercase())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Tier {
    High,
    Medium,
    Low,
}

impl Tier {
    fn as_str(self) -> &'static str {
        match self {
            Tier::High => "high",
            Tier::Medium => "medium",
            Tier::Low => "low",
        }
    }
}

struct EdgeAcc {
    src: EntityKey,
    dst: EntityKey,
    link_type: &'static str,
    tier: Tier,
    confidence: f64,
    evidence: u64,
}

fn norm_key(raw_key: &str) -> String {
    raw_key
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn find_raw<'a>(
    obj: &'a serde_json::Map<String, serde_json::Value>,
    names: &[&str],
) -> Option<&'a str> {
    for (k, v) in obj {
        let nk = norm_key(k);
        if names.contains(&nk.as_str()) {
            if let Some(s) = v.as_str() {
                let t = s.trim();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
    }
    None
}

fn normalize_phone(p: &str) -> String {
    p.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect()
}

pub async fn resolve_case(pool: &SqlitePool, case_id: Uuid) -> Result<ResolveStats, String> {
    let case_str = case_id.to_string();

    // Load all events for the case
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT entity_id, entity_type, source_type, raw FROM events WHERE case_id = ?1",
    )
    .bind(&case_str)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load events failed: {e}"))?;

    let mut stats = ResolveStats { events_scanned: rows.len() as u64, ..Default::default() };
    let mut entities: HashMap<EntityKey, Uuid> = HashMap::new();
    let mut display_names: HashMap<EntityKey, String> = HashMap::new();
    let mut type_of: HashMap<EntityKey, String> = HashMap::new();
    let mut edges: HashMap<(EntityKey, EntityKey, &'static str), EdgeAcc> = HashMap::new();

    // ─── Trackers for probabilistic pass ───────────────────────────────
    // entity_key → list of raw field values extracted from events
    let mut entity_raw_fields: HashMap<EntityKey, Vec<String>> = HashMap::new();
    // entity_key → list of (timestamp, lat, lng) for temporal/spatial proximity
    let mut entity_spacetime: HashMap<EntityKey, Vec<(String, Option<f64>, Option<f64>)>> = HashMap::new();

    macro_rules! ensure_entity {
        ($t:expr, $id:expr) => {{
            let k = key($t, &$id);
            entities.entry(k.clone()).or_insert_with(Uuid::new_v4);
            type_of.entry(k.clone()).or_insert_with(|| $t.to_string());
            k
        }};
    }

    macro_rules! add_edge {
        ($src:expr, $dst:expr, $lt:expr, $tier:expr, $conf:expr) => {{
            if $src != $dst {
                let e = edges
                    .entry(($src.to_string(), $dst.to_string(), $lt))
                    .or_insert(EdgeAcc {
                        src: $src.to_string(),
                        dst: $dst.to_string(),
                        link_type: $lt,
                        tier: $tier,
                        confidence: $conf,
                        evidence: 0,
                    });
                e.evidence += 1;
            }
        }};
    }

    // ─── Deterministic pass ────────────────────────────────────────────
    for (entity_id, entity_type, source_type, raw_str) in &rows {
        let ek = ensure_entity!(entity_type.as_str(), entity_id.as_str());

        // Extract raw fields for probabilistic matching
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) {
            if let Some(obj) = json.as_object() {
                // Collect all string values as potential matching fields
                for (_k, v) in obj.iter() {
                    if let Some(s) = v.as_str() {
                        let trimmed = s.trim().to_lowercase();
                        if !trimmed.is_empty() && !trimmed.starts_with('_') {
                            entity_raw_fields.entry(ek.clone()).or_default().push(trimmed);
                        }
                    }
                }
            }
        }

        match source_type.as_str() {
            "cdr" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) else {
                    continue;
                };
                let Some(obj) = json.as_object() else { continue };

                // Extract timestamp and location for temporal/spatial analysis
                let ts = find_raw(obj, &["datetime", "ts", "date", "time"]).unwrap_or("").to_string();
                let lat = find_raw(obj, &["lat", "latitude"]).and_then(|s| s.parse::<f64>().ok());
                let lng = find_raw(obj, &["lng", "lon", "longitude"]).and_then(|s| s.parse::<f64>().ok());
                if !ts.is_empty() {
                    entity_spacetime.entry(ek.clone()).or_default().push((ts, lat, lng));
                }

                // Deterministic: IMEI → used_device
                if let Some(imei) = find_raw(obj, &["imei"]) {
                    let ik = ensure_entity!("imei", imei.to_string());
                    add_edge!(ek, ik, "used_device", Tier::High, 1.0);
                    stats.device_links += 1;
                }

                // Deterministic: b-party → communication
                if let Some(b) = find_raw(obj, &["bnumber", "bparty", "callednumber"]) {
                    let bp = normalize_phone(b);
                    if bp.len() >= 8 {
                        let bk = ensure_entity!("phone", bp.clone());
                        display_names.entry(bk.clone()).or_insert_with(|| format!("+{bp}"));
                        add_edge!(ek, bk, "communication", Tier::High, 1.0);
                        stats.communication_links += 1;
                    }
                }

                // Deterministic: IMSI link
                if let Some(imsi) = find_raw(obj, &["imsi"]) {
                    let ik = ensure_entity!("imsi", imsi.to_string());
                    add_edge!(ek, ik, "subscriber_identity", Tier::High, 1.0);
                }
            }
            "bank" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) else {
                    continue;
                };
                let Some(obj) = json.as_object() else { continue };

                // Extract timestamp for temporal proximity
                let ts = find_raw(obj, &["date", "valuedate", "txndate"]).unwrap_or("").to_string();
                if !ts.is_empty() {
                    entity_spacetime.entry(ek.clone()).or_default().push((ts, None, None));
                }

                // Deterministic: counterparty bank account
                if let Some(counterparty) = find_raw(obj, &["counterparty", "beneficiary", "payee"]) {
                    let cpk = ensure_entity!("bank_acc", counterparty.to_string());
                    add_edge!(ek, cpk, "fund_transfer", Tier::High, 1.0);
                }

                // Extract account holder name for name matching
                if let Some(name) = find_raw(obj, &["name", "accountname", "holder", "beneficiaryname"]) {
                    entity_raw_fields.entry(ek.clone()).or_default().push(name.to_lowercase());
                }
            }
            "social" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) else {
                    continue;
                };
                let Some(obj) = json.as_object() else { continue };

                // Extract platform info
                if let Some(platform) = find_raw(obj, &["platform", "source", "app"]) {
                    entity_raw_fields.entry(ek.clone()).or_default().push(format!("platform:{platform}"));
                }
            }
            _ => {}
        }
    }

    // ─── Probabilistic pass ────────────────────────────────────────────

    // 1. Name similarity (Jaro-Winkler) across entities with names
    let name_entities: Vec<(EntityKey, String)> = entity_raw_fields.iter()
        .filter_map(|(k, fields)| {
            // Look for fields that look like names (contain letters, reasonable length)
            let name = fields.iter().find(|f| {
                f.len() >= 3 && f.len() <= 100 && f.chars().any(|c| c.is_alphabetic())
                    && !f.starts_with("platform:") && !f.starts_with("operator:")
            })?;
            Some((k.clone(), name.clone()))
        })
        .collect();

    for i in 0..name_entities.len() {
        for j in (i + 1)..name_entities.len() {
            let (ref k_a, ref name_a) = name_entities[i];
            let (ref k_b, ref name_b) = name_entities[j];

            // Skip same entity type with same identifier (already linked deterministically)
            if key_type(k_a) == key_type(k_b) {
                continue;
            }

            let similarity = jaro_winkler(name_a, name_b);
            if similarity >= 0.85 {
                let tier = if similarity >= 0.95 { Tier::High } else { Tier::Medium };
                add_edge!(k_a, k_b, "name_match", tier, similarity);
                stats.probabilistic_links += 1;
            }
        }
    }

    // 2. Temporal proximity: events from different entities happening within 5 minutes
    let spacetime_list: Vec<(&EntityKey, &Vec<(String, Option<f64>, Option<f64>)>)> = entity_spacetime.iter().collect();
    for i in 0..spacetime_list.len() {
        for j in (i + 1)..spacetime_list.len() {
            let (ref k_a, events_a) = spacetime_list[i];
            let (ref k_b, events_b) = spacetime_list[j];

            // Only link different entity types
            if key_type(k_a) == key_type(k_b) {
                continue;
            }

            // Check for co-located events (same lat/lng within 0.001 degrees ≈ 100m)
            for (ts_a, lat_a, lng_a) in events_a {
                if let (Some(la_a), Some(lo_a)) = (lat_a, lng_a) {
                    for (ts_b, lat_b, lng_b) in events_b {
                        if let (Some(la_b), Some(lo_b)) = (lat_b, lng_b) {
                            let dist = ((la_a - la_b).powi(2) + (lo_a - lo_b).powi(2)).sqrt();
                            if dist < 0.001 {
                                // Check temporal proximity (within 5 minutes)
                                if let (Some(dt_a), Some(dt_b)) = (
                                    chrono::DateTime::parse_from_rfc3339(ts_a).ok(),
                                    chrono::DateTime::parse_from_rfc3339(ts_b).ok(),
                                ) {
                                    let diff = (dt_a - dt_b).num_seconds().abs();
                                    if diff <= 300 { // 5 minutes
                                        let conf = 0.8 - (diff as f64 / 3000.0); // Higher conf for closer times
                                        add_edge!(k_a, k_b, "co_location", Tier::Medium, conf);
                                        stats.probabilistic_links += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Cross-domain: if same phone appears in CDR as subscriber AND in bank as account holder reference
    // Look for phone numbers that appear in bank raw fields
    let phone_keys: Vec<EntityKey> = entities.iter()
        .filter(|(k, _)| k.starts_with("phone|"))
        .map(|(k, _)| k.clone())
        .collect();

    let bank_keys: Vec<EntityKey> = entities.iter()
        .filter(|(k, _)| k.starts_with("bank_acc|"))
        .map(|(k, _)| k.clone())
        .collect();

    for pk in &phone_keys {
        let phone_id = pk.splitn(2, '|').nth(1).unwrap_or("");
        let normalized = phone_id.chars().filter(|c| c.is_ascii_digit()).collect::<String>();

        for bk in &bank_keys {
            // Check if the bank account's raw fields contain the phone number
            if let Some(fields) = entity_raw_fields.get(bk) {
                for field in fields {
                    let field_digits: String = field.chars().filter(|c| c.is_ascii_digit()).collect();
                    if field_digits.len() >= 8 && field_digits == normalized {
                        add_edge!(pk, bk, "phone_account_link", Tier::Medium, 0.75);
                        stats.probabilistic_links += 1;
                    }
                }
            }
        }
    }

    // ─── Write to database ─────────────────────────────────────────────
    let mut tx = pool.begin().await.map_err(|e| format!("begin transaction failed: {e}"))?;

    sqlx::query("DELETE FROM entity_edges WHERE case_id = ?1")
        .bind(&case_str)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("clear edges failed: {e}"))?;
    sqlx::query("DELETE FROM entities WHERE case_id = ?1")
        .bind(&case_str)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("clear entities failed: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    let now_ref = &now;
    for (k, uuid) in &entities {
        sqlx::query(
            "INSERT OR IGNORE INTO entities (id, case_id, type, identifier, display_name, link_tier, tags, created_at) VALUES (?1, ?2, ?3, ?4, ?5, NULL, '[]', ?6)",
        )
        .bind(uuid.to_string())
        .bind(&case_str)
        .bind(type_of.get(k).cloned().unwrap_or_default())
        .bind(k.splitn(2, '|').nth(1).unwrap_or(k).to_string())
        .bind(display_names.get(k).cloned())
        .bind(now_ref)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("entity insert failed: {e}"))?;
        stats.entities += 1;
    }

    for acc in edges.values() {
        let Some(src_uuid) = entities.get(&acc.src) else { continue };
        let Some(dst_uuid) = entities.get(&acc.dst) else { continue };
        sqlx::query(
            "INSERT INTO entity_edges (id, case_id, source_entity_id, target_entity_id, link_type, tier, confidence, evidence_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&case_str)
        .bind(src_uuid.to_string())
        .bind(dst_uuid.to_string())
        .bind(acc.link_type)
        .bind(acc.tier.as_str())
        .bind(acc.confidence)
        .bind(acc.evidence as i64)
        .bind(now_ref)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("edge insert failed: {e}"))?;
        stats.edges += 1;
    }

    tx.commit().await.map_err(|e| format!("commit failed: {e}"))?;

    Ok(stats)
}

// ─── Helpers ────────────────────────────────────────────────────────────

/// Extract entity type from a key like "phone|+919812345678"
fn key_type(key: &str) -> &str {
    key.splitn(2, '|').next().unwrap_or(key)
}

/// Jaro-Winkler string similarity (0.0 to 1.0)
fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    if s1 == s2 { return 1.0; }
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 || len2 == 0 { return 0.0; }

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);
    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];
    let mut matches = 0.0;
    let mut transpositions = 0.0;

    // Count matches
    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);
        for j in start..end {
            if s2_matches[j] || s1[i] != s2[j] { continue; }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1.0;
            break;
        }
    }

    if matches == 0.0 { return 0.0; }

    // Count transpositions
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] { continue; }
        while !s2_matches[k] { k += 1; }
        if s1[i] != s2[k] { transpositions += 1.0; }
        k += 1;
    }

    let jaro = (matches / len1 as f64
        + matches / len2 as f64
        + (matches - transpositions / 2.0) / matches) / 3.0;

    // Winkler modification: boost for common prefix
    let mut prefix_len = 0usize;
    for i in 0..4.min(len1).min(len2) {
        if s1[i] == s2[i] { prefix_len += 1; } else { break; }
    }

    jaro + prefix_len as f64 * 0.1 * (1.0 - jaro)
}
