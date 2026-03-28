use sqlx::PgPool;
use uuid::Uuid;
use vpn_shared::AppError;
use vpn_shared::types::VpnPeer;
use vpn_data::repository::{PeerRepository, SubscriptionRepository};
use vpn_crypto::KeyGenerator;

pub struct PeerService;

impl PeerService {
    /// Create a new peer for the user
    pub async fn create_peer(
        pool: &PgPool,
        user_id: Uuid,
        node_id: Uuid,
        name: &str,
    ) -> Result<VpnPeer, AppError> {
        // Check subscription limit
        let subscription = SubscriptionRepository::find_by_user_id(pool, user_id)
            .await?;

        let current_count = PeerRepository::count_by_user(pool, user_id)
            .await?;

        if current_count >= subscription.max_peers as i64 {
            return Err(AppError::SubscriptionLimitExceeded);
        }

        // Generate WireGuard keypair
        let (public_key, _) = KeyGenerator::generate_keypair();

        // Create peer in db
        let peer = PeerRepository::create(pool, user_id, node_id, name, &public_key)
            .await?;

        Ok(peer)
    }

    /// Get a specific peer (with authorization check)
    pub async fn get_peer(
        pool: &PgPool,
        peer_id: Uuid,
        user_id: Uuid,
    ) -> Result<VpnPeer, AppError> {
        PeerRepository::find_by_id_for_user(pool, peer_id, user_id)
            .await
    }

    /// List all peers for a user
    pub async fn list_peers(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<VpnPeer>, AppError> {
        PeerRepository::list_by_user(pool, user_id)
            .await
    }

    /// Delete a peer (revoke access)
    pub async fn delete_peer(
        pool: &PgPool,
        peer_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        // Verify ownership first
        let _peer = PeerRepository::find_by_id_for_user(pool, peer_id, user_id)
            .await?;

        PeerRepository::delete(pool, peer_id)
            .await?;

        Ok(())
    }
}
