use sqlx::PgPool;
use uuid::Uuid;
use vpn_shared::types::VpnPeer;
use vpn_shared::AppError;
use chrono::Utc;

#[derive(Debug, sqlx::FromRow)]
struct PeerRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub node_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
    pub last_connected_at: Option<chrono::DateTime<Utc>>,
}

impl From<PeerRow> for VpnPeer {
    fn from(row: PeerRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            node_id: row.node_id,
            name: row.name,
            public_key: row.public_key,
            status: row.status,
            created_at: row.created_at,
            last_connected_at: row.last_connected_at,
        }
    }
}

pub struct PeerRepository;

impl PeerRepository {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        node_id: Uuid,
        name: &str,
        public_key: &str,
    ) -> Result<VpnPeer, AppError> {
        let peer_id = Uuid::new_v4();
        let now = Utc::now();
        
        let row = sqlx::query_as::<_, PeerRow>(
            r#"
            INSERT INTO vpn_peers (id, user_id, node_id, name, public_key, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, node_id, name, public_key, status, created_at, last_connected_at
            "#,
        )
        .bind(peer_id)
        .bind(user_id)
        .bind(node_id)
        .bind(name)
        .bind(public_key)
        .bind("active")
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("foreign key") {
                AppError::NotFound
            } else {
                AppError::DatabaseError(e.to_string())
            }
        })?;

        Ok(row.into())
    }

    pub async fn find_by_id(pool: &PgPool, peer_id: Uuid) -> Result<VpnPeer, AppError> {
        let row = sqlx::query_as::<_, PeerRow>(
            "SELECT id, user_id, node_id, name, public_key, status, created_at, last_connected_at FROM vpn_peers WHERE id = $1"
        )
        .bind(peer_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or(AppError::NotFound)?;

        Ok(row.into())
    }

    pub async fn find_by_id_for_user(
        pool: &PgPool,
        peer_id: Uuid,
        user_id: Uuid,
    ) -> Result<VpnPeer, AppError> {
        let row = sqlx::query_as::<_, PeerRow>(
            "SELECT id, user_id, node_id, name, public_key, status, created_at, last_connected_at FROM vpn_peers WHERE id = $1 AND user_id = $2"
        )
        .bind(peer_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or(AppError::NotFound)?;

        Ok(row.into())
    }

    pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<VpnPeer>, AppError> {
        let rows = sqlx::query_as::<_, PeerRow>(
            "SELECT id, user_id, node_id, name, public_key, status, created_at, last_connected_at FROM vpn_peers WHERE user_id = $1 AND status != 'revoked' ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete(pool: &PgPool, peer_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM vpn_peers WHERE id = $1")
            .bind(peer_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    pub async fn count_by_user(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM vpn_peers WHERE user_id = $1 AND status != 'revoked'"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.0)
    }
}
