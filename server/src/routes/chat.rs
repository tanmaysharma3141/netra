use std::convert::Infallible;
use std::time::Duration;

use futures_util::stream;
use futures_util::StreamExt;
use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::Response;
use axum::Json;

use crate::models::{ApiError, ChatFrame, ChatRequest};
use crate::state::AppState;

// ── Intent detection ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Intent {
    SuspiciousEntities,
    AlertSummary,
    EntityLookup,
    WhoCalled,
    ConnectionsBetween,
    Timeline,
    Locations,
    Financial,
    CaseOverview,
    ImeiReuse,
    FreeSearch,
}

fn detect_intent(question: &str) -> (Intent, Option<String>) {
    let q = question.to_lowercase();
    let q_words: Vec<&str> = q.split_whitespace().collect();

    // Extract phone/IMEI-like tokens for lookup
    fn extract_id_token(words: &[&str]) -> Option<String> {
        for w in words {
            let clean: String = w.chars().filter(|c| c.is_alphanumeric() || *c == '+' || *c == '-').collect();
            // Phone number: starts with +91 or 10+ digits
            if clean.starts_with("+91") && clean.len() >= 12 {
                return Some(clean);
            }
            // IMEI: 15 digits
            let digits: String = clean.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() == 15 {
                return Some(digits);
            }
            // Bank account: 8-18 digits starting with specific patterns
            if digits.len() >= 8 && digits.len() <= 18
                && (digits.starts_with("3") || digits.starts_with("4") || digits.starts_with("5") || digits.starts_with("6"))
            {
                return Some(digits);
            }
        }
        None
    }

    // Pattern matching — most specific first
    if q.contains("imei reuse") || q.contains("shared across") || q.contains("same imei") {
        return (Intent::ImeiReuse, None);
    }
    if q.contains("suspicious") || q.contains("dangerous") || q.contains("risk")
        || q.contains("threat") || q.contains("concern")
    {
        return (Intent::SuspiciousEntities, None);
    }
    if q.contains("alert") || q.contains("anomal") || q.contains("pattern")
        || q.contains("detect") || q.contains("flag")
    {
        return (Intent::AlertSummary, None);
    }
    if q.contains("who called") || q.contains("who phoned") || q.contains("caller")
        || (q.contains("call") && q.contains("between"))
    {
        return (Intent::WhoCalled, extract_id_token(&q_words));
    }
    if q.contains("connect") || q.contains("link") || q.contains("relat")
        || q.contains("association") || q.contains("tied") || q.contains("together")
    {
        return (Intent::ConnectionsBetween, extract_id_token(&q_words));
    }
    if q.contains("timeline") || q.contains("chronolog") || q.contains("sequence")
        || q.contains("when") || q.contains("recent") || q.contains("latest")
        || q.contains("last")
    {
        return (Intent::Timeline, None);
    }
    if q.contains("location") || q.contains("where") || q.contains("movement")
        || q.contains("tower") || q.contains("travel") || q.contains("place")
    {
        return (Intent::Locations, None);
    }
    if q.contains("bank") || q.contains("transaction") || q.contains("money")
        || q.contains("fund") || q.contains("transfer") || q.contains("hawala")
        || q.contains("account")
    {
        return (Intent::Financial, None);
    }
    if q.contains("summar") || q.contains("overview") || q.contains("about this")
        || q.contains("what is") || q.contains("tell me") || q.contains("brief")
        || q.contains("case") && !q.contains("entity")
    {
        return (Intent::CaseOverview, None);
    }

    // Check for entity-like tokens → entity lookup
    if let Some(token) = extract_id_token(&q_words) {
        return (Intent::EntityLookup, Some(token));
    }

    // Entity type keywords
    if q.contains("phone") || q.contains("imei") || q.contains("ip")
        || q.contains("handle") || q.contains("entity") || q.contains("entities")
    {
        return (Intent::EntityLookup, None);
    }

    (Intent::FreeSearch, None)
}

// ── Query builders ────────────────────────────────────────────────────────────

async fn get_case_stats(pool: &sqlx::SqlitePool, cid: &str) -> (i64, i64, i64, i64) {
    sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT \
           (SELECT COUNT(*) FROM events WHERE case_id = ?1), \
           (SELECT COUNT(*) FROM entities WHERE case_id = ?1), \
           (SELECT COUNT(*) FROM alerts WHERE case_id = ?1), \
           (SELECT COUNT(*) FROM entity_edges WHERE case_id = ?1)",
    )
    .bind(cid)
    .fetch_one(pool)
    .await
    .unwrap_or((0, 0, 0, 0))
}

async fn get_case_title(pool: &sqlx::SqlitePool, cid: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT title FROM cases WHERE id = ?1")
        .bind(cid)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "Unknown case".into())
}

async fn answer_suspicious(pool: &sqlx::SqlitePool, cid: &str) -> String {
    let title = get_case_title(pool, cid).await;
    let stats = get_case_stats(pool, cid).await;

    // Top alerts by score
    let alerts: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT pattern, severity, score, summary FROM alerts WHERE case_id = ?1 ORDER BY score DESC LIMIT 10",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Entities involved in critical alerts
    let critical_entities: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT DISTINCT e.identifier, e.type, e.display_name, COALESCE(a.score, 0) as score \
         FROM entities e \
         JOIN alerts a ON a.case_id = e.case_id \
         WHERE e.case_id = ?1 AND a.score >= 80 \
         ORDER BY score DESC LIMIT 10",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut answer = format!("📊 **Case: {}**\n", title);
    answer += &format!("   {} events · {} entities · {} alerts · {} relationships\n\n", stats.0, stats.1, stats.2, stats.3);

    answer += "**🚨 Most Suspicious Entities**\n";
    if critical_entities.is_empty() {
        answer += "No entities flagged with critical scores. Try checking alerts tab for medium-severity patterns.\n";
    } else {
        for (id, etype, name, score) in &critical_entities {
            let label = if name.is_empty() { id.as_str() } else { name };
            answer += &format!("   • **{}** ({}) — Risk score: {}/100\n", label, etype, score);
        }
    }

    answer += "\n**🔍 Top Alert Patterns**\n";
    if alerts.is_empty() {
        answer += "No alerts generated yet. Run analysis from the Alerts tab.";
    } else {
        for (pattern, severity, score, summary) in alerts.iter().take(5) {
            answer += &format!("   • [{}] {} (score: {}/100)\n     ↳ {}\n", severity.to_uppercase(), pattern, score, summary);
        }
    }

    answer += "\n💡 Click on any alert in the Alerts tab to triage it as confirmed or false positive.";
    answer
}

async fn answer_alert_summary(pool: &sqlx::SqlitePool, cid: &str) -> String {
    let title = get_case_title(pool, cid).await;

    // Alert counts by severity
    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT severity, COUNT(*) as cnt FROM alerts WHERE case_id = ?1 GROUP BY severity ORDER BY cnt DESC",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Pattern breakdown
    let patterns: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT pattern, severity, COUNT(*) as cnt FROM alerts WHERE case_id = ?1 GROUP BY pattern ORDER BY cnt DESC",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Status breakdown
    let statuses: Vec<(String, i64)> = sqlx::query_as(
        "SELECT status, COUNT(*) FROM alerts WHERE case_id = ?1 GROUP BY status",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let total: i64 = counts.iter().map(|(_, c)| c).sum();

    let mut answer = format!("📊 **Alert Summary — {}**\n\n", title);
    answer += &format!("**{} total alerts**\n\n", total);

    answer += "**By Severity:**\n";
    for (sev, cnt) in &counts {
        let bar = "█".repeat(*cnt as usize);
        answer += &format!("   {} {:>3}  {}\n", sev.to_uppercase(), cnt, bar);
    }

    answer += "\n**By Pattern:**\n";
    for (pattern, severity, cnt) in &patterns {
        answer += &format!("   • {} ({}) — {} alert{}\n", pattern, severity, cnt, if *cnt == 1 { "" } else { "s" });
    }

    if !statuses.is_empty() {
        answer += "\n**Triage Status:**\n";
        for (status, cnt) in &statuses {
            answer += &format!("   • {} — {}\n", status, cnt);
        }
    }

    answer += "\n💡 Go to Alert Center to triage all alerts at once.";
    answer
}

async fn answer_entity_lookup(pool: &sqlx::SqlitePool, cid: &str, query: Option<String>) -> String {
    let title = get_case_title(pool, cid).await;

    let (entities, pattern_used) = if let Some(ref token) = query {
        // Direct identifier search
        let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, type, identifier, display_name FROM entities WHERE case_id = ?1 AND identifier LIKE ?2 LIMIT 10",
        )
        .bind(cid)
        .bind(format!("%{}%", token))
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        (rows, token.clone())
    } else {
        // Keyword search on type/name
        let q = query.as_deref().unwrap_or("");
        let type_filter = match q {
            s if s.contains("phone") => "phone",
            s if s.contains("imei") => "imei",
            s if s.contains("bank") => "bank_acc",
            s if s.contains("ip") => "ip",
            s if s.contains("handle") => "handle",
            _ => "%",
        };
        let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, type, identifier, display_name FROM entities WHERE case_id = ?1 AND type LIKE ?2 LIMIT 15",
        )
        .bind(cid)
        .bind(type_filter)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        (rows, type_filter.to_string())
    };

    let mut answer = format!("🔍 **Entity Search — {}**\n", title);
    answer += &format!("Pattern: `{}`\n\n", pattern_used);

    if entities.is_empty() {
        answer += "No matching entities found. Try a different phone number, IMEI, or identifier.";
    } else {
        answer += &format!("Found {} matching entit{}:\n\n", entities.len(), if entities.len() == 1 { "y" } else { "ies" });
        for (_id, etype, identifier, display_name) in &entities {
            let label = display_name.as_deref().unwrap_or(identifier);
            answer += &format!("   • **{}** ({}) — `{}`\n", label, etype, identifier);

            // Show alerts for this entity
            let alerts: Vec<(String, String, i64)> = sqlx::query_as(
                "SELECT pattern, severity, score FROM alerts WHERE case_id = ?1 AND summary LIKE ?2 LIMIT 3",
            )
            .bind(cid)
            .bind(format!("%{}%", identifier))
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            for (pattern, severity, score) in &alerts {
                answer += &format!("     ↳ ⚠️ [{}] {} (score: {}/100)\n", severity.to_uppercase(), pattern, score);
            }
        }
    }

    answer += "\n💡 Click a node in the Graph tab to see full connections for any entity.";
    answer
}

async fn answer_timeline(pool: &sqlx::SqlitePool, cid: &str) -> String {
    let title = get_case_title(pool, cid).await;

    // Recent events with entity info
    let events: Vec<(String, String, String, String, String, Option<f64>)> = sqlx::query_as(
        "SELECT e.entity_id, e.entity_type, e.event_type, e.source_type, e.timestamp, e.value \
         FROM events e WHERE e.case_id = ?1 ORDER BY e.timestamp DESC LIMIT 20",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Activity by hour
    let hourly: Vec<(String, i64)> = sqlx::query_as(
        "SELECT SUBSTR(timestamp, 12, 2) as hour, COUNT(*) as cnt \
         FROM events WHERE case_id = ?1 GROUP BY hour ORDER BY hour",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Activity by event type
    let by_type: Vec<(String, i64)> = sqlx::query_as(
        "SELECT event_type, COUNT(*) FROM events WHERE case_id = ?1 GROUP BY event_type ORDER BY COUNT(*) DESC",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut answer = format!("📅 **Timeline — {}**\n\n", title);

    // Activity distribution
    answer += "**Activity Distribution:**\n";
    for (etype, cnt) in &by_type {
        let bar_len = (*cnt as usize).min(30);
        let bar = "█".repeat(bar_len);
        answer += &format!("   {} {:>5}  {}\n", etype.to_uppercase(), cnt, bar);
    }

    // Peak hours
    if !hourly.is_empty() {
        let max_hour = hourly.iter().max_by_key(|(_, c)| c).unwrap();
        answer += &format!("\n⏰ **Peak activity hour:** {}:00 UTC ({} events)\n", max_hour.0, max_hour.1);
    }

    answer += "\n**Recent Events (latest 10):**\n";
    for (entity_id, etype, event_type, source, ts, value) in events.iter().take(10) {
        let val_str = value.map(|v| format!(" — value: {:.0}", v)).unwrap_or_default();
        answer += &format!("   [{}] {} {} → {}{}\n", source.to_uppercase(), etype, event_type, entity_id, val_str);
        // Format timestamp nicely
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f") {
            answer = answer.replacen(ts, &dt.format("%d/%m %H:%M").to_string(), 1);
        }
    }

    answer += "\n💡 Use the Timeline tab for interactive chronological view with filters.";
    answer
}

async fn answer_locations(pool: &sqlx::SqlitePool, cid: &str) -> String {
    let title = get_case_title(pool, cid).await;

    // Entities with movement trails
    let trails: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT entity_id, entity_type, COUNT(*) as pings \
         FROM events WHERE case_id = ?1 AND location_lat IS NOT NULL \
         GROUP BY entity_id ORDER BY pings DESC LIMIT 10",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Tower distribution
    let towers: Vec<(String, i64)> = sqlx::query_as(
        "SELECT COALESCE(tower_id, 'unknown'), COUNT(*) FROM events WHERE case_id = ?1 AND tower_id IS NOT NULL \
         GROUP BY tower_id ORDER BY COUNT(*) DESC LIMIT 10",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let total_pings: i64 = trails.iter().map(|(_, _, c)| c).sum();

    let mut answer = format!("📍 **Location Analysis — {}**\n\n", title);
    answer += &format!("**{} geo-located pings** from {} entities\n\n", total_pings, trails.len());

    if !trails.is_empty() {
        answer += "**Entities with movement trails:**\n";
        for (id, etype, pings) in &trails {
            answer += &format!("   • **{}** ({}) — {} location pings\n", id, etype, pings);
        }
    } else {
        answer += "No geo-located events found. Tower resolution may not have run yet.\n";
    }

    if !towers.is_empty() {
        answer += "\n**Most active cell towers:**\n";
        for (tower, cnt) in towers.iter().take(5) {
            answer += &format!("   • Tower {} — {} events\n", tower, cnt);
        }
    }

    answer += "\n💡 Switch to the Map tab to see movement trails visually with playback animation.";
    answer
}

async fn answer_financial(pool: &sqlx::SqlitePool, cid: &str) -> String {
    let title = get_case_title(pool, cid).await;

    // Bank accounts
    let accounts: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT identifier, display_name FROM entities WHERE case_id = ?1 AND type = 'bank_acc' LIMIT 10",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Financial transactions
    let txns: Vec<(String, String, String, Option<f64>)> = sqlx::query_as(
        "SELECT e.entity_id, e.event_type, e.timestamp, e.value \
         FROM events e WHERE e.case_id = ?1 AND e.source_type = 'bank' \
         ORDER BY e.timestamp DESC LIMIT 15",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let total_txn_value: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(value) FROM events WHERE case_id = ?1 AND source_type = 'bank' AND value IS NOT NULL",
    )
    .bind(cid)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten();

    let mut answer = format!("💰 **Financial Analysis — {}**\n\n", title);

    if accounts.is_empty() {
        answer += "No banking data ingested yet. Upload bank statements via the Ingest tab.\n";
        answer += "\nThis case currently only contains telecom CDR data.";
    } else {
        answer += &format!("**{} bank accounts identified:**\n", accounts.len());
        for (id, name) in &accounts {
            let label = name.as_deref().unwrap_or(id);
            answer += &format!("   • **{}** — `{}`\n", label, id);
        }

        if let Some(total) = total_txn_value {
            answer += &format!("\n**Total transaction value:** ₹{:.2}\n", total);
        }

        if !txns.is_empty() {
            answer += "\n**Recent transactions:**\n";
            for (id, etype, _ts, value) in txns.iter().take(10) {
                let val_str = value.map(|v| format!("₹{:.2}", v)).unwrap_or_else(|| "N/A".to_string());
                answer += &format!("   • {} {} — {}\n", id, etype, val_str);
            }
        }

        // Cross-domain links
        let cross: Vec<(String, String)> = sqlx::query_as(
            "SELECT e.identifier, e2.identifier FROM entity_edges ee \
             JOIN entities e ON ee.source_entity_id = e.id \
             JOIN entities e2 ON ee.target_entity_id = e2.id \
             WHERE ee.case_id = ?1 AND e.type = 'bank_acc' \
             LIMIT 5",
        )
        .bind(cid)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        if !cross.is_empty() {
            answer += "\n**Linked entities (bank → telecom):**\n";
            for (bank, other) in &cross {
                answer += &format!("   • {} ↔ {}\n", bank, other);
            }
        }
    }

    answer
}

async fn answer_case_overview(pool: &sqlx::SqlitePool, cid: &str) -> String {
    let title = get_case_title(pool, cid).await;
    let stats = get_case_stats(pool, cid).await;

    // Source breakdown
    let sources: Vec<(String, i64)> = sqlx::query_as(
        "SELECT source_type, COUNT(*) FROM events WHERE case_id = ?1 GROUP BY source_type",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Entity type breakdown
    let entity_types: Vec<(String, i64)> = sqlx::query_as(
        "SELECT type, COUNT(*) FROM entities WHERE case_id = ?1 GROUP BY type ORDER BY COUNT(*) DESC",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Top alerts
    let top_alerts: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT pattern, severity, score FROM alerts WHERE case_id = ?1 ORDER BY score DESC LIMIT 3",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Case tags (stored as JSON array text)
    let tags_raw: Option<String> = sqlx::query_scalar(
        "SELECT tags FROM cases WHERE id = ?1",
    )
    .bind(cid)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten();
    let tags: Vec<String> = tags_raw
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default();

    let mut answer = format!("📋 **Case Overview — {}**\n\n", title);

    if !tags.is_empty() {
        answer += &format!("Tags: {}\n", tags.join(", "));
    }

    answer += "**Data Summary:**\n";
    answer += &format!("   📞 Events: {}\n", stats.0);
    answer += &format!("   👤 Entities: {}\n", stats.1);
    answer += &format!("   🚨 Alerts: {}\n", stats.2);
    answer += &format!("   🔗 Relationships: {}\n\n", stats.3);

    answer += "**Events by Source:**\n";
    for (source, cnt) in &sources {
        answer += &format!("   • {} — {}\n", source.to_uppercase(), cnt);
    }

    if !entity_types.is_empty() {
        answer += "\n**Entities by Type:**\n";
        for (etype, cnt) in &entity_types {
            answer += &format!("   • {} — {}\n", etype, cnt);
        }
    }

    if !top_alerts.is_empty() {
        answer += "\n**Top Alerts:**\n";
        for (pattern, severity, score) in &top_alerts {
            answer += &format!("   • [{}] {} — score: {}/100\n", severity.to_uppercase(), pattern, score);
        }
    }

    answer += "\n💡 Navigate to individual tabs (Timeline, Graph, Map, Alerts) for detailed views.";
    answer
}

async fn answer_imei_reuse(pool: &sqlx::SqlitePool, cid: &str) -> String {
    let title = get_case_title(pool, cid).await;

    // IMEI reuse alerts
    let imei_alerts: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT summary, score, severity FROM alerts WHERE case_id = ?1 AND pattern = 'imei_reuse' ORDER BY score DESC",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // IMEI entities
    let imeis: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT e.identifier, e.display_name, e.link_tier \
         FROM entities e WHERE e.case_id = ?1 AND e.type = 'imei' LIMIT 10",
    )
    .bind(cid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut answer = format!("📱 **IMEI Reuse Analysis — {}**\n\n", title);

    if imei_alerts.is_empty() {
        answer += "No IMEI reuse patterns detected in this case.";
    } else {
        answer += &format!("**{} IMEI reuse alert(s) detected:**\n\n", imei_alerts.len());
        for (summary, score, severity) in &imei_alerts {
            answer += &format!("🚨 **[{}] Score: {}/100**\n", severity.to_uppercase(), score);
            answer += &format!("   ↳ {}\n\n", summary);
        }

        answer += "**Affected IMEIs:**\n";
        for (imei, name, tier) in &imeis {
            let label = name.as_deref().unwrap_or("Unknown");
            let tier_str = if tier.is_empty() { "ungraded" } else { tier };
            answer += &format!("   • IMEI `{}` — {} (tier: {})\n", imei, label, tier_str);
        }

        answer += "\n⚠️ IMEI reuse across multiple SIM cards strongly suggests device sharing or cloning.\n";
        answer += "This is a key indicator in fraud and organized crime investigations.";
    }

    answer += "\n💡 Click the IMEI entity in the Graph tab to see all subscriber lines linked to it.";
    answer
}

async fn answer_free_search(pool: &sqlx::SqlitePool, cid: &str, question: &str) -> String {
    let title = get_case_title(pool, cid).await;

    // Broad search across all tables
    let q = format!("%{}%", question);

    let events: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT entity_id, entity_type, event_type, source_type FROM events \
         WHERE case_id = ?1 AND (raw LIKE ?2 OR entity_id LIKE ?2 OR event_type LIKE ?2) LIMIT 10",
    )
    .bind(cid)
    .bind(&q)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let entities: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, type, identifier, display_name FROM entities \
         WHERE case_id = ?1 AND (identifier LIKE ?2 OR display_name LIKE ?2) LIMIT 10",
    )
    .bind(cid)
    .bind(&q)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let alerts: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT pattern, severity, score, summary FROM alerts \
         WHERE case_id = ?1 AND (summary LIKE ?2 OR pattern LIKE ?2) LIMIT 5",
    )
    .bind(cid)
    .bind(&q)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let stats = get_case_stats(pool, cid).await;

    let mut answer = format!("🔍 **Search Results — {}**\n", title);

    if entities.is_empty() && events.is_empty() && alerts.is_empty() {
        answer += &format!("\nNo results found for \"{}\".\n\n", question);
        answer += &format!("This case has {} events, {} entities, {} alerts, and {} relationships.\n\n", stats.0, stats.1, stats.2, stats.3);
        answer += "Try asking about:\n";
        answer += "   • Suspicious entities or risk scores\n";
        answer += "   • Alert patterns or anomalies\n";
        answer += "   • Timeline or recent events\n";
        answer += "   • Locations or movement\n";
        answer += "   • Financial transactions\n";
        answer += "   • IMEI reuse patterns\n";
        answer += "   • Specific phone numbers or identifiers";
    } else {
        if !entities.is_empty() {
            answer += &format!("\n**Entities ({} found):**\n", entities.len());
            for (_, etype, id, name) in &entities {
                let label = name.as_deref().unwrap_or(id);
                answer += &format!("   • {} ({}) — `{}`\n", label, etype, id);
            }
        }
        if !alerts.is_empty() {
            answer += &format!("\n**Alerts ({} found):**\n", alerts.len());
            for (pattern, severity, score, summary) in &alerts {
                answer += &format!("   • [{}] {} — {}/100\n     ↳ {}\n", severity.to_uppercase(), pattern, score, summary);
            }
        }
        if !events.is_empty() {
            answer += &format!("\n**Events ({} found):**\n", events.len());
            for (eid, etype, ev_type, source) in events.iter().take(5) {
                answer += &format!("   • [{}] {} {} — {}\n", source.to_uppercase(), etype, ev_type, eid);
            }
        }
    }

    answer += &format!("\n\n📊 Case totals: {} events · {} entities · {} alerts · {} relationships", stats.0, stats.1, stats.2, stats.3);
    answer
}

// ── Main handler ──────────────────────────────────────────────────────────────

pub async fn ask(
    State(state): State<AppState>,
    _authed: crate::auth::Authed,
    Path(case_id): Path<Uuid>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, Response> {
    let question = req.question.trim().to_string();
    if question.is_empty() {
        return Err(ApiError::new("bad_request", "question cannot be empty")
            .into_response(StatusCode::BAD_REQUEST));
    }

    let cid = case_id.to_string();

    // Check case exists
    let case_exists: Option<String> = sqlx::query_scalar("SELECT id FROM cases WHERE id = ?1")
        .bind(&cid)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    if case_exists.is_none() {
        return Err(ApiError::new("not_found", "case not found")
            .into_response(StatusCode::NOT_FOUND));
    }

    // Detect intent and build answer
    let (intent, token) = detect_intent(&question);

    let answer = match intent {
        Intent::SuspiciousEntities => answer_suspicious(&state.pool, &cid).await,
        Intent::AlertSummary => answer_alert_summary(&state.pool, &cid).await,
        Intent::EntityLookup => answer_entity_lookup(&state.pool, &cid, token).await,
        Intent::WhoCalled | Intent::ConnectionsBetween => answer_entity_lookup(&state.pool, &cid, token).await,
        Intent::Timeline => answer_timeline(&state.pool, &cid).await,
        Intent::Locations => answer_locations(&state.pool, &cid).await,
        Intent::Financial => answer_financial(&state.pool, &cid).await,
        Intent::CaseOverview => answer_case_overview(&state.pool, &cid).await,
        Intent::ImeiReuse => answer_imei_reuse(&state.pool, &cid).await,
        Intent::FreeSearch => answer_free_search(&state.pool, &cid, &question).await,
    };

    // Stream the answer in chunks
    let chunks: Vec<String> = answer
        .chars()
        .collect::<Vec<char>>()
        .chunks(20)
        .map(|c| c.iter().collect())
        .collect();

    let state_clone = state.clone();
    let cid_clone = cid;

    // Collect source IDs from entity matches
    let source_ids: Vec<uuid::Uuid> = sqlx::query_scalar::<_, String>(
        "SELECT id FROM entities WHERE case_id = ?1 AND (identifier LIKE ?2 OR id IN \
         (SELECT id FROM alerts WHERE case_id = ?1)) LIMIT 10",
    )
    .bind(&cid_clone)
    .bind(format!("%{}%", question))
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default()
    .iter()
    .filter_map(|s| uuid::Uuid::parse_str(s).ok())
    .collect();

    let s = stream::iter(chunks.into_iter().enumerate())
        .then(move |(_, chunk)| {
            let _ = &state_clone;
            async move {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok(Event::default()
                    .data(serde_json::to_string(&ChatFrame::delta(chunk)).unwrap()))
            }
        })
        .chain(stream::once(async move {
            Ok(Event::default().data(
                serde_json::to_string(&ChatFrame::sources(source_ids)).unwrap(),
            ))
        }))
        .chain(stream::once(async {
            Ok(Event::default().data(serde_json::to_string(&ChatFrame::done()).unwrap()))
        }));

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
