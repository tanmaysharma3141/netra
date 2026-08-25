use std::convert::Infallible;
use std::time::Duration;

use futures_util::stream;
use futures_util::StreamExt;
use uuid::Uuid;

use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;

use crate::models::{ChatFrame, ChatRequest};
use crate::state::AppState;
use crate::stub_data;

pub async fn ask(
    State(state): State<AppState>,
    Path(_case_id): Path<Uuid>,
    Json(req): Json<ChatRequest>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    tracing::info!(question = %req.question, "copilot query (stub)");
    let answer: Vec<String> = format!(
        "Stub response. The correlation engine is not yet wired; based on the demo case the most \
         suspicious activity clusters around IMEI 354809104512345 shared by two subscribers. \
         Question was: {}",
        req.question
    )
    .chars()
    .collect::<Vec<char>>()
    .chunks(24)
    .map(|c| c.iter().collect::<String>())
    .collect();

    let event_ids: Vec<Uuid> = stub_data::demo_events()
        .into_iter()
        .map(|e| e.id)
        .collect();
    let state = state.clone();

    let s = stream::iter(answer.into_iter().enumerate())
        .then(move |(_, chunk)| {
            let _ = &state;
            async move {
                tokio::time::sleep(Duration::from_millis(60)).await;
                Ok(Event::default().data(
                    serde_json::to_string(&ChatFrame::Delta { delta: chunk }).unwrap(),
                ))
            }
        })
        .chain(stream::once(async move {
            Ok(Event::default().data(
                serde_json::to_string(&ChatFrame::Sources {
                    sources: event_ids,
                })
                .unwrap(),
            ))
        }))
        .chain(stream::once(async {
            Ok(Event::default().data(serde_json::to_string(&ChatFrame::Done).unwrap()))
        }));

    Sse::new(s).keep_alive(KeepAlive::default())
}
