use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Data Models matching the planning document specs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub id: String,                             // Primary key, e.g., "ETH", "BTC"
    pub display_name: String,                   // User-friendly name, e.g., "Ethereum"
    pub network_name_pioneer: String,           // Internal mapping name, e.g. "base", "ripple"
    pub chain_id_caip2: String,                 // CAIP-2 ID, e.g., "eip155:1"
    pub symbol: String,                         // Native currency symbol, e.g., "ETH"
    pub native_asset_decimals: u8,
    pub slip44: u32,
    pub default_derivation_path_str: Option<String>,
    pub is_evm: bool,
    pub is_testnet: bool,
    pub explorer_url_mainnet: Option<String>,
    pub explorer_address_url_mainnet: Option<String>,
    pub explorer_tx_url_mainnet: Option<String>,
    pub logo_uri: Option<String>,
    pub supported_asset_namespaces_caip19: Vec<String>, // e.g., ["erc20", "erc721"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id_caip19: String,                      // Primary key, CAIP-19 ID, e.g., "eip155:1/erc20:0x..."
    pub network_id_caip2: String,               // Foreign key to networks.chain_id_caip2
    pub asset_namespace_caip19: String,         // e.g., "erc20", "slip44"
    pub contract_address: Option<String>,       // Token contract address
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub description: Option<String>,
    pub logo_uri: Option<String>,
    pub source_list_name: Option<String>,       // e.g., "CoinGeckoList"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationPath {
    pub id: i64,                                // Auto-incrementing primary key
    pub network_id_caip2: String,               // Foreign key to networks.chain_id_caip2
    pub path_string: String,                    // e.g., "m/44'/60'/0'/0/0"
    pub description: Option<String>,
    pub path_type: Option<String>,              // e.g., "legacy", "segwit", "taproot", "account"
    pub purpose: Option<u32>,
}

// Query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct NetworkQuery {
    pub symbol: Option<String>,
    pub chain_id: Option<String>,
    pub is_evm: Option<bool>,
    pub is_testnet: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AssetQuery {
    pub network_id_caip2: Option<String>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub contract_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssetSearchQuery {
    pub query: String,
    pub network_id_caip2: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PathQuery {
    pub network_id_caip2: Option<String>,
    pub path_type: Option<String>,
}

// Temporary in-memory storage for development
// TODO: Replace with SQLite database implementation
lazy_static::lazy_static! {
    static ref NETWORKS: std::sync::Mutex<Vec<Network>> = std::sync::Mutex::new(get_initial_networks());
    static ref ASSETS: std::sync::Mutex<Vec<Asset>> = std::sync::Mutex::new(get_initial_assets());
    static ref PATHS: std::sync::Mutex<Vec<DerivationPath>> = std::sync::Mutex::new(get_initial_paths());
}

// Network endpoints
pub async fn list_networks(Query(params): Query<NetworkQuery>) -> Json<Vec<Network>> {
    let networks = NETWORKS.lock().unwrap();
    let mut filtered: Vec<Network> = networks.clone();
    
    // Apply filters
    if let Some(symbol) = params.symbol {
        filtered.retain(|n| n.symbol == symbol);
    }
    if let Some(chain_id) = params.chain_id {
        filtered.retain(|n| n.chain_id_caip2 == chain_id);
    }
    if let Some(is_evm) = params.is_evm {
        filtered.retain(|n| n.is_evm == is_evm);
    }
    if let Some(is_testnet) = params.is_testnet {
        filtered.retain(|n| n.is_testnet == is_testnet);
    }
    
    Json(filtered)
}

pub async fn get_network_by_caip2(Path(chain_id_caip2): Path<String>) -> Result<Json<Network>, StatusCode> {
    let networks = NETWORKS.lock().unwrap();
    
    if let Some(network) = networks.iter().find(|n| n.chain_id_caip2 == chain_id_caip2) {
        Ok(Json(network.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn get_network_by_symbol(Path(symbol): Path<String>) -> Result<Json<Network>, StatusCode> {
    let networks = NETWORKS.lock().unwrap();
    
    if let Some(network) = networks.iter().find(|n| n.symbol == symbol) {
        Ok(Json(network.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Asset endpoints
pub async fn list_assets(Query(params): Query<AssetQuery>) -> Json<Vec<Asset>> {
    let assets = ASSETS.lock().unwrap();
    let mut filtered: Vec<Asset> = assets.clone();
    
    // Apply filters
    if let Some(network_id) = params.network_id_caip2 {
        filtered.retain(|a| a.network_id_caip2 == network_id);
    }
    if let Some(symbol) = params.symbol {
        filtered.retain(|a| a.symbol == symbol);
    }
    if let Some(name) = params.name {
        filtered.retain(|a| a.name.to_lowercase().contains(&name.to_lowercase()));
    }
    if let Some(contract_address) = params.contract_address {
        filtered.retain(|a| a.contract_address.as_ref().map_or(false, |addr| *addr == contract_address));
    }
    
    Json(filtered)
}

pub async fn get_asset_by_caip19(Path(asset_id_caip19): Path<String>) -> Result<Json<Asset>, StatusCode> {
    let assets = ASSETS.lock().unwrap();
    
    if let Some(asset) = assets.iter().find(|a| a.id_caip19 == asset_id_caip19) {
        Ok(Json(asset.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn search_assets(Query(params): Query<AssetSearchQuery>) -> Json<Vec<Asset>> {
    let assets = ASSETS.lock().unwrap();
    let query_lower = params.query.to_lowercase();
    
    let mut filtered: Vec<Asset> = assets
        .iter()
        .filter(|a| {
            a.name.to_lowercase().contains(&query_lower) ||
            a.symbol.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect();
    
    // Further filter by network if specified
    if let Some(network_id) = params.network_id_caip2 {
        filtered.retain(|a| a.network_id_caip2 == network_id);
    }
    
    Json(filtered)
}

// Derivation path endpoints
pub async fn list_paths(Query(params): Query<PathQuery>) -> Json<Vec<DerivationPath>> {
    let paths = PATHS.lock().unwrap();
    let mut filtered: Vec<DerivationPath> = paths.clone();
    
    // Apply filters
    if let Some(network_id) = params.network_id_caip2 {
        filtered.retain(|p| p.network_id_caip2 == network_id);
    }
    if let Some(path_type) = params.path_type {
        filtered.retain(|p| p.path_type.as_ref().map_or(false, |pt| *pt == path_type));
    }
    
    Json(filtered)
}

// Initialize with sample data from Pioneer SDK
fn get_initial_networks() -> Vec<Network> {
    vec![
        Network {
            id: "ETH".to_string(),
            display_name: "Ethereum".to_string(),
            network_name_pioneer: "ethereum".to_string(),
            chain_id_caip2: "eip155:1".to_string(),
            symbol: "ETH".to_string(),
            native_asset_decimals: 18,
            slip44: 60,
            default_derivation_path_str: Some("m/44'/60'/0'/0/0".to_string()),
            is_evm: true,
            is_testnet: false,
            explorer_url_mainnet: Some("https://etherscan.io".to_string()),
            explorer_address_url_mainnet: Some("https://etherscan.io/address/{}".to_string()),
            explorer_tx_url_mainnet: Some("https://etherscan.io/tx/{}".to_string()),
            logo_uri: None,
            supported_asset_namespaces_caip19: vec!["erc20".to_string(), "erc721".to_string()],
        },
        Network {
            id: "BTC".to_string(),
            display_name: "Bitcoin".to_string(),
            network_name_pioneer: "bitcoin".to_string(),
            chain_id_caip2: "bip122:000000000019d6689c085ae165831e93".to_string(),
            symbol: "BTC".to_string(),
            native_asset_decimals: 8,
            slip44: 0,
            default_derivation_path_str: Some("m/84'/0'/0'/0/0".to_string()),
            is_evm: false,
            is_testnet: false,
            explorer_url_mainnet: Some("https://blockstream.info".to_string()),
            explorer_address_url_mainnet: Some("https://blockstream.info/address/{}".to_string()),
            explorer_tx_url_mainnet: Some("https://blockstream.info/tx/{}".to_string()),
            logo_uri: None,
            supported_asset_namespaces_caip19: vec!["slip44".to_string()],
        },
        Network {
            id: "ARB".to_string(),
            display_name: "Arbitrum".to_string(),
            network_name_pioneer: "arbitrum".to_string(),
            chain_id_caip2: "eip155:42161".to_string(),
            symbol: "ETH".to_string(),
            native_asset_decimals: 18,
            slip44: 60,
            default_derivation_path_str: Some("m/44'/60'/0'/0/0".to_string()),
            is_evm: true,
            is_testnet: false,
            explorer_url_mainnet: Some("https://arbiscan.io".to_string()),
            explorer_address_url_mainnet: Some("https://arbiscan.io/address/{}".to_string()),
            explorer_tx_url_mainnet: Some("https://arbiscan.io/tx/{}".to_string()),
            logo_uri: None,
            supported_asset_namespaces_caip19: vec!["erc20".to_string(), "erc721".to_string()],
        },
    ]
}

fn get_initial_assets() -> Vec<Asset> {
    vec![
        Asset {
            id_caip19: "eip155:1/slip44:60".to_string(),
            network_id_caip2: "eip155:1".to_string(),
            asset_namespace_caip19: "slip44".to_string(),
            contract_address: None,
            name: "Ethereum".to_string(),
            symbol: "ETH".to_string(),
            decimals: 18,
            description: Some("Native Ethereum token".to_string()),
            logo_uri: None,
            source_list_name: Some("Pioneer".to_string()),
        },
        Asset {
            id_caip19: "eip155:1/erc20:0xA0b86a33E6441F8C8B18B8F0b862C62b16C7DC4C".to_string(),
            network_id_caip2: "eip155:1".to_string(),
            asset_namespace_caip19: "erc20".to_string(),
            contract_address: Some("0xA0b86a33E6441F8C8B18B8F0b862C62b16C7DC4C".to_string()),
            name: "USD Coin".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            description: Some("USD Coin stablecoin".to_string()),
            logo_uri: None,
            source_list_name: Some("Pioneer".to_string()),
        },
        Asset {
            id_caip19: "bip122:000000000019d6689c085ae165831e93/slip44:0".to_string(),
            network_id_caip2: "bip122:000000000019d6689c085ae165831e93".to_string(),
            asset_namespace_caip19: "slip44".to_string(),
            contract_address: None,
            name: "Bitcoin".to_string(),
            symbol: "BTC".to_string(),
            decimals: 8,
            description: Some("Native Bitcoin".to_string()),
            logo_uri: None,
            source_list_name: Some("Pioneer".to_string()),
        },
    ]
}

fn get_initial_paths() -> Vec<DerivationPath> {
    vec![
        DerivationPath {
            id: 1,
            network_id_caip2: "eip155:1".to_string(),
            path_string: "m/44'/60'/0'/0/0".to_string(),
            description: Some("Ethereum default path".to_string()),
            path_type: Some("default".to_string()),
            purpose: Some(44),
        },
        DerivationPath {
            id: 2,
            network_id_caip2: "bip122:000000000019d6689c085ae165831e93".to_string(),
            path_string: "m/84'/0'/0'/0/0".to_string(),
            description: Some("Bitcoin SegWit native".to_string()),
            path_type: Some("segwit".to_string()),
            purpose: Some(84),
        },
        DerivationPath {
            id: 3,
            network_id_caip2: "bip122:000000000019d6689c085ae165831e93".to_string(),
            path_string: "m/44'/0'/0'/0/0".to_string(),
            description: Some("Bitcoin legacy".to_string()),
            path_type: Some("legacy".to_string()),
            purpose: Some(44),
        },
    ]
}

// Router setup - these routes don't need device state since they serve static metadata
pub fn pioneer_routes() -> Router {
    Router::new()
        // Network routes
        .route("/networks", get(list_networks))
        .route("/networks/:chain_id_caip2", get(get_network_by_caip2))
        .route("/networks/symbol/:symbol", get(get_network_by_symbol))
        // Asset routes
        .route("/assets", get(list_assets))
        .route("/assets/:asset_id_caip19", get(get_asset_by_caip19))
        .route("/assets/search", get(search_assets))
        // Derivation path routes
        .route("/paths", get(list_paths))
} 