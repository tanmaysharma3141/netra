use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Id = Uuid;

macro_rules! str_enum {
    ($t:ty { $($v:ident => $s:literal),+ $(,)? }) => {
        impl std::str::FromStr for $t {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let lc = s.trim().to_lowercase();
                $(if lc == $s.to_lowercase() { return Ok(<$t>::$v); })+
                Err(format!("unknown {} variant: {}", stringify!($t), s))
            }
        }
    };
}

str_enum!(Role {
    Admin => "admin",
    Supervisor => "supervisor",
    Investigator => "investigator",
    Analyst => "analyst",
});

str_enum!(SourceType {
    Cdr => "CDR",
    Ipdr => "IPDR",
    Bank => "BANK",
    Social => "SOCIAL",
});

impl SourceType {
    pub fn db_str(&self) -> &'static str {
        match self {
            Self::Cdr => "cdr",
            Self::Ipdr => "ipdr",
            Self::Bank => "bank",
            Self::Social => "social",
        }
    }
}

str_enum!(EntityType {
    Phone => "PHONE",
    Imei => "IMEI",
    BankAcc => "BANK_ACC",
    Ip => "IP",
    Handle => "HANDLE",
});

impl EntityType {
    pub fn db_str(&self) -> &'static str {
        match self {
            Self::Phone => "phone",
            Self::Imei => "imei",
            Self::BankAcc => "bank_acc",
            Self::Ip => "ip",
            Self::Handle => "handle",
        }
    }
}

str_enum!(EventType {
    Call => "CALL",
    Sms => "SMS",
    Data => "DATA",
    Txn => "TXN",
    Post => "POST",
    Login => "LOGIN",
    Other => "OTHER",
});

impl EventType {
    pub fn db_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Sms => "sms",
            Self::Data => "data",
            Self::Txn => "txn",
            Self::Post => "post",
            Self::Login => "login",
            Self::Other => "other",
        }
    }
}

str_enum!(LinkTier {
    High => "high",
    Medium => "medium",
    Low => "low",
});

str_enum!(Severity {
    Low => "low",
    Medium => "medium",
    High => "high",
    Critical => "critical",
});

str_enum!(AlertStatus {
    Open => "open",
    Reviewing => "reviewing",
    Confirmed => "confirmed",
    FalsePositive => "false_positive",
});

str_enum!(CaseStatus {
    Active => "active",
    Archived => "archived",
    Closed => "closed",
});

str_enum!(IngestJobStatus {
    Queued => "queued",
    Running => "running",
    Done => "done",
    Failed => "failed",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Supervisor,
    Investigator,
    Analyst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Id,
    pub username: String,
    pub role: Role,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SourceType {
    Cdr,
    Ipdr,
    Bank,
    Social,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EntityType {
    Phone,
    Imei,
    BankAcc,
    Ip,
    Handle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EventType {
    Call,
    Sms,
    Data,
    Txn,
    Post,
    Login,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkTier {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Open,
    Reviewing,
    Confirmed,
    FalsePositive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    Active,
    Archived,
    Closed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaseStats {
    #[serde(default)]
    pub events_by_source: std::collections::HashMap<SourceType, u64>,
    #[serde(default)]
    pub alerts_by_severity: std::collections::HashMap<Severity, u64>,
    #[serde(default)]
    pub entity_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: Id,
    pub title: String,
    pub status: CaseStatus,
    pub classification: String,
    pub created_by: Id,
    pub created_at: DateTime<Utc>,
    pub assignees: Vec<Id>,
    pub tags: Vec<String>,
    pub stats: CaseStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Id,
    pub case_id: Id,
    pub timestamp: DateTime<Utc>,
    pub source_type: SourceType,
    pub entity_id: String,
    pub entity_type: EntityType,
    pub event_type: EventType,
    pub value: Option<f64>,
    pub location: Option<LatLng>,
    pub raw: serde_json::Value,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Id,
    pub case_id: Id,
    #[serde(rename = "type")]
    pub entity_type: EntityType,
    pub identifier: String,
    pub display_name: Option<String>,
    pub link_tier: Option<LinkTier>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: EntityType,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub link_type: String,
    pub tier: LinkTier,
    pub confidence: f64,
    pub evidence_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Id,
    pub case_id: Id,
    pub pattern: String,
    pub severity: Severity,
    pub score: u8,
    pub status: AlertStatus,
    pub entity_ids: Vec<Id>,
    pub evidence_event_ids: Vec<Id>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestJobStatus {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestJob {
    pub id: Id,
    pub case_id: Id,
    pub status: IngestJobStatus,
    pub records_parsed: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub entity_id: String,
    pub lat: f64,
    pub lng: f64,
    pub tower_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrailsResponse {
    pub trails: Vec<Trail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trail {
    pub entity_id: String,
    pub points: Vec<GeoPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Id,
    pub case_id: Id,
    pub version: u32,
    #[serde(rename = "generated_by")]
    pub generated_by: GeneratedBy,
    pub approved_by: Option<Id>,
    pub created_at: DateTime<Utc>,
    pub summary_md: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedBy {
    Llm,
    Template,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Id,
    pub case_id: Option<Id>,
    pub user_id: Id,
    pub action: String,
    pub detail: serde_json::Value,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub struct CreateCaseRequest {
    pub title: String,
    #[serde(default)]
    pub classification: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertStatusRequest {
    pub status: AlertStatus,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub question: String,
}

#[derive(Debug, Serialize)]
pub struct ChatFrame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<Id>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

impl ChatFrame {
    pub fn delta(chunk: String) -> Self {
        Self { delta: Some(chunk), sources: None, done: None }
    }

    pub fn sources(ids: Vec<Id>) -> Self {
        Self { delta: None, sources: Some(ids), done: None }
    }

    pub fn done() -> Self {
        Self { delta: None, sources: None, done: Some(true) }
    }
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub version: String,
    pub active: bool,
    pub trained_at: Option<DateTime<Utc>>,
    pub base_model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingQueueInfo {
    pub queued_events: u64,
    pub minimum_batch: u64,
    pub last_run: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct WebhookConfig {
    pub discord_url: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.into(),
            },
        }
    }

    pub fn into_response(self, status: axum::http::StatusCode) -> axum::response::Response {
        use axum::response::IntoResponse;
        (status, axum::Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event")]
pub enum WsEvent {
    #[serde(rename = "alert.created")]
    AlertCreated { payload: Alert },
    #[serde(rename = "ingest.progress")]
    IngestProgress { payload: IngestProgress },
    #[serde(rename = "training.progress")]
    TrainingProgress { payload: TrainingProgress },
    #[serde(rename = "model.updated")]
    ModelUpdated { payload: ModelUpdated },
}

#[derive(Debug, Clone, Serialize)]
pub struct WsEnvelope {
    pub topic: String,
    #[serde(flatten)]
    pub event: WsEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestProgress {
    pub job_id: Id,
    pub parsed: u64,
    pub total_est: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingProgress {
    pub epoch: u32,
    pub loss: f64,
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdated {
    pub version: String,
}
