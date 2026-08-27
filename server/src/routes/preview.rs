use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use uuid::Uuid;

use crate::auth::Authed;
use crate::ingest::detect::{detect_domain, detect_operator, sniff_delimiter};
use crate::models::{ApiError, Role};
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct PreviewResult {
    pub headers: Vec<String>,
    pub sample_rows: Vec<Vec<String>>,
    pub domain: String,
    pub domain_score: usize,
    pub estimated_rows: usize,
    pub operator: Option<String>,
}

pub async fn preview(
    State(_state): State<AppState>,
    authed: Authed,
    Path(_case_id): Path<Uuid>,
    mut multipart: axum::extract::multipart::Multipart,
) -> Result<Json<PreviewResult>, Response> {
    authed.require(&[Role::Admin, Role::Investigator])?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::new("bad_request", format!("multipart error: {e}")).into_response(StatusCode::BAD_REQUEST))?
    {
        let Some(file_name) = field.file_name().map(String::from) else {
            continue;
        };

        let ext = std::path::Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Read file bytes
        let data = field.bytes().await.map_err(|e| {
            ApiError::new("bad_request", format!("read error: {e}")).into_response(StatusCode::BAD_REQUEST)
        })?;

        match ext.as_str() {
            "csv" | "tsv" | "txt" => {
                return preview_text(&data, &file_name);
            }
            "xlsx" | "xls" => {
                return preview_excel(&data, &file_name);
            }
            "pdf" => {
                return preview_pdf(&data, &file_name);
            }
            _ => {
                return Err(ApiError::new("bad_request", format!("unsupported type '.{ext}'"))
                    .into_response(StatusCode::BAD_REQUEST));
            }
        }
    }

    Err(ApiError::new("bad_request", "no file parts found").into_response(StatusCode::BAD_REQUEST))
}

fn preview_text(data: &[u8], file_name: &str) -> Result<Json<PreviewResult>, Response> {
    let text = String::from_utf8_lossy(data);
    let delim = sniff_delimiter(&text);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(true)
        .flexible(true)
        .from_reader(data);

    let headers: Vec<String> = match reader.headers() {
        Ok(h) => h.iter().map(String::from).collect(),
        Err(e) => return Err(ApiError::new("bad_request", format!("header parse: {e}")).into_response(StatusCode::BAD_REQUEST)),
    };

    let fp = detect_domain(&headers);
    let operator = detect_operator(file_name);

    let mut sample_rows = Vec::new();
    let mut total = 0usize;
    for rec in reader.records() {
        total += 1;
        if sample_rows.len() < 10 {
            if let Ok(row) = rec {
                sample_rows.push(row.iter().map(String::from).collect());
            }
        }
    }

    Ok(Json(PreviewResult {
        headers,
        sample_rows,
        domain: fp.domain.as_str().to_string(),
        domain_score: fp.score,
        estimated_rows: total,
        operator: operator.map(String::from),
    }))
}

fn preview_excel(data: &[u8], file_name: &str) -> Result<Json<PreviewResult>, Response> {
    // Write to temp file for calamine
    let tmp = std::env::temp_dir().join(format!("netra_preview_{}.tmp", Uuid::new_v4()));
    std::fs::write(&tmp, data).map_err(|e| {
        ApiError::new("internal", format!("write temp: {e}")).into_response(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let result = crate::ingest::excel_parser::parse_excel(&tmp);
    let _ = std::fs::remove_file(&tmp);

    let (headers, all_rows) = result.map_err(|e| {
        ApiError::new("bad_request", e).into_response(StatusCode::BAD_REQUEST)
    })?;

    let fp = detect_domain(&headers);
    let operator = detect_operator(file_name);
    let total = all_rows.len();
    let sample_rows: Vec<Vec<String>> = all_rows.into_iter().take(10).collect();

    Ok(Json(PreviewResult {
        headers,
        sample_rows,
        domain: fp.domain.as_str().to_string(),
        domain_score: fp.score,
        estimated_rows: total,
        operator: operator.map(String::from),
    }))
}

fn preview_pdf(data: &[u8], file_name: &str) -> Result<Json<PreviewResult>, Response> {
    let tmp = std::env::temp_dir().join(format!("netra_preview_{}.pdf", Uuid::new_v4()));
    std::fs::write(&tmp, data).map_err(|e| {
        ApiError::new("internal", format!("write temp: {e}")).into_response(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let result = crate::ingest::pdf_parser::parse_pdf(&tmp);
    let _ = std::fs::remove_file(&tmp);

    let (headers, all_rows) = result.map_err(|e| {
        ApiError::new("bad_request", e).into_response(StatusCode::BAD_REQUEST)
    })?;

    let fp = detect_domain(&headers);
    let operator = detect_operator(file_name);
    let total = all_rows.len();
    let sample_rows: Vec<Vec<String>> = all_rows.into_iter().take(10).collect();

    Ok(Json(PreviewResult {
        headers,
        sample_rows,
        domain: fp.domain.as_str().to_string(),
        domain_score: fp.score,
        estimated_rows: total,
        operator: operator.map(String::from),
    }))
}
