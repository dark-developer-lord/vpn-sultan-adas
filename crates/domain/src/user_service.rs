use sqlx::PgPool;
use uuid::Uuid;
use vpn_data::repository::{UserRepository, SubscriptionRepository};
use vpn_shared::types::User;
use vpn_shared::AppError;

pub struct UserService;

impl UserService {
    pub async fn create_user(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError> {
        // Check if user already exists
        if let Ok(_) = UserRepository::find_by_email(pool, email).await {
            return Err(AppError::UserAlreadyExists);
        }

        let user = UserRepository::create(pool, email, password_hash).await?;

        // Create default free subscription for new user
        let _ = SubscriptionRepository::create_default_subscription(pool, user.id)
            .await
            .map_err(|_| AppError::DatabaseError("Failed to create subscription".to_string()))?;

        Ok(user)
    }

    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<User, AppError> {
        UserRepository::find_by_email(pool, email).await
    }

    pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
        UserRepository::find_by_id(pool, user_id).await
    }
}
