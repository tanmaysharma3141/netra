
#![allow(dead_code)]

use chrono::Duration;
use uuid::{uuid, Uuid};

use crate::models::*;

pub const CASE_ID: Uuid = uuid!("11111111-1111-1111-1111-111111111111");

pub fn demo_events() -> Vec<Event> {
    let now = chrono::Utc::now();
    vec![
        Event {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            timestamp: now - Duration::hours(5),
            source_type: SourceType::Cdr,
            entity_id: "+919812345678".into(),
            entity_type: EntityType::Phone,
            event_type: EventType::Call,
            value: Some(184.0),
            location: Some(LatLng { lat: 30.7333, lng: 76.7794 }),
            raw: serde_json::json!({
                "operator": "JIO",
                "call_type": "OUT",
                "b_party": "+919876543210",
                "duration_sec": 184,
                "cell_id": "JIO-PB-40221"
            }),
            notes: vec![],
        },
        Event {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            timestamp: now - Duration::hours(4),
            source_type: SourceType::Bank,
            entity_id: "XXXX4412".into(),
            entity_type: EntityType::BankAcc,
            event_type: EventType::Txn,
            value: Some(49_999.0),
            location: None,
            raw: serde_json::json!({ "mode": "IMPS", "counterparty": "XXXX8811", "ref": "IMPS-99213" }),
            notes: vec![],
        },
        Event {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            timestamp: now - Duration::hours(2),
            source_type: SourceType::Social,
            entity_id: "@ghost_handle".into(),
            entity_type: EntityType::Handle,
            event_type: EventType::Post,
            value: None,
            location: None,
            raw: serde_json::json!({ "platform": "twitter", "content_hash": "a91f..." }),
            notes: vec![],
        },
    ]
}

pub fn demo_alerts() -> Vec<Alert> {
    vec![
        Alert {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            pattern: "imei_reuse".into(),
            severity: Severity::Critical,
            score: 94,
            status: AlertStatus::Open,
            entity_ids: vec![],
            evidence_event_ids: vec![],
            summary: "Stub: IMEI reused across multiple subscriber lines".into(),
            created_at: (chrono::Utc::now() - Duration::hours(3)).to_rfc3339(),
        },
        Alert {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            pattern: "hawala_signature".into(),
            severity: Severity::High,
            score: 87,
            status: AlertStatus::Reviewing,
            entity_ids: vec![],
            evidence_event_ids: vec![],
            summary: "Stub: structured small deposits aggregating quickly".into(),
            created_at: (chrono::Utc::now() - Duration::hours(8)).to_rfc3339(),
        },
        Alert {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            pattern: "coordinated_silence".into(),
            severity: Severity::Medium,
            score: 61,
            status: AlertStatus::Open,
            entity_ids: vec![],
            evidence_event_ids: vec![],
            summary: "Stub: linked phones went quiet simultaneously".into(),
            created_at: (chrono::Utc::now() - Duration::hours(20)).to_rfc3339(),
        },
    ]
}
