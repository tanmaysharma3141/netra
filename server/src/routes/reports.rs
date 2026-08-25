use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::models::{Report};
use crate::state::AppState;
use crate::stub_data;

pub async fn generate(State(_state): State<AppState>, Path(case_id): Path<Uuid>) -> Json<Report> {
    Json(Report {
        id: Uuid::new_v4(),
        case_id,
        version: "v0.1-draft".into(),
        ..stub_data::demo_report()
    })
}

pub async fn list(Path(_case_id): Path<Uuid>) -> Json<Vec<Report>> {
    Json(vec![stub_data::demo_report()])
}

pub async fn detail(Path(_id): Path<Uuid>) -> Json<Report> {
    Json(stub_data::demo_report())
}

pub async fn export_pdf(Path(_id): Path<Uuid>) -> Response {
    let pdf_stub: &[u8] = b"%PDF-1.4\n% NETRA stub report PDF\n%%EOF";
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"netra-report.pdf\""),
        ],
        pdf_stub.to_vec(),
    )
        .into_response()
}

pub async fn approve(Path(_id): Path<Uuid>) -> Json<Report> {
    Json(Report {
        approved_by: Some(stub_data::USER_ID),
        ..stub_data::demo_report()
    })
}
