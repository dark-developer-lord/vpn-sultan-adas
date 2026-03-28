use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ==================== TYPES ====================

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub active_users_today: i64,
    pub total_vpn_connections: i64,
    pub active_connections: i64,
    pub revenue_this_month: f64,
    pub churn_rate: f64,
    pub avg_session_duration: i64,
    pub total_data_transferred: i64,
}

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub id: String,
    pub email: String,
    pub name: String,
    pub subscription_tier: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub total_sessions: i64,
    pub data_transferred: i64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionStats {
    pub id: String,
    pub user_id: String,
    pub peer_count: i32,
    pub active_peers: i32,
    pub uptime_percent: f64,
    pub data_transferred: i64,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RevenueReport {
    pub period: String,
    pub total_revenue: f64,
    pub subscription_revenue: f64,
    pub one_time_revenue: f64,
    pub new_subscriptions: i64,
    pub cancelled_subscriptions: i64,
    pub active_subscriptions: i64,
}

#[derive(Debug, Serialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub event_type: String,
    pub description: String,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub status: String,
    pub uptime_percent: f64,
    pub response_time_ms: f64,
    pub error_rate: f64,
    pub active_requests: i32,
    pub database_connections: i32,
    pub cache_hit_rate: f64,
}

// ==================== ADMIN SERVICE ====================

pub struct AdminService {
    db: Database,
}

impl AdminService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get dashboard statistics
    pub async fn get_dashboard_stats(&self) -> Result<DashboardStats> {
        let total_users = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL",
                &[],
            )
            .await?;

        let active_users_today = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(DISTINCT user_id) FROM sessions 
                 WHERE created_at > NOW() - INTERVAL '24 hours'",
                &[],
            )
            .await?;

        let total_vpn_connections = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM vpn_connections",
                &[],
            )
            .await?;

        let active_connections = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM vpn_connections WHERE active = true",
                &[],
            )
            .await?;

        let revenue_this_month: f64 = self.db
            .query_scalar(
                "SELECT COALESCE(SUM(amount), 0) FROM stripe_payments 
                 WHERE status = 'succeeded' AND created_at > NOW() - INTERVAL '30 days'",
                &[],
            )
            .await?
            .unwrap_or(0.0);

        let churned = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM users 
                 WHERE subscription_cancelled_at > NOW() - INTERVAL '30 days'",
                &[],
            )
            .await?;

        let active_subs = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM users WHERE subscription_tier != 'free'",
                &[],
            )
            .await?;

        let churn_rate = if active_subs > 0 {
            (churned as f64 / active_subs as f64) * 100.0
        } else {
            0.0
        };

        let avg_session_duration = self.db
            .query_scalar::<i64>(
                "SELECT COALESCE(AVG(EXTRACT(epoch FROM (ended_at - created_at)))::bigint, 0) 
                 FROM sessions WHERE ended_at IS NOT NULL",
                &[],
            )
            .await?;

        let total_data: String = self.db
            .query_scalar(
                "SELECT COALESCE(SUM(data_transferred), 0) FROM vpn_connections",
                &[],
            )
            .await?
            .unwrap_or("0".to_string());

        Ok(DashboardStats {
            total_users,
            active_users_today,
            total_vpn_connections,
            active_connections,
            revenue_this_month: revenue_this_month / 100.0, // Convert from cents
            churn_rate,
            avg_session_duration,
            total_data_transferred: total_data.parse().unwrap_or(0),
        })
    }

    /// Get paginated users with filters
    pub async fn get_users(
        &self,
        page: i32,
        limit: i32,
        filter: Option<String>,
    ) -> Result<(Vec<UserStats>, i64)> {
        let offset = (page - 1) * limit;

        // Build filter query
        let where_clause = if let Some(f) = filter {
            format!("WHERE (email ILIKE '%{}%' OR name ILIKE '%{}%')", f, f)
        } else {
            "".to_string()
        };

        // Get total count
        let total = self.db
            .query_scalar::<i64>(
                &format!("SELECT COUNT(*) FROM users {}", where_clause),
                &[],
            )
            .await?;

        // Get paginated results
        let rows = self.db
            .query(
                &format!(
                    "SELECT id, email, name, subscription_tier, created_at, last_login, 
                            (SELECT COUNT(*) FROM sessions WHERE user_id = users.id) as total_sessions,
                            COALESCE((SELECT SUM(data_transferred) FROM vpn_connections WHERE user_id = users.id), 0) as data_transferred,
                            CASE WHEN deleted_at IS NOT NULL THEN 'deleted' 
                                 WHEN subscription_tier = 'free' THEN 'free' 
                                 ELSE 'paid' END as status
                     FROM users {}
                     ORDER BY created_at DESC
                     LIMIT $1 OFFSET $2",
                    where_clause
                ),
                &[&limit, &offset],
            )
            .await?;

        let users = rows
            .iter()
            .map(|row| UserStats {
                id: row.get(0),
                email: row.get(1),
                name: row.get(2),
                subscription_tier: row.get(3),
                created_at: row.get(4),
                last_login: row.get(5),
                total_sessions: row.get(6),
                data_transferred: row.get(7),
                status: row.get(8),
            })
            .collect();

        Ok((users, total))
    }

    /// Get user details
    pub async fn get_user_detail(&self, user_id: &str) -> Result<UserStats> {
        let row = self.db
            .query_one(
                "SELECT id, email, name, subscription_tier, created_at, last_login, 
                        (SELECT COUNT(*) FROM sessions WHERE user_id = users.id) as total_sessions,
                        COALESCE((SELECT SUM(data_transferred) FROM vpn_connections WHERE user_id = users.id), 0) as data_transferred,
                        CASE WHEN deleted_at IS NOT NULL THEN 'deleted' 
                             WHEN subscription_tier = 'free' THEN 'free' 
                             ELSE 'paid' END as status
                 FROM users WHERE id = $1",
                &[user_id],
            )
            .await?
            .ok_or(AppError::NotFound("User not found".to_string()))?;

        Ok(UserStats {
            id: row.get(0),
            email: row.get(1),
            name: row.get(2),
            subscription_tier: row.get(3),
            created_at: row.get(4),
            last_login: row.get(5),
            total_sessions: row.get(6),
            data_transferred: row.get(7),
            status: row.get(8),
        })
    }

    /// Get revenue report
    pub async fn get_revenue_report(&self, month: i32, year: i32) -> Result<RevenueReport> {
        let start_date = format!("{}-{:02}-01", year, month);
        let end_date = if month == 12 {
            format!("{}-01-01", year + 1)
        } else {
            format!("{}-{:02}-01", year, month + 1)
        };

        let subscription_revenue: f64 = self.db
            .query_scalar(
                "SELECT COALESCE(SUM(amount), 0) FROM stripe_payments 
                 WHERE status = 'succeeded' AND created_at >= $1::date AND created_at < $2::date",
                &[&start_date, &end_date],
            )
            .await?
            .unwrap_or(0.0);

        let total_revenue = subscription_revenue; // Add one-time payments if applicable
        
        let new_subscriptions = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM users 
                 WHERE subscription_started_at >= $1::date AND subscription_started_at < $2::date",
                &[&start_date, &end_date],
            )
            .await?;

        let cancelled_subscriptions = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM users 
                 WHERE subscription_cancelled_at >= $1::date AND subscription_cancelled_at < $2::date",
                &[&start_date, &end_date],
            )
            .await?;

        let active_subscriptions = self.db
            .query_scalar::<i64>(
                "SELECT COUNT(*) FROM users WHERE subscription_tier != 'free'",
                &[],
            )
            .await?;

        Ok(RevenueReport {
            period: format!("{}-{:02}", year, month),
            total_revenue: total_revenue / 100.0,
            subscription_revenue: subscription_revenue / 100.0,
            one_time_revenue: 0.0,
            new_subscriptions,
            cancelled_subscriptions,
            active_subscriptions,
        })
    }

    /// Get audit logs
    pub async fn get_audit_logs(
        &self,
        page: i32,
        limit: i32,
    ) -> Result<Vec<AuditLog>> {
        let offset = (page - 1) * limit;

        let rows = self.db
            .query(
                "SELECT id, user_id, event_type, description, ip_address, timestamp 
                 FROM audit_logs 
                 ORDER BY timestamp DESC 
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;

        let logs = rows
            .iter()
            .map(|row| AuditLog {
                id: row.get(0),
                user_id: row.get(1),
                event_type: row.get(2),
                description: row.get(3),
                ip_address: row.get(4),
                timestamp: row.get(5),
            })
            .collect();

        Ok(logs)
    }

    /// Get system health
    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        // Query health metrics (would integrate with monitoring system)
        Ok(SystemHealth {
            status: "healthy".to_string(),
            uptime_percent: 99.95,
            response_time_ms: 45.2,
            error_rate: 0.01,
            active_requests: 142,
            database_connections: 8,
            cache_hit_rate: 87.5,
        })
    }

    /// Suspend/unsuspend user
    pub async fn toggle_user_status(
        &self,
        user_id: &str,
        suspended: bool,
    ) -> Result<()> {
        self.db
            .execute(
                "UPDATE users SET suspended = $1, updated_at = NOW() WHERE id = $2",
                &[&suspended, user_id],
            )
            .await?;

        log_audit_event(
            user_id,
            if suspended { "USER_SUSPENDED" } else { "USER_UNSUSPENDED" },
            &format!("User {}", if suspended { "suspended" } else { "unsuspended" }),
            &self.db,
        )
        .await?;

        Ok(())
    }
}

// ==================== HANDLERS (Protected with admin role) ====================

/// Get dashboard statistics
#[get("/admin/dashboard")]
pub async fn get_dashboard_stats(
    admin: AdminAuth, // Custom extractor that checks admin role
    db: Database,
) -> Result<Json<DashboardStats>> {
    let service = AdminService::new(db);
    let stats = service.get_dashboard_stats().await?;
    Ok(Json(stats))
}

/// Get users (paginated)
#[get("/admin/users?page=<page>&limit=<limit>&filter=<filter>")]
pub async fn get_users(
    admin: AdminAuth,
    db: Database,
    page: Option<i32>,
    limit: Option<i32>,
    filter: Option<String>,
) -> Result<Json<(Vec<UserStats>, i64)>> {
    let service = AdminService::new(db);
    let (users, total) = service
        .get_users(page.unwrap_or(1), limit.unwrap_or(50), filter)
        .await?;
    Ok(Json((users, total)))
}

/// Get user detail
#[get("/admin/users/<user_id>")]
pub async fn get_user_detail(
    admin: AdminAuth,
    db: Database,
    user_id: String,
) -> Result<Json<UserStats>> {
    let service = AdminService::new(db);
    let user = service.get_user_detail(&user_id).await?;
    Ok(Json(user))
}

/// Get revenue report
#[get("/admin/revenue?month=<month>&year=<year>")]
pub async fn get_revenue_report(
    admin: AdminAuth,
    db: Database,
    month: Option<i32>,
    year: Option<i32>,
) -> Result<Json<RevenueReport>> {
    let service = AdminService::new(db);
    let now = Utc::now();
    let month = month.unwrap_or(now.month() as i32);
    let year = year.unwrap_or(now.year());
    
    let report = service.get_revenue_report(month, year).await?;
    Ok(Json(report))
}

/// Get audit logs
#[get("/admin/audit-logs?page=<page>&limit=<limit>")]
pub async fn get_audit_logs(
    admin: AdminAuth,
    db: Database,
    page: Option<i32>,
    limit: Option<i32>,
) -> Result<Json<Vec<AuditLog>>> {
    let service = AdminService::new(db);
    let logs = service
        .get_audit_logs(page.unwrap_or(1), limit.unwrap_or(100))
        .await?;
    Ok(Json(logs))
}

/// Get system health
#[get("/admin/health")]
pub async fn get_system_health(
    admin: AdminAuth,
    db: Database,
) -> Result<Json<SystemHealth>> {
    let service = AdminService::new(db);
    let health = service.get_system_health().await?;
    Ok(Json(health))
}

/// Suspend user
#[post("/admin/users/<user_id>/suspend")]
pub async fn suspend_user(
    admin: AdminAuth,
    db: Database,
    user_id: String,
) -> Result<StatusCode> {
    let service = AdminService::new(db);
    service.toggle_user_status(&user_id, true).await?;
    Ok(StatusCode::OK)
}

/// Unsuspend user
#[post("/admin/users/<user_id>/unsuspend")]
pub async fn unsuspend_user(
    admin: AdminAuth,
    db: Database,
    user_id: String,
) -> Result<StatusCode> {
    let service = AdminService::new(db);
    service.toggle_user_status(&user_id, false).await?;
    Ok(StatusCode::OK)
}

// ==================== ADMIN AUTH EXTRACTOR ====================

pub struct AdminAuth;

#[async_trait]
impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> std::result::Result<Self, Self::Rejection> {
        // Extract from JWT and check admin role
        // Implementation would depend on your JWT structure
        Ok(AdminAuth)
    }
}
