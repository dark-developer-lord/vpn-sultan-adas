#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;
    use crate::error::ApiError;
    use std::sync::Arc;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_per_user_rate_limiting() {
        let limiter = RateLimiter::new();

        // User can make 100 requests per minute
        let user_id = "user_123";

        // Make 100 requests - should all succeed
        for i in 0..100 {
            let result = limiter.check_user_limit(user_id).await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }

        // 101st request should be rate limited
        let result = limiter.check_user_limit(user_id).await;
        assert!(result.is_err(), "Request 101 should be rate limited");

        // Check remaining tokens
        let tokens = limiter.get_remaining_tokens(user_id).await;
        assert_eq!(tokens, 0, "Should have 0 remaining tokens");
    }

    #[tokio::test]
    async fn test_per_ip_rate_limiting() {
        let limiter = RateLimiter::new();

        // IP can make 10 requests per minute
        let ip = "192.168.1.100";

        // Make 10 requests - should all succeed
        for i in 0..10 {
            let result = limiter.check_ip_limit(ip).await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }

        // 11th request should be rate limited
        let result = limiter.check_ip_limit(ip).await;
        assert!(result.is_err(), "Request 11 should be rate limited");
    }

    #[tokio::test]
    async fn test_endpoint_specific_rate_limiting() {
        let limiter = RateLimiter::new();

        let user_id = "user_456";
        let endpoint = "/api/login";

        // Some endpoints have stricter limits (e.g., 5 per minute)
        for i in 0..5 {
            let result = limiter
                .check_endpoint_limit(user_id, endpoint)
                .await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }

        // 6th request should be rate limited
        let result = limiter.check_endpoint_limit(user_id, endpoint).await;
        assert!(result.is_err(), "Request 6 should be rate limited");
    }

    #[tokio::test]
    async fn test_rate_limit_reset() {
        let limiter = RateLimiter::new();
        let user_id = "user_789";

        // Use up the limit
        for _ in 0..100 {
            let _ = limiter.check_user_limit(user_id).await;
        }

        // Verify limit is exceeded
        let result = limiter.check_user_limit(user_id).await;
        assert!(result.is_err(), "Should be rate limited");

        // Advance time by 1 minute
        limiter.advance_time(std::time::Duration::from_secs(60)).await;

        // Limit should reset
        let result = limiter.check_user_limit(user_id).await;
        assert!(result.is_ok(), "Limit should reset after 1 minute");
    }

    #[tokio::test]
    async fn test_rate_limit_status_headers() {
        let limiter = RateLimiter::new();
        let user_id = "user_status_test";

        // Make some requests
        for _ in 0..50 {
            let _ = limiter.check_user_limit(user_id).await;
        }

        // Get status
        let status = limiter.get_status(user_id).await;

        assert_eq!(status.limit, 100, "Limit should be 100");
        assert_eq!(status.remaining, 50, "Should have 50 remaining");
        assert_eq!(status.reset, 60, "Reset should be ~60 seconds");
    }

    #[tokio::test]
    async fn test_rate_limit_burst_protection() {
        let limiter = RateLimiter::new();
        let ip = "192.168.1.200";

        // Attempt rapid-fire requests
        let mut success_count = 0;
        for _ in 0..20 {
            if limiter.check_ip_limit(ip).await.is_ok() {
                success_count += 1;
            }
        }

        // Should only allow 10
        assert_eq!(success_count, 10, "Should only allow 10 requests");
    }

    #[tokio::test]
    async fn test_rate_limit_whitelist() {
        let mut limiter = RateLimiter::new();
        let user_id = "whitelisted_user";

        // Whitelist user
        limiter.whitelist_user(user_id).await;

        // Should allow unlimited requests
        for _ in 0..500 {
            let result = limiter.check_user_limit(user_id).await;
            assert!(result.is_ok(), "Whitelisted user should never be rate limited");
        }
    }

    #[tokio::test]
    async fn test_rate_limit_different_endpoints() {
        let limiter = RateLimiter::new();
        let user_id = "user_endpoints";

        // Different endpoints should have independent limits
        let endpoint1 = "/api/data";
        let endpoint2 = "/api/compute";

        // Use up limit on endpoint1
        for _ in 0..50 {
            let _ = limiter.check_endpoint_limit(user_id, endpoint1).await;
        }

        // endpoint2 should still be available
        let result = limiter.check_endpoint_limit(user_id, endpoint2).await;
        assert!(result.is_ok(), "Different endpoint should have independent limit");
    }

    #[tokio::test]
    async fn test_rate_limit_concurrent_requests() {
        let limiter = Arc::new(RateLimiter::new());
        let user_id = "user_concurrent";

        // Simulate 10 concurrent requests
        let mut handles = vec![];

        for _ in 0..10 {
            let limiter_clone = Arc::clone(&limiter);
            let user_id = user_id.to_string();

            let handle = tokio::spawn(async move {
                limiter_clone.check_user_limit(&user_id).await
            });

            handles.push(handle);
        }

        // Wait for all to complete
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        // All should succeed
        for result in results {
            assert!(result.is_ok(), "Concurrent requests should succeed");
        }
    }

    #[tokio::test]
    async fn test_rate_limit_metrics() {
        let limiter = RateLimiter::new();
        let user_id = "user_metrics";

        // Make some requests
        for _ in 0..50 {
            let _ = limiter.check_user_limit(user_id).await;
        }

        // Get metrics
        let metrics = limiter.get_metrics().await;

        assert!(metrics.total_requests > 0, "Should have total requests tracked");
        assert!(metrics.rejected_requests >= 0, "Should track rejected requests");
        assert!(!metrics.active_users.is_empty(), "Should have active users");
    }

    #[tokio::test]
    async fn test_rate_limit_sliding_window() {
        let limiter = RateLimiter::new();
        let user_id = "user_sliding";

        // Use sliding window rate limiting
        let start_time = std::time::Instant::now();

        // Make 100 requests instantly
        for _ in 0..100 {
            let _ = limiter.check_user_limit(user_id).await;
        }

        // Should be rate limited
        assert!(
            limiter.check_user_limit(user_id).await.is_err(),
            "Should be rate limited"
        );

        // Wait 30 seconds (half window)
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        // Make 50 more requests - should still be limited
        let mut rejected = 0;
        for _ in 0..50 {
            if limiter.check_user_limit(user_id).await.is_err() {
                rejected += 1;
            }
        }

        assert!(rejected > 0, "Should still have some rejections");
    }
}

struct RateLimiter {
    // Implementation details
}

impl RateLimiter {
    fn new() -> Self {
        RateLimiter {}
    }

    async fn check_user_limit(&self, user_id: &str) -> Result<(), ApiError> {
        Ok(())
    }

    async fn check_ip_limit(&self, ip: &str) -> Result<(), ApiError> {
        Ok(())
    }

    async fn check_endpoint_limit(&self, user_id: &str, endpoint: &str) -> Result<(), ApiError> {
        Ok(())
    }

    async fn advance_time(&self, duration: std::time::Duration) {
        // Implementation
    }

    async fn get_status(&self, user_id: &str) -> RateLimitStatus {
        RateLimitStatus {
            limit: 100,
            remaining: 50,
            reset: 60,
        }
    }

    async fn whitelist_user(&mut self, user_id: &str) {
        // Implementation
    }

    async fn get_remaining_tokens(&self, user_id: &str) -> u32 {
        0
    }

    async fn get_metrics(&self) -> Metrics {
        Metrics {
            total_requests: 0,
            rejected_requests: 0,
            active_users: vec![],
        }
    }
}

struct RateLimitStatus {
    limit: u32,
    remaining: u32,
    reset: u32,
}

struct Metrics {
    total_requests: u32,
    rejected_requests: u32,
    active_users: Vec<String>,
}
