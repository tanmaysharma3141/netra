use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::auth::Authed;
use crate::models::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default)]
    pub search_type: Option<String>, // "entity", "alert", "case", or None for all
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(serde::Serialize)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total: usize,
}

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub result_type: String, // "entity", "alert", "case"
    pub case_id: String,
    pub case_title: String,
    pub identifier: String,
    pub detail: serde_json::Value,
}

pub async fn search(
    State(state): State<AppState>,
    _authed: Authed,
    Query(q): Query<SearchQuery>,
) -> Result<Json<SearchResults>, Response> {
    if q.q.trim().is_empty() {
        return Err(ApiError::new("bad_request", "search query cannot be empty")
            .into_response(StatusCode::BAD_REQUEST));
    }

    let pattern = format!("%{}%", q.q.trim());
    let limit = q.limit.min(200) as i64;
    let mut results: Vec<SearchResult> = Vec::new();

    let include_entities = q.search_type.as_deref() != Some("alert") && q.search_type.as_deref() != Some("case");
    let include_alerts = q.search_type.as_deref() != Some("entity") && q.search_type.as_deref() != Some("case");
    let include_cases = q.search_type.as_deref() != Some("entity") && q.search_type.as_deref() != Some("alert");

    // Search entities
    if include_entities {
        let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT e.id, e.case_id, c.title, e.identifier, e.type FROM entities e JOIN cases c ON c.id = e.case_id WHERE e.identifier LIKE ?1 LIMIT ?2",
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

        for (eid, cid, title, identifier, etype) in rows {
            results.push(SearchResult {
                result_type: "entity".into(),
                case_id: cid,
                case_title: title,
                identifier,
                detail: serde_json::json!({ "entity_id": eid, "entity_type": etype }),
            });
        }
    }

    // Search alerts
    if include_alerts {
        let rows: Vec<(String, String, String, String, String, i64, String)> = sqlx::query_as(
            "SELECT a.id, a.case_id, c.title, a.pattern, a.severity, a.score, a.summary FROM alerts a JOIN cases c ON c.id = a.case_id WHERE a.summary LIKE ?1 OR a.pattern LIKE ?1 LIMIT ?2",
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

        for (aid, cid, title, pattern, severity, score, summary) in rows {
            results.push(SearchResult {
                result_type: "alert".into(),
                case_id: cid,
                case_title: title,
                identifier: pattern,
                detail: serde_json::json!({ "alert_id": aid, "severity": severity, "score": score, "summary": summary }),
            });
        }
    }

    // Search cases
    if include_cases {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT id, title, tags FROM cases WHERE title LIKE ?1 OR tags LIKE ?1 OR id LIKE ?1 LIMIT ?2",
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

        for (cid, title, tags) in rows {
            results.push(SearchResult {
                result_type: "case".into(),
                case_id: cid.clone(),
                case_title: title.clone(),
                identifier: title,
                detail: serde_json::json!({ "tags": tags }),
            });
        }
    }

    let total = results.len();
    results.truncate(limit as usize);

    Ok(Json(SearchResults { results, total }))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
