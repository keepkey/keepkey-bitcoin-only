use axum::{extract::State, Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;

// --- Trait for State --- 
pub trait HasDbConnection: Send + Sync + 'static {
    fn get_db_connection(&self) -> Arc<Mutex<Connection>>;
}

// --- Data Models ---

#[derive(Debug, Serialize, ToSchema)]
pub struct Network {
    pub id: String,
    pub chain_id_caip2: String,
    pub display_name: String,
    pub network_name: String,
    pub symbol: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Balance {
    pub caip: String,
    pub pubkey: String,
    pub balance: String,
    pub price_usd: Option<f64>,
    pub value_usd: Option<f64>,
    pub symbol: String,
    pub network_id: String,
    pub last_updated: Option<String>,
    pub age: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PortfolioSummary {
    pub total_value_usd: Option<f64>,
    pub network_count: u32,
    pub asset_count: u32,
    pub last_updated: Option<String>,
}

// --- Handlers ---

#[utoipa::path(
    get,
    path = "/v2/networks",
    responses(
        (status = 200, description = "List networks", body = [Network])
    ),
    tag = "v2"
)]
pub async fn get_networks<S_Actual: HasDbConnection>(State(state_arc): State<Arc<S_Actual>>) -> Json<Vec<Network>> {
    let db_arc = state_arc.get_db_connection();
    let networks = tokio::task::spawn_blocking(move || {
        let db = db_arc.blocking_lock();
        let mut stmt = db.prepare(
            "SELECT id, chain_id_caip2, display_name, network_name, symbol, enabled FROM networks"
        )?;
        let networks = stmt
            .query_map([], |row| {
                Ok(Network {
                    id: row.get(0)?,
                    chain_id_caip2: row.get(1)?,
                    display_name: row.get(2)?,
                    network_name: row.get(3)?,
                    symbol: row.get(4)?,
                    enabled: row.get(5)?,
                })
            })?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        Ok::<_, rusqlite::Error>(networks)
    }).await.unwrap().unwrap_or_default();
    Json(networks)
}

#[utoipa::path(
    get,
    path = "/v2/balances",
    responses(
        (status = 200, description = "List balances", body = [Balance])
    ),
    tag = "v2"
)]
pub async fn get_balances<S_Actual: HasDbConnection>(State(state_arc): State<Arc<S_Actual>>) -> Json<Vec<Balance>> {
    let db_arc = state_arc.get_db_connection();
    let balances = tokio::task::spawn_blocking(move || {
        let db = db_arc.blocking_lock();
        let mut stmt = db.prepare(
            "SELECT caip, pubkey, balance, price_usd, value_usd, symbol, network_id, last_updated, age FROM cached_balances"
        )?;
        let balances = stmt
            .query_map([], |row| {
                Ok(Balance {
                    caip: row.get(0)?,
                    pubkey: row.get(1)?,
                    balance: row.get(2)?,
                    price_usd: row.get(3).ok(),
                    value_usd: row.get(4).ok(),
                    symbol: row.get(5)?,
                    network_id: row.get(6)?,
                    last_updated: row.get(7).ok(),
                    age: row.get(8).ok(),
                })
            })?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        Ok::<_, rusqlite::Error>(balances)
    }).await.unwrap().unwrap_or_default();
    Json(balances)
}

#[utoipa::path(
    get,
    path = "/v2/portfolio/summary",
    responses(
        (status = 200, description = "Portfolio summary", body = PortfolioSummary)
    ),
    tag = "v2"
)]
pub async fn get_portfolio_summary<S_Actual: HasDbConnection>(State(state_arc): State<Arc<S_Actual>>) -> Json<PortfolioSummary> {
    let db_arc = state_arc.get_db_connection();
    let summary = tokio::task::spawn_blocking(move || {
        let db = db_arc.blocking_lock();
        let mut stmt = db.prepare(
            "SELECT total_value_usd, network_count, asset_count, last_updated FROM portfolio_summaries ORDER BY last_updated DESC LIMIT 1"
        )?;
        let summary = stmt
            .query_row([], |row| {
                Ok(PortfolioSummary {
                    total_value_usd: row.get(0).ok(),
                    network_count: row.get(1).unwrap_or(0),
                    asset_count: row.get(2).unwrap_or(0),
                    last_updated: row.get(3).ok(),
                })
            })
            .unwrap_or(PortfolioSummary {
                total_value_usd: None,
                network_count: 0,
                asset_count: 0,
                last_updated: None,
            });
        Ok::<_, rusqlite::Error>(summary)
    }).await.unwrap().unwrap_or(PortfolioSummary {
        total_value_usd: None,
        network_count: 0,
        asset_count: 0,
        last_updated: None,
    });
    Json(summary)
}

pub fn v2_router<S_Actual: HasDbConnection + Clone + Send + Sync + 'static>() -> Router<Arc<S_Actual>> {
    Router::new()
        .route("/networks", get(get_networks::<S_Actual>))
        .route("/balances", get(get_balances::<S_Actual>))
        .route("/portfolio/summary", get(get_portfolio_summary::<S_Actual>))
}
