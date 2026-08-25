use std::time::Duration;

use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::models::{IngestJob, IngestJobStatus};
use crate::state::AppState;
use crate::stub_data;

pub async fn upload(
    State(state): State<AppState>,
    Path(case_id): Path<Uuid>,
) -> (StatusCode, Json<IngestJob>) {
    let job = IngestJob {
        id: Uuid::new_v4(),
        case_id,
        status: IngestJobStatus::Running,
        records_parsed: 0,
        errors: vec![],
    };
    let tx = state.tx.clone();
    let progress_job = job.clone();
    tokio::spawn(async move {
        for parsed in [2_500_u64, 5_240] {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = tx.send(crate::models::WsEnvelope {
                topic: format!("case:{case_id}"),
                event: crate::models::WsEvent::IngestProgress {
                    payload: crate::models::IngestProgress {
                        job_id: progress_job.id,
                        parsed,
                        total_est: 5_240,
                    },
                },
            });
        }
    });
    (StatusCode::ACCEPTED, Json(job))
}

pub async fn job(Path(_id): Path<Uuid>) -> Json<IngestJob> {
    let mut j = stub_data::demo_job();
    j.status = IngestJobStatus::Done;
    j.records_parsed = 5_240;
    Json(j)
}
