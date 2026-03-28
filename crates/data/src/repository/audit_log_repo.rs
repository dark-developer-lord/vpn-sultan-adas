use sqlx::PgPool;
use uuid::Uuid;
use vpn_shared::AppError;
use chrono::Utc;

#[derive(Debug, sqlx::FromRow)]
struct AuditLogRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub status: String,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct AuditLogRepository;

impl AuditLogRepository {
    pub async fn log_action(
        pool: &PgPool,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        status: &str,
        details: Option<serde_json::Value>,
        ip_address: Option<&str>,
    ) -> Result<(), AppError> {
        let log_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, status, details, ip_address, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(log_id)
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(status)
        .bind(details)
        .bind(ip_address)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Log authentication attempt
    pub async fn log_auth_attempt(
        pool: &PgPool,
        email: &str,
        success: bool,
        ip_address: Option<&str>,
    ) -> Result<(), AppError> {
        let status = if success { "success" } else { "failed" };
        
        Self::log_action(
            pool,
            None,
            if success { "login" } else { "login_failed" },
            "user",
            None,
            status,
            Some(serde_json::json!({ "email": email })),
            ip_address,
        )
        .await
    }

    /// Log peer creation
    pub async fn log_peer_creation(
        pool: &PgPool,
        user_id: Uuid,
        peer_id: Uuid,
        peer_name: &str,
        node_id: Uuid,
        success: bool,
        ip_address: Option<&str>,
    ) -> Result<(), AppError> {
        let status = if success { "success" } else { "failed" };
        
        Self::log_action(
            pool,
            Some(user_id),
            "create_peer",
            "peer",
            Some(peer_id),
            status,
            Some(serde_json::json!({
                "peer_name": peer_name,
                "node_id": node_id
            })),
            ip_address,
        )
        .await
    }

    /// Log peer deletion
    pub async fn log_peer_deletion(
        pool: &PgPool,
        user_id: Uuid,
        peer_id: Uuid,
        success: bool,
        ip_address: Option<&str>,
    ) -> Result<(), AppError> {
        let status = if success { "success" } else { "failed" };
        
        Self::log_action(
            pool,
            Some(user_id),
            "delete_peer",
            "peer",
            Some(peer_id),
            status,
            None,
            ip_address,
        )
        .await
    }

    /// Log subscription limit exceeded
    pub async fn log_subscription_limit_exceeded(
        pool: &PgPool,
        user_id: Uuid,
        reason: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AppError> {
        Self::log_action(
            pool,
            Some(user_id),
            "limit_exceeded",
            "subscription",
            None,
            "blocked",
            Some(serde_json::json!({ "reason": reason })),
            ip_address,
        )
        .await
    }
}
