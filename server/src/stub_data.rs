#![allow(dead_code)]

use std::collections::HashMap;

use chrono::Duration;
use uuid::{uuid, Uuid};

use crate::models::*;

pub const CASE_ID: Uuid = uuid!("11111111-1111-1111-1111-111111111111");
pub const USER_ID: Uuid = uuid!("22222222-2222-2222-2222-222222222222");

pub fn admin_user() -> User {
    User {
        id: USER_ID,
        username: "chirag".into(),
        role: Role::Admin,
        active: true,
    }
}

pub fn demo_case() -> Case {
    let mut events_by_source = HashMap::new();
    events_by_source.insert(SourceType::Cdr, 128_400);
    events_by_source.insert(SourceType::Ipdr, 41_220);
    events_by_source.insert(SourceType::Bank, 3_910);
    events_by_source.insert(SourceType::Social, 12_050);

    let mut alerts_by_severity = HashMap::new();
    alerts_by_severity.insert(Severity::Low, 4);
    alerts_by_severity.insert(Severity::Medium, 6);
    alerts_by_severity.insert(Severity::High, 3);
    alerts_by_severity.insert(Severity::Critical, 1);

    Case {
        id: CASE_ID,
        title: "OP-2026-041: Cross-border hawala ring".into(),
        status: CaseStatus::Active,
        classification: "RESTRICTED".into(),
        created_by: USER_ID,
        created_at: chrono::Utc::now() - Duration::days(14),
        assignees: vec![USER_ID],
        tags: vec!["hawala".into(), "financial-fraud".into()],
        stats: CaseStats {
            events_by_source,
            alerts_by_severity,
            entity_count: 47,
        },
    }
}

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

pub fn demo_entities() -> Vec<Entity> {
    vec![
        Entity {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            entity_type: EntityType::Phone,
            identifier: "+919812345678".into(),
            display_name: Some("Suspect A".into()),
            link_tier: None,
            tags: vec!["primary".into()],
        },
        Entity {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            entity_type: EntityType::Imei,
            identifier: "354809104512345".into(),
            display_name: None,
            link_tier: Some(LinkTier::High),
            tags: vec!["shared-device".into()],
        },
        Entity {
            id: Uuid::new_v4(),
            case_id: CASE_ID,
            entity_type: EntityType::BankAcc,
            identifier: "XXXX4412".into(),
            display_name: Some("R. Kumar".into()),
            link_tier: Some(LinkTier::Medium),
            tags: vec![],
        },
    ]
}

pub fn demo_graph() -> Graph {
    Graph {
        nodes: vec![
            GraphNode { id: "+919812345678".into(), node_type: EntityType::Phone, label: "Suspect A".into() },
            GraphNode { id: "354809104512345".into(), node_type: EntityType::Imei, label: "IMEI ...345".into() },
            GraphNode { id: "+919899001122".into(), node_type: EntityType::Phone, label: "Suspect B".into() },
            GraphNode { id: "XXXX4412".into(), node_type: EntityType::BankAcc, label: "Acc XXXX4412".into() },
            GraphNode { id: "@ghost_handle".into(), node_type: EntityType::Handle, label: "@ghost_handle".into() },
        ],
        edges: vec![
            GraphEdge { source: "+919812345678".into(), target: "354809104512345".into(), link_type: "used_device".into(), tier: LinkTier::High, confidence: 1.0, evidence_count: 214 },
            GraphEdge { source: "+919899001122".into(), target: "354809104512345".into(), link_type: "used_device".into(), tier: LinkTier::High, confidence: 1.0, evidence_count: 96 },
            GraphEdge { source: "+919812345678".into(), target: "XXXX4412".into(), link_type: "kyc_name_match".into(), tier: LinkTier::Medium, confidence: 0.82, evidence_count: 7 },
            GraphEdge { source: "XXXX4412".into(), target: "@ghost_handle".into(), link_type: "txn_reference".into(), tier: LinkTier::Medium, confidence: 0.71, evidence_count: 3 },
        ],
    }
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

pub fn demo_job() -> IngestJob {
    IngestJob {
        id: Uuid::new_v4(),
        case_id: CASE_ID,
        status: IngestJobStatus::Running,
        records_parsed: 5_240,
        errors: vec![],
    }
}

pub fn demo_trails() -> TrailsResponse {
    let base = chrono::Utc::now() - Duration::hours(6);
    let points = (0..6)
        .map(|i| GeoPoint {
            entity_id: "+919812345678".into(),
            lat: 30.7333 + f64::from(i) * 0.01,
            lng: 76.7794 + f64::from(i) * 0.008,
            tower_id: Some(format!("JIO-PB-{i}")),
            timestamp: base + Duration::minutes(i64::from(i) * 55),
        })
        .collect();
    TrailsResponse {
        trails: vec![Trail { entity_id: "+919812345678".into(), points }],
    }
}

pub fn demo_report() -> Report {
    Report {
        id: Uuid::new_v4(),
        case_id: CASE_ID,
        version: 1,
        generated_by: GeneratedBy::Template,
        approved_by: None,
        created_at: chrono::Utc::now(),
        summary_md: "# Executive Summary\n\nStub report pending LLM integration.\n\n- 47 entities resolved\n- 14 alerts raised (1 critical)\n- IMEI reuse links Suspects A and B to one device.".into(),
    }
}

pub fn demo_audit() -> Vec<AuditEntry> {
    vec![
        AuditEntry {
            id: Uuid::new_v4(),
            case_id: Some(CASE_ID),
            user_id: USER_ID,
            action: "case.created".into(),
            detail: serde_json::json!({ "title": "OP-2026-041" }),
            at: chrono::Utc::now() - Duration::days(14),
        },
        AuditEntry {
            id: Uuid::new_v4(),
            case_id: Some(CASE_ID),
            user_id: USER_ID,
            action: "ingest.completed".into(),
            detail: serde_json::json!({ "file": "jio_cdr_march.csv", "sha256": "ab12...", "records": 128_400 }),
            at: chrono::Utc::now() - Duration::days(12),
        },
    ]
}
