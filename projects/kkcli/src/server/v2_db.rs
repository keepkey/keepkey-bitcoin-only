use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, FromRow};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct V2Network {
    pub id: i64,
    pub chain_id_caip2: String,
    pub display_name: String,
    pub network_name_v2: String,
    pub symbol: String,
    pub is_evm: bool,
    pub is_testnet: bool,
    pub enabled: bool,
}

impl V2Network {
    pub async fn get_enabled_v2(pool: &SqlitePool) -> sqlx::Result<Vec<V2Network>> {
        sqlx::query_as::<_, V2Network>("SELECT * FROM networks WHERE enabled = 1")
            .fetch_all(pool)
            .await
    }
}

// (Future) More CRUD functions as needed for v2_db
