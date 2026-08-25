use uuid::Uuid;

use axum::extract::{Path, Query};
use axum::Json;
use serde::Deserialize;

use crate::models::{Entity, Graph};
use crate::stub_data;

#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    pub entity_id: Option<String>,
    #[serde(default = "default_hops")]
    pub hops: u32,
}

fn default_hops() -> u32 {
    2
}

pub async fn list(Path(_case_id): Path<Uuid>) -> Json<Vec<Entity>> {
    Json(stub_data::demo_entities())
}

pub async fn graph(
    Path(_case_id): Path<Uuid>,
    Query(q): Query<GraphQuery>,
) -> Json<Graph> {
    let mut g = stub_data::demo_graph();
    if let Some(root) = q.entity_id {
        let connected: Vec<String> = g
            .edges
            .iter()
            .filter(|e| e.source == root || e.target == root)
            .flat_map(|e| [e.source.clone(), e.target.clone()])
            .collect();
        g.nodes.retain(|n| n.id == root || connected.contains(&n.id));
        g.edges.retain(|e| e.source == root || e.target == root);
    }
    Json(g)
}

pub async fn profile(Path(id): Path<Uuid>) -> Json<Option<Entity>> {
    Json(stub_data::demo_entities().into_iter().find(|e| e.id == id))
}

#[derive(Debug, Deserialize)]
pub struct AnnotateRequest {
    pub tags: Option<Vec<String>>,
}

pub async fn annotate(Path(_id): Path<Uuid>, Json(req): Json<AnnotateRequest>) -> axum::http::StatusCode {
    tracing::info!(tags = ?req.tags, "entity annotate (stub)");
    axum::http::StatusCode::NO_CONTENT
}
