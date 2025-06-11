use axum::{
    extract::{State, Path as AxumPath, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
// Keep Router import separately to avoid unused warning when compiling without v2_router use in this module
#[allow(unused_imports)]
use axum::Router;
use crate::cache::device_cache::{DeviceCache, Network, Path, CachedBalance, PortfolioSummary};
use crate::server::AppState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, error, debug, warn};
use anyhow::Result;

// Import try_get_device directly from the server module (for future use)
// use crate::server::try_get_device;

/// Get networks endpoint
#[utoipa::path(
    get,
    context_path = "/v2",
    path = "/networks",
    tag = "Networks",
    responses(
        (status = 200, body = Vec<Network>, description = "List of networks"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn get_networks(State(app_state): State<Arc<AppState>>) -> Json<Vec<Network>> {
    let cache = &app_state.device_cache;
    let mut networks = match cache.get_enabled_networks().await {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to get networks: {}", e);
            vec![]
        },
    };
    // Special logic: If there are any eip155 networks, replace with eip155:*
    let has_evm = networks.iter().any(|n| n.chain_id_caip2.starts_with("eip155:"));
    if has_evm {
        networks.retain(|n| !n.chain_id_caip2.starts_with("eip155:"));
        networks.push(Network {
            id: 0,
            chain_id_caip2: "eip155:*".to_string(),
            display_name: "All EVM Chains".to_string(),
            network_name: "evm".to_string(),
            symbol: "EVM".to_string(),
            is_evm: true,
            is_testnet: false,
            enabled: true,
        });
    }
    Json(networks)
}

/// Input model for network creation - doesn't require ID field
#[derive(Debug, Deserialize, ToSchema)]
pub struct NetworkInput {
    pub chain_id_caip2: String,
    pub display_name: String,
    pub network_name: String,
    pub symbol: String,
    pub is_evm: bool,
    pub is_testnet: bool,
    pub enabled: bool,
}

/// Add a new network via POST request
#[utoipa::path(
    post,
    context_path = "/v2",
    path = "/networks",
    tag = "Networks",
    request_body = NetworkInput,
    responses(
        (status = 200, body = String, description = "Network added successfully"),
        (status = 400, body = String, description = "Missing required field"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn post_network(
    State(app_state): State<Arc<AppState>>,
    Json(network_input): Json<NetworkInput>,
) -> impl IntoResponse {
    let cache = &app_state.device_cache;
    // Validate required fields
    if network_input.chain_id_caip2.trim().is_empty() {
        error!("Missing required field: chain_id_caip2");
        return (StatusCode::BAD_REQUEST, "Missing required field: chain_id_caip2").into_response();
    }
    if network_input.display_name.trim().is_empty() {
        error!("Missing required field: display_name");
        return (StatusCode::BAD_REQUEST, "Missing required field: display_name").into_response();
    }
    if network_input.network_name.trim().is_empty() {
        error!("Missing required field: network_name");
        return (StatusCode::BAD_REQUEST, "Missing required field: network_name").into_response();
    }
    if network_input.symbol.trim().is_empty() {
        error!("Missing required field: symbol");
        return (StatusCode::BAD_REQUEST, "Missing required field: symbol").into_response();
    }

    // Convert the input to a Network object
    // The ID will be determined by the database
    let network = Network {
        id: 0, // This will be replaced by the DB
        chain_id_caip2: network_input.chain_id_caip2,
        display_name: network_input.display_name,
        network_name: network_input.network_name,
        symbol: network_input.symbol,
        is_evm: network_input.is_evm,
        is_testnet: network_input.is_testnet,
        enabled: network_input.enabled,
    };

    // Try to add the network to the database
    match cache.add_network(&network).await {
        Ok(id) => {
            info!("Added/updated network '{}' with ID {}", network.display_name, id);
            (StatusCode::OK, format!("{{ \"id\": {} }}", id)).into_response()
        }
        Err(e) => {
            error!("Failed to add network: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to add network: {}", e)).into_response()
        }
    }
}

// Path API endpoints
#[utoipa::path(
    get,
    context_path = "/v2",
    path = "/paths",
    tag = "Paths",
    responses(
        (status = 200, body = Vec<Path>, description = "List of paths"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn get_paths(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let cache = &app_state.device_cache;
    match cache.get_paths().await {
        Ok(paths) => (StatusCode::OK, Json(paths)).into_response(),
        Err(e) => {
            error!("Failed to get paths: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get paths: {}", e)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    context_path = "/v2",
    path = "/paths/{id}",
    tag = "Paths",
    params(
        ("id" = i64, Path, description = "Path ID"),
    ),
    responses(
        (status = 200, body = Path, description = "Path details"),
        (status = 404, body = String, description = "Path not found"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn get_path(State(app_state): State<Arc<AppState>>, AxumPath(id): AxumPath<i64>) -> impl IntoResponse {
    let cache = &app_state.device_cache;
    match cache.get_path(id).await {
        Ok(Some(path)) => (StatusCode::OK, Json(path)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, format!("Path with ID {} not found", id)).into_response(),
        Err(e) => {
            error!("Failed to get path {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get path: {}", e)).into_response()
        }
    }
}

#[utoipa::path(
    post,
    context_path = "/v2",
    path = "/paths",
    tag = "Paths",
    request_body = Path,
    responses(
        (status = 201, body = String, description = "Path added successfully"),
        (status = 400, body = String, description = "Missing required field"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn post_path(State(app_state): State<Arc<AppState>>, Json(path): Json<Path>) -> impl IntoResponse {
    let cache = &app_state.device_cache;
    // Validate required fields
    if path.note.trim().is_empty() {
        error!("Missing required field: note");
        return (StatusCode::BAD_REQUEST, "Missing required field: note").into_response();
    }
    if path.script_type.trim().is_empty() {
        error!("Missing required field: script_type");
        return (StatusCode::BAD_REQUEST, "Missing required field: script_type").into_response();
    }
    if path.path_type.trim().is_empty() {
        error!("Missing required field: type");
        return (StatusCode::BAD_REQUEST, "Missing required field: type").into_response();
    }
    if path.networks.is_empty() {
        error!("Missing required field: networks");
        return (StatusCode::BAD_REQUEST, "Missing required field: networks").into_response();
    }
    if path.address_n_list.is_empty() {
        error!("Missing required field: addressNList");
        return (StatusCode::BAD_REQUEST, "Missing required field: addressNList").into_response();
    }
    if path.address_n_list_master.is_empty() {
        error!("Missing required field: addressNListMaster");
        return (StatusCode::BAD_REQUEST, "Missing required field: addressNListMaster").into_response();
    }

    match cache.add_path(&path).await {
        Ok(id) => {
            info!("Added path '{}' with ID {}", path.note, id);
            (StatusCode::CREATED, format!("{{ \"id\": {} }}", id)).into_response()
        }
        Err(e) => {
            error!("Failed to add path: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to add path: {}", e)).into_response()
        }
    }
}

#[utoipa::path(
    put,
    context_path = "/v2",
    path = "/paths/{id}",
    tag = "Paths",
    request_body = Path,
    params(
        ("id" = i64, Path, description = "Path ID"),
    ),
    responses(
        (status = 200, body = String, description = "Path updated successfully"),
        (status = 400, body = String, description = "Missing required field"),
        (status = 404, body = String, description = "Path not found"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn put_path(State(app_state): State<Arc<AppState>>, AxumPath(id): AxumPath<i64>, Json(path): Json<Path>) -> impl IntoResponse {
    let cache = &app_state.device_cache;
    // Validate required fields (same as post)
    if path.note.trim().is_empty() {
        error!("Missing required field: note");
        return (StatusCode::BAD_REQUEST, "Missing required field: note").into_response();
    }
    if path.script_type.trim().is_empty() {
        error!("Missing required field: script_type");
        return (StatusCode::BAD_REQUEST, "Missing required field: script_type").into_response();
    }
    if path.path_type.trim().is_empty() {
        error!("Missing required field: type");
        return (StatusCode::BAD_REQUEST, "Missing required field: type").into_response();
    }
    if path.networks.is_empty() {
        error!("Missing required field: networks");
        return (StatusCode::BAD_REQUEST, "Missing required field: networks").into_response();
    }
    if path.address_n_list.is_empty() {
        error!("Missing required field: addressNList");
        return (StatusCode::BAD_REQUEST, "Missing required field: addressNList").into_response();
    }
    if path.address_n_list_master.is_empty() {
        error!("Missing required field: addressNListMaster");
        return (StatusCode::BAD_REQUEST, "Missing required field: addressNListMaster").into_response();
    }

    match cache.update_path(id, &path).await {
        Ok(_) => {
            info!("Updated path '{}' with ID {}", path.note, id);
            (StatusCode::OK, format!("Path with ID {} updated", id)).into_response()
        }
        Err(e) => {
            error!("Failed to update path: {}", e);
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, format!("Path with ID {} not found", id)).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update path: {}", e)).into_response()
            }
        }
    }
}

#[utoipa::path(
    delete,
    context_path = "/v2",
    path = "/paths/{id}",
    tag = "Paths",
    params(
        ("id" = i64, Path, description = "Path ID"),
    ),
    responses(
        (status = 200, body = String, description = "Path deleted successfully"),
        (status = 404, body = String, description = "Path not found"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn delete_path(State(app_state): State<Arc<AppState>>, AxumPath(id): AxumPath<i64>) -> impl IntoResponse {
    let cache = &app_state.device_cache;
    match cache.delete_path(id).await {
        Ok(_) => {
            info!("Deleted path with ID {}", id);
            (StatusCode::OK, format!("Path with ID {} deleted", id)).into_response()
        }
        Err(e) => {
            error!("Failed to delete path: {}", e);
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, format!("Path with ID {} not found", id)).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete path: {}", e)).into_response()
            }
        }
    }
}

/// Query parameters for filtering pubkeys by network
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetPubkeysQuery {
    pub network: Option<String>,
}

/// PubkeyResponse represents a structured public key response for a specific network/path combination
#[derive(Debug, Serialize, ToSchema)]
pub struct PubkeyResponse {
    #[serde(rename = "type")]
    pub key_type: String,
    pub master: Option<String>,
    pub address: String,
    pub pubkey: String,
    pub path: String,
    pub pathMaster: String,
    pub scriptType: String,
    pub note: String,
    pub available_scripts_types: Option<Vec<String>>,
    pub networks: Vec<String>,
    pub context: Option<String>,
}

/// Get device pubkeys - ONLY REAL DATA, NO MOCKING, NO FALLBACKS
#[utoipa::path(
    get,
    context_path = "/v2",
    path = "/pubkeys",
    tag = "Pubkeys",
    params(
        ("network" = String, Query, description = "Filter by network"),
    ),
    responses(
        (status = 200, body = Vec<PubkeyResponse>, description = "List of pubkeys"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn get_pubkeys(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<GetPubkeysQuery>,
) -> impl IntoResponse {
    let tag = "get_pubkeys";
    debug!("{}: Getting pubkeys with params: {:?}", tag, params);
    
    // Get actual device ID from cache - FAIL FAST if no device
    let device_id = match app_state.device_cache.get_device_id() {
        Some(id) => id,
        None => {
            error!("{}: No device found in cache", tag);
            return Json::<Vec<PubkeyResponse>>(vec![]).into_response();
        }
    };
    
    // Get paths from database, filtered by network if specified
    let paths = match app_state.device_cache.get_paths().await {
        Ok(all_paths) => {
            if let Some(network_filter) = &params.network {
                all_paths.into_iter()
                    .filter(|path| path.networks.iter().any(|net| net == network_filter))
                    .collect()
            } else {
                all_paths
            }
        }
        Err(e) => {
            error!("{}: Failed to get paths: {}", tag, e);
            return Json::<Vec<PubkeyResponse>>(vec![]).into_response();
        }
    };

    debug!("{}: Found {} paths for query", tag, paths.len());
    
    let mut pubkey_responses = Vec::new();
    
    // Process each path and ONLY return real cached addresses
    for path in paths {
        for network in &path.networks {
            // Determine coin name and script type from network and path
            let (coin_name, script_type) = match get_coin_info_from_network(network, &path.script_type) {
                Ok(info) => info,
                Err(e) => {
                    warn!("{}: Skipping unsupported network {}: {}", tag, network, e);
                    continue;
                }
            };
            
            // Use the addressNListMaster directly - it's already a complete address path
            let address_path = &path.address_n_list_master;
            
            // Check if this address is cached - ONLY USE REAL DATA
            if let Some(cached_addr) = app_state.device_cache.get_cached_address(&coin_name, &script_type, address_path) {
                // Format BIP32 path strings
                let path_str = format_bip32_path(&path.address_n_list);
                let path_master_str = format_bip32_path(address_path);
                
                let pubkey_response = PubkeyResponse {
                    key_type: path.path_type.clone(),
                    master: None,
                    address: cached_addr.address.clone(),
                    pubkey: cached_addr.pubkey.unwrap_or_else(|| cached_addr.address.clone()),
                    path: path_str,
                    pathMaster: path_master_str,
                    scriptType: script_type.clone(),
                    note: path.note.clone(),
                    available_scripts_types: path.available_script_types.clone(),
                    networks: vec![network.clone()],
                    context: Some(format!("Real cached address for {}", network)),
                };
                
                pubkey_responses.push(pubkey_response);
                debug!("{}: Added real cached address for {} {}", tag, coin_name, script_type);
            } else {
                // NO FALLBACK, NO MOCK DATA - just log the missing address
                debug!("{}: No cached address found for {} {} at path {:?} - skipping", tag, coin_name, script_type, address_path);
            }
        }
    }
    
    info!("{}: Returning {} REAL pubkey responses (no mock data)", tag, pubkey_responses.len());
    Json(pubkey_responses).into_response()
}

/// Get coin and script type info from network identifier
fn get_coin_info_from_network(network: &str, script_type: &str) -> Result<(String, String), anyhow::Error> {
    let coin_name = match network {
        // Bitcoin networks
        n if n.starts_with("bip122:000000000019d6689c085ae165831e93") => "Bitcoin",
        n if n.starts_with("bip122:000000000000000000651ef99cb9fcbe") => "BitcoinCash",
        n if n.starts_with("bip122:000000000933ea01ad0ee984209779ba") => "Testnet", // Bitcoin testnet

        // UTXO altcoins
        n if n.starts_with("bip122:12a765e31ffd4059bada1e25190f6e98") => "Litecoin",
        n if n.starts_with("bip122:000007d91d1254d60e2dd1ae58038307") => "Dash",
        n if n.starts_with("bip122:00000000001a91e3dace36e2be3bf030") => "Dogecoin",
        n if n.starts_with("bip122:7497ea1b465eb39f1c8f507bc877078f") => "DigiByte",
        n if n.starts_with("bip122:0000000000196a45") => "Zcash",
        n if n.starts_with("bip122:027e3758c3a65b12aa1046462b486d0a") => "Komodo",

        // Account-based networks
        n if n.starts_with("eip155:") => "Ethereum",
        n if n.starts_with("cosmos:") => "Cosmos",
        n if n.starts_with("thorchain:") => "Thorchain",
        n if n.starts_with("mayachain:") => "Mayachain",
        n if n.starts_with("ripple:") => "Ripple",

        _ => return Err(anyhow::anyhow!("Unknown network: {}", network)),
    };

    Ok((coin_name.to_string(), script_type.to_string()))
}

/// Helper function to format address_n_list as BIP32 path string
fn format_bip32_path(address_n_list: &[u32]) -> String {
    let parts: Vec<String> = address_n_list.iter()
        .map(|&n| {
            if n >= 0x80000000 {
                format!("{}′", (n - 0x80000000))
            } else {
                n.to_string()
            }
        })
        .collect();

    format!("m/{}", parts.join("/"))
}

/// Query parameters for filtering balances
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetBalancesQuery {
    pub network: Option<String>,
    pub force_refresh: Option<bool>,
}

/// Balance response structure
#[derive(Debug, Serialize, ToSchema)]
pub struct BalanceResponse {
    pub caip: String,
    pub pubkey: String,
    pub balance: String,
    pub price_usd: String,
    pub value_usd: String,
    pub symbol: Option<String>,
    pub network_id: Option<String>,
    pub last_updated: i64,
    pub age: String, // Human readable age like "2 minutes ago"
}

/// Portfolio balance request (for POST endpoint)
#[derive(Debug, Deserialize, ToSchema)]
pub struct PortfolioBalanceRequest {
    pub caip: String,
    pub pubkey: String,
}

/// Get cached balances with optional refresh if > 10 minutes old
#[utoipa::path(
    get,
    context_path = "/v2",
    path = "/balances",
    tag = "Balances",
    params(
        ("network" = String, Query, description = "Filter by network"),
        ("force_refresh" = bool, Query, description = "Force refresh of balances"),
    ),
    responses(
        (status = 200, body = Vec<BalanceResponse>, description = "List of balances"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn get_balances(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<GetBalancesQuery>,
) -> impl IntoResponse {
    let tag = "get_balances";
    debug!("{}: Getting balances with params: {:?}", tag, params);
    
    // Get actual device ID from cache - FAIL FAST if no device
    let device_id = match app_state.device_cache.get_device_id() {
        Some(id) => id,
        None => {
            error!("{}: No device found in cache", tag);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "No device available - server requires connected KeepKey device"
            }))).into_response();
        }
    };
    
    // Check if balances need refresh or force refresh is requested
    let force_refresh = params.force_refresh.unwrap_or(false);
    let needs_refresh = match app_state.device_cache.balances_need_refresh(&device_id).await {
        Ok(needs) => needs || force_refresh,
        Err(e) => {
            error!("{}: Error checking refresh status: {}", tag, e);
            true // Default to refresh on error
        }
    };
    
    if needs_refresh {
        info!("{}: Balances need refresh - fetching from Pioneer API", tag);
        if let Err(e) = refresh_balances_from_pioneer(&*app_state.device_cache, &device_id).await {
            error!("{}: Failed to refresh balances - FAIL FAST: {}", tag, e);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": format!("Failed to refresh balances: {}", e)
            }))).into_response();
        }
    }
    
    // Get cached balances
    let balances = match app_state.device_cache.get_cached_balances(&device_id).await {
        Ok(balances) => balances,
        Err(e) => {
            error!("{}: Failed to get cached balances: {}", tag, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to get balances"
            }))).into_response();
        }
    };
    
    // Filter by network if specified
    let filtered_balances: Vec<BalanceResponse> = balances.into_iter()
        .filter(|balance| {
            if let Some(network_filter) = &params.network {
                balance.network_id.as_ref().map_or(false, |net| net == network_filter) ||
                balance.caip.contains(network_filter)
            } else {
                true
            }
        })
        .map(|balance| BalanceResponse {
            caip: balance.caip,
            pubkey: balance.pubkey,
            balance: balance.balance,
            price_usd: balance.price_usd,
            value_usd: balance.value_usd,
            symbol: balance.symbol,
            network_id: balance.network_id,
            last_updated: balance.last_updated,
            age: format_age(balance.last_updated),
        })
        .collect();
    
    info!("{}: Returning {} balances", tag, filtered_balances.len());
    Json(filtered_balances).into_response()
}

/// Get portfolio balances for specific caip/pubkey pairs
#[utoipa::path(
    post,
    context_path = "/v2",
    path = "/portfolio",
    tag = "Portfolio",
    request_body = Vec<PortfolioBalanceRequest>,
    responses(
        (status = 200, body = Vec<BalanceResponse>, description = "List of portfolio balances"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn post_portfolio_balances(
    State(app_state): State<Arc<AppState>>,
    Json(requests): Json<Vec<PortfolioBalanceRequest>>,
) -> impl IntoResponse {
    let tag = "post_portfolio_balances";
    debug!("{}: Getting portfolio balances for {} requests", tag, requests.len());
    
    // Get actual device ID from cache - FAIL FAST if no device
    let device_id = match app_state.device_cache.get_device_id() {
        Some(id) => id,
        None => {
            error!("{}: No device found in cache", tag);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "No device available - server requires connected KeepKey device"
            }))).into_response();
        }
    };
    
    // Check if balances need refresh
    let needs_refresh = match app_state.device_cache.balances_need_refresh(&device_id).await {
        Ok(needs) => needs,
        Err(e) => {
            error!("{}: Error checking refresh status: {}", tag, e);
            true
        }
    };
    
    if needs_refresh {
        info!("{}: Balances need refresh - fetching from Pioneer API", tag);
        if let Err(e) = refresh_balances_from_pioneer(&*app_state.device_cache, &device_id).await {
            error!("{}: Failed to refresh balances - FAIL FAST: {}", tag, e);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": format!("Failed to refresh balances: {}", e)
            }))).into_response();
        }
    }
    
    // Get all cached balances
    let cached_balances = match app_state.device_cache.get_cached_balances(&device_id).await {
        Ok(balances) => balances,
        Err(e) => {
            error!("{}: Failed to get cached balances: {}", tag, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to get balances"
            }))).into_response();
        }
    };
    
    // Create lookup map for efficient searching
    let balance_map: HashMap<(String, String), &CachedBalance> = cached_balances.iter()
        .map(|balance| ((balance.caip.clone(), balance.pubkey.clone()), balance))
        .collect();
    
    // Build response for each request
    let mut responses = Vec::new();
    for request in requests {
        if let Some(cached_balance) = balance_map.get(&(request.caip.clone(), request.pubkey.clone())) {
            responses.push(BalanceResponse {
                caip: cached_balance.caip.clone(),
                pubkey: cached_balance.pubkey.clone(),
                balance: cached_balance.balance.clone(),
                price_usd: cached_balance.price_usd.clone(),
                value_usd: cached_balance.value_usd.clone(),
                symbol: cached_balance.symbol.clone(),
                network_id: cached_balance.network_id.clone(),
                last_updated: cached_balance.last_updated,
                age: format_age(cached_balance.last_updated),
            });
        } else {
            // Return zero balance for missing data (no fallbacks)
            responses.push(BalanceResponse {
                caip: request.caip,
                pubkey: request.pubkey,
                balance: "0.00000000".to_string(),
                price_usd: "0.00".to_string(),
                value_usd: "0.00".to_string(),
                symbol: None,
                network_id: None,
                last_updated: chrono::Utc::now().timestamp(),
                age: format_age(chrono::Utc::now().timestamp()),
            });
        }
    }
    
    info!("{}: Returning {} portfolio balance responses", tag, responses.len());
    Json(responses).into_response()
}

/// Get portfolio summary (total values, counts)
#[utoipa::path(
    get,
    context_path = "/v2",
    path = "/portfolio/summary",
    tag = "Portfolio",
    responses(
        (status = 200, body = PortfolioSummary, description = "Portfolio summary"),
        (status = 500, body = String, description = "Internal server error"),
    ),
)]
pub async fn get_portfolio_summary(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let tag = "get_portfolio_summary";
    
    // Get actual device ID from cache - FAIL FAST if no device
    let device_id = match app_state.device_cache.get_device_id() {
        Some(id) => id,
        None => {
            error!("{}: No device found in cache", tag);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "No device available - server requires connected KeepKey device"
            }))).into_response();
        }
    };
    
    match app_state.device_cache.get_portfolio_summary(&device_id).await {
        Ok(Some(summary)) => Json(summary).into_response(),
        Ok(None) => {
            // Generate summary from current balances
            match app_state.device_cache.get_cached_balances(&device_id).await {
                Ok(balances) => {
                    let mut total_value_usd = 0.0;
                    let mut networks = std::collections::HashSet::new();
                    
                    for balance in &balances {
                        if let Ok(value) = balance.value_usd.parse::<f64>() {
                            total_value_usd += value;
                        }
                        if let Some(network_id) = &balance.network_id {
                            networks.insert(network_id.clone());
                        }
                    }
                    
                    let summary = PortfolioSummary {
                        id: 0,
                        device_id: device_id.to_string(),
                        total_value_usd: format!("{:.2}", total_value_usd),
                        network_count: networks.len() as i64,
                        asset_count: balances.len() as i64,
                        last_updated: chrono::Utc::now().timestamp(),
                    };
                    
                    // Save the summary
                    if let Err(e) = app_state.device_cache.save_portfolio_summary(&device_id, &summary).await {
                        warn!("{}: Failed to save portfolio summary: {}", tag, e);
                    }
                    
                    Json(summary).into_response()
                }
                Err(e) => {
                    error!("{}: Failed to get balances for summary: {}", tag, e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": "Failed to generate portfolio summary"
                    }))).into_response()
                }
            }
        }
        Err(e) => {
            error!("{}: Failed to get portfolio summary: {}", tag, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to get portfolio summary"
            }))).into_response()
        }
    }
}

/// Fetch balances from Pioneer API and cache them
async fn refresh_balances_from_pioneer(cache: &DeviceCache, device_id: &str) -> Result<()> {
    let tag = "refresh_balances_from_pioneer";
    
    // Get Pioneer server URL from config
    let pioneer_url = cache.get_pioneer_server_url().await?;
    info!("{}: Using Pioneer server URL: {}", tag, pioneer_url);
    
    // Get all pubkeys for building asset query
    let paths = cache.get_paths().await?;
    let mut asset_queries = Vec::new();
    
    for path in paths {
        for network in &path.networks {
            // Get CAIP for this network
            let caip = match network_to_caip(network) {
                Ok(caip) => caip,
                Err(e) => {
                    debug!("{}: Skipping unsupported network {}: {}", tag, network, e);
                    continue;
                }
            };
            
            // Find cached address for this path
            let (coin_name, script_type) = match get_coin_info_from_network(network, &path.script_type) {
                Ok(info) => info,
                Err(e) => {
                    debug!("{}: Skipping unsupported network {}: {}", tag, network, e);
                    continue;
                }
            };
            
            if let Some(cached_addr) = cache.get_cached_address(&coin_name, &script_type, &path.address_n_list_master) {
                let pubkey = cached_addr.pubkey.unwrap_or_else(|| cached_addr.address.clone());
                info!("{}: Adding asset query: caip={}, pubkey={}, address={}", tag, caip, pubkey, cached_addr.address);
                asset_queries.push(serde_json::json!({
                    "caip": caip,
                    "pubkey": pubkey
                }));
            } else {
                warn!("{}: No cached address found for {} {} at path {:?}", tag, coin_name, script_type, path.address_n_list_master);
            }
        }
    }
    
    if asset_queries.is_empty() {
        error!("{}: No asset queries to process - no cached addresses found!", tag);
        return Err(anyhow::anyhow!("No cached addresses available for balance lookup"));
    }
    
    info!("{}: Fetching balances for {} assets from {}", tag, asset_queries.len(), pioneer_url);
    
    // Build the full URL
    let portfolio_url = format!("{}/api/v1/portfolio", pioneer_url);
    info!("{}: Making POST request to: {}", tag, portfolio_url);
    
    // Log the request payload
    let payload = serde_json::to_string_pretty(&asset_queries)?;
    info!("{}: Request payload:\n{}", tag, payload);
    
    // Make HTTP request to Pioneer API
    let client = reqwest::Client::new();
    let response = client
        .post(&portfolio_url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "kkcli/1.0")
        .json(&asset_queries)
        .send()
        .await?;
    
    let status = response.status();
    let headers = response.headers().clone();
    info!("{}: Response status: {}", tag, status);
    info!("{}: Response headers: {:?}", tag, headers);
    
    if !status.is_success() {
        // Try to get response body for debugging
        let body = response.text().await.unwrap_or_else(|_| "Could not read response body".to_string());
        error!("{}: Pioneer API error response body: {}", tag, body);
        return Err(anyhow::anyhow!("Pioneer API returned status: {} {}", status, body));
    }
    
    let response_text = response.text().await?;
    info!("{}: Raw response body:\n{}", tag, response_text);
    
    // Parse the response
    let balance_responses: Vec<serde_json::Value> = serde_json::from_str(&response_text)?;
    info!("{}: Parsed {} balance responses from Pioneer API", tag, balance_responses.len());
    
    // Convert to CachedBalance objects
    let mut cached_balances = Vec::new();
    for balance_data in balance_responses {
        info!("{}: Processing balance entry: {}", tag, serde_json::to_string_pretty(&balance_data)?);
        
        // Try different field name variations for balance/price/value
        let balance_str = balance_data.get("balance")
            .or_else(|| balance_data.get("balanceUsd"))
            .and_then(|v| v.as_str());
            
        let price_str = balance_data.get("priceUsd")
            .or_else(|| balance_data.get("price"))
            .and_then(|v| v.as_str());
            
        let value_str = balance_data.get("valueUsd")
            .or_else(|| balance_data.get("value"))
            .and_then(|v| v.as_str());
        
        if let (Some(caip), Some(pubkey)) = (
            balance_data.get("caip").and_then(|v| v.as_str()),
            balance_data.get("pubkey").and_then(|v| v.as_str()),
        ) {
            let balance = balance_str.unwrap_or("0.00000000").to_string();
            let price_usd = price_str.unwrap_or("0.00").to_string();
            let value_usd = value_str.unwrap_or("0.00").to_string();
            
            let network_id = caip_to_network_id(caip);
            
            let cached_balance = CachedBalance {
                id: 0, // Will be set by database
                device_id: device_id.to_string(),
                caip: caip.to_string(),
                pubkey: pubkey.to_string(),
                balance,
                price_usd: price_usd.clone(),
                value_usd: value_usd.clone(),
                symbol: extract_symbol_from_caip(caip),
                network_id: Some(network_id),
                last_updated: chrono::Utc::now().timestamp(),
            };
            
            info!("{}: Created cached balance: caip={}, balance={}, price_usd={}, value_usd={}", 
                tag, caip, cached_balance.balance, price_usd, value_usd);
            cached_balances.push(cached_balance);
        } else {
            warn!("{}: Skipping balance entry with missing caip/pubkey: {}", tag, 
                serde_json::to_string(&balance_data)?);
        }
    }
    
    if cached_balances.is_empty() {
        warn!("{}: No valid balances received from Pioneer API", tag);
        return Ok(());
    }
    
    // Save to cache
    cache.save_balances(device_id, &cached_balances).await?;
    
    // Clean up old balances
    cache.clear_old_balances(device_id).await?;
    
    info!("{}: Successfully cached {} balances", tag, cached_balances.len());
    Ok(())
}

/// Convert network identifier to CAIP format
fn network_to_caip(network: &str) -> Result<String> {
    match network {
        // Bitcoin networks
        n if n.starts_with("bip122:000000000019d6689c085ae165831e93") => Ok("bip122:000000000019d6689c085ae165831e93/slip44:0".to_string()),
        n if n.starts_with("bip122:000000000000000000651ef99cb9fcbe") => Ok("bip122:000000000000000000651ef99cb9fcbe/slip44:145".to_string()), // Bitcoin Cash
        n if n.starts_with("bip122:12a765e31ffd4059bada1e25190f6e98") => Ok("bip122:12a765e31ffd4059bada1e25190f6e98/slip44:2".to_string()), // Litecoin
        
        // Ethereum networks  
        n if n.starts_with("eip155:1") => Ok("eip155:1/slip44:60".to_string()), // Ethereum mainnet
        n if n.starts_with("eip155:10") => Ok("eip155:10/slip44:60".to_string()), // Optimism
        n if n.starts_with("eip155:137") => Ok("eip155:137/slip44:60".to_string()), // Polygon
        n if n.starts_with("eip155:42161") => Ok("eip155:42161/slip44:60".to_string()), // Arbitrum
        n if n.starts_with("eip155:8453") => Ok("eip155:8453/slip44:60".to_string()), // Base
        n if n.starts_with("eip155:56") => Ok("eip155:56/slip44:60".to_string()), // BSC
        n if n.starts_with("eip155:43114") => Ok("eip155:43114/slip44:60".to_string()), // Avalanche
        n if n.starts_with("eip155:250") => Ok("eip155:250/slip44:60".to_string()), // Fantom
        
        // Handle generic eip155 networks
        n if n.starts_with("eip155:") => {
            let chain_id = n.strip_prefix("eip155:").unwrap_or("1");
            if chain_id == "*" {
                Ok("eip155:1/slip44:60".to_string()) // Default to Ethereum mainnet
            } else {
                Ok(format!("eip155:{}/slip44:60", chain_id))
            }
        }
        
        // Cosmos networks
        n if n.starts_with("cosmos:cosmoshub-4") => Ok("cosmos:cosmoshub-4/slip44:118".to_string()), // Cosmos Hub
        n if n.starts_with("cosmos:osmosis-1") => Ok("cosmos:osmosis-1/slip44:118".to_string()), // Osmosis
        n if n.starts_with("cosmos:") => Ok("cosmos:cosmoshub-4/slip44:118".to_string()), // Default Cosmos
        
        // THORChain
        n if n.starts_with("thorchain:") => Ok("thorchain:thorchain-mainnet-v1/slip44:931".to_string()),
        
        // Mayachain  
        n if n.starts_with("mayachain:") => Ok("mayachain:mayachain-mainnet-v1/slip44:931".to_string()),
        
        // Additional UTXO networks
        n if n.starts_with("bip122:000007d91d1254d60e2dd1ae58038307") => Ok("bip122:000007d91d1254d60e2dd1ae58038307/slip44:5".to_string()), // Dash
        n if n.starts_with("bip122:00000000001a91e3dace36e2be3bf030") => Ok("bip122:00000000001a91e3dace36e2be3bf030/slip44:3".to_string()), // Dogecoin
        n if n.starts_with("bip122:7497ea1b465eb39f1c8f507bc877078f") => Ok("bip122:7497ea1b465eb39f1c8f507bc877078f/slip44:20".to_string()), // DigiByte
        n if n.starts_with("bip122:0000000000196a45") => Ok("bip122:0000000000196a45/slip44:133".to_string()), // Zcash
        n if n.starts_with("bip122:027e3758c3a65b12aa1046462b486d0a") => Ok("bip122:027e3758c3a65b12aa1046462b486d0a/slip44:141".to_string()), // Komodo
        
        // Ripple
        n if n.starts_with("ripple:") => Ok("ripple:1/slip44:144".to_string()),
        
        _ => Err(anyhow::anyhow!("Unsupported network for CAIP conversion: {}", network)),
    }
}

/// Convert CAIP to network identifier
fn caip_to_network_id(caip: &str) -> String {
    if caip.starts_with("bip122:000000000019d6689c085ae165831e93") {
        "bip122:000000000019d6689c085ae165831e93".to_string() // Bitcoin
    } else if caip.starts_with("bip122:000000000000000000651ef99cb9fcbe") {
        "bip122:000000000000000000651ef99cb9fcbe".to_string() // Bitcoin Cash
    } else if caip.starts_with("bip122:12a765e31ffd4059bada1e25190f6e98") {
        "bip122:12a765e31ffd4059bada1e25190f6e98".to_string() // Litecoin
    } else if caip.starts_with("eip155:1/") {
        "eip155:1".to_string() // Ethereum mainnet
    } else if caip.starts_with("eip155:10/") {
        "eip155:10".to_string() // Optimism
    } else if caip.starts_with("eip155:137/") {
        "eip155:137".to_string() // Polygon
    } else if caip.starts_with("eip155:42161/") {
        "eip155:42161".to_string() // Arbitrum
    } else if caip.starts_with("eip155:8453/") {
        "eip155:8453".to_string() // Base
    } else if caip.starts_with("eip155:56/") {
        "eip155:56".to_string() // BSC
    } else if caip.starts_with("eip155:43114/") {
        "eip155:43114".to_string() // Avalanche
    } else if caip.starts_with("eip155:250/") {
        "eip155:250".to_string() // Fantom
    } else if caip.starts_with("eip155:") {
        let chain_id = caip.split('/').next().unwrap_or("eip155:1");
        chain_id.to_string()
    } else if caip.starts_with("cosmos:cosmoshub-4") {
        "cosmos:cosmoshub-4".to_string()
    } else if caip.starts_with("cosmos:osmosis-1") {
        "cosmos:osmosis-1".to_string()
    } else if caip.starts_with("cosmos:") {
        "cosmos:cosmoshub-4".to_string()
    } else if caip.starts_with("thorchain:") {
        "thorchain:thorchain-mainnet-v1".to_string()
    } else if caip.starts_with("mayachain:") {
        "mayachain:mayachain-mainnet-v1".to_string()
    } else if caip.starts_with("bip122:000007d91d1254d60e2dd1ae58038307") {
        "bip122:000007d91d1254d60e2dd1ae58038307".to_string() // Dash
    } else if caip.starts_with("bip122:00000000001a91e3dace36e2be3bf030") {
        "bip122:00000000001a91e3dace36e2be3bf030".to_string() // Dogecoin
    } else if caip.starts_with("bip122:7497ea1b465eb39f1c8f507bc877078f") {
        "bip122:7497ea1b465eb39f1c8f507bc877078f".to_string() // DigiByte
    } else if caip.starts_with("bip122:0000000000196a45") {
        "bip122:0000000000196a45".to_string() // Zcash
    } else if caip.starts_with("bip122:027e3758c3a65b12aa1046462b486d0a") {
        "bip122:027e3758c3a65b12aa1046462b486d0a".to_string() // Komodo
    } else if caip.starts_with("ripple:") {
        "ripple:1".to_string()
    } else {
        caip.split('/').next().unwrap_or(caip).to_string()
    }
}

/// Extract symbol from CAIP
fn extract_symbol_from_caip(caip: &str) -> Option<String> {
    if caip.contains("bip122:000000000019d6689c085ae165831e93") {
        Some("BTC".to_string())
    } else if caip.contains("bip122:000000000000000000651ef99cb9fcbe") {
        Some("BCH".to_string()) // Bitcoin Cash
    } else if caip.contains("bip122:12a765e31ffd4059bada1e25190f6e98") {
        Some("LTC".to_string()) // Litecoin
    } else if caip.starts_with("eip155:1/") {
        Some("ETH".to_string()) // Ethereum mainnet
    } else if caip.starts_with("eip155:10/") {
        Some("OP".to_string()) // Optimism 
    } else if caip.starts_with("eip155:137/") {
        Some("MATIC".to_string()) // Polygon
    } else if caip.starts_with("eip155:42161/") {
        Some("ARB".to_string()) // Arbitrum
    } else if caip.starts_with("eip155:8453/") {
        Some("ETH".to_string()) // Base (uses ETH)
    } else if caip.starts_with("eip155:56/") {
        Some("BNB".to_string()) // BSC
    } else if caip.starts_with("eip155:43114/") {
        Some("AVAX".to_string()) // Avalanche
    } else if caip.starts_with("eip155:250/") {
        Some("FTM".to_string()) // Fantom
    } else if caip.starts_with("eip155:") {
        Some("ETH".to_string()) // Default to ETH for unknown EIP155 networks
    } else if caip.starts_with("cosmos:") {
        Some("ATOM".to_string())
    } else if caip.starts_with("thorchain:") {
        Some("RUNE".to_string()) // THORChain
    } else if caip.starts_with("mayachain:") {
        Some("CACAO".to_string()) // Mayachain
    } else if caip.contains("bip122:000007d91d1254d60e2dd1ae58038307") {
        Some("DASH".to_string()) // Dash
    } else if caip.contains("bip122:00000000001a91e3dace36e2be3bf030") {
        Some("DOGE".to_string()) // Dogecoin
    } else if caip.contains("bip122:7497ea1b465eb39f1c8f507bc877078f") {
        Some("DGB".to_string()) // DigiByte
    } else if caip.contains("bip122:0000000000196a45") {
        Some("ZEC".to_string()) // Zcash
    } else if caip.contains("bip122:027e3758c3a65b12aa1046462b486d0a") {
        Some("KMD".to_string()) // Komodo
    } else if caip.starts_with("ripple:") {
        Some("XRP".to_string()) // Ripple
    } else {
        None
    }
}

/// Helper function to format duration as human readable age
fn format_age(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;
    
    if diff < 60 {
        format!("{} seconds ago", diff)
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

/// Create the v2 router with all v2 endpoints (uses AppState as router state)
pub fn v2_router() -> axum::Router<Arc<AppState>> {
    use axum::routing::{get, post, put, delete};
    
    axum::Router::new()
        .route("/networks", get(get_networks).post(post_network))
        .route("/paths", get(get_paths).post(post_path))
        .route("/paths/:id", get(get_path).put(put_path).delete(delete_path))
        .route("/pubkeys", get(get_pubkeys))
        .route("/balances", get(get_balances))
        .route("/portfolio", post(post_portfolio_balances))
        .route("/portfolio/summary", get(get_portfolio_summary))
        .route("/debug/cache", get(debug_device_cache))
        .route("/sync-device", post(sync_device_to_cache))
}

/// Debug endpoint to check what device is in cache
pub async fn debug_device_cache(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let device_id = app_state.device_cache.get_device_id();
    let cached_features = app_state.device_cache.get_cached_features();
    
    Json(serde_json::json!({
        "device_id_in_cache": device_id,
        "has_cached_features": cached_features.is_some(),
        "features_device_id": cached_features.as_ref().map(|f| &f.device_id),
        "cache_address": format!("{:p}", &*app_state.device_cache),
    })).into_response()
}

/// Manual device sync endpoint - force sync device from registry to cache
pub async fn sync_device_to_cache(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let tag = "sync_device_to_cache";
    
    // Get device from registry
    let entries = match crate::device_registry::get_all_device_entries() {
        Ok(entries) => entries,
        Err(e) => {
            error!("{}: Failed to get device registry: {}", tag, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to get device registry: {}", e)
            }))).into_response();
        }
    };
    
    if entries.is_empty() {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "No devices found in registry"
        }))).into_response();
    }
    
    let device_entry = &entries[0]; // Use first device
    let device_id = &device_entry.device.unique_id;
    
    if let Some(features) = &device_entry.features {
        // Convert device registry features to cache features format
        let cache_features = crate::cache::device_cache::CachedFeatures {
    device_id: device_id.clone(),
    label: features.label.clone(),
    vendor: features.vendor.clone(),
    major_version: None, // No longer present in DeviceFeatures
    minor_version: None, // No longer present in DeviceFeatures
    patch_version: None, // No longer present in DeviceFeatures
    revision: features.version.clone().into(), // Use version string as revision
    firmware_hash: features.firmware_hash.clone(),
    bootloader_hash: features.bootloader_hash.clone(),
    features_json: serde_json::to_string(features).unwrap_or_else(|_| "{}".to_string()),
    last_seen: chrono::Utc::now().timestamp(),
};
        
        // Force-set device in memory cache using the new method
        app_state.device_cache.force_set_device_features(device_id.clone(), cache_features);
        
        info!("{}: Successfully synced device {} to cache", tag, device_id);
        
        Json(serde_json::json!({
            "success": true,
            "device_id": device_id,
            "message": "Device synced to cache successfully"
        })).into_response()
    } else {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Device has no features in registry"
        }))).into_response()
    }
}
