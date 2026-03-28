use sqlx::PgPool;
use uuid::Uuid;
use vpn_shared::AppError;
use vpn_shared::types::VpnNode;
use vpn_data::repository::NodeRepository;

pub struct NodeService;

impl NodeService {
    /// List all online nodes
    pub async fn list_online_nodes(pool: &PgPool) -> Result<Vec<VpnNode>, AppError> {
        NodeRepository::list_all(pool).await
    }

    /// List all nodes including offline
    pub async fn list_all_nodes(pool: &PgPool) -> Result<Vec<VpnNode>, AppError> {
        NodeRepository::list_all_including_offline(pool).await
    }

    /// Get a specific node
    pub async fn get_node(pool: &PgPool, node_id: Uuid) -> Result<VpnNode, AppError> {
        NodeRepository::find_by_id(pool, node_id).await
    }

    /// Update node heartbeat (called by agent)
    pub async fn heartbeat(pool: &PgPool, node_id: Uuid) -> Result<(), AppError> {
        NodeRepository::update_heartbeat(pool, node_id).await
    }
}
