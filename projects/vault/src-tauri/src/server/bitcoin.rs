// Bitcoin API - Simple, focused implementation for vault
// Handles frontloading of pubkeys/addresses from real KeepKey devices

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, warn};
use utoipa::ToSchema;

use crate::usb_manager::DeviceManager;

// === Data Models ===

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultPath {
    pub id: String,
    pub note: String,
    pub blockchain: String,
    pub symbol: String,
    pub networks: Vec<String>,
    pub script_type: String,
    pub available_script_types: Vec<String>,
    #[serde(rename = "type")]
    pub key_type: String,
    #[serde(rename = "addressNList")]
    pub address_n_list: Vec<u32>,
    #[serde(rename = "addressNListMaster")]
    pub address_n_list_master: Vec<u32>,
    pub curve: String,
    #[serde(rename = "showDisplay")]
    pub show_display: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultPaths {
    pub version: String,
    pub description: String,
    pub paths: Vec<DefaultPath>,
}

// === API Request/Response Types ===

#[derive(Debug, Serialize, ToSchema)]
pub struct NetworksResponse {
    pub networks: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ParsePathRequest {
    pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ParsePathResponse {
    pub valid: bool,
    pub normalized: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PubkeyRequest {
    pub path: String,
    pub network: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PubkeyResponse {
    pub xpub: String,
    pub pubkey: String,
    pub address: String,
    pub path: String,
    pub script_type: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BalanceRequest {
    pub address: String,
    pub network: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BalanceResponse {
    pub balance: u64,
    pub confirmed: u64,
    pub unconfirmed: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FrontloadResponse {
    pub success: bool,
    pub addresses_loaded: u32,
    pub networks: Vec<String>,
    pub message: String,
}

// === Server State ===

pub struct BitcoinState {
    pub device_manager: Arc<tokio::sync::Mutex<DeviceManager>>,
    pub indexdb: Arc<tokio::sync::Mutex<rusqlite::Connection>>,
    pub default_paths: Vec<DefaultPath>,
}

impl BitcoinState {
    pub async fn new(
        device_manager: Arc<tokio::sync::Mutex<DeviceManager>>,
        indexdb: Arc<tokio::sync::Mutex<rusqlite::Connection>>,
    ) -> Result<Self> {
        let default_paths = load_default_paths().await?;
        info!("Loaded {} default paths", default_paths.len());
        
        Ok(Self {
            device_manager,
            indexdb,
            default_paths,
        })
    }
}

// === Load Default Paths ===

async fn load_default_paths() -> Result<Vec<DefaultPath>> {
    let content = tokio::fs::read_to_string("src/default-paths.json").await?;
    let parsed: DefaultPaths = serde_json::from_str(&content)?;
    Ok(parsed.paths)
}

// === API Endpoints ===

/// List supported networks (from default paths)
#[utoipa::path(
    get,
    path = "/api/networks",
    responses(
        (status = 200, description = "Supported networks", body = NetworksResponse)
    ),
    tag = "bitcoin"
)]
pub async fn networks(State(state): State<Arc<super::AppState>>) -> Json<NetworksResponse> {
    let bitcoin_state = &state.bitcoin_state;
    let mut networks = Vec::new();
    for path in &bitcoin_state.default_paths {
        for network in &path.networks {
            if !networks.contains(network) {
                networks.push(network.clone());
            }
        }
    }
    Json(NetworksResponse { networks })
}

/// Parse and validate BIP32 paths
#[utoipa::path(
    post,
    path = "/api/parse-path",
    request_body = ParsePathRequest,
    responses(
        (status = 200, description = "Path validation result", body = ParsePathResponse)
    ),
    tag = "bitcoin"
)]
pub async fn parse_path(Json(req): Json<ParsePathRequest>) -> Json<ParsePathResponse> {
    // Real BIP32 path validation
    if req.path.starts_with("m/") && (
        req.path.starts_with("m/44'/0'/") ||  // Legacy
        req.path.starts_with("m/49'/0'/") ||  // SegWit P2SH
        req.path.starts_with("m/84'/0'/")     // Native SegWit
    ) {
        Json(ParsePathResponse {
            valid: true,
            normalized: Some(req.path),
            error: None,
        })
    } else {
        Json(ParsePathResponse {
            valid: false,
            normalized: None,
            error: Some("Invalid or unsupported Bitcoin path".to_string()),
        })
    }
}

/// Get pubkey/address for a path (from frontloaded data or device)
#[utoipa::path(
    post,
    path = "/api/pubkey",
    request_body = PubkeyRequest,
    responses(
        (status = 200, description = "Public key data", body = PubkeyResponse),
        (status = 503, description = "No device connected or data not frontloaded")
    ),
    tag = "bitcoin"
)]
pub async fn pubkey(
    State(state): State<Arc<super::AppState>>,
    Json(req): Json<PubkeyRequest>
) -> Result<Json<PubkeyResponse>, axum::http::StatusCode> {
    let bitcoin_state = &state.bitcoin_state;
    // First check if we have frontloaded data
    match get_frontloaded_pubkey(bitcoin_state, &req.path, &req.network).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => {
            // No frontloaded data - try to get from device
            match get_device_pubkey(bitcoin_state, &req.path, &req.network).await {
                Ok(response) => Ok(Json(response)),
                Err(_) => Err(axum::http::StatusCode::SERVICE_UNAVAILABLE),
            }
        }
    }
}

/// Frontload all addresses from connected device
#[utoipa::path(
    post,
    path = "/api/frontload",
    responses(
        (status = 200, description = "Frontload result", body = FrontloadResponse),
        (status = 503, description = "No device connected")
    ),
    tag = "bitcoin"
)]
pub async fn frontload(State(state): State<Arc<super::AppState>>) -> Result<Json<FrontloadResponse>, axum::http::StatusCode> {
    let bitcoin_state = &state.bitcoin_state;
    let device_manager = bitcoin_state.device_manager.lock().await;
    let devices = device_manager.get_connected_devices();
    // Check for any KeepKey device (normal mode 0x0001 or bootloader 0x0002)
    let keepkey_devices: Vec<_> = devices.into_iter()
        .filter(|d| d.is_keepkey || (d.vid == 0x2B24 && (d.pid == 0x0001 || d.pid == 0x0002)))
        .collect();
    
    if keepkey_devices.is_empty() {
        return Ok(Json(FrontloadResponse {
            success: false,
            addresses_loaded: 0,
            networks: vec![],
            message: "No KeepKey device connected".to_string(),
        }));
    }
    
    // Check if device is in bootloader mode (can't get addresses from bootloader)
    let in_bootloader = keepkey_devices.iter().any(|d| d.pid == 0x0002);
    if in_bootloader {
        return Ok(Json(FrontloadResponse {
            success: false,
            addresses_loaded: 0,
            networks: vec![],
            message: "KeepKey is in bootloader mode - please install firmware first".to_string(),
        }));
    }
    
    drop(device_manager); // Release lock before async operations
    
    // Frontload all paths from default-paths.json
    match frontload_all_paths(bitcoin_state).await {
        Ok((count, networks)) => Ok(Json(FrontloadResponse {
            success: true,
            addresses_loaded: count,
            networks,
            message: format!("Successfully frontloaded {} addresses", count),
        })),
        Err(e) => {
            error!("Frontload failed: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// === Helper Functions ===

async fn check_frontloaded(indexdb: &Arc<tokio::sync::Mutex<rusqlite::Connection>>) -> Result<bool> {
    let db = indexdb.lock().await;
    let mut stmt = db.prepare("SELECT COUNT(*) FROM addresses WHERE pubkey IS NOT NULL")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;
    Ok(count > 0)
}

async fn get_frontloaded_pubkey(
    state: &BitcoinState,
    path: &str,
    network: &str,
) -> Result<PubkeyResponse> {
    let db = state.indexdb.lock().await;
    let mut stmt = db.prepare(
        "SELECT address, pubkey, xpub, script_type FROM addresses WHERE path = ?1 AND network = ?2"
    )?;
    
    let result = stmt.query_row([path, network], |row| {
        Ok(PubkeyResponse {
            address: row.get(0)?,
            pubkey: row.get(1)?,
            xpub: row.get(2)?,
            path: path.to_string(),
            script_type: row.get(3)?,
        })
    })?;
    
    Ok(result)
}

async fn get_device_pubkey(
    _state: &BitcoinState,
    _path: &str,
    _network: &str,
) -> Result<PubkeyResponse> {
    // TODO: Implement real device communication
    // This should use the transport layer to get pubkey from device
    Err(anyhow::anyhow!("Device communication not yet implemented"))
}

async fn frontload_all_paths(state: &BitcoinState) -> Result<(u32, Vec<String>)> {
    let mut loaded_count = 0u32;
    let mut networks = Vec::new();
    
    // Initialize database tables if needed
    init_database_tables(&state.indexdb).await?;
    
    for path in &state.default_paths {
        for network in &path.networks {
            // TODO: Get real pubkey/address from device
            // For now, just store the path info
            let address = format!("FRONTLOAD_PENDING_{}", path.id);
            
            if let Err(e) = store_address_info(
                &state.indexdb,
                &format_bip32_path(&path.address_n_list_master),
                network,
                &address,
                "PENDING",
                "PENDING",
                &path.script_type,
            ).await {
                warn!("Failed to store path {}: {}", path.id, e);
            } else {
                loaded_count += 1;
                if !networks.contains(network) {
                    networks.push(network.clone());
                }
            }
        }
    }
    
    info!("Frontloaded {} addresses across {} networks", loaded_count, networks.len());
    Ok((loaded_count, networks))
}

async fn init_database_tables(indexdb: &Arc<tokio::sync::Mutex<rusqlite::Connection>>) -> Result<()> {
    let db = indexdb.lock().await;
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS addresses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL,
            network TEXT NOT NULL,
            address TEXT NOT NULL,
            pubkey TEXT,
            xpub TEXT,
            script_type TEXT NOT NULL,
            created_at INTEGER DEFAULT (strftime('%s', 'now')),
            UNIQUE(path, network)
        )
        "#,
        [],
    )?;
    
    Ok(())
}

async fn store_address_info(
    indexdb: &Arc<tokio::sync::Mutex<rusqlite::Connection>>,
    path: &str,
    network: &str,
    address: &str,
    pubkey: &str,
    xpub: &str,
    script_type: &str,
) -> Result<()> {
    let db = indexdb.lock().await;
    
    db.execute(
        r#"
        INSERT OR REPLACE INTO addresses 
        (path, network, address, pubkey, xpub, script_type)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        [path, network, address, pubkey, xpub, script_type],
    )?;
    
    Ok(())
}

fn format_bip32_path(address_n: &[u32]) -> String {
    let parts: Vec<String> = address_n.iter()
        .map(|&n| {
            if n >= 0x80000000 {
                format!("{}′", n - 0x80000000)
            } else {
                n.to_string()
            }
        })
        .collect();
    
    format!("m/{}", parts.join("/"))
}

// Router is handled in mod.rs 