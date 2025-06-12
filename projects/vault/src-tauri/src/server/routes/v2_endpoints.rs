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
use axum::routing::{get, post};

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
            // Memory cache is empty - try to load first device from database
            debug!("{}: Memory cache empty, trying to load device from database", tag);
            
            // Get first device from database
            match app_state.device_cache.get_first_device_from_db().await {
                Ok(Some(device_id)) => {
                    debug!("{}: Found device {} in database, loading into memory", tag, device_id);
                    // Load device into memory cache
                    match app_state.device_cache.load_device(&device_id).await {
                        Ok(Some(_)) => {
                            info!("{}: Successfully loaded device {} from database", tag, device_id);
                            device_id
                        }
                        Ok(None) => {
                            error!("{}: Device {} not found in database after lookup", tag, device_id);
                            return Json::<Vec<PubkeyResponse>>(vec![]).into_response();
                        }
                        Err(e) => {
                            error!("{}: Failed to load device {} into memory: {}", tag, device_id, e);
                            return Json::<Vec<PubkeyResponse>>(vec![]).into_response();
                        }
                    }
                }
                Ok(None) => {
                    error!("{}: No devices found in database", tag);
                    return Json::<Vec<PubkeyResponse>>(vec![]).into_response();
                }
                Err(e) => {
                    error!("{}: Failed to query database for devices: {}", tag, e);
                    return Json::<Vec<PubkeyResponse>>(vec![]).into_response();
                }
            }
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
    for path in &paths {
        for network in &path.networks {
            // Get coin info for this network/path combination
            let (coin_name, script_type) = match get_coin_info_from_network(network, &path.script_type) {
                Ok(info) => info,
                Err(e) => {
                    debug!("{}: Skipping unsupported network {}: {}", tag, network, e);
                    continue;
                }
            };
            
            // Use different address paths for UTXO vs account-based networks
            let is_utxo_network = network.starts_with("bip122:");
            let address_path = if is_utxo_network { 
                &path.address_n_list 
            } else { 
                &path.address_n_list_master 
            };
            
            // For UTXO networks, try to find both the XPUB and individual addresses
            if is_utxo_network {
                // First try to find the XPUB
                let xpub_script_type = format!("{}_xpub", script_type);
                if let Some(cached_addr) = app_state.device_cache.get_cached_address(&coin_name, &xpub_script_type, address_path) {
                    // Format BIP32 path strings
                    let path_str = format_bip32_path(&path.address_n_list);
                    let path_master_str = format_bip32_path(address_path);
                    
                    // Determine XPUB type from address content
                    let key_type = if cached_addr.address.starts_with("xpub") {
                        "xpub".to_string()
                    } else if cached_addr.address.starts_with("ypub") {
                        "ypub".to_string()
                    } else if cached_addr.address.starts_with("zpub") {
                        "zpub".to_string()
                    } else {
                        path.path_type.clone()
                    };
                    
                    // Use proper Bitcoin CAIP-2 context for Bitcoin mainnet
                    let bitcoin_context = if network == "bitcoin" || network == "mainnet" {
                        "bip122:000000000019d6689c085ae165831e93"  // Bitcoin mainnet genesis hash
                    } else if network == "testnet" {
                        "bip122:000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943"  // Bitcoin testnet genesis hash
                    } else {
                        &format!("XPUB for {}", network)
                    };

                    let pubkey_response = PubkeyResponse {
                        key_type: key_type.clone(),
                        master: None,
                        address: cached_addr.address.clone(),
                        pubkey: cached_addr.pubkey.unwrap_or_else(|| cached_addr.address.clone()),
                        path: path_str,
                        pathMaster: path_master_str,
                        scriptType: xpub_script_type.clone(),
                        note: path.note.clone(),
                        available_scripts_types: path.available_script_types.clone(),
                        networks: vec![network.clone()],
                        context: Some(bitcoin_context.to_string()),
                    };
                    
                    pubkey_responses.push(pubkey_response);
                    debug!("{}: Added {} XPUB for {} {}", tag, key_type, coin_name, script_type);
                }
                
                // Also try to find individual addresses (first 5)
                for i in 0..5 {
                    let mut individual_path = address_path.to_vec();
                    if individual_path.len() == 3 {
                        individual_path.push(0); // change = 0 (receiving)
                        individual_path.push(i); // address_index
                    } else {
                        continue; // Skip malformed paths
                    }
                    
                    if let Some(cached_addr) = app_state.device_cache.get_cached_address(&coin_name, &script_type, &individual_path) {
                        // Format BIP32 path strings
                        let path_str = format_bip32_path(&individual_path);
                        let path_master_str = format_bip32_path(address_path);
                        
                        let pubkey_response = PubkeyResponse {
                            key_type: "address".to_string(),
                            master: None,
                            address: cached_addr.address.clone(),
                            pubkey: cached_addr.pubkey.unwrap_or_else(|| cached_addr.address.clone()),
                            path: path_str,
                            pathMaster: path_master_str,
                            scriptType: script_type.clone(),
                            note: path.note.clone(),
                            available_scripts_types: path.available_script_types.clone(),
                            networks: vec![network.clone()],
                            context: Some(format!("Address for {}", network)),
                        };
                        
                        pubkey_responses.push(pubkey_response);
                        debug!("{}: Added individual address for {} {} at index {}", tag, coin_name, script_type, i);
                    }
                }
            } else {
                // Account-based networks - just look for the main address
                if let Some(cached_addr) = app_state.device_cache.get_cached_address(&coin_name, &script_type, address_path) {
                    // Format BIP32 path strings
                    let path_str = format_bip32_path(&path.address_n_list);
                    let path_master_str = format_bip32_path(address_path);
                    
                    let key_type = path.path_type.clone();
                    
                    // Use proper CAIP-2 context
                    let context = if network == "bitcoin" || network == "mainnet" {
                        "bip122:000000000019d6689c085ae165831e93"  // Bitcoin mainnet genesis hash
                    } else if network == "testnet" {
                        "bip122:000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943"  // Bitcoin testnet genesis hash
                    } else {
                        &format!("Real cached address for {}", network)
                    };

                    let pubkey_response = PubkeyResponse {
                        key_type,
                        master: None,
                        address: cached_addr.address.clone(),
                        pubkey: cached_addr.pubkey.unwrap_or_else(|| cached_addr.address.clone()),
                        path: path_str,
                        pathMaster: path_master_str,
                        scriptType: script_type.clone(),
                        note: path.note.clone(),
                        available_scripts_types: path.available_script_types.clone(),
                        networks: vec![network.clone()],
                        context: Some(context.to_string()),
                    };
                    
                    pubkey_responses.push(pubkey_response);
                    debug!("{}: Added account-based address for {} {}", tag, coin_name, script_type);
                }
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
    let _device_id = match app_state.device_cache.get_device_id() {
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
    let needs_refresh = match app_state.device_cache.balances_need_refresh(&_device_id).await {
        Ok(needs) => needs || force_refresh,
        Err(e) => {
            error!("{}: Error checking refresh status: {}", tag, e);
            true // Default to refresh on error
        }
    };
    
    if needs_refresh {
        info!("{}: Balances need refresh - fetching from Pioneer API", tag);
        if let Err(e) = refresh_balances_from_pioneer(&*app_state.device_cache, &_device_id).await {
            error!("{}: Failed to refresh balances - FAIL FAST: {}", tag, e);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": format!("Failed to refresh balances: {}", e)
            }))).into_response();
        }
    }
    
    // Get cached balances
    let balances = match app_state.device_cache.get_cached_balances(&_device_id).await {
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
    let _device_id = match app_state.device_cache.get_device_id() {
        Some(id) => id,
        None => {
            error!("{}: No device found in cache", tag);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "No device available - server requires connected KeepKey device"
            }))).into_response();
        }
    };
    
    // Check if balances need refresh
    let needs_refresh = match app_state.device_cache.balances_need_refresh(&_device_id).await {
        Ok(needs) => needs,
        Err(e) => {
            error!("{}: Error checking refresh status: {}", tag, e);
            true
        }
    };
    
    if needs_refresh {
        info!("{}: Balances need refresh - fetching from Pioneer API", tag);
        if let Err(e) = refresh_balances_from_pioneer(&*app_state.device_cache, &_device_id).await {
            error!("{}: Failed to refresh balances - FAIL FAST: {}", tag, e);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": format!("Failed to refresh balances: {}", e)
            }))).into_response();
        }
    }
    
    // Get all cached balances
    let cached_balances = match app_state.device_cache.get_cached_balances(&_device_id).await {
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
    let _device_id = match app_state.device_cache.get_device_id() {
        Some(id) => id,
        None => {
            error!("{}: No device found in cache", tag);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "No device available - server requires connected KeepKey device"
            }))).into_response();
        }
    };
    
    match app_state.device_cache.get_portfolio_summary(&_device_id).await {
        Ok(Some(summary)) => Json(summary).into_response(),
        Ok(None) => {
            // Generate summary from current balances
            match app_state.device_cache.get_cached_balances(&_device_id).await {
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
                        device_id: _device_id.to_string(),
                        total_value_usd: format!("{:.2}", total_value_usd),
                        network_count: networks.len() as i64,
                        asset_count: balances.len() as i64,
                        last_updated: chrono::Utc::now().timestamp(),
                    };
                    
                    // Save the summary
                    if let Err(e) = app_state.device_cache.save_portfolio_summary(&_device_id, &summary).await {
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
            
            // For UTXO networks (Bitcoin), look for XPUBs at account level with _xpub suffix
            // For account-based networks, look at master level with regular script type
            let is_utxo_network = network.starts_with("bip122:");
            let (lookup_script_type, lookup_path) = if is_utxo_network {
                (format!("{}_xpub", script_type), &path.address_n_list) // Bitcoin XPUBs stored as p2wpkh_xpub etc.
            } else {
                (script_type.clone(), &path.address_n_list_master) // Ethereum addresses etc.
            };
            
            if let Some(cached_addr) = cache.get_cached_address(&coin_name, &lookup_script_type, lookup_path) {
                let pubkey = cached_addr.pubkey.unwrap_or_else(|| cached_addr.address.clone());
                info!("{}: Adding asset query: caip={}, pubkey={}, address={} ({})", tag, caip, pubkey, cached_addr.address, lookup_script_type);
                asset_queries.push(serde_json::json!({
                    "caip": caip,
                    "pubkey": pubkey
                }));
            } else {
                warn!("{}: No cached address found for {} {} at path {:?} (looked for script_type: {})", tag, coin_name, script_type, lookup_path, lookup_script_type);
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
    axum::Router::new()
        .route("/networks", get(get_networks).post(post_network))
        .route("/paths", get(get_paths).post(post_path))
        .route("/paths/:id", get(get_path).put(put_path).delete(delete_path))
        .route("/pubkeys", get(get_pubkeys))
        .route("/balances", get(get_balances))
        .route("/portfolio", post(post_portfolio_balances))
        .route("/portfolio/summary", get(get_portfolio_summary))
        .route("/listUnspent", get(list_unspent_v2))
        .route("/txHistory", get(tx_history_v2))
        .route("/getNewAddress", get(get_new_address_v2))
        .route("/getChangeAddress", get(get_change_address_v2))
        .route("/tx/build", post(build_transaction_v2))
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

/// Manual device sync endpoint - force sync device from registry to cache AND refresh balances
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
        
        info!("{}: ✅ Device {} synced to cache, now refreshing balances...", tag, device_id);
        
        // Also refresh balances from Pioneer API to ensure portfolio endpoints work
        match refresh_balances_from_pioneer(&*app_state.device_cache, device_id).await {
            Ok(_) => {
                // Get balance count for confirmation
                let balance_count = match app_state.device_cache.get_cached_balances(device_id).await {
                    Ok(balances) => balances.len(),
                    Err(_) => 0,
                };
                
                info!("{}: ✅ Successfully synced device {} and refreshed {} balances", tag, device_id, balance_count);
                
                Json(serde_json::json!({
                    "success": true,
                    "device_id": device_id,
                    "balances_cached": balance_count,
                    "message": format!("Device synced and {} balances refreshed successfully", balance_count)
                })).into_response()
            }
            Err(e) => {
                warn!("{}: Device synced but balance refresh failed: {}", tag, e);
                Json(serde_json::json!({
                    "success": true,
                    "device_id": device_id,
                    "balances_cached": 0,
                    "warning": format!("Device synced but balance refresh failed: {}", e),
                    "message": "Device synced successfully but balances may be stale"
                })).into_response()
            }
        }
    } else {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Device has no features in registry"
        }))).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PubkeyQueryParams {
    pub include_indices: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XpubQueryParams {
    pub xpub: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UtxoResponse {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub height: Option<u64>,
    pub confirmations: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxHistoryResponse {
    pub balance: u64,
    #[serde(rename = "totalReceived")]
    pub total_received: u64,
    #[serde(rename = "totalSent")]
    pub total_sent: u64,
    pub txs: u64,
    pub page: u32,
    #[serde(rename = "totalPages")]
    pub total_pages: u32,
    pub txids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressGenerationResponse {
    pub address: String,
    #[serde(rename = "addressIndex")]
    pub address_index: u32,
    #[serde(rename = "changeIndex")]
    pub change_index: Option<u32>,
    #[serde(rename = "receiveIndex")]
    pub receive_index: Option<u32>,
    pub path: String,
}

// Pioneer API client for comparison
async fn call_pioneer_api(endpoint: &str) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:9001{}", endpoint);
    
    debug!("Calling Pioneer API: {}", url);
    let response = client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await?;
    
    Ok(response)
}



/// List unspent outputs for an XPUB - hits pioneers.dev
pub async fn list_unspent_v2(
    State(_app_state): State<Arc<AppState>>,
    Query(params): Query<XpubQueryParams>,
) -> impl IntoResponse {
    let tag = "list_unspent_v2";
    debug!("{}: Getting unspent outputs for XPUB: {}", tag, &params.xpub[..20]);
    
    // Call pioneers.dev for real data - NO FALLBACK
    match call_pioneer_api(&format!("/api/v1/listUnspent/BTC/{}", params.xpub)).await {
        Ok(response) => {
            match response.json::<Vec<UtxoResponse>>().await {
                Ok(utxos) => {
                    info!("{}: ✅ Found {} UTXOs from pioneers.dev", tag, utxos.len());
                    Json(utxos).into_response()
                }
                Err(e) => {
                    error!("{}: ❌ Failed to parse pioneers.dev response: {}", tag, e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            error!("{}: ❌ Failed to call pioneers.dev: {}", tag, e);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

/// Get transaction history for an XPUB - hits pioneers.dev  
pub async fn tx_history_v2(
    State(_app_state): State<Arc<AppState>>,
    Query(params): Query<XpubQueryParams>,
) -> impl IntoResponse {
    let tag = "tx_history_v2";
    debug!("{}: Getting tx history for XPUB: {}", tag, &params.xpub[..20]);
    
    // Call pioneers.dev for real data - NO FALLBACK
    match call_pioneer_api(&format!("/api/v1/txsByXpub/BTC/{}", params.xpub)).await {
        Ok(response) => {
            match response.json::<TxHistoryResponse>().await {
                Ok(history) => {
                    info!("{}: ✅ Found {} transactions from pioneers.dev", tag, history.txs);
                    Json(history).into_response()
                }
                Err(e) => {
                    error!("{}: ❌ Failed to parse pioneers.dev response: {}", tag, e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            error!("{}: ❌ Failed to call pioneers.dev: {}", tag, e);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

/// Generate new receive address for an XPUB - hits pioneers.dev
pub async fn get_new_address_v2(
    State(_app_state): State<Arc<AppState>>,
    Query(params): Query<XpubQueryParams>,
) -> impl IntoResponse {
    let tag = "get_new_address_v2";
    debug!("{}: Getting new receive address for XPUB: {}", tag, &params.xpub[..20]);
    
    // Call pioneers.dev for real data - NO FALLBACK
    match call_pioneer_api(&format!("/api/v1/getNewAddress/BTC/{}", params.xpub)).await {
        Ok(response) => {
            match response.json::<AddressGenerationResponse>().await {
                Ok(address_data) => {
                    info!("{}: ✅ Generated receive address from pioneers.dev", tag);
                    Json(address_data).into_response()
                }
                Err(e) => {
                    error!("{}: ❌ Failed to parse pioneers.dev response: {}", tag, e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            error!("{}: ❌ Failed to call pioneers.dev: {}", tag, e);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

/// Generate new change address for an XPUB - hits pioneers.dev
pub async fn get_change_address_v2(
    State(_app_state): State<Arc<AppState>>,
    Query(params): Query<XpubQueryParams>,
) -> impl IntoResponse {
    let tag = "get_change_address_v2";
    debug!("{}: Getting new change address for XPUB: {}", tag, &params.xpub[..20]);
    
    // Call pioneers.dev for real data - NO FALLBACK
    match call_pioneer_api(&format!("/api/v1/getChangeAddress/BTC/{}", params.xpub)).await {
        Ok(response) => {
            match response.json::<AddressGenerationResponse>().await {
                Ok(address_data) => {
                    info!("{}: ✅ Generated change address from pioneers.dev", tag);
                    Json(address_data).into_response()
                }
                Err(e) => {
                    error!("{}: ❌ Failed to parse pioneers.dev response: {}", tag, e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            error!("{}: ❌ Failed to call pioneers.dev: {}", tag, e);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

// ===== TRANSACTION BUILDING API =====

/// 🚨 SAFETY CONSTANTS - NEVER ALLOW FEES ABOVE THESE LIMITS
const MAX_FEE_BTC: f64 = 0.1; // 0.1 BTC absolute maximum
const MAX_FEE_PERCENT: f64 = 50.0; // 50% of transaction value maximum  
const MIN_FEE_RATE: f64 = 1.0; // 1 sat/vByte minimum
const MAX_FEE_RATE: f64 = 1000.0; // 1000 sat/vByte maximum (sanity check)
const DUST_LIMIT: u64 = 546; // Standard dust limit in satoshis

#[derive(Debug, Deserialize, ToSchema)]
pub struct BuildTxRequest {
    pub device_id: String,
    pub recipients: Vec<TxOutputRequest>,
    pub fee_rate: f64,                    // sat/vByte
    pub input_selection: InputSelection,
    pub max_fee_btc: Option<f64>,        // Optional override (default: 0.1 BTC max)
    pub script_type: Option<String>,     // Optional script type (default: p2wpkh)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TxOutputRequest {
    pub address: String,
    pub amount: String,                  // BTC amount as string (e.g., "0.00100000")
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum InputSelection {
    Auto { percent: u8 },               // 0-100% of available UTXOs
    Manual { utxos: Vec<String> },      // Specific txid:vout identifiers
    Max,                                // Send maximum (all UTXOs minus fee)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BuildTxResponse {
    pub success: bool,
    pub tx: Option<UnsignedTx>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnsignedTx {
    pub inputs: Vec<TxInputDetail>,
    pub outputs: Vec<TxOutputDetail>,
    pub fee_sats: u64,
    pub fee_btc: String,
    pub fee_usd: String,
    pub size_bytes: u32,
    pub fee_rate: f64,                  // Actual sat/vByte achieved
    pub total_input_sats: u64,
    pub total_output_sats: u64,
    pub change_output: Option<ChangeOutputDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TxInputDetail {
    pub txid: String,
    pub vout: u32,
    pub amount_sats: u64,
    pub script_type: String,
    pub address_n_list: Vec<u32>,
    pub confirmations: u32,
    pub address: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TxOutputDetail {
    pub address: String,
    pub amount_sats: u64,
    pub is_change: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChangeOutputDetail {
    pub address: String,
    pub amount_sats: u64,
    pub address_n_list: Vec<u32>,
    pub script_type: String,
}

/// Build an unsigned Bitcoin transaction with comprehensive safety checks
#[utoipa::path(
    post,
    context_path = "/v2",
    path = "/tx/build",
    tag = "Transaction",
    request_body = BuildTxRequest,
    responses(
        (status = 200, body = BuildTxResponse, description = "Transaction built successfully"),
        (status = 400, body = BuildTxResponse, description = "Invalid request or excessive fee"),
        (status = 404, body = BuildTxResponse, description = "Device not found or no UTXOs"),
        (status = 500, body = BuildTxResponse, description = "Internal server error"),
    ),
)]
pub async fn build_transaction_v2(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<BuildTxRequest>,
) -> impl IntoResponse {
    let tag = "build_transaction_v2";
    debug!("{}: Building transaction for device: {}", tag, request.device_id);
    
    // 🚨 PHASE 1: INPUT VALIDATION - FAIL FAST
    if let Err(error_msg) = validate_build_request(&request) {
        warn!("{}: Validation failed: {}", tag, error_msg);
        return (StatusCode::BAD_REQUEST, Json(BuildTxResponse {
            success: false,
            tx: None,
            error: Some(error_msg),
            warnings: vec![],
        })).into_response();
    }
    
    // 🚨 PHASE 2: GET DEVICE AND UTXOS - FAIL FAST IF NO DATA
    let device_id = request.device_id.trim();
    let script_type = request.script_type.as_deref().unwrap_or("p2wpkh");
    
    // Get Bitcoin XPUBs from device cache
    let xpubs = match get_bitcoin_xpubs_for_script_type(&app_state.device_cache, script_type).await {
        Ok(xpubs) => xpubs,
        Err(e) => {
            error!("{}: Failed to get XPUBs: {}", tag, e);
            return (StatusCode::NOT_FOUND, Json(BuildTxResponse {
                success: false,
                tx: None,
                error: Some(format!("Failed to get XPUBs for device: {}", e)),
                warnings: vec![],
            })).into_response();
        }
    };
    
    // Get UTXOs from pioneers.dev
    let all_utxos = match get_utxos_from_pioneer(&xpubs).await {
        Ok(utxos) => utxos,
        Err(e) => {
            error!("{}: Failed to get UTXOs: {}", tag, e);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(BuildTxResponse {
                success: false,
                tx: None,
                error: Some(format!("Failed to get UTXOs: {}", e)),
                warnings: vec![],
            })).into_response();
        }
    };
    
    if all_utxos.is_empty() {
        warn!("{}: No UTXOs found for device", tag);
        return (StatusCode::NOT_FOUND, Json(BuildTxResponse {
            success: false,
            tx: None,
            error: Some("No UTXOs found for this device".to_string()),
            warnings: vec![],
        })).into_response();
    }
    
    // 🚨 PHASE 3: UTXO SELECTION
    let selected_utxos = match select_utxos(&all_utxos, &request.input_selection) {
        Ok(utxos) => utxos,
        Err(e) => {
            error!("{}: UTXO selection failed: {}", tag, e);
            return (StatusCode::BAD_REQUEST, Json(BuildTxResponse {
                success: false,
                tx: None,
                error: Some(e),
                warnings: vec![],
            })).into_response();
        }
    };
    
    // 🚨 PHASE 4: COIN SELECTION & TRANSACTION BUILDING
    let is_max_send = matches!(request.input_selection, InputSelection::Max);
    
    match build_bitcoin_transaction(selected_utxos, request.recipients, request.fee_rate, is_max_send, request.max_fee_btc).await {
        Ok((unsigned_tx, warnings)) => {
            info!("{}: ✅ Transaction built successfully - fee: {} BTC", tag, unsigned_tx.fee_btc);
            (StatusCode::OK, Json(BuildTxResponse {
                success: true,
                tx: Some(unsigned_tx),
                error: None,
                warnings,
            })).into_response()
        }
        Err(e) => {
            error!("{}: Transaction building failed: {}", tag, e);
            let status_code = if e.contains("Fee too high") || e.contains("exceeds limit") {
                StatusCode::BAD_REQUEST
            } else if e.contains("Insufficient funds") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            (status_code, Json(BuildTxResponse {
                success: false,
                tx: None,
                error: Some(format_user_friendly_error(&e)),
                warnings: vec![],
            })).into_response()
        }
    }
}

/// Validate the build request - FAIL FAST on invalid input
fn validate_build_request(req: &BuildTxRequest) -> Result<(), String> {
    // Device ID validation
    if req.device_id.trim().is_empty() {
        return Err("Device ID is required".to_string());
    }
    
    // Recipients validation
    if req.recipients.is_empty() {
        return Err("At least one recipient is required".to_string());
    }
    
    for (i, recipient) in req.recipients.iter().enumerate() {
        // Bitcoin address validation (basic check)
        if recipient.address.trim().is_empty() {
            return Err(format!("Empty address at recipient {}", i + 1));
        }
        
        // Amount validation
        let amount = parse_btc_amount(&recipient.amount)?;
        if amount <= 0.0 {
            return Err(format!("Invalid amount at recipient {}: must be > 0", i + 1));
        }
        
        // Dust limit check (546 sats for P2PKH)
        let amount_sats = (amount * 100_000_000.0) as u64;
        if amount_sats < DUST_LIMIT {
            return Err(format!("Amount too small at recipient {}: below dust limit of {} sats", i + 1, DUST_LIMIT));
        }
    }
    
    // Fee rate validation
    if req.fee_rate < MIN_FEE_RATE {
        return Err(format!("Fee rate must be at least {} sat/vByte", MIN_FEE_RATE));
    }
    if req.fee_rate > MAX_FEE_RATE {
        return Err(format!("Fee rate of {} sat/vByte is unreasonably high", req.fee_rate));
    }
    
    // Input selection validation
    match &req.input_selection {
        InputSelection::Auto { percent } => {
            if *percent == 0 || *percent > 100 {
                return Err("Percentage must be between 1 and 100".to_string());
            }
        }
        InputSelection::Manual { utxos } => {
            if utxos.is_empty() {
                return Err("Manual selection requires at least one UTXO".to_string());
            }
            for utxo in utxos {
                if !utxo.contains(':') {
                    return Err(format!("Invalid UTXO format: {} (expected txid:vout)", utxo));
                }
            }
        }
        InputSelection::Max => {
            // Max send is always valid
        }
    }
    
    Ok(())
}

/// Parse BTC amount string to f64
fn parse_btc_amount(amount_str: &str) -> Result<f64, String> {
    amount_str.parse::<f64>()
        .map_err(|_| format!("Invalid amount format: {}", amount_str))
}

/// Get Bitcoin XPUBs for specific script type
async fn get_bitcoin_xpubs_for_script_type(cache: &DeviceCache, script_type: &str) -> Result<Vec<String>, String> {
    // This would ideally get XPUBs from cache based on script type
    // For now, implement a basic version
    
    let device_id = cache.get_device_id()
        .ok_or("No device found in cache")?;
    
    // Try to get a cached XPUB for this script type
    let xpub_script_type = format!("{}_xpub", script_type);
    
    // For now, return placeholder - this would be integrated with actual cache lookup
    // In real implementation, this would search cache for XPUBs
    Ok(vec!["xpub_placeholder".to_string()])
}

/// Get UTXOs from pioneers.dev for given XPUBs
async fn get_utxos_from_pioneer(xpubs: &[String]) -> Result<Vec<UtxoResponse>, String> {
    let mut all_utxos = Vec::new();
    
    for xpub in xpubs {
        if xpub == "xpub_placeholder" {
            // Skip placeholder for now
            continue;
        }
        
        match call_pioneer_api(&format!("/api/v1/listUnspent/BTC/{}", xpub)).await {
            Ok(response) => {
                match response.json::<Vec<UtxoResponse>>().await {
                    Ok(mut utxos) => {
                        all_utxos.append(&mut utxos);
                    }
                    Err(e) => {
                        return Err(format!("Failed to parse UTXO response: {}", e));
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to call pioneers.dev: {}", e));
            }
        }
    }
    
    Ok(all_utxos)
}

/// Select UTXOs based on selection strategy
fn select_utxos(all_utxos: &[UtxoResponse], selection: &InputSelection) -> Result<Vec<UtxoResponse>, String> {
    match selection {
        InputSelection::Auto { percent } => {
            let mut sorted_utxos = all_utxos.to_vec();
            
            // Sort by confirmations (desc) then by value (asc) for privacy
            sorted_utxos.sort_by(|a, b| {
                match b.confirmations.unwrap_or(0).cmp(&a.confirmations.unwrap_or(0)) {
                    std::cmp::Ordering::Equal => a.value.cmp(&b.value),
                    other => other,
                }
            });
            
            let count = ((sorted_utxos.len() as f64 * *percent as f64) / 100.0).ceil() as usize;
            let count = count.max(1).min(sorted_utxos.len());
            
            Ok(sorted_utxos.into_iter().take(count).collect())
        }
        InputSelection::Manual { utxos: selected_outpoints } => {
            let mut selected = Vec::new();
            
            for outpoint in selected_outpoints {
                let parts: Vec<&str> = outpoint.split(':').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid outpoint format: {}", outpoint));
                }
                
                let txid = parts[0];
                let vout: u32 = parts[1].parse()
                    .map_err(|_| format!("Invalid vout in outpoint: {}", outpoint))?;
                
                match all_utxos.iter().find(|u| u.txid == txid && u.vout == vout) {
                    Some(utxo) => selected.push(utxo.clone()),
                    None => return Err(format!("UTXO not found: {}", outpoint)),
                }
            }
            
            Ok(selected)
        }
        InputSelection::Max => {
            // For max send, use all UTXOs
            Ok(all_utxos.to_vec())
        }
    }
}

/// Build Bitcoin transaction with comprehensive safety checks
async fn build_bitcoin_transaction(
    utxos: Vec<UtxoResponse>,
    recipients: Vec<TxOutputRequest>,
    fee_rate: f64,
    is_max_send: bool,
    max_fee_override: Option<f64>,
) -> Result<(UnsignedTx, Vec<String>), String> {
    
    // Calculate total input value
    let total_input_sats: u64 = utxos.iter().map(|u| u.value).sum();
    
    // Calculate total output value (for non-max sends)
    let mut total_output_sats = 0u64;
    for recipient in &recipients {
        let amount = parse_btc_amount(&recipient.amount)?;
        total_output_sats += (amount * 100_000_000.0) as u64;
    }
    
    // Estimate transaction size (rough calculation)
    let estimated_size = estimate_tx_size(utxos.len(), recipients.len() + 1); // +1 for potential change
    let estimated_fee_sats = (estimated_size as f64 * fee_rate).ceil() as u64;
    
    // 🚨 CRITICAL SAFETY CHECK: Validate fee
    validate_fee_safety(estimated_fee_sats, total_output_sats, max_fee_override)?;
    
    // Check if we have enough funds
    if !is_max_send && total_input_sats < total_output_sats + estimated_fee_sats {
        let needed_btc = (total_output_sats + estimated_fee_sats) as f64 / 100_000_000.0;
        let available_btc = total_input_sats as f64 / 100_000_000.0;
        return Err(format!("Insufficient funds: need {:.8} BTC, have {:.8} BTC", needed_btc, available_btc));
    }
    
    // Build transaction inputs
    let inputs: Vec<TxInputDetail> = utxos.iter().map(|utxo| TxInputDetail {
        txid: utxo.txid.clone(),
        vout: utxo.vout,
        amount_sats: utxo.value,
        script_type: "p2wpkh".to_string(), // Default for now
        address_n_list: vec![], // Would be populated from cache in real implementation
        confirmations: utxo.confirmations.unwrap_or(0) as u32,
        address: "".to_string(), // Would be populated from cache
    }).collect();
    
    // Build transaction outputs
    let mut outputs = Vec::new();
    
    if is_max_send {
        // Max send: single output for all funds minus fee
        if recipients.len() != 1 {
            return Err("Max send only supports single recipient".to_string());
        }
        let max_amount_sats = total_input_sats - estimated_fee_sats;
        outputs.push(TxOutputDetail {
            address: recipients[0].address.clone(),
            amount_sats: max_amount_sats,
            is_change: false,
        });
        total_output_sats = max_amount_sats;
    } else {
        // Regular send: specified outputs + change if needed
        for recipient in recipients {
            let amount = parse_btc_amount(&recipient.amount)?;
            outputs.push(TxOutputDetail {
                address: recipient.address,
                amount_sats: (amount * 100_000_000.0) as u64,
                is_change: false,
            });
        }
        
        // Add change output if needed
        let change_amount = total_input_sats - total_output_sats - estimated_fee_sats;
        if change_amount > DUST_LIMIT {
            outputs.push(TxOutputDetail {
                address: "change_address_placeholder".to_string(), // Would get real change address
                amount_sats: change_amount,
                is_change: true,
            });
        }
    }
    
    // Generate privacy warnings
    let warnings = generate_privacy_warnings(&inputs, &outputs);
    
    let unsigned_tx = UnsignedTx {
        inputs,
        outputs,
        fee_sats: estimated_fee_sats,
        fee_btc: format!("{:.8}", estimated_fee_sats as f64 / 100_000_000.0),
        fee_usd: "N/A".to_string(), // Keep BTC-only, no USD conversion needed
        size_bytes: estimated_size as u32,
        fee_rate,
        total_input_sats,
        total_output_sats,
        change_output: None, // Would be populated if change output exists
    };
    
    Ok((unsigned_tx, warnings))
}

/// 🚨 CRITICAL SAFETY FUNCTION: Validate fee safety
fn validate_fee_safety(fee_sats: u64, total_output_sats: u64, max_fee_override: Option<f64>) -> Result<(), String> {
    let fee_btc = fee_sats as f64 / 100_000_000.0;
    let max_allowed = max_fee_override.unwrap_or(MAX_FEE_BTC);
    
    // 🚨 HARD LIMIT: Never allow fees > 0.1 BTC (or override limit)
    if fee_btc > max_allowed {
        return Err(format!(
            "Fee of {:.8} BTC exceeds maximum allowed limit of {:.1} BTC. This appears to be an error.",
            fee_btc, max_allowed
        ));
    }
    
    // 🚨 PERCENTAGE CHECK: Fee shouldn't exceed 50% of transaction value
    if total_output_sats > 0 {
        let fee_percentage = (fee_sats as f64 / total_output_sats as f64) * 100.0;
        if fee_percentage > MAX_FEE_PERCENT {
            return Err(format!(
                "Fee of {:.1}% is excessive. Maximum allowed is {:.1}% of transaction value.",
                fee_percentage, MAX_FEE_PERCENT
            ));
        }
    }
    
    Ok(())
}

/// Estimate transaction size in bytes
fn estimate_tx_size(input_count: usize, output_count: usize) -> usize {
    // Rough estimation for P2WPKH transactions
    // Base: 10 bytes
    // Input: ~68 bytes each (for P2WPKH)
    // Output: ~31 bytes each (for P2WPKH)
    10 + (input_count * 68) + (output_count * 31)
}

/// Generate privacy warnings
fn generate_privacy_warnings(inputs: &[TxInputDetail], outputs: &[TxOutputDetail]) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // Check for low confirmations
    let low_conf_count = inputs.iter().filter(|i| i.confirmations < 3).count();
    if low_conf_count > 0 {
        warnings.push(format!("⚠️ {} input(s) have low confirmations", low_conf_count));
    }
    
    // Check for round number amounts
    for output in outputs {
        if !output.is_change && is_round_number(output.amount_sats) {
            warnings.push("⚠️ Round number amounts may reduce privacy".to_string());
            break;
        }
    }
    
    warnings
}

/// Check if amount is a round number that might reduce privacy
fn is_round_number(sats: u64) -> bool {
    let btc = sats as f64 / 100_000_000.0;
    btc.fract() == 0.0 || // Whole BTC amounts
    sats % 1_000_000 == 0 || // Whole mBTC amounts  
    sats % 100_000 == 0 // 0.001 BTC increments
}

/// Format error messages in user-friendly way
fn format_user_friendly_error(error: &str) -> String {
    if error.contains("Fee") && error.contains("exceeds") {
        format!("Transaction fee seems unusually high. {}", error)
    } else if error.contains("Insufficient funds") {
        format!("Not enough funds available. {}", error)
    } else if error.contains("dust limit") {
        "Amount is too small (below Bitcoin dust limit of 546 satoshis)".to_string()
    } else {
        error.to_string()
    }
}
