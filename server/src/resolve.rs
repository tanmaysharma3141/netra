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
}

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
                    .entry(($src.clone(), $dst.clone(), $lt))
                    .or_insert(EdgeAcc {
                        src: $src.clone(),
                        dst: $dst.clone(),
                        link_type: $lt,
                        tier: $tier,
                        confidence: $conf,
                        evidence: 0,
                    });
                e.evidence += 1;
            }
        }};
    }

    for (entity_id, entity_type, source_type, raw_str) in &rows {
        let ek = ensure_entity!(entity_type.as_str(), entity_id.as_str());

        match source_type.as_str() {
            "cdr" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) else {
                    continue;
                };
                let Some(obj) = json.as_object() else { continue };

                if let Some(imei) = find_raw(obj, &["imei"]) {
                    let ik = ensure_entity!("imei", imei.to_string());
                    add_edge!(ek, ik, "used_device", Tier::High, 1.0);
                    stats.device_links += 1;
                }

                if let Some(b) = find_raw(obj, &["bnumber", "bparty", "callednumber"]) {
                    let bp = normalize_phone(b);
                    if bp.len() >= 8 {
                        let bk = ensure_entity!("phone", bp.clone());
                        display_names
                            .entry(bk.clone())
                            .or_insert_with(|| format!("+{bp}"));
                        add_edge!(ek, bk, "communication", Tier::High, 1.0);
                        stats.communication_links += 1;
                    }
                }
            }
            _ => {}
        }
    }

    // Wrap the full rebuild in a transaction for atomicity
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
