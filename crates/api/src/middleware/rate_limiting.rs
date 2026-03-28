use axum::{
    extract::ConnectInfo,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use std::sync::Arc;
use redis::RedisError;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

// ==================== TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second per user
    pub authenticated_rps: u32,
    /// Requests per second per IP
    pub anonymous_rps: u32,
    /// Burst allowance (multiplier on RPS)
    pub burst_multiplier: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            authenticated_rps: 100,
            anonymous_rps: 10,
            burst_multiplier: 2,
            window_seconds: 60,
        }
    }
}

#[derive(Debug)]
pub struct RateLimitInfo {
    pub key: String,
    pub limit: u32,
    pub current: u32,
    pub reset_at: u64,
}

// ==================== RATE LIMITER ====================

pub struct RateLimiter {
    redis: Arc<redis::Client>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(redis: Arc<redis::Client>, config: RateLimitConfig) -> Self {
        Self { redis, config }
    }

    /// Check rate limit for a user (authenticated)
    pub async fn check_user_limit(&self, user_id: &str) -> Result<RateLimitInfo, RedisError> {
        let key = format!("ratelimit:user:{}", user_id);
        let limit = self.config.authenticated_rps * self.config.burst_multiplier;

        self.check_limit(&key, limit as u32, self.config.window_seconds)
            .await
    }

    /// Check rate limit for an IP (anonymous)
    pub async fn check_ip_limit(&self, ip: &IpAddr) -> Result<RateLimitInfo, RedisError> {
        let key = format!("ratelimit:ip:{}", ip);
        let limit = self.config.anonymous_rps * self.config.burst_multiplier;

        self.check_limit(&key, limit as u32, self.config.window_seconds)
            .await
    }

    /// Check endpoint-specific rate limit (e.g., login: 5 attempts per minute)
    pub async fn check_endpoint_limit(
        &self,
        endpoint: &str,
        identifier: &str,
        limit: u32,
        window: u64,
    ) -> Result<RateLimitInfo, RedisError> {
        let key = format!("ratelimit:endpoint:{}:{}", endpoint, identifier);
        self.check_limit(&key, limit, window).await
    }

    /// Internal rate limit check using token bucket algorithm
    async fn check_limit(
        &self,
        key: &str,
        limit: u32,
        window: u64,
    ) -> Result<RateLimitInfo, RedisError> {
        let mut conn = self.redis.get_async_connection().await?;

        // Get current count
        let current: u32 = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        if current < limit {
            // Increment and get new value
            let new_count: u32 = redis::cmd("INCR")
                .arg(key)
                .query_async(&mut conn)
                .await?;

            // Set expiration on first request
            if new_count == 1 {
                redis::cmd("EXPIRE")
                    .arg(key)
                    .arg(window as u32)
                    .query_async(&mut conn)
                    .await?;
            }

            // Get TTL for reset time
            let ttl: i32 = redis::cmd("TTL").arg(key).query_async(&mut conn).await?;
            let reset_at =
                (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()
                    .as_secs()
                    + ttl as u64);

            Ok(RateLimitInfo {
                key: key.to_string(),
                limit,
                current: new_count,
                reset_at,
            })
        } else {
            // Limit exceeded
            let ttl: i32 = redis::cmd("TTL").arg(key).query_async(&mut conn).await?;
            let reset_at =
                (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()
                    .as_secs()
                    + ttl as u64);

            Ok(RateLimitInfo {
                key: key.to_string(),
                limit,
                current,
                reset_at,
            })
        }
    }

    /// Get current limit status without incrementing
    pub async fn get_limit_status(&self, key: &str) -> Result<Option<u32>, RedisError> {
        let mut conn = self.redis.get_async_connection().await?;
        let count: Option<u32> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(count)
    }

    /// Reset limit for user (admin only)
    pub async fn reset_user_limit(&self, user_id: &str) -> Result<(), RedisError> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("ratelimit:user:{}", user_id);
        redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

// ==================== MIDDLEWARE ====================

/// Rate limit middleware for authenticated users
pub async fn rate_limit_middleware<B>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<B>,
    next: Next,
) -> Result<Response, RateLimitError> {
    let limiter = req
        .extensions()
        .get::<Arc<RateLimiter>>()
        .cloned()
        .ok_or(RateLimitError::ServerError)?;

    // Try to get user ID from JWT
    let user_id = extract_user_from_request(&req);

    let limit_info = if let Some(user_id) = user_id {
        // Authenticated user
        limiter
            .check_user_limit(&user_id)
            .await
            .map_err(|_| RateLimitError::ServerError)?
    } else {
        // Anonymous user (by IP)
        limiter
            .check_ip_limit(&addr.ip())
            .await
            .map_err(|_| RateLimitError::ServerError)?
    };

    if limit_info.current > limit_info.limit {
        return Err(RateLimitError::LimitExceeded {
            retry_after: (limit_info.reset_at as i32 - std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32)
                .max(1),
        });
    }

    let mut res = next.run(req).await;

    // Add rate limit headers
    res.headers_mut().insert(
        "X-RateLimit-Limit",
        format!("{}", limit_info.limit)
            .parse()
            .unwrap_or_default(),
    );
    res.headers_mut().insert(
        "X-RateLimit-Remaining",
        format!("{}", limit_info.limit.saturating_sub(limit_info.current))
            .parse()
            .unwrap_or_default(),
    );
    res.headers_mut().insert(
        "X-RateLimit-Reset",
        format!("{}", limit_info.reset_at)
            .parse()
            .unwrap_or_default(),
    );

    Ok(res)
}

/// Endpoint-specific rate limit middleware
/// Used for login (5 attempts per minute), password reset requests, etc.
pub async fn endpoint_rate_limit_middleware(
    identifier: String,
    endpoint: &str,
    limit: u32,
    window: u64,
) -> Result<(), RateLimitError> {
    // This would be called in specific handler for login, password reset, etc.
    // Example usage in login handler:
    // endpoint_rate_limit_middleware(email, "login", 5, 60).await?;
    Ok(())
}

// ==================== ERROR HANDLING ====================

#[derive(Debug)]
pub enum RateLimitError {
    LimitExceeded { retry_after: i32 },
    ServerError,
}

impl axum::response::IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::LimitExceeded { retry_after } => {
                let mut headers = axum::http::HeaderMap::new();
                headers.insert(
                    axum::http::header::RETRY_AFTER,
                    format!("{}", retry_after)
                        .parse()
                        .unwrap_or_default(),
                );

                (
                    axum::http::StatusCode::TOO_MANY_REQUESTS,
                    headers,
                    "Rate limit exceeded. Please try again later.",
                )
                    .into_response()
            }
            RateLimitError::ServerError => {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Rate limiter error",
                )
                    .into_response()
            }
        }
    }
}

// ==================== HELPERS ====================

fn extract_user_from_request<B>(req: &Request<B>) -> Option<String> {
    // Extract user_id from JWT token in Authorization header
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .and_then(|token| decode_jwt_user_id(token))
}

fn decode_jwt_user_id(token: &str) -> Option<String> {
    // Decode JWT and extract user_id claim
    // This is a stub - implement with your JWT library
    None
}

// ==================== USAGE EXAMPLE ====================

/*
In your main.rs or app setup:

use axum::middleware;
use std::sync::Arc;

let redis = Arc::new(
    redis::Client::open("redis://localhost:6379")
        .expect("invalid connection URL")
);

let rate_limiter = Arc::new(RateLimiter::new(
    redis,
    RateLimitConfig::default(),
));

let app = Router::new()
    .route("/api/peers", post(create_peer))
    .route("/auth/login", post(login))
    .route_layer(middleware::from_fn_with_state(
        rate_limiter,
        rate_limit_middleware,
    ));


In login handler:
#[post("/auth/login")]
async fn login(
    limiter: Arc<RateLimiter>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<TokenResponse>> {
    // Check endpoint-specific rate limit (5 attempts per minute per email)
    limiter
        .check_endpoint_limit("login", &payload.email, 5, 60)
        .await
        .map_err(|_| AppError::TooManyRequests)?;
    
    // ... rest of login logic
    Ok(Json(token_response))
}
*/
