use sqlx::PgPool;
use uuid::Uuid;
use vpn_shared::types::VpnNode;
use vpn_shared::AppError;
use chrono::Utc;

#[derive(Debug, sqlx::FromRow)]
struct NodeRow {
    pub id: Uuid,
    pub name: String,
    pub public_ip: String,
    pub internal_ip: String,
    pub wg_public_key: String,
    pub status: String,
    pub last_heartbeat_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<NodeRow> for VpnNode {
    fn from(row: NodeRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            public_ip: row.public_ip,
            internal_ip: row.internal_ip,
            wg_public_key: row.wg_public_key,
            status: row.status,
            last_heartbeat_at: row.last_heartbeat_at,
            created_at: row.created_at,
        }
    }
}

pub struct NodeRepository;

impl NodeRepository {
    pub async fn create(
        pool: &PgPool,
        name: &str,
        public_ip: &str,
        internal_ip: &str,
        wg_public_key: &str,
    ) -> Result<VpnNode, AppError> {
        let node_id = Uuid::new_v4();
        let now = Utc::now();
        
        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            INSERT INTO vpn_nodes (id, name, public_ip, internal_ip, wg_public_key, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, public_ip, internal_ip, wg_public_key, status, last_heartbeat_at, created_at
            "#,
        )
        .bind(node_id)
        .bind(name)
        .bind(public_ip)
        .bind(internal_ip)
        .bind(wg_public_key)
        .bind("offline")
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.into())
    }

    pub async fn find_by_id(pool: &PgPool, node_id: Uuid) -> Result<VpnNode, AppError> {
        let row = sqlx::query_as::<_, NodeRow>(
            "SELECT id, name, public_ip, internal_ip, wg_public_key, status, last_heartbeat_at, created_at FROM vpn_nodes WHERE id = $1"
        )
        .bind(node_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or(AppError::NotFound)?;

        Ok(row.into())
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<VpnNode>, AppError> {
        let rows = sqlx::query_as::<_, NodeRow>(
            "SELECT id, name, public_ip, internal_ip, wg_public_key, status, last_heartbeat_at, created_at FROM vpn_nodes WHERE status = 'online' ORDER BY created_at ASC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_all_including_offline(pool: &PgPool) -> Result<Vec<VpnNode>, AppError> {
        let rows = sqlx::query_as::<_, NodeRow>(
            "SELECT id, name, public_ip, internal_ip, wg_public_key, status, last_heartbeat_at, created_at FROM vpn_nodes ORDER BY created_at ASC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_heartbeat(pool: &PgPool, node_id: Uuid) -> Result<(), AppError> {
        let now = Utc::now();
        sqlx::query("UPDATE vpn_nodes SET last_heartbeat_at = $1, status = 'online' WHERE id = $2")
            .bind(now)
            .bind(node_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
