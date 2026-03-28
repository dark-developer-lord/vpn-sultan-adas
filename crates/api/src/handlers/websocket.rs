use axum::extract::{ws::WebSocket, ConnectInfo};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error};

// ==================== WEBSOCKET EVENTS ====================

#[derive(Clone, Debug)]
pub enum PeerEvent {
    PeerCreated { user_id: String, peer_id: String },
    PeerDeleted { user_id: String, peer_id: String },
    PeerStatusChanged { peer_id: String, status: String },
    NodeOnline { node_id: String },
    NodeOffline { node_id: String },
    TrafficUpdated { peer_id: String, bytes_in: u64, bytes_out: u64 },
}

// ==================== WEBSOCKET STATE ====================

#[derive(Clone)]
pub struct WebSocketState {
    pub tx: broadcast::Sender<PeerEvent>,
}

impl WebSocketState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub async fn broadcast_event(&self, event: PeerEvent) {
        if let Err(e) = self.tx.send(event) {
            error!("Failed to broadcast event: {}", e);
        }
    }
}

// ==================== WEBSOCKET HANDLERS ====================

pub async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: Arc<WebSocketState>,
    auth: AuthLayer,
) -> impl axum::response::IntoResponse {
    info!("WebSocket connection from {}", addr);
    
    ws.on_upgrade(move |socket| handle_socket(socket, auth.user_id, state))
}

async fn handle_socket(
    mut socket: WebSocket,
    user_id: String,
    state: Arc<WebSocketState>,
) {
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            // Handle incoming messages from client
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        info!("Client {} requested close", user_id);
                        break;
                    }
                    Ok(axum::extract::ws::Message::Ping(ping)) => {
                        if socket.send(axum::extract::ws::Message::Pong(ping)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            
            // Handle events from broadcast channel
            Ok(event) = rx.recv() => {
                // Filter events relevant to this user
                if should_send_event(&event, &user_id).await {
                    if let Ok(msg) = serde_json::to_string(&event) {
                        if socket
                            .send(axum::extract::ws::Message::Text(msg))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        }
    }

    info!("WebSocket connection closed for user {}", user_id);
}

// ==================== EVENT FILTERING ====================

async fn should_send_event(event: &PeerEvent, user_id: &str) -> bool {
    match event {
        PeerEvent::PeerCreated { user_id: u, .. } => u == user_id,
        PeerEvent::PeerDeleted { user_id: u, .. } => u == user_id,
        PeerEvent::PeerStatusChanged { .. } => true, // Enable for peers user owns
        PeerEvent::NodeOnline { .. } => true,         // Admin feature
        PeerEvent::NodeOffline { .. } => true,        // Admin feature
        PeerEvent::TrafficUpdated { .. } => true,     // Enable for peers user owns
    }
}

// ==================== EVENT EMITTERS ====================

pub mod event_emitters {
    use super::*;

    pub async fn emit_peer_created(
        state: &Arc<WebSocketState>,
        user_id: String,
        peer_id: String,
    ) {
        state
            .broadcast_event(PeerEvent::PeerCreated { user_id, peer_id })
            .await;
    }

    pub async fn emit_peer_deleted(
        state: &Arc<WebSocketState>,
        user_id: String,
        peer_id: String,
    ) {
        state
            .broadcast_event(PeerEvent::PeerDeleted { user_id, peer_id })
            .await;
    }

    pub async fn emit_node_status_changed(
        state: &Arc<WebSocketState>,
        node_id: String,
        is_online: bool,
    ) {
        let event = if is_online {
            PeerEvent::NodeOnline { node_id }
        } else {
            PeerEvent::NodeOffline { node_id }
        };
        state.broadcast_event(event).await;
    }

    pub async fn emit_traffic_updated(
        state: &Arc<WebSocketState>,
        peer_id: String,
        bytes_in: u64,
        bytes_out: u64,
    ) {
        state
            .broadcast_event(PeerEvent::TrafficUpdated {
                peer_id,
                bytes_in,
                bytes_out,
            })
            .await;
    }
}

// ==================== FRONTEND INTEGRATION ====================

// Use in frontend:
/*
const ws = new WebSocket('wss://api.vpn-service.com/ws');

ws.onmessage = (event) => {
    const peer_event = JSON.parse(event.data);
    
    switch(peer_event.type) {
        case 'PeerCreated':
            updatePeersList();
            break;
        case 'PeerDeleted':
            removePeerFromUI(peer_event.peer_id);
            break;
        case 'TrafficUpdated':
            updateTrafficStats(peer_event);
            break;
    }
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
    // Fallback to polling
};
*/
