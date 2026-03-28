use totp_rs::{TOTP, Algorithm, Secret, SecretFetcher};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};

// ==================== TYPES ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TwoFASetup {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTwoFARequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnableTwoFAResponse {
    pub setup: TwoFASetup,
    pub enrollment_token: String,
}

// ==================== 2FA SERVICE ====================

pub struct TwoFAService;

impl TwoFAService {
    /// Generate a new 2FA secret and QR code
    pub fn generate_setup(user_email: &str, app_name: &str) -> Result<TwoFASetup> {
        // Generate TOTP secret
        let secret = Secret::generate_secret();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,           // 6-digit code
            1,           // 1 issuer
            30,          // 30-second window
            secret.to_bytes()?,
            Some(app_name.to_string()),
            user_email.to_string(),
        )?;

        // Generate QR code
        let qr_code = totp.get_qr_base64()?;

        // Generate backup codes (10 codes for emergency access)
        let backup_codes = Self::generate_backup_codes();

        Ok(TwoFASetup {
            secret: secret.to_string(),
            qr_code,
            backup_codes: backup_codes.clone(),
        })
    }

    /// Verify a TOTP code
    pub fn verify_code(secret: &str, code: &str) -> Result<bool> {
        let secret = Secret::Encoded(secret.to_string());
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes()?,
            None,
            "2FA".to_string(),
        )?;

        Ok(totp.check_current(code)?)
    }

    /// Verify a backup code and mark as used
    pub async fn verify_backup_code(
        user_id: &str,
        code: &str,
        db: &Database,
    ) -> Result<bool> {
        let backup_code = db
            .query_one(
                "SELECT code FROM backup_codes WHERE user_id = $1 AND code = $2 AND used = false",
                &[&user_id, &code],
            )
            .await?;

        if backup_code.is_some() {
            // Mark code as used
            db.execute(
                "UPDATE backup_codes SET used = true, used_at = NOW() WHERE user_id = $1 AND code = $2",
                &[&user_id, &code],
            )
            .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Generate 10 backup codes
    fn generate_backup_codes() -> Vec<String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..10)
            .map(|_| {
                (0..8)
                    .map(|_| {
                        let idx = rng.gen_range(0..36);
                        if idx < 10 {
                            (b'0' + idx as u8) as char
                        } else {
                            (b'a' + (idx - 10) as u8) as char
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

// ==================== HANDLERS ====================

/// Enable 2FA for authenticated user
#[post("/auth/2fa/enable")]
pub async fn enable_2fa(
    auth: AuthLayer,
    db: Database,
) -> Result<Json<EnableTwoFAResponse>> {
    // Check if 2FA already enabled
    let existing = db
        .query_one(
            "SELECT id FROM two_factor WHERE user_id = $1 AND enabled = true",
            &[&auth.user_id],
        )
        .await;

    if existing.is_ok() {
        return Err(AppError::Conflict("2FA already enabled".to_string()));
    }

    // Generate setup (secret + QR code + backup codes)
    let setup = TwoFAService::generate_setup(&auth.user.email, "VPN Service")?;

    // Create enrollment token (temporary, valid for 10 minutes)
    let enrollment_token = generate_token()?;
    let token_hash = hash_token(&enrollment_token);

    db.execute(
        "INSERT INTO 2fa_enrollments (user_id, secret, backup_codes, token_hash, expires_at, created_at)
         VALUES ($1, $2, $3, $4, NOW() + INTERVAL '10 minutes', NOW())",
        &[
            &auth.user_id,
            &setup.secret,
            &serde_json::to_string(&setup.backup_codes)?,
            &token_hash,
        ],
    )
    .await?;

    // Log this action
    log_audit_event(
        &auth.user_id,
        "2FA_SETUP_INITIATED",
        &format!("2FA setup started"),
        &db,
    )
    .await?;

    Ok(Json(EnableTwoFAResponse {
        setup,
        enrollment_token,
    }))
}

/// Verify 2FA code and complete enrollment
#[post("/auth/2fa/verify-setup")]
pub async fn verify_2fa_setup(
    auth: AuthLayer,
    req: Json<VerifyTwoFARequest>,
    db: Database,
) -> Result<Json<serde_json::Value>> {
    // Get active enrollment
    let enrollment = db
        .query_one(
            "SELECT secret, backup_codes FROM 2fa_enrollments 
             WHERE user_id = $1 AND expires_at > NOW()
             ORDER BY created_at DESC LIMIT 1",
            &[&auth.user_id],
        )
        .await?;

    let enrollment = enrollment.ok_or(AppError::BadRequest("No active enrollment".to_string()))?;
    let secret: String = enrollment.get(0);

    // Verify the code
    if !TwoFAService::verify_code(&secret, &req.code)? {
        return Err(AppError::BadRequest("Invalid 2FA code".to_string()));
    }

    // Enable 2FA for user
    let backup_codes: String = enrollment.get(1);
    db.execute(
        "UPDATE users SET two_factor_enabled = true WHERE id = $1",
        &[&auth.user_id],
    )
    .await?;

    db.execute(
        "INSERT INTO two_factor (user_id, secret, backup_codes, enabled, created_at)
         VALUES ($1, $2, $3, true, NOW())",
        &[&auth.user_id, &secret, &backup_codes],
    )
    .await?;

    // Clear enrollment
    db.execute(
        "DELETE FROM 2fa_enrollments WHERE user_id = $1",
        &[&auth.user_id],
    )
    .await?;

    log_audit_event(
        &auth.user_id,
        "2FA_ENABLED",
        "2FA successfully enabled",
        &db,
    )
    .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "2FA enabled successfully"
    })))
}

/// Disable 2FA (requires email verification + current password)
#[post("/auth/2fa/disable")]
pub async fn disable_2fa(
    auth: AuthLayer,
    Json(payload): Json<DisableTwoFARequest>,
    db: Database,
) -> Result<Json<serde_json::Value>> {
    // Verify password
    let user = db
        .query_one("SELECT password_hash FROM users WHERE id = $1", &[&auth.user_id])
        .await?
        .ok_or(AppError::Unauthorized)?;

    let password_hash: String = user.get(0);
    if !argon2_verify_password(&payload.password, &password_hash)? {
        return Err(AppError::Unauthorized);
    }

    // Disable 2FA
    db.execute(
        "UPDATE two_factor SET enabled = false WHERE user_id = $1",
        &[&auth.user_id],
    )
    .await?;

    log_audit_event(
        &auth.user_id,
        "2FA_DISABLED",
        "2FA disabled by user",
        &db,
    )
    .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "2FA disabled"
    })))
}

/// Verify 2FA code during login
#[post("/auth/2fa/verify-login")]
pub async fn verify_2fa_login(
    req: Json<VerifyTwoFALoginRequest>,
    db: Database,
) -> Result<Json<TokenResponse>> {
    // Validate temporary login token
    let session = db
        .query_one(
            "SELECT user_id FROM temp_login_sessions WHERE token_hash = $1 AND expires_at > NOW()",
            &[&validate_token(&req.temp_token)?],
        )
        .await?
        .ok_or(AppError::Unauthorized)?;

    let user_id: String = session.get(0);

    // Get 2FA secret
    let two_fa = db
        .query_one(
            "SELECT secret FROM two_factor WHERE user_id = $1 AND enabled = true",
            &[&user_id],
        )
        .await?
        .ok_or(AppError::Unauthorized)?;

    let secret: String = two_fa.get(0);

    // Verify code (or try backup code)
    let verified = TwoFAService::verify_code(&secret, &req.code)?
        || TwoFAService::verify_backup_code(&user_id, &req.code, &db).await?;

    if !verified {
        return Err(AppError::BadRequest("Invalid 2FA code".to_string()));
    }

    // Generate JWT token
    let token = generate_jwt_token(&user_id)?;

    // Clear temp session
    db.execute(
        "DELETE FROM temp_login_sessions WHERE token_hash = $1",
        &[&validate_token(&req.temp_token)?],
    )
    .await?;

    log_audit_event(
        &user_id,
        "LOGIN_2FA",
        "User logged in with 2FA",
        &db,
    )
    .await?;

    Ok(Json(TokenResponse { access_token: token }))
}

// ==================== DATABASE SCHEMA ====================

pub const SETUP_2FA_TABLES: &str = r#"
-- 2FA configuration
CREATE TABLE IF NOT EXISTS two_factor (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    secret TEXT NOT NULL,
    backup_codes JSONB NOT NULL,
    enabled BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id)
);

-- 2FA enrollment (temporary)
CREATE TABLE IF NOT EXISTS 2fa_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    secret TEXT NOT NULL,
    backup_codes JSONB NOT NULL,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Used backup codes
CREATE TABLE IF NOT EXISTS backup_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    used BOOLEAN DEFAULT false,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Temporary login sessions (used when 2FA not yet verified)
CREATE TABLE IF NOT EXISTS temp_login_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_two_factor_user_id ON two_factor(user_id);
CREATE INDEX IF NOT EXISTS idx_two_factor_enabled ON two_factor(user_id, enabled);
CREATE INDEX IF NOT EXISTS idx_2fa_enrollments_user_id ON 2fa_enrollments(user_id);
CREATE INDEX IF NOT EXISTS idx_temp_login_sessions_user_id ON temp_login_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_backup_codes_user_id ON backup_codes(user_id);
"#;
