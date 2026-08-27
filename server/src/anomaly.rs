use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Default, serde::Serialize)]
pub struct AnalyzeStats {
    pub events_scanned: u64,
    pub alerts_raised: u64,
    pub by_rule: HashMap<String, u64>,
}

struct DraftAlert {
    pattern: &'static str,
    severity: Severity,
    score: i64,
    entity_ids: Vec<String>,
    evidence_event_ids: Vec<String>,
    summary: String,
}

#[derive(Debug, Clone, Copy)]
enum Severity {
    Medium,
    High,
    Critical,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    fn base_score(self) -> i64 {
        match self {
            Severity::Medium => 55,
            Severity::High => 75,
            Severity::Critical => 90,
        }
    }
}

const IMEI_REUSE_MIN_SUBSCRIBERS: usize = 3;
const IMEI_REUSE_MIN_TOTAL_EVIDENCE: i64 = 40;
const HAWALA_WINDOW_HOURS: i64 = 48;
const HAWALA_MIN_TXNS: usize = 4;
const RAPID_WINDOW_MINUTES: i64 = 60;
const RAPID_MIN_TXNS: usize = 3;
const RAPID_MIN_FLOW: f64 = 300_000.0;
const SILENCE_MIN_PARTIES: usize = 3;

pub async fn analyze_case(pool: &SqlitePool, case_id: Uuid) -> Result<AnalyzeStats, String> {
    let case_str = case_id.to_string();
    let mut stats = AnalyzeStats::default();

    let rows: Vec<(String, String, String, String, String, Option<f64>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, source_type, entity_id, event_type, ts, value, raw FROM events WHERE case_id = ?1 ORDER BY ts",
        )
        .bind(&case_str)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("load events failed: {e}"))?;

    stats.events_scanned = rows.len() as u64;
    let mut drafts: Vec<DraftAlert> = Vec::new();

    imei_reuse_rule(pool, &case_str, &mut drafts).await?;
    hawala_signature_rule(&rows, &case_str, &mut drafts);
    rapid_transfer_rule(&rows, &mut drafts);
    coordinated_silence_rule(&rows, &mut drafts);
    bot_social_rule(&rows, &mut drafts);
    round_trip_rule(&rows, &mut drafts);
    tower_jump_rule(&rows, &mut drafts);

    sqlx::query("DELETE FROM alerts WHERE case_id = ?1 AND status = 'open'")
        .bind(&case_str)
        .execute(pool)
        .await
        .map_err(|e| format!("clear alerts failed: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    let now_ref = &now;
    for d in drafts {
        sqlx::query(
            "INSERT INTO alerts (id, case_id, pattern, severity, score, status, entity_ids, evidence_event_ids, summary, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6, ?7, ?8, ?9, ?9)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&case_str)
        .bind(d.pattern)
        .bind(d.severity.as_str())
        .bind(d.score.clamp(0, 100))
        .bind(serde_json::to_string(&d.entity_ids).unwrap_or_else(|_| "[]".into()))
        .bind(serde_json::to_string(&d.evidence_event_ids).unwrap_or_else(|_| "[]".into()))
        .bind(&d.summary)
        .bind(now_ref)
        .execute(pool)
        .await
        .map_err(|e| format!("alert insert failed: {e}"))?;
        *stats.by_rule.entry(d.pattern.to_string()).or_insert(0) += 1;
        stats.alerts_raised += 1;
    }

    Ok(stats)
}

async fn imei_reuse_rule(
    pool: &SqlitePool,
    case_id: &str,
    drafts: &mut Vec<DraftAlert>,
) -> Result<(), String> {
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT e.identifier, ee.source_entity_id, ee.evidence_count \
         FROM entity_edges ee JOIN entities e ON e.id = ee.target_entity_id \
         WHERE ee.case_id = ?1 AND ee.link_type = 'used_device' AND e.type = 'imei'",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("imei rule query failed: {e}"))?;

    let mut by_imei: HashMap<String, Vec<(String, i64)>> = HashMap::new();
    for (imei_identifier, phone_entity_id, evidence) in rows {
        by_imei.entry(imei_identifier).or_default().push((phone_entity_id, evidence));
    }

    for (imei, subscribers) in by_imei {
        if subscribers.len() < IMEI_REUSE_MIN_SUBSCRIBERS {
            continue;
        }
        let total_evidence: i64 = subscribers.iter().map(|(_, c)| c).sum();
        if total_evidence < IMEI_REUSE_MIN_TOTAL_EVIDENCE {
            continue;
        }
        let score = (Severity::Critical.base_score() + total_evidence.min(200) / 4).clamp(0, 100);
        drafts.push(DraftAlert {
            pattern: "imei_reuse",
            severity: Severity::Critical,
            score,
            entity_ids: subscribers.iter().map(|(id, _)| id.clone()).collect(),
            evidence_event_ids: vec![],
            summary: format!(
                "IMEI {} shared across {} subscriber lines ({} call bindings)",
                imei,
                subscribers.len(),
                total_evidence
            ),
        });
    }
    Ok(())
}

fn hawala_signature_rule(
    rows: &[(String, String, String, String, String, Option<f64>, Option<String>)],
    _case_id: &str,
    drafts: &mut Vec<DraftAlert>,
) {
    // Track event_id alongside timestamp and value
    let mut per_account: HashMap<&str, Vec<(&str, f64, &str)>> = HashMap::new();
    for (id, src, entity, etype, ts, value, _raw) in rows {
        if src != "bank" || etype != "txn" {
            continue;
        }
        if let (Some(v), Some(t)) = (value, parse_ts(ts)) {
            per_account.entry(entity.as_str()).or_default().push((t, v.abs(), id.as_str()));
        }
    }

    for (account, txns) in per_account {
        let window = chrono::Duration::hours(HAWALA_WINDOW_HOURS);
        let mut idx = 0;
        while idx + HAWALA_MIN_TXNS <= txns.len() {
            let start = match chrono::DateTime::parse_from_rfc3339(txns[idx].0) {
                Ok(s) => s.with_timezone(&chrono::Utc),
                Err(_) => {
                    idx += 1;
                    continue;
                }
            };
            let slice: Vec<(&str, f64, &str)> = txns[idx..]
                .iter()
                .take_while(|(t, _, _)| {
                    chrono::DateTime::parse_from_rfc3339(t)
                        .map(|dt| dt.with_timezone(&chrono::Utc) - start <= window)
                        .unwrap_or(false)
                })
                .copied()
                .collect();

            let deposits: f64 = slice.iter().map(|(_, v, _)| *v).sum();
            let small_count = slice.iter().filter(|(_, v, _)| *v < 10_000.0).count();

            if small_count >= HAWALA_MIN_TXNS && deposits >= 40_000.0 && deposits < 150_000.0 {
                let score = (Severity::High.base_score() + (small_count as i64 - 4) * 3).clamp(0, 100);
                drafts.push(DraftAlert {
                    pattern: "hawala_signature",
                    severity: Severity::High,
                    score,
                    entity_ids: vec![format!("bank_acc:{account}")],
                    evidence_event_ids: slice.iter().map(|(_, _, id)| (*id).to_string()).collect(),
                    summary: format!(
                        "Account {} shows {} small sub-10k txns aggregating {:.0} within {}h - structured deposit pattern",
                        account, small_count, deposits, HAWALA_WINDOW_HOURS
                    ),
                });
                idx += slice.len().max(1);
            } else {
                idx += 1;
            }
        }
    }
}

fn rapid_transfer_rule(
    rows: &[(String, String, String, String, String, Option<f64>, Option<String>)],
    drafts: &mut Vec<DraftAlert>,
) {
    // Track event_id alongside timestamp and value
    let mut per_account: HashMap<&str, Vec<(&str, f64, &str)>> = HashMap::new();
    for (id, src, entity, etype, ts, value, _raw) in rows {
        if src != "bank" || etype != "txn" {
            continue;
        }
        if let (Some(v), Some(t)) = (value, parse_ts(ts)) {
            per_account.entry(entity.as_str()).or_default().push((t, v.abs(), id.as_str()));
        }
    }

    for (account, txns) in per_account {
        let window = chrono::Duration::minutes(RAPID_WINDOW_MINUTES);
        let mut idx = 0;
        let mut seen_keys: HashSet<String> = HashSet::new();
        while idx < txns.len() {
            let start_ts = match chrono::DateTime::parse_from_rfc3339(txns[idx].0) {
                Ok(s) => s.with_timezone(&chrono::Utc),
                Err(_) => {
                    idx += 1;
                    continue;
                }
            };
            let slice: Vec<(&str, f64, &str)> = txns[idx..]
                .iter()
                .take_while(|(t, _, _)| {
                    chrono::DateTime::parse_from_rfc3339(t)
                        .map(|dt| dt.with_timezone(&chrono::Utc) - start_ts <= window)
                        .unwrap_or(false)
                })
                .copied()
                .collect();

            if slice.len() >= RAPID_MIN_TXNS {
                let flow: f64 = slice.iter().map(|(_, v, _)| *v).sum();
                if flow >= RAPID_MIN_FLOW {
                    let key = format!("{account}|{}", slice[0].0);
                    if seen_keys.insert(key) {
                        drafts.push(DraftAlert {
                            pattern: "rapid_transfer",
                            severity: Severity::High,
                            score: (Severity::High.base_score()
                                + ((flow - RAPID_MIN_FLOW) / 20_000.0) as i64)
                                .clamp(0, 100),
                            entity_ids: vec![format!("bank_acc:{account}")],
                            evidence_event_ids: slice.iter().map(|(_, _, id)| (*id).to_string()).collect(),
                            summary: format!(
                                "Account {} moved cumulative {:.0} across {} txns within {} minutes",
                                account, flow, slice.len(), RAPID_WINDOW_MINUTES
                            ),
                        });
                    }
                }
            }
            idx += 1;
        }
    }
}

fn coordinated_silence_rule(
    rows: &[(String, String, String, String, String, Option<f64>, Option<String>)],
    drafts: &mut Vec<DraftAlert>,
) {
    // Track last activity per entity, including the event ID
    let mut last_activity: HashMap<&str, (&str, &str)> = HashMap::new(); // entity -> (timestamp, event_id)
    for (id, src, entity, _etype, ts, _value, _raw) in rows {
        if src != "cdr" {
            continue;
        }
        last_activity.insert(entity.as_str(), (ts.as_str(), id.as_str()));
    }
    if last_activity.len() < SILENCE_MIN_PARTIES {
        return;
    }

    let mut cutoff_candidates: Vec<&str> = last_activity.values().map(|(ts, _)| *ts).collect();
    cutoff_candidates.sort_unstable();
    cutoff_candidates.dedup();

    for window_end in cutoff_candidates.windows(SILENCE_MIN_PARTIES) {
        let Ok(end_dt) = chrono::DateTime::parse_from_rfc3339(window_end[SILENCE_MIN_PARTIES - 1])
        else {
            continue;
        };
        let end_utc = end_dt.with_timezone(&chrono::Utc);
        let went_quiet_ids: Vec<String> = last_activity
            .iter()
            .filter(|(_, (ts, _))| {
                chrono::DateTime::parse_from_rfc3339(ts)
                    .map(|dt| dt.with_timezone(&chrono::Utc) <= end_utc)
                    .unwrap_or(false)
            })
            .map(|(_, (_, id))| id.to_string())
            .collect();
        if went_quiet_ids.len() >= SILENCE_MIN_PARTIES {
            drafts.push(DraftAlert {
                pattern: "coordinated_silence",
                severity: Severity::Medium,
                score: Severity::Medium.base_score(),
                entity_ids: vec![],
                evidence_event_ids: went_quiet_ids.clone(),
                summary: format!(
                    "{} linked phones stopped activity simultaneously before {}",
                    went_quiet_ids.len(),
                    window_end[SILENCE_MIN_PARTIES - 1]
                ),
            });
            break;
        }
    }
}

fn parse_ts(s: &str) -> Option<&str> {
    if s.contains("T") && s.len() >= 20 {
        Some(s)
    } else {
        None
    }
}

// ─── Additional Rules ──────────────────────────────────────────────────

const BOT_MIN_POSTS: usize = 10;
const BOT_MAX_INTERVAL_SECS: i64 = 300; // 5 minutes
const ROUND_TRIP_WINDOW_HOURS: i64 = 48;
const TOWER_JUMP_MAX_MINUTES: i64 = 30;
const TOWER_JUMP_MIN_KM: f64 = 50.0;

/// Rule: Bot-like social behavior — suspiciously regular posting intervals
fn bot_social_rule(
    rows: &[(String, String, String, String, String, Option<f64>, Option<String>)],
    drafts: &mut Vec<DraftAlert>,
) {
    let mut per_handle: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    for (id, src, entity, _etype, ts, _value, _raw) in rows {
        if src != "social" {
            continue;
        }
        if let Some(t) = parse_ts(ts) {
            per_handle.entry(entity.as_str()).or_default().push((t, id.as_str()));
        }
    }

    for (handle, posts) in per_handle {
        if posts.len() < BOT_MIN_POSTS {
            continue;
        }

        // Check for regular intervals
        let mut intervals: Vec<i64> = Vec::new();
        for window in posts.windows(2) {
            if let (Some(t1), Some(t2)) = (
                chrono::DateTime::parse_from_rfc3339(window[0].0).ok(),
                chrono::DateTime::parse_from_rfc3339(window[1].0).ok(),
            ) {
                intervals.push((t2 - t1).num_seconds().abs());
            }
        }

        if intervals.is_empty() {
            continue;
        }

        // Check if intervals are suspiciously consistent (std dev < 30% of mean)
        let mean = intervals.iter().sum::<i64>() as f64 / intervals.len() as f64;
        let variance: f64 = intervals.iter()
            .map(|i| (*i as f64 - mean).powi(2))
            .sum::<f64>() / intervals.len() as f64;
        let std_dev = variance.sqrt();

        if mean > 0.0 && std_dev / mean < 0.3 && mean < BOT_MAX_INTERVAL_SECS as f64 {
            let score = (Severity::Medium.base_score() + (posts.len() as i64 - BOT_MIN_POSTS as i64) * 2).clamp(0, 100);
            drafts.push(DraftAlert {
                pattern: "bot_social",
                severity: Severity::Medium,
                score,
                entity_ids: vec![format!("handle:{handle}")],
                evidence_event_ids: posts.iter().map(|(_, id)| (*id).to_string()).collect(),
                summary: format!(
                    "Handle {} posted {} times with suspiciously regular intervals (avg {:.0}s between posts)",
                    handle, posts.len(), mean
                ),
            });
        }
    }
}

/// Rule: Round-tripping — money leaves account A → arrives B → returns to A within 48h
fn round_trip_rule(
    rows: &[(String, String, String, String, String, Option<f64>, Option<String>)],
    drafts: &mut Vec<DraftAlert>,
) {
    let mut transfers: Vec<(String, String, String, String)> = Vec::new();
    for (id, src, entity, etype, ts, value, raw) in rows {
        if src != "bank" || etype != "txn" {
            continue;
        }
        let Some(t) = parse_ts(ts) else { continue };
        let Some(raw_str) = raw else { continue };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) else { continue };
        let Some(obj) = json.as_object() else { continue };
        let Some(counterparty) = obj.get("counterparty").and_then(|v| v.as_str()) else { continue };
        if let Some(v) = value {
            if *v < 0.0 {
                transfers.push((entity.to_string(), counterparty.to_string(), t.to_string(), id.to_string()));
            } else if *v > 0.0 {
                transfers.push((counterparty.to_string(), entity.to_string(), t.to_string(), id.to_string()));
            }
        }
    }

    let window = chrono::Duration::hours(ROUND_TRIP_WINDOW_HOURS);
    let mut seen = HashSet::new();

    for t1 in &transfers {
        for t2 in &transfers {
            if t1.0 == t2.0 && t1.1 == t2.1 && t1.2 != t2.2 {
                if let (Some(ts1), Some(ts2)) = (
                    chrono::DateTime::parse_from_rfc3339(&t1.2).ok(),
                    chrono::DateTime::parse_from_rfc3339(&t2.2).ok(),
                ) {
                    let diff = (ts2 - ts1).num_seconds().abs();
                    if diff <= window.num_seconds() as i64 {
                        let key = format!("{}|{}|{}|{}", t1.0, t1.1, t1.2, t2.2);
                        if seen.insert(key) {
                            drafts.push(DraftAlert {
                                pattern: "round_trip",
                                severity: Severity::High,
                                score: Severity::High.base_score(),
                                entity_ids: vec![format!("bank_acc:{}", t1.0), format!("bank_acc:{}", t1.1)],
                                evidence_event_ids: vec![t1.3.clone(), t2.3.clone()],
                                summary: format!(
                                    "Round-trip detected: {} \u{2192} {} \u{2192} {} within {}h",
                                    t1.0, t1.1, t1.0,
                                    diff / 3600
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Rule: Tower jump — impossible travel between towers (>50km in <30min)
fn tower_jump_rule(
    rows: &[(String, String, String, String, String, Option<f64>, Option<String>)],
    drafts: &mut Vec<DraftAlert>,
) {
    let mut per_entity: HashMap<String, Vec<(f64, f64, String, String)>> = HashMap::new();
    for (id, src, entity, _etype, ts, _value, raw) in rows {
        if src != "cdr" {
            continue;
        }
        let Some(raw_str) = raw.as_ref() else { continue };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_str) else { continue };
        let Some(obj) = json.as_object() else { continue };
        let lat = match obj.get("Lat").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()) {
            Some(v) => v,
            None => continue,
        };
        let lng = match obj.get("Lng").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()) {
            Some(v) => v,
            None => continue,
        };
        if lat != 0.0 && lng != 0.0 {
            if let Some(t) = parse_ts(ts) {
                per_entity.entry(entity.clone()).or_default().push((lat, lng, t.to_string(), id.clone()));
            }
        }
    }

    for (entity, points) in &per_entity {
        if points.len() < 2 {
            continue;
        }
        let mut sorted = points.clone();
        sorted.sort_by(|a, b| a.2.cmp(&b.2));

        for window in sorted.windows(2) {
            if let (Some(t1), Some(t2)) = (
                chrono::DateTime::parse_from_rfc3339(&window[0].2).ok(),
                chrono::DateTime::parse_from_rfc3339(&window[1].2).ok(),
            ) {
                let time_diff = (t2 - t1).num_minutes().abs();
                if time_diff > TOWER_JUMP_MAX_MINUTES {
                    continue;
                }
                let dist = haversine_km(window[0].0, window[0].1, window[1].0, window[1].1);
                if dist > TOWER_JUMP_MIN_KM {
                    let speed_kmh = dist / (time_diff as f64 / 60.0);
                    let score = (Severity::Medium.base_score() + (dist / 10.0) as i64).clamp(0, 100);
                    drafts.push(DraftAlert {
                        pattern: "tower_jump",
                        severity: Severity::Medium,
                        score,
                        entity_ids: vec![format!("phone:{entity}")],
                        evidence_event_ids: vec![window[0].3.clone(), window[1].3.clone()],
                        summary: format!(
                            "Entity {} moved {:.0}km in {} minutes (avg {:.0} km/h) \u{2014} impossible travel",
                            entity, dist, time_diff, speed_kmh
                        ),
                    });
                    break;
                }
            }
        }
    }
}

/// Haversine distance between two lat/lng points (in km)
fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Earth radius in km
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}

