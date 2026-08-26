use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::{Case, CaseStats, Role, SourceType, Severity, User};

pub const DEMO_CASE_ID: &str = "11111111-1111-1111-1111-111111111111";

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub active: i64,
    pub failed_attempts: i64,
    pub locked_until: Option<String>,
    pub created_at: String,
}

impl UserRow {
    pub fn to_api(&self) -> User {
        User {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::nil()),
            username: self.username.clone(),
            role: self.role.parse().unwrap_or(Role::Analyst),
            active: self.active != 0,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct CaseRow {
    pub id: String,
    pub title: String,
    pub status: String,
    pub classification: String,
    pub created_by: String,
    pub created_at: String,
    pub tags: String,
    pub assignees: String,
}

impl CaseRow {
    pub fn to_api(&self) -> Case {
        let stats = CaseStats::default();
        Case {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::nil()),
            title: self.title.clone(),
            status: self.status.parse().unwrap_or(crate::models::CaseStatus::Active),
            classification: self.classification.clone(),
            created_by: Uuid::parse_str(&self.created_by).unwrap_or_else(|_| Uuid::nil()),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            assignees: serde_json::from_str(&self.assignees).unwrap_or_default(),
            tags: serde_json::from_str(&self.tags).unwrap_or_default(),
            stats,
        }
    }

    pub fn assignee_ids(&self) -> Vec<String> {
        serde_json::from_str(&self.assignees).unwrap_or_default()
    }
}

#[derive(Debug, FromRow)]
pub struct AuditRow {
    pub id: String,
    pub user_id: String,
    pub case_id: Option<String>,
    pub action: String,
    pub detail: String,
    pub at: String,
}

pub async fn init(database_url: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let opts: SqliteConnectOptions = database_url.parse()?;
    let opts = opts
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(30))
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    seed_admin(&pool).await?;
    seed_demo_case(&pool).await?;
    Ok(pool)
}

async fn seed_admin(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Ok(());
    }
    let password =
        std::env::var("NETRA_ADMIN_PASSWORD").unwrap_or_else(|_| "netra-admin".to_string());
    if std::env::var("NETRA_ADMIN_PASSWORD").is_err() {
        tracing::warn!("NETRA_ADMIN_PASSWORD not set; seeding admin with default password 'netra-admin' — CHANGE IT");
    }
    let hash = bcrypt::hash(&password, 12)?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, password_hash, role, active, failed_attempts, created_at) VALUES (?1, ?2, ?3, 'admin', 1, 0, ?4)")
        .bind(id.to_string())
        .bind("admin")
        .bind(hash)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
    tracing::info!("seeded admin user");
    Ok(())
}

async fn seed_demo_case(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cases")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Ok(());
    }
    let admin_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = 'admin'")
        .fetch_one(pool)
        .await?;
    sqlx::query("INSERT INTO cases (id, title, status, classification, created_by, created_at, tags, assignees) VALUES (?1, ?2, 'active', 'RESTRICTED', ?3, ?4, ?5, ?6)")
        .bind(DEMO_CASE_ID)
        .bind("OP-2026-041: Cross-border hawala ring")
        .bind(&admin_id)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(r#"["hawala","financial-fraud"]"#)
        .bind(serde_json::json!([admin_id]).to_string())
        .execute(pool)
        .await?;
    tracing::info!(case_id = DEMO_CASE_ID, "seeded demo case");
    Ok(())
}

pub async fn audit(
    pool: &SqlitePool,
    user_id: &str,
    case_id: Option<&str>,
    action: &str,
    detail: serde_json::Value,
) {
    let _ = sqlx::query(
        "INSERT INTO audit_log (id, user_id, case_id, action, detail, at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(case_id)
    .bind(action)
    .bind(detail.to_string())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await;
}

pub fn severity_counts(rows: Vec<(String, i64)>) -> std::collections::HashMap<Severity, u64> {
    let mut map = std::collections::HashMap::new();
    for (sev, n) in rows {
        if let Ok(s) = sev.parse::<Severity>() {
            *map.entry(s).or_insert(0) += n as u64;
        }
    }
    map
}

pub fn source_counts(rows: Vec<(String, i64)>) -> std::collections::HashMap<SourceType, u64> {
    let mut map = std::collections::HashMap::new();
    for (src, n) in rows {
        if let Ok(s) = src.parse::<SourceType>() {
            *map.entry(s).or_insert(0) += n as u64;
        }
    }
    map
}
