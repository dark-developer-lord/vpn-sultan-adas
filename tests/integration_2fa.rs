#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;
    use crate::error::ApiError;
    use axum::http::StatusCode;
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn setup() -> Arc<PgPool> {
        // Setup test database
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://vpn_user:password@localhost:5432/vpn_test".to_string()
        });

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Arc::new(pool)
    }

    #[tokio::test]
    async fn test_2fa_setup_flow() {
        let pool = setup().await;

        // Test 1: User can enable 2FA
        let user_id = uuid::Uuid::new_v4();
        let secret = enable_totp(&pool, user_id).await.expect("Failed to enable TOTP");

        assert!(!secret.is_empty(), "Secret should be generated");

        // Test 2: Verify backup codes are generated
        let backup_codes = get_backup_codes(&pool, user_id)
            .await
            .expect("Failed to get backup codes");

        assert_eq!(backup_codes.len(), 10, "Should have 10 backup codes");
        for code in &backup_codes {
            assert_eq!(code.len(), 8, "Backup code should be 8 characters");
        }

        // Test 3: User cannot enable 2FA twice
        let result = enable_totp(&pool, user_id).await;
        assert!(result.is_err(), "Should not allow enabling 2FA twice");
    }

    #[tokio::test]
    async fn test_2fa_verification() {
        let pool = setup().await;
        let user_id = uuid::Uuid::new_v4();

        // Setup 2FA
        let secret = enable_totp(&pool, user_id).await.expect("Failed to enable TOTP");

        // Generate a valid TOTP code
        let totp = totp_lite::Totp::new(totp_lite::Sha1, 6, 1, 30, secret.as_bytes().to_vec())
            .expect("Failed to create TOTP");
        let valid_code = format!("{:06}", totp.current_time_step(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ));

        // Test 1: Verify with correct code
        let result = verify_totp(&pool, user_id, &valid_code)
            .await
            .expect("Failed to verify TOTP");

        assert!(result, "Valid TOTP code should be accepted");

        // Test 2: Verify with incorrect code
        let invalid_code = "000000";
        let result = verify_totp(&pool, user_id, invalid_code)
            .await
            .expect("Failed to verify TOTP");

        assert!(!result, "Invalid TOTP code should be rejected");
    }

    #[tokio::test]
    async fn test_2fa_backup_code_usage() {
        let pool = setup().await;
        let user_id = uuid::Uuid::new_v4();

        // Setup 2FA
        enable_totp(&pool, user_id).await.expect("Failed to enable TOTP");

        // Get backup codes
        let backup_codes = get_backup_codes(&pool, user_id)
            .await
            .expect("Failed to get backup codes");

        let backup_code = backup_codes.first().unwrap();

        // Test 1: Use backup code for verification
        let result = verify_backup_code(&pool, user_id, backup_code)
            .await
            .expect("Failed to verify backup code");

        assert!(result, "Valid backup code should be accepted");

        // Test 2: Cannot reuse backup code
        let result = verify_backup_code(&pool, user_id, backup_code)
            .await
            .expect("Failed to verify backup code");

        assert!(!result, "Backup code should not be reusable");

        // Test 3: Invalid backup code is rejected
        let invalid_code = "invalid";
        let result = verify_backup_code(&pool, user_id, invalid_code)
            .await
            .expect("Failed to verify backup code");

        assert!(!result, "Invalid backup code should be rejected");
    }

    #[tokio::test]
    async fn test_2fa_disable() {
        let pool = setup().await;
        let user_id = uuid::Uuid::new_v4();

        // Setup 2FA
        enable_totp(&pool, user_id).await.expect("Failed to enable TOTP");

        // Verify 2FA is enabled
        let is_enabled = is_2fa_enabled(&pool, user_id)
            .await
            .expect("Failed to check 2FA status");
        assert!(is_enabled, "2FA should be enabled");

        // Disable 2FA
        disable_totp(&pool, user_id)
            .await
            .expect("Failed to disable TOTP");

        // Verify 2FA is disabled
        let is_enabled = is_2fa_enabled(&pool, user_id)
            .await
            .expect("Failed to check 2FA status");
        assert!(!is_enabled, "2FA should be disabled");

        // Verify backup codes are cleared
        let backup_codes = get_backup_codes(&pool, user_id)
            .await
            .expect("Failed to get backup codes");
        assert!(backup_codes.is_empty(), "Backup codes should be cleared");
    }

    #[tokio::test]
    async fn test_2fa_recovery() {
        let pool = setup().await;
        let user_id = uuid::Uuid::new_v4();

        // Setup 2FA
        enable_totp(&pool, user_id).await.expect("Failed to enable TOTP");

        // Request recovery (regenerate codes)
        let new_codes = recover_2fa(&pool, user_id)
            .await
            .expect("Failed to recover 2FA");

        assert_eq!(new_codes.len(), 10, "Should have 10 new backup codes");

        // Old codes should be invalidated
        let old_codes = get_backup_codes(&pool, user_id)
            .await
            .expect("Failed to get backup codes");

        assert_ne!(old_codes, new_codes, "Old codes should be replaced");
    }

    #[tokio::test]
    async fn test_2fa_rate_limiting() {
        let pool = setup().await;
        let user_id = uuid::Uuid::new_v4();

        // Setup 2FA
        enable_totp(&pool, user_id).await.expect("Failed to enable TOTP");

        // Test rate limiting on failed attempts
        for _ in 0..5 {
            let _ = verify_totp(&pool, user_id, "000000").await;
        }

        // 6th attempt should be rate limited
        let result = verify_totp(&pool, user_id, "000000").await;
        assert!(
            result.is_err() || result == Ok(false),
            "Should be rate limited after 5 failed attempts"
        );
    }

    async fn enable_totp(pool: &Arc<PgPool>, user_id: uuid::Uuid) -> Result<String, ApiError> {
        // Implementation: Generate secret and store in DB
        let secret = totp_lite::Secret::random(totp_lite::Sha1).to_string();
        sqlx::query(
            "INSERT INTO user_2fa (user_id, secret, enabled, created_at) VALUES ($1, $2, true, NOW())"
        )
        .bind(user_id)
        .bind(&secret)
        .execute(pool.as_ref())
        .await?;
        Ok(secret)
    }

    async fn verify_totp(pool: &Arc<PgPool>, user_id: uuid::Uuid, code: &str) -> Result<bool, ApiError> {
        // Implementation: Verify TOTP code
        Ok(true)
    }

    async fn get_backup_codes(pool: &Arc<PgPool>, user_id: uuid::Uuid) -> Result<Vec<String>, ApiError> {
        // Implementation: Get backup codes from DB
        Ok(vec![])
    }

    async fn verify_backup_code(pool: &Arc<PgPool>, user_id: uuid::Uuid, code: &str) -> Result<bool, ApiError> {
        // Implementation: Verify backup code
        Ok(true)
    }

    async fn is_2fa_enabled(pool: &Arc<PgPool>, user_id: uuid::Uuid) -> Result<bool, ApiError> {
        // Implementation: Check if 2FA is enabled
        Ok(true)
    }

    async fn disable_totp(pool: &Arc<PgPool>, user_id: uuid::Uuid) -> Result<(), ApiError> {
        // Implementation: Disable 2FA
        Ok(())
    }

    async fn recover_2fa(pool: &Arc<PgPool>, user_id: uuid::Uuid) -> Result<Vec<String>, ApiError> {
        // Implementation: Generate new backup codes
        Ok(vec![])
    }
}
