use axum::{
    routing::{get, post, delete, put},
    Router,
};
use std::sync::Arc;
use vpn_data::Database;
use crate::handlers;

pub fn build_router(db: Database) -> Router {
    let db = Arc::new(db);
    
    Router::new()
        .route("/health", get(handlers::health))
        .route("/health/ready", get(handlers::ready))
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/peers", get(handlers::list_peers))
        .route("/peers", post(handlers::create_peer))
        .route("/peers/:peer_id", get(handlers::get_peer))
        .route("/peers/:peer_id", delete(handlers::delete_peer))
        .route("/peers/:peer_id/config", get(handlers::get_peer_config))
        .route("/nodes", get(handlers::list_nodes))
        .route("/agents/register", post(handlers::register_agent))
        .route("/agents/:node_id/heartbeat", put(handlers::agent_heartbeat))
        .with_state(db)
}
