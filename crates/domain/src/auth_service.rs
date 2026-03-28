use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use uuid::Uuid;
use vpn_shared::types::JwtClaims;
use vpn_shared::AppError;

pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    /// Hash password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(rand::thread_rng());
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AppError::InternalServerError)
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AppError::InternalServerError)?;
        
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Generate JWT token
    pub fn generate_jwt(
        &self,
        user_id: Uuid,
        email: &str,
        role: &str,
    ) -> Result<String, AppError> {
        let now = Utc::now().timestamp();
        let expiry = now + 900;  // 15 minutes

        let claims = JwtClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            iat: now,
            exp: expiry,
        };

        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());
        encode(&Header::default(), &claims, &encoding_key)
            .map_err(|_| AppError::InternalServerError)
    }

    /// Validate JWT token
    pub fn validate_jwt(&self, token: &str) -> Result<JwtClaims, AppError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        decode::<JwtClaims>(token, &decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AppError::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let service = AuthService::new("test-secret".to_string());
        let password = "my-secure-password";
        
        let hash = service.hash_password(password).unwrap();
        let is_valid = service.verify_password(password, &hash).unwrap();
        assert!(is_valid);
        
        let is_invalid = service.verify_password("wrong-password", &hash).unwrap();
        assert!(!is_invalid);
    }
}
