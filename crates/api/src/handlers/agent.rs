use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use vpn_data::Database;
use vpn_shared::types::ApiResponse;

#[derive(Deserialize)]
pub struct AgentRegisterRequest {
    pub node_name: String,
    pub public_ip: String,
    pub internal_ip: String,
    pub wg_public_key: String,
}

#[derive(Serialize)]
pub struct AgentRegisterResponse {
    pub node_id: Uuid,
    pub api_key: String,
    pub heartbeat_interval_secs: u32,
}

/// Register a new VPN node agent
pub async fn register_agent(
    State(db): State<Arc<Database>>,
    Json(payload): Json<AgentRegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AgentRegisterResponse>>), (StatusCode, Json<serde_json::Value>)> {
    // Validate input
    if payload.node_name.is_empty()
        || payload.public_ip.is_empty()
        || payload.internal_ip.is_empty()
        || payload.wg_public_key.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "error": "All fields are required"
            })),
        ));
    }

    // Validate IP format (basic)
    if !payload.public_ip.contains('.') || !payload.internal_ip.contains('.') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "error": "Invalid IP address format"
            })),
        ));
    }

    use vpn_data::repository::NodeRepository;

    // Create node in database
    let node = NodeRepository::create(
        &db.pool,
        &payload.node_name,
        &payload.public_ip,
        &payload.internal_ip,
        &payload.wg_public_key,
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "error": "Failed to register node"
            })),
        )
    })?;

    // Generate API key (simple UUID-based for now, would be in production: bcrypt hash of random token)
    let api_key = format!("node_{}", Uuid::new_v4().to_string());

    tracing::info!(
        "Node registered: {} ({}:{})",
        node.name,
        node.public_ip,
        node.internal_ip
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(AgentRegisterResponse {
            node_id: node.id,
            api_key,
            heartbeat_interval_secs: 30,
        })),
    ))
}

/// Node heartbeat - called periodically by agents to report status
pub async fn agent_heartbeat(
    State(db): State<Arc<Database>>,
    axum::extract::Path(node_id): axum::extract::Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Validate API key from Authorization header
    // For now, just update the heartbeat timestamp
    use vpn_data::repository::NodeRepository;

    NodeRepository::update_heartbeat(&db.pool, node_id)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "status": "error",
                    "error": "Node not found"
                })),
            )
        })?;

    tracing::debug!("Heartbeat received from node {}", node_id);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "status": "ok"
    }))))
}
