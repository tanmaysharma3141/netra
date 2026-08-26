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

pub async fn ask(
    State(state): State<AppState>,
    _authed: crate::auth::Authed,
    Path(case_id): Path<Uuid>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, Response> {
    let question = req.question.trim().to_lowercase();
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

    // Extract keywords from the question (split on whitespace, filter short words)
    let keywords: Vec<String> = question
        .split_whitespace()
        .filter(|w| w.len() >= 2 && !["the", "is", "at", "in", "of", "to", "and", "or", "for", "show", "get", "find", "list", "what", "how", "all", "any", "this", "that", "with", "from", "about", "have", "has", "are", "was", "were", "been", "being", "do", "does", "did", "will", "would", "could", "should", "may", "might", "can"].contains(w))
        .map(String::from)
        .collect();

    let search_term = if keywords.is_empty() {
        question.clone()
    } else {
        keywords.join(" ")
    };

    // Search events
    let event_rows: Vec<(String, String, String, String, String, Option<f64>, String)> =
        sqlx::query_as(
            "SELECT id, source_type, entity_id, entity_type, event_type, value, raw FROM events WHERE case_id = ?1 AND (raw LIKE ?2 OR entity_id LIKE ?2) LIMIT 20",
        )
        .bind(&cid)
        .bind(format!("%{search_term}%"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

    // Search entities
    let entity_rows: Vec<(String, String, String, Option<String>)> =
        sqlx::query_as(
            "SELECT id, type, identifier, display_name FROM entities WHERE case_id = ?1 AND (identifier LIKE ?2 OR display_name LIKE ?2) LIMIT 10",
        )
        .bind(&cid)
        .bind(format!("%{search_term}%"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

    // Search alerts
    let alert_rows: Vec<(String, String, String, i64, String)> =
        sqlx::query_as(
            "SELECT id, pattern, severity, score, summary FROM alerts WHERE case_id = ?1 AND (summary LIKE ?2 OR pattern LIKE ?2) LIMIT 10",
        )
        .bind(&cid)
        .bind(format!("%{search_term}%"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

    // Get case stats for context
    let stats: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM events WHERE case_id = ?1), (SELECT COUNT(*) FROM entities WHERE case_id = ?1), (SELECT COUNT(*) FROM alerts WHERE case_id = ?1)",
    )
    .bind(&cid)
    .fetch_one(&state.pool)
    .await
    .map_err(internal)?;

    // Build answer
    let mut answer = String::new();
    let mut source_event_ids: Vec<uuid::Uuid> = Vec::new();

    if !entity_rows.is_empty() {
        answer += &format!("Found {} matching entities:\n", entity_rows.len());
        for (id, etype, identifier, display_name) in &entity_rows {
            let label = display_name.as_deref().unwrap_or(identifier);
            answer += &format!("  • {} ({}) — {}\n", label, etype, identifier);
            if let Ok(uid) = Uuid::parse_str(id) {
                source_event_ids.push(uid);
            }
        }
        answer += "\n";
    }

    if !event_rows.is_empty() {
        answer += &format!("Found {} matching events:\n", event_rows.len());
        for (id, source, entity, _etype, etype, value, _raw) in &event_rows {
            let val_str = value.map(|v| format!(" (value: {:.2})", v)).unwrap_or_default();
            answer += &format!("  • [{}] {} {}{}\n", source, entity, etype, val_str);
            if let Ok(uid) = Uuid::parse_str(id) {
                source_event_ids.push(uid);
            }
        }
        answer += "\n";
    }

    if !alert_rows.is_empty() {
        answer += &format!("Found {} matching alerts:\n", alert_rows.len());
        for (id, pattern, severity, score, summary) in &alert_rows {
            answer += &format!("  • [{}] {} (score: {}/100) — {}\n", severity, pattern, score, summary);
            if let Ok(uid) = Uuid::parse_str(id) {
                source_event_ids.push(uid);
            }
        }
        answer += "\n";
    }

    if answer.is_empty() {
        answer = format!(
            "No results found for \"{}\" in this case. The case has {} events, {} entities, and {} alerts. Try searching for phone numbers, IMEI, account numbers, or alert patterns.",
            req.question, stats.0, stats.1, stats.2
        );
    } else {
        // Add summary at the end
        answer += &format!(
            "Case summary: {} events, {} entities, {} alerts total.",
            stats.0, stats.1, stats.2
        );
    }

    // Stream the answer in chunks
    let chunks: Vec<String> = answer
        .chars()
        .collect::<Vec<char>>()
        .chunks(30)
        .map(|c| c.iter().collect())
        .collect();

    let state_clone = state.clone();
    let sources = source_event_ids;

    let s = stream::iter(chunks.into_iter().enumerate())
        .then(move |(_, chunk)| {
            let _ = &state_clone;
            async move {
                tokio::time::sleep(Duration::from_millis(40)).await;
                Ok(Event::default()
                    .data(serde_json::to_string(&ChatFrame::delta(chunk)).unwrap()))
            }
        })
        .chain(stream::once(async move {
            Ok(Event::default().data(
                serde_json::to_string(&ChatFrame::sources(sources)).unwrap(),
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
