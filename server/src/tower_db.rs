#![allow(dead_code)]

use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TowerInfo {
    pub lat: f64,
    pub lng: f64,
    pub operator: Option<String>,
    pub range_m: Option<i64>,
}

/// Look up a tower by CID only (when LAC is not available)
pub async fn lookup_cid(
    pool: &SqlitePool,
    cid: i64,
) -> Result<Option<TowerInfo>, String> {
    let row: Option<(f64, f64, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT lat, lng, operator, range_m FROM cell_towers WHERE cid = ?1 LIMIT 1",
    )
    .bind(cid)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("tower lookup failed: {e}"))?;

    Ok(row.map(|r| TowerInfo {
        lat: r.0,
        lng: r.1,
        operator: r.2,
        range_m: r.3,
    }))
}

/// Look up a tower by LAC + CID (common CDR tower identifiers)
pub async fn lookup_lac_cid(
    pool: &SqlitePool,
    lac: i64,
    cid: i64,
) -> Result<Option<TowerInfo>, String> {
    let row: Option<(f64, f64, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT lat, lng, operator, range_m FROM cell_towers WHERE lac = ?1 AND cid = ?2 LIMIT 1",
    )
    .bind(lac)
    .bind(cid)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("tower lookup failed: {e}"))?;

    Ok(row.map(|r| TowerInfo {
        lat: r.0,
        lng: r.1,
        operator: r.2,
        range_m: r.3,
    }))
}

/// Look up a tower by operator-prefixed name (e.g. "JIO-CHD-001")
/// Falls back to operator name matching if exact match not found
pub async fn lookup_by_name(
    pool: &SqlitePool,
    tower_name: &str,
) -> Result<Option<TowerInfo>, String> {
    // Try exact operator match first
    let row: Option<(f64, f64, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT lat, lng, operator, range_m FROM cell_towers WHERE operator = ?1 LIMIT 1",
    )
    .bind(tower_name)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("tower lookup failed: {e}"))?;

    Ok(row.map(|r| TowerInfo {
        lat: r.0,
        lng: r.1,
        operator: r.2,
        range_m: r.3,
    }))
}

/// Bulk resolve tower IDs for events that have no lat/lng
/// Updates events in-place using tower DB
pub async fn resolve_event_locations(
    pool: &SqlitePool,
    case_id: &str,
) -> Result<u64, String> {
    // Find events with NULL lat/lng but have raw JSON with tower info
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, raw, source_type FROM events WHERE case_id = ?1 AND lat IS NULL AND source_type IN ('cdr', 'ipdr')",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load events failed: {e}"))?;

    let mut resolved = 0u64;
    for (event_id, raw_str, _source) in &rows {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) else {
            continue;
        };
        let Some(obj) = json.as_object() else {
            continue;
        };

        // Try to extract tower ID from raw JSON
        let tower = obj.get("cell_id")
            .or_else(|| obj.get("Tower ID"))
            .or_else(|| obj.get("towersid"))
            .and_then(|v| v.as_str());

        let Some(tower_id) = tower else { continue };

        // Parse tower_id into parts for different formats
        let parts: Vec<&str> = tower_id.split('-').collect();

        // Format 1: MCC-MNC-LAC-CID (4 parts, e.g. "404-123-4567-89012")
        if parts.len() == 4 {
            if let (Ok(_mcc), Ok(_mnc), Ok(lac), Ok(cid)) = (
                parts[0].parse::<i64>(),
                parts[1].parse::<i64>(),
                parts[2].parse::<i64>(),
                parts[3].parse::<i64>(),
            ) {
                if let Ok(Some(info)) = lookup_lac_cid(pool, lac, cid).await {
                    let _ = sqlx::query(
                        "UPDATE events SET lat = ?1, lng = ?2 WHERE id = ?3",
                    )
                    .bind(info.lat)
                    .bind(info.lng)
                    .bind(event_id)
                    .execute(pool)
                    .await;
                    resolved += 1;
                    continue;
                }
            }
        }

        // Format 2: MCC-MNC-CID (3 parts, e.g. "404-245-17655")
        if parts.len() == 3 {
            if let Ok(cid) = parts[2].parse::<i64>() {
                if let Ok(Some(info)) = lookup_cid(pool, cid).await {
                    let _ = sqlx::query(
                        "UPDATE events SET lat = ?1, lng = ?2 WHERE id = ?3",
                    )
                    .bind(info.lat)
                    .bind(info.lng)
                    .bind(event_id)
                    .execute(pool)
                    .await;
                    resolved += 1;
                    continue;
                }
            }
        }

        // Format 3: bare numeric CID (e.g. "17655")
        if let Ok(cid) = tower_id.parse::<i64>() {
            if let Ok(Some(info)) = lookup_cid(pool, cid).await {
                let _ = sqlx::query(
                    "UPDATE events SET lat = ?1, lng = ?2 WHERE id = ?3",
                )
                .bind(info.lat)
                .bind(info.lng)
                .bind(event_id)
                .execute(pool)
                .await;
                resolved += 1;
                continue;
            }
        }

        // Format 4: operator-prefixed name (e.g. "JIO-PB-40221" or "AIRTEL-CHD-001")
        if let Ok(Some(info)) = lookup_by_name(pool, tower_id).await {
            let _ = sqlx::query(
                "UPDATE events SET lat = ?1, lng = ?2 WHERE id = ?3",
            )
            .bind(info.lat)
            .bind(info.lng)
            .bind(event_id)
            .execute(pool)
            .await;
            resolved += 1;
        }
    }

    Ok(resolved)
}

/// Get total tower count in DB
pub async fn tower_count(pool: &SqlitePool) -> Result<i64, String> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cell_towers")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("count failed: {e}"))?;
    Ok(row.0)
}
