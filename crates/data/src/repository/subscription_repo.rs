use sqlx::PgPool;
use uuid::Uuid;
use vpn_shared::types::Subscription;
use vpn_shared::AppError;
use chrono::Utc;

#[derive(Debug, sqlx::FromRow)]
struct SubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan: String,
    pub status: String,
    pub data_limit_gb: Option<i32>,
    pub max_peers: i32,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<SubscriptionRow> for Subscription {
    fn from(row: SubscriptionRow) -> Self {
        let plan = match row.plan.as_str() {
            "pro" => vpn_shared::types::SubscriptionPlan::Pro,
            "enterprise" => vpn_shared::types::SubscriptionPlan::Enterprise,
            _ => vpn_shared::types::SubscriptionPlan::Free,
        };

        Self {
            id: row.id,
            user_id: row.user_id,
            plan,
            status: row.status,
            data_limit_gb: row.data_limit_gb,
            max_peers: row.max_peers,
            created_at: row.created_at,
        }
    }
}

pub struct SubscriptionRepository;

impl SubscriptionRepository {
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Subscription, AppError> {
        let row = sqlx::query_as::<_, SubscriptionRow>(
            "SELECT id, user_id, plan, status, data_limit_gb, max_peers, created_at FROM subscriptions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or(AppError::NotFound)?;

        Ok(row.into())
    }

    pub async fn create_default_subscription(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Subscription, AppError> {
        let sub_id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query_as::<_, SubscriptionRow>(
            r#"
            INSERT INTO subscriptions (id, user_id, plan, status, max_peers, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, plan, status, data_limit_gb, max_peers, created_at
            "#,
        )
        .bind(sub_id)
        .bind(user_id)
        .bind("free")
        .bind("active")
        .bind(2) // Free plan has 2 peers max
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.into())
    }
}
