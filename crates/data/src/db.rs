use sqlx::{postgres::PgPoolOptions, PgPool};
use vpn_shared::AppError;
use tracing::info;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Create new database connection
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        info!("Connecting to database: {}", database_url);
        
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .connect(database_url)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to connect: {}", e))
            })?;
        
        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to test connection: {}", e))
            })?;
        
        info!("Database connected and migrations applied");
        
        Ok(Self { pool })
    }
}
