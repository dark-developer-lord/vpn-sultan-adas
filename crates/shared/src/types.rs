use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// User types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

// Subscription types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionPlan {
    Free,
    Pro,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan: SubscriptionPlan,
    pub status: String,
    pub data_limit_gb: Option<i32>,
    pub max_peers: i32,
    pub created_at: DateTime<Utc>,
}

// VPN Node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnNode {
    pub id: Uuid,
    pub name: String,
    pub public_ip: String,
    pub internal_ip: String,
    pub wg_public_key: String,
    pub status: String,  // "online", "offline", "degraded"
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// VPN Peer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnPeer {
    pub id: Uuid,
    pub user_id: Uuid,
    pub node_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub status: String,  // "active", "revoked", "inactive"
    pub created_at: DateTime<Utc>,
    pub last_connected_at: Option<DateTime<Utc>>,
}

// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,  // user_id
    pub email: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

// API Response wrappers
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub status: String,
    pub error: String,
    pub timestamp: DateTime<Utc>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            data,
        }
    }
}
