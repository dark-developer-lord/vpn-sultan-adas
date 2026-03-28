use sqlx::PgPool;
use uuid::Uuid;
use vpn_shared::types::User;
use vpn_shared::AppError;
use chrono::Utc;

pub struct UserRepository;

#[derive(sqlx::FromRow)]
pub struct UserWithPassword {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl UserRepository {
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError> {
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, created_at, updated_at
            "#
        )
        .bind(&user_id)
        .bind(email)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<User, AppError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, created_at, updated_at FROM users WHERE email = $1 AND deleted_at IS NULL
            "#
        )
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::UserNotFound)
    }

    pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, created_at, updated_at FROM users WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::UserNotFound)
    }

    pub async fn find_by_email_with_password(
        pool: &PgPool,
        email: &str,
    ) -> Result<UserWithPassword, AppError> {
        sqlx::query_as::<_, UserWithPassword>(
            r#"
            SELECT id, email, password_hash, created_at, updated_at 
            FROM users 
            WHERE email = $1 AND deleted_at IS NULL
            "#
        )
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::UserNotFound)
    }
}
