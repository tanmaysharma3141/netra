use std::collections::{HashMap, HashSet, VecDeque};

use sqlx::FromRow;
use uuid::Uuid;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::auth::Authed;
use crate::models::{ApiError, Entity, EntityType, Graph, GraphEdge, GraphNode, Role};
use crate::state::AppState;

#[derive(Debug, FromRow)]
pub struct EntityRow {
    pub id: String,
    pub case_id: String,
    pub r#type: String,
    pub identifier: String,
    pub display_name: Option<String>,
    pub link_tier: Option<String>,
    pub tags: String,
}

impl EntityRow {
    fn to_api(&self) -> Entity {
        Entity {
            id: Uuid::parse_str(&self.id).unwrap_or_default(),
            case_id: Uuid::parse_str(&self.case_id).unwrap_or_default(),
            entity_type: self.r#type.parse().unwrap_or(EntityType::Phone),
            identifier: self.identifier.clone(),
            display_name: self.display_name.clone(),
            link_tier: self.link_tier.as_deref().and_then(|t| t.parse().ok()),
            tags: serde_json::from_str(&self.tags).unwrap_or_default(),
        }
    }
}

#[derive(Debug, FromRow)]
struct EdgeRow {
    id: String,
    source_entity_id: String,
    target_entity_id: String,
    link_type: String,
    tier: String,
    confidence: f64,
    evidence_count: i64,
}

pub async fn list(
    State(state): State<AppState>,
    _authed: Authed,
    Path(case_id): Path<Uuid>,
) -> Result<Json<Vec<Entity>>, Response> {
    let rows: Vec<EntityRow> =
        sqlx::query_as("SELECT * FROM entities WHERE case_id = ?1 ORDER BY identifier")
            .bind(case_id.to_string())
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;
    Ok(Json(rows.iter().map(|r| r.to_api()).collect()))
}

#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    pub entity_id: Option<Uuid>,
    #[serde(default = "default_hops")]
    pub hops: u32,
}

fn default_hops() -> u32 {
    2
}

async fn load_graph(
    state: &AppState,
    case_id: Uuid,
) -> Result<(Vec<EntityRow>, Vec<EdgeRow>), Response> {
    let ents: Vec<EntityRow> =
        sqlx::query_as("SELECT * FROM entities WHERE case_id = ?1")
            .bind(case_id.to_string())
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;
    let edges: Vec<EdgeRow> =
        sqlx::query_as("SELECT id, source_entity_id, target_entity_id, link_type, tier, confidence, evidence_count FROM entity_edges WHERE case_id = ?1")
            .bind(case_id.to_string())
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;
    Ok((ents, edges))
}

pub async fn graph(
    State(state): State<AppState>,
    _authed: Authed,
    Path(case_id): Path<Uuid>,
    Query(q): Query<GraphQuery>,
) -> Result<Json<Graph>, Response> {
    let (ents, edges) = load_graph(&state, case_id).await?;

    if q.entity_id.is_none() {
        return Ok(Json(build_graph(&ents, &edges, None, q.hops)));
    }

    Ok(Json(build_graph(
        &ents,
        &edges,
        Some(q.entity_id.unwrap().to_string()),
        q.hops,
    )))
}

fn build_graph(
    ents: &[EntityRow],
    edges: &[EdgeRow],
    root: Option<String>,
    hops: u32,
) -> Graph {
    let mut adjacency: HashMap<&str, Vec<&EdgeRow>> = HashMap::new();
    for e in edges {
        adjacency.entry(e.source_entity_id.as_str()).or_default().push(e);
        adjacency.entry(e.target_entity_id.as_str()).or_default().push(e);
    }
    let ent_by_id: HashMap<&str, &EntityRow> =
        ents.iter().map(|e| (e.id.as_str(), e)).collect();

    let keep_nodes: HashSet<&str> = match (&root, hops) {
        (None, _) => ents.iter().map(|e| e.id.as_str()).collect(),
        (Some(r), h) => {
            let mut visited: HashSet<&str> = HashSet::new();
            let mut queue: VecDeque<(&str, u32)> = VecDeque::new();
            visited.insert(r.as_str());
            queue.push_back((r.as_str(), 0));
            while let Some((node, depth)) = queue.pop_front() {
                if depth >= h {
                    continue;
                }
                if let Some(neighbors) = adjacency.get(node) {
                    for e in neighbors {
                        let other = if e.source_entity_id == node {
                            e.target_entity_id.as_str()
                        } else {
                            e.source_entity_id.as_str()
                        };
                        if visited.insert(other) {
                            queue.push_back((other, depth + 1));
                        }
                    }
                }
            }
            visited
        }
    };

    let nodes = ents
        .iter()
        .filter(|e| keep_nodes.contains(e.id.as_str()))
        .map(|e| GraphNode {
            id: e.id.clone(),
            node_type: e.r#type.parse().unwrap_or(EntityType::Phone),
            label: e.display_name.clone().unwrap_or_else(|| e.identifier.clone()),
        })
        .collect();

    let out_edges = edges
        .iter()
        .filter(|e| {
            keep_nodes.contains(e.source_entity_id.as_str())
                && keep_nodes.contains(e.target_entity_id.as_str())
        })
        .map(|e| GraphEdge {
            source: e.source_entity_id.clone(),
            target: e.target_entity_id.clone(),
            link_type: e.link_type.clone(),
            tier: e.tier.parse().unwrap_or(crate::models::LinkTier::Medium),
            confidence: e.confidence,
            evidence_count: e.evidence_count as u32,
        })
        .collect();

    Graph { nodes, edges: out_edges }
}

#[derive(serde::Serialize)]
pub struct ConnectionView {
    pub link_type: String,
    pub tier: String,
    pub confidence: f64,
    pub evidence_count: i64,
    pub other_entity_id: String,
    pub other_identifier: String,
    pub other_type: String,
}

#[derive(serde::Serialize)]
pub struct ProfileResponse {
    pub entity: Entity,
    pub stats: ProfileStats,
    pub connections: Vec<ConnectionView>,
}

#[derive(serde::Serialize)]
pub struct ProfileStats {
    pub events: i64,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

pub async fn profile(
    State(state): State<AppState>,
    _authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Json<ProfileResponse>, Response> {
    let row: Option<EntityRow> = sqlx::query_as("SELECT * FROM entities WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    let Some(row) = row else {
        return Err(ApiError::new("not_found", "entity not found").into_response(StatusCode::NOT_FOUND));
    };

    let stats: (i64, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT COUNT(*), MIN(ts), MAX(ts) FROM events WHERE case_id = ?1 AND entity_type = ?2 AND LOWER(entity_id) = LOWER(?3)",
    )
    .bind(&row.case_id)
    .bind(&row.r#type)
    .bind(&row.identifier)
    .fetch_one(&state.pool)
    .await
    .map_err(internal)?;

    let edge_rows: Vec<EdgeRow> = sqlx::query_as(
        "SELECT id, source_entity_id, target_entity_id, link_type, tier, confidence, evidence_count FROM entity_edges WHERE case_id = ?1 AND (source_entity_id = ?2 OR target_entity_id = ?2)",
    )
    .bind(&row.case_id)
    .bind(&row.id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;

    let all_ents: Vec<EntityRow> =
        sqlx::query_as("SELECT * FROM entities WHERE case_id = ?1")
            .bind(&row.case_id)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;
    let by_id: HashMap<&str, &EntityRow> =
        all_ents.iter().map(|e| (e.id.as_str(), e)).collect();

    let connections = edge_rows
        .iter()
        .map(|e| {
            let other_id = if e.source_entity_id == row.id {
                e.target_entity_id.as_str()
            } else {
                e.source_entity_id.as_str()
            };
            let other = by_id.get(other_id).copied();
            ConnectionView {
                link_type: e.link_type.clone(),
                tier: e.tier.clone(),
                confidence: e.confidence,
                evidence_count: e.evidence_count,
                other_entity_id: other_id.to_string(),
                other_identifier: other.map(|o| o.identifier.clone()).unwrap_or_default(),
                other_type: other.map(|o| o.r#type.clone()).unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(ProfileResponse {
        entity: row.to_api(),
        stats: ProfileStats {
            events: stats.0,
            first_seen: stats.1,
            last_seen: stats.2,
        },
        connections,
    }))
}

#[derive(Debug, Deserialize)]
pub struct AnnotateRequest {
    pub tags: Option<Vec<String>>,
}

pub async fn annotate(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
    Json(req): Json<AnnotateRequest>,
) -> Result<Json<Entity>, Response> {
    authed.require(&[Role::Admin, Role::Investigator])?;
    let tags_json = serde_json::to_string(&req.tags.as_ref().unwrap_or(&Vec::new()))
        .unwrap_or_else(|_| "[]".into());
    let res = sqlx::query("UPDATE entities SET tags = ?2 WHERE id = ?1")
        .bind(id.to_string())
        .bind(tags_json)
        .execute(&state.pool)
        .await
        .map_err(internal)?;
    if res.rows_affected() == 0 {
        return Err(ApiError::new("not_found", "entity not found").into_response(StatusCode::NOT_FOUND));
    }

    let row: EntityRow = sqlx::query_as("SELECT * FROM entities WHERE id = ?1")
        .bind(id.to_string())
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;

    crate::db::audit(
        &state.pool,
        &authed.id,
        Some(&row.case_id),
        "entity.annotated",
        serde_json::json!({ "entity_id": id.to_string(), "tags": req.tags }),
    )
    .await;

    Ok(Json(row.to_api()))
}

#[derive(Debug, Deserialize)]
pub struct ResolveBody {}

pub async fn resolve_endpoint(
    State(state): State<AppState>,
    authed: Authed,
    Path(case_id): Path<Uuid>,
    Json(_body): Json<ResolveBody>,
) -> Result<Json<crate::resolve::ResolveStats>, Response> {
    authed.require(&[Role::Admin, Role::Investigator])?;
    let pool = state.pool.clone();
    let stats = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(crate::resolve::resolve_case(&pool, case_id))
    })
    .await
    .map_err(internal)?
    .map_err(internal)?;

    crate::db::audit(
        &state.pool,
        &authed.id,
        Some(&case_id.to_string()),
        "resolution.run",
        serde_json::json!({
            "entities": stats.entities, "edges": stats.edges,
            "device_links": stats.device_links, "communication_links": stats.communication_links
        }),
    )
    .await;

    Ok(Json(stats))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error").into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
