use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use vpn_data::Database;
use vpn_domain::PeerService;
use vpn_shared::types::{ApiResponse, VpnPeer};
use crate::extractors::AuthUser;

#[derive(Deserialize)]
pub struct CreatePeerRequest {
    pub node_id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct PeerResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub node_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct PeerConfigResponse {
    pub public_key: String,
    pub private_key_encrypted: String,
    pub private_key_nonce: String,
    pub node_endpoint: String,
    pub internal_ip: String,
}

impl From<VpnPeer> for PeerResponse {
    fn from(peer: VpnPeer) -> Self {
        Self {
            id: peer.id,
            user_id: peer.user_id,
            node_id: peer.node_id,
            name: peer.name,
            public_key: peer.public_key,
            status: peer.status,
            created_at: peer.created_at,
            last_connected_at: peer.last_connected_at,
        }
    }
}

/// List all peers for authenticated user
pub async fn list_peers(
    State(db): State<Arc<Database>>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<PeerResponse>>>, (StatusCode, Json<serde_json::Value>)> {
    let peers = PeerService::list_peers(&db.pool, auth.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Failed to fetch peers"
                })),
            )
        })?;

    let responses: Vec<PeerResponse> = peers.into_iter().map(PeerResponse::from).collect();
    tracing::info!("Listed {} peers for user {}", responses.len(), auth.user_id);

    Ok(Json(ApiResponse::ok(responses)))
}

/// Get WireGuard configuration for a peer
pub async fn get_peer_config(
    State(db): State<Arc<Database>>,
    auth: AuthUser,
    Path(peer_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    use vpn_domain::NodeService;

    // Get peer details
    let peer = PeerService::get_peer(&db.pool, peer_id, auth.user_id)
        .await
        .map_err(|e| {
            let (status, message) = match e {
                vpn_shared::AppError::NotFound => (StatusCode::NOT_FOUND, "Peer not found"),
                vpn_shared::AppError::Unauthorized | vpn_shared::AppError::Forbidden => {
                    (StatusCode::FORBIDDEN, "Access denied to this peer")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch peer"),
            };
            (
                status,
                Json(serde_json::json!({
                    "status": "error",
                    "error": message
                })),
            )
        })?;

    // Get node details for endpoint
    let node = NodeService::get_node(&db.pool, peer.node_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Node not available"
                })),
            )
        })?;

    // Generate WireGuard config (INI format)
    let config = format!(
        r#"[Interface]
PrivateKey = {private_key}
ListenPort = 51820
Address = 10.0.0.{addr}/32

[Peer]
PublicKey = {peer_public_key}
AllowedIPs = 10.0.0.0/24
Endpoint = {endpoint}:51820
PersistentKeepalive = 25
"#,
        private_key = "PLACEHOLDER_PRIVATE_KEY", // Mock key - in production, retrieve from encrypted storage
        peer_public_key = peer.public_key,
        endpoint = node.public_ip,
        addr = (auth.user_id.as_u128() % 254) + 2, // Derive IP from user_id
    );

    tracing::info!("Generated config for peer {}", peer_id);

    Ok(Json(serde_json::json!({
        "status": "ok",
        "data": {
            "config": config,
            "format": "wg-quick",
            "peer_name": peer.name,
            "node_name": node.name,
        }
    })))
}

/// Create a new peer (WireGuard configuration)
pub async fn create_peer(
    State(db): State<Arc<Database>>,
    auth: AuthUser,
    Json(payload): Json<CreatePeerRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PeerResponse>>), (StatusCode, Json<serde_json::Value>)> {
    if payload.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "error": "Peer name is required"
            })),
        ));
    }

    let peer = PeerService::create_peer(&db.pool, auth.user_id, payload.node_id, &payload.name)
        .await
        .map_err(|e| {
            let (status, message) = match e {
                vpn_shared::AppError::SubscriptionLimitExceeded => (
                    StatusCode::PAYMENT_REQUIRED,
                    "Maximum peers limit reached for your subscription",
                ),
                vpn_shared::AppError::NotFound => (StatusCode::NOT_FOUND, "Node not found"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create peer"),
            };
            (
                status,
                Json(serde_json::json!({
                    "status": "error",
                    "error": message
                })),
            )
        })?;

    tracing::info!(
        "Created peer {} for user {} on node {}",
        peer.id, auth.user_id, payload.node_id
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(PeerResponse::from(peer))),
    ))
}

/// Get a specific peer
pub async fn get_peer(
    State(db): State<Arc<Database>>,
    auth: AuthUser,
    Path(peer_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PeerResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let peer = PeerService::get_peer(&db.pool, peer_id, auth.user_id)
        .await
        .map_err(|e| {
            let (status, message) = match e {
                vpn_shared::AppError::NotFound => (StatusCode::NOT_FOUND, "Peer not found"),
                vpn_shared::AppError::Unauthorized => {
                    (StatusCode::FORBIDDEN, "Access denied to this peer")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch peer"),
            };
            (
                status,
                Json(serde_json::json!({
                    "status": "error",
                    "error": message
                })),
            )
        })?;

    Ok(Json(ApiResponse::ok(PeerResponse::from(peer))))
}

/// Delete a peer (revoke WireGuard access)
pub async fn delete_peer(
    State(db): State<Arc<Database>>,
    auth: AuthUser,
    Path(peer_id): Path<Uuid>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    PeerService::delete_peer(&db.pool, peer_id, auth.user_id)
        .await
        .map_err(|e| {
            let (status, message) = match e {
                vpn_shared::AppError::NotFound => (StatusCode::NOT_FOUND, "Peer not found"),
                vpn_shared::AppError::Unauthorized => {
                    (StatusCode::FORBIDDEN, "Access denied to this peer")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete peer"),
            };
            (
                status,
                Json(serde_json::json!({
                    "status": "error",
                    "error": message
                })),
            )
        })?;

    tracing::info!("Deleted peer {} for user {}", peer_id, auth.user_id);

    Ok((
        StatusCode::NO_CONTENT,
        Json(serde_json::json!({
            "status": "ok"
        })),
    ))
}

/// List available VPN nodes
pub async fn list_nodes(
    State(db): State<Arc<Database>>,
) -> Result<Json<ApiResponse<Vec<vpn_shared::types::VpnNode>>>, (StatusCode, Json<serde_json::Value>)> {
    use vpn_domain::NodeService;

    let nodes = NodeService::list_online_nodes(&db.pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Failed to fetch nodes"
                })),
            )
        })?;

    tracing::info!("Listed {} available nodes", nodes.len());

    Ok(Json(ApiResponse::ok(nodes)))
}
