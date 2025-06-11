use axum::{
    extract::State,
    http::StatusCode,
    Json,
    routing::post,
    Router,
    response::IntoResponse, // Import IntoResponse trait
};
use std::sync::Arc;
use utoipa::ToSchema;
use std::fs;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use log;
use hex;

use super::AppState;
use crate::device_update;

// === V1 API Types (matching swagger.json) ===

#[derive(Debug, Deserialize, ToSchema)]
pub struct UtxoAddressRequest {
    pub show_display: Option<bool>,
    pub script_type: Option<String>,
    pub coin: String,
    pub address_n: Vec<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UtxoAddressResponse {
    pub address: String,
    pub address_n: Vec<u32>,
}

// === Existing Types ===

#[derive(serde::Serialize, ToSchema)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    connected: bool,
    device: Option<DeviceInfo>,
}

#[derive(serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// Unique device ID (based on serial number or bus/address)
    pub device_id: String,
    /// User-friendly device name
    pub name: String,
    /// USB Vendor ID
    pub vendor_id: u16,
    /// USB Product ID  
    pub product_id: u16,
    /// Manufacturer string from USB descriptor
    pub manufacturer: Option<String>,
    /// Product string from USB descriptor
    pub product: Option<String>,
    /// Serial number if available
    pub serial_number: Option<String>,
    /// Whether this is a KeepKey device
    pub is_keepkey: bool,
    /// Additional KeepKey-specific info (if available and device is KeepKey)
    pub keepkey_info: Option<KeepKeyInfo>,
}

#[derive(serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsbDeviceInfo {
    vendor_id: u16,
    product_id: u16,
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
}

#[derive(serde::Serialize, ToSchema)]
pub struct HealthResponse {
    status: String,
    timestamp: String,
    service: String,
    version: String,
}

#[derive(serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KeepKeyInfo {
    pub label: Option<String>,
    pub device_id: Option<String>,
    pub firmware_version: String,
    pub revision: Option<String>,
    pub bootloader_hash: Option<String>,
    pub bootloader_version: Option<String>,
    pub initialized: bool,
    pub bootloader_mode: bool,
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse)
    ),
    tag = "system"
)]
pub async fn health_check() -> Json<HealthResponse> {
    // Read version from package.json
    let version = get_package_version().unwrap_or_else(|_| "0.1.0".to_string());
    
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        service: "KeepKey Desktop API".to_string(),
        version,
    })
}

/// Reads the version from the package.json file
fn get_package_version() -> Result<String, Box<dyn std::error::Error>> {
    // Read the package.json file
    let package_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Failed to find parent directory")?        
        .join("package.json");
    
    let package_content = fs::read_to_string(package_path)?;
    
    // Parse the JSON
    let package_json: Value = serde_json::from_str(&package_content)?;
    
    // Extract the version
    let version = package_json["version"]
        .as_str()
        .ok_or("Version not found in package.json")?;
    
    Ok(version.to_string())
}

#[utoipa::path(
    get,
    path = "/api/status",
    responses(
        (status = 200, description = "Device status retrieved successfully", body = DeviceStatus),
        (status = 500, description = "Internal server error")
    ),
    tag = "device"
)]
pub async fn device_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DeviceStatus>, StatusCode> {
    let _device_manager = state.server_state.device_manager.lock().await;
    
    // Get device features using the features module
    let status = match crate::features::get_device_features_impl() {
        Ok(device_features) => {
            // For device_status, we'll create a simplified DeviceInfo
            // In a real implementation, you might want to get the actual USB info
            let label = device_features.label.clone();
            DeviceStatus {
                connected: true,
                device: Some(DeviceInfo {
                    device_id: device_features.device_id.as_ref().map(|s| s.clone()).unwrap_or_else(|| "Unknown".to_string()),
                    name: label.clone().unwrap_or_else(|| "KeepKey".to_string()),
                    vendor_id: 0x2B24, // KeepKey VID
                    product_id: if device_features.bootloader_mode { 0x0002 } else { 0x0001 },
                    manufacturer: Some(device_features.vendor.unwrap_or_else(|| "ShapeShift".to_string())),
                    product: Some(device_features.model.unwrap_or_else(|| "KeepKey".to_string())),
                    serial_number: None,
                    is_keepkey: true,
                    keepkey_info: Some(KeepKeyInfo {
                        label,
                        device_id: device_features.device_id.clone(),
                        firmware_version: device_features.version,
                        revision: device_features.firmware_hash.clone(),
                        bootloader_hash: device_features.bootloader_hash.clone(),
                        bootloader_version: device_features.bootloader_version.clone(),
                        initialized: device_features.initialized,
                        bootloader_mode: device_features.bootloader_mode,
                    }),
                }),
            }
        }
        Err(_) => {
            DeviceStatus {
                connected: false,
                device: None,
            }
        }
    };
    
    Ok(Json(status))
}

#[utoipa::path(
    get,
    path = "/api/firmware",
    responses(
        (status = 200, description = "Firmware releases information from releases.json"),
        (status = 500, description = "Internal server error")
    ),
    tag = "firmware"
)]
pub async fn firmware_releases() -> Result<Json<serde_json::Value>, StatusCode> {
    match device_update::get_releases_data() {
        Ok(releases_data) => Ok(Json(releases_data)),
        Err(e) => {
            log::error!("Failed to load releases.json: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/devices",
    responses(
        (status = 200, description = "List of connected KeepKey devices", body = Vec<DeviceInfo>),
        (status = 500, description = "Internal server error")
    ),
    tag = "device"
)]
pub async fn list_devices(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<DeviceInfo>>, StatusCode> {
    // Read from the cached device registry - no scanning or blocking operations
    let entries = match crate::device_registry::get_all_device_entries() {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("Failed to get devices from registry: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    log::debug!("Registry has {} device entries", entries.len());
    
    // Convert registry entries to API response format
    let device_infos: Vec<DeviceInfo> = entries
        .into_iter()
        .filter(|entry| entry.device.is_keepkey)
        .map(|entry| {
            log::debug!("Processing device: {} ({})", entry.device.name, entry.device.unique_id);
            
            let keepkey_info = if let Some(features) = &entry.features {
                log::debug!("Device {} has features: version={}", entry.device.unique_id, features.version);
                Some(KeepKeyInfo {
                    label: features.label.clone(),
                    device_id: features.device_id.clone(),
                    firmware_version: features.version.clone(),
                    revision: features.firmware_hash.clone(),
                    bootloader_hash: features.bootloader_hash.clone(),
                    bootloader_version: features.bootloader_version.clone(),
                    initialized: features.initialized,
                    bootloader_mode: features.bootloader_mode,
                })
            } else {
                log::debug!("Device {} has no features yet", entry.device.unique_id);
                Some(KeepKeyInfo {
                    label: None,
                    device_id: None,
                    firmware_version: "Unknown".to_string(),
                    revision: None,
                    bootloader_hash: None,
                    bootloader_version: None,
                    initialized: false,
                    bootloader_mode: entry.device.pid == 0x0002,
                })
            };
            
            DeviceInfo {
                device_id: entry.device.unique_id,
                name: entry.device.name,
                vendor_id: entry.device.vid,
                product_id: entry.device.pid,
                manufacturer: entry.device.manufacturer,
                product: entry.device.product,
                serial_number: entry.device.serial_number,
                is_keepkey: true,
                keepkey_info,
            }
        })
        .collect();
    
    log::debug!("Returning {} KeepKey device(s)", device_infos.len());
    
    Ok(Json(device_infos))
}

#[utoipa::path(
    get,
    path = "/api/devices/debug",
    responses(
        (status = 200, description = "Debug USB device list"),
        (status = 500, description = "Internal server error")
    ),
    tag = "device"
)]
pub async fn debug_devices() -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    use rusb;
    let mut devices = Vec::new();
    
    match rusb::devices() {
        Ok(device_list) => {
            for device in device_list.iter() {
                match device.device_descriptor() {
                    Ok(desc) => {
                        let vid = desc.vendor_id();
                        let pid = desc.product_id();
                        
                        let device_info = serde_json::json!({
                            "vid": format!("{:04x}", vid),
                            "pid": format!("{:04x}", pid),
                            "bus": device.bus_number(),
                            "address": device.address(),
                            "is_keepkey": vid == 0x2B24,
                            "can_open": device.open().is_ok()
                        });
                        
                        devices.push(device_info);
                    }
                    Err(e) => {
                        log::error!("Failed to get device descriptor: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to enumerate USB devices: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    
    log::info!("Debug scan found {} total USB devices", devices.len());
    Ok(Json(devices))
}

#[utoipa::path(
    get,
    path = "/api/devices/registry",
    responses(
        (status = 200, description = "Current device registry state"),
        (status = 500, description = "Internal server error")
    ),
    tag = "device"
)]
pub async fn registry_status() -> Result<Json<serde_json::Value>, StatusCode> {
    let entries = match crate::device_registry::get_all_device_entries() {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("Failed to get registry entries: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let registry_info = serde_json::json!({
        "device_count": entries.len(),
        "devices": entries.iter().map(|entry| {
            serde_json::json!({
                "device_id": entry.device.unique_id,
                "name": entry.device.name,
                "vid": format!("{:04x}", entry.device.vid),
                "pid": format!("{:04x}", entry.device.pid),
                "has_features": entry.features.is_some(),
                "firmware_version": entry.features.as_ref().map(|f| &f.version),
                "initialized": entry.features.as_ref().map(|f| f.initialized),
                "last_updated": entry.last_updated,
                "time_since_update": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .saturating_sub(entry.last_updated)
            })
        }).collect::<Vec<_>>()
    });
    
    Ok(Json(registry_info))
}

// === V1 API Endpoints ===

/// V1 UTXO Address Generation - matches swagger.json specification
#[utoipa::path(
    post,
    path = "/addresses/utxo",
    request_body = UtxoAddressRequest,
    responses(
        (status = 200, description = "Bitcoin address generated", body = UtxoAddressResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Error processing request")
    ),
    tag = "Address"
)]
pub async fn utxo_get_address(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<UtxoAddressRequest>
) -> impl axum::response::IntoResponse {
    info!("🚀 V1 API: UTXO address request - coin: {}, script_type: {:?}, path: {:?}", 
        request.coin, request.script_type, request.address_n);
    
    // Use the existing implementation from impl_addresses.rs
    let cache = &app_state.device_cache;
    let device_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(()));
    
    match self::impl_addresses::generate_utxo_address_impl(request, cache, device_mutex).await {
        Ok(response) => {
            info!("✅ V1 API: Generated address: {}", response.address);
            // Return address data directly for SDK compatibility (not wrapped in ApiResponse)
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(self::impl_addresses::UtxoAddressError::NotCached) => {
            error!("❌ V1 API: Address not found in cache and device not available.");
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<UtxoAddressResponse> {
                    success: false,
                    data: None,
                    error: Some("Address not cached and device not available.".to_string()),
                })
            ).into_response()
        }
        Err(self::impl_addresses::UtxoAddressError::NoContext) => {
            error!("❌ V1 API: No device context set.");
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UtxoAddressResponse> {
                    success: false,
                    data: None,
                    error: Some("No device context set. Please select a device first using the /api/context endpoint.".to_string()),
                })
            ).into_response()
        }
        Err(self::impl_addresses::UtxoAddressError::DeviceNotFound(device_id)) => {
            error!("❌ V1 API: Device not found: {}", device_id);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<UtxoAddressResponse> {
                    success: false,
                    data: None,
                    error: Some(format!("Device not found: {}", device_id)),
                })
            ).into_response()
        }
        Err(e) => {
            error!("❌ V1 API: Unexpected error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UtxoAddressResponse> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                })
            ).into_response()
        }
    }
}

/// Create the v1 router with legacy endpoints for backward compatibility
pub fn v1_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/addresses/utxo", post(utxo_get_address))
        .route("/utxo/sign-transaction", post(utxo_sign_transaction))
        // Add more v1 endpoints as needed
}

#[utoipa::path(
    get,
    path = "/api/context",
    responses(
        (status = 200, description = "Get current device context", body = crate::server::context::ContextResponse)
    ),
    tag = "device"
)]
pub async fn api_get_context() -> axum::Json<crate::server::context::ContextResponse> {
    crate::server::context::get_context().await
}

#[utoipa::path(
    post,
    path = "/api/context",
    request_body = crate::server::context::SetContextRequest,
    responses(
        (status = 204, description = "Set device context")
    ),
    tag = "device"
)]
pub async fn api_set_context(payload: axum::Json<crate::server::context::SetContextRequest>) -> axum::http::StatusCode {
    crate::server::context::set_context(payload).await
}

#[utoipa::path(
    delete,
    path = "/api/context",
    responses(
        (status = 204, description = "Clear device context")
    ),
    tag = "device"
)]
pub async fn api_clear_context() -> axum::http::StatusCode {
    crate::server::context::clear_context().await
}

pub mod v2_endpoints;
pub mod impl_addresses;
pub mod auth;

// Only include impl_bitcoin since that's what we need for UTXO signing
pub mod impl_bitcoin;

// === UTXO Sign Transaction Types (SDK Compatible) ===

// Helper type to handle amounts that can be either strings or numbers
#[derive(Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum AmountValue {
    String(String),
    Number(u64),
}

impl AmountValue {
    fn as_string(&self) -> String {
        match self {
            AmountValue::String(s) => s.clone(),
            AmountValue::Number(n) => n.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoSignTransactionRequest {
    pub coin: String,
    pub inputs: Vec<UtxoInput>,
    pub outputs: Vec<UtxoOutput>,
    pub version: Option<u32>,
    pub locktime: Option<u32>,
    pub op_return_data: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoInput {
    pub address_n_list: Vec<u32>,  // SDK uses addressNList
    pub txid: String,
    pub vout: u32,
    pub amount: AmountValue,
    pub script_type: String,
    pub hex: Option<String>,
    pub tx: Option<PrevTransaction>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrevTransaction {
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<PrevTransactionInput>,
    pub vout: Vec<PrevTransactionOutput>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrevTransactionInput {
    pub txid: String,
    pub vout: u32,
    pub script_sig: ScriptSig,
    pub sequence: u32,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScriptSig {
    pub hex: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrevTransactionOutput {
    pub value: String,
    pub script_pub_key: ScriptPubKey,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPubKey {
    pub hex: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoOutput {
    pub address: String,
    pub amount: AmountValue,
    pub address_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoSignTransactionResponse {
    pub serialized_tx: String,
}

// Helper to serialize PrevTransaction to hex
fn serialize_prev_transaction(tx: &PrevTransaction) -> Result<String, anyhow::Error> {
    use std::io::Write;
    let mut buffer = Vec::new();

    buffer.write_all(&tx.version.to_le_bytes())?;
    write_varint(&mut buffer, tx.vin.len() as u64)?;

    for input in &tx.vin {
        let txid_bytes = hex::decode(&input.txid)?;
        buffer.write_all(&txid_bytes.iter().rev().cloned().collect::<Vec<u8>>())?;
        buffer.write_all(&input.vout.to_le_bytes())?;

        let script_bytes = hex::decode(&input.script_sig.hex)?;
        write_varint(&mut buffer, script_bytes.len() as u64)?;
        buffer.write_all(&script_bytes)?;

        buffer.write_all(&input.sequence.to_le_bytes())?;
    }

    write_varint(&mut buffer, tx.vout.len() as u64)?;
    for output in &tx.vout {
        let value: u64 = output.value.parse()?;
        buffer.write_all(&value.to_le_bytes())?;
        let script_bytes = hex::decode(&output.script_pub_key.hex)?;
        write_varint(&mut buffer, script_bytes.len() as u64)?;
        buffer.write_all(&script_bytes)?;
    }

    buffer.write_all(&tx.locktime.to_le_bytes())?;
    Ok(hex::encode(buffer))
}

fn write_varint(buffer: &mut Vec<u8>, n: u64) -> std::io::Result<()> {
    use std::io::Write;
    if n < 0xfd {
        buffer.write_all(&[n as u8])?;
    } else if n <= 0xffff {
        buffer.write_all(&[0xfd])?;
        buffer.write_all(&(n as u16).to_le_bytes())?;
    } else if n <= 0xffffffff {
        buffer.write_all(&[0xfe])?;
        buffer.write_all(&(n as u32).to_le_bytes())?;
    } else {
        buffer.write_all(&[0xff])?;
        buffer.write_all(&n.to_le_bytes())?;
    }
    Ok(())
}

/// UTXO sign transaction endpoint (SDK compatible)
#[utoipa::path(
    post,
    path = "/utxo/sign-transaction",
    request_body = UtxoSignTransactionRequest,
    responses(
        (status = 200, description = "Transaction signed successfully", body = UtxoSignTransactionResponse),
        (status = 404, description = "No KeepKey device found"),
        (status = 422, description = "Invalid request data"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "utxo"
)]
pub async fn utxo_sign_transaction(
    State(_app_state): State<Arc<AppState>>,
    Json(request): Json<UtxoSignTransactionRequest>,
) -> Result<Json<UtxoSignTransactionResponse>, StatusCode> {
    info!("🚀 V1 API: utxo/sign-transaction request ({} inputs, {} outputs)", 
        request.inputs.len(), request.outputs.len());

    // Convert SDK format to internal BitcoinSignRequest format
    let mut inputs = Vec::new();
    for (idx, input) in request.inputs.iter().enumerate() {
        // Get the hex for the previous transaction
        let prev_tx_hex = if let Some(hex) = &input.hex {
            // Direct hex provided
            hex.clone()
        } else if let Some(tx) = &input.tx {
            // Convert tx object to hex
            match serialize_prev_transaction(tx) {
                Ok(hex) => hex,
                Err(e) => {
                    error!("Failed to serialize previous transaction at input {}: {}", idx, e);
                    return Err(StatusCode::UNPROCESSABLE_ENTITY);
                }
            }
        } else {
            error!("Input {} missing both hex and tx fields", idx);
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        };

        inputs.push(BitcoinInput {
            address_n: input.address_n_list.clone(),
            prev_hash: input.txid.clone(),
            prev_index: input.vout,
            amount: input.amount.as_string(),
            script_type: input.script_type.clone(),
            hex: Some(prev_tx_hex),
        });
    }

    let mut outputs = Vec::new();
    for output in &request.outputs {
        // Determine script type based on address prefix
        let script_type = if output.address.starts_with("bc1q") {
            "p2wpkh".to_string()
        } else if output.address.starts_with("bc1p") {
            "p2tr".to_string()
        } else if output.address.starts_with('3') {
            "p2sh".to_string()
        } else if output.address.starts_with('1') {
            "p2pkh".to_string()
        } else {
            warn!("Unknown address format for {}, defaulting to p2pkh", output.address);
            "p2pkh".to_string()
        };

        outputs.push(BitcoinOutput {
            address: Some(output.address.clone()),
            address_n: None,
            amount: output.amount.as_string(),
            script_type,
        });
    }

    let bitcoin_request = BitcoinSignRequest {
        tx_hex: "".to_string(), // Not used in our implementation
        inputs,
        outputs,
    };

    // Log the request for debugging
    if let Ok(json) = serde_json::to_string_pretty(&bitcoin_request) {
        info!("🔍 Bitcoin request body:\n{}", json);
    }

    // Use the existing fresh implementation
    match super::routes::impl_bitcoin::bitcoin_sign_tx_fresh_impl(bitcoin_request).await {
        Ok(response) => {
            info!("✅ Transaction signed successfully");
            Ok(Json(UtxoSignTransactionResponse {
                serialized_tx: response.serialized_tx,
            }))
        }
        Err(e) => {
            error!("❌ Failed to sign transaction: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

// === Bitcoin Transaction Types ===

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BitcoinSignRequest {
    pub tx_hex: String,
    pub inputs: Vec<BitcoinInput>,
    pub outputs: Vec<BitcoinOutput>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BitcoinInput {
    pub address_n: Vec<u32>,
    pub prev_hash: String,
    pub prev_index: u32,
    pub amount: String,
    pub script_type: String,
    pub hex: Option<String>, // Optional previous transaction hex
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BitcoinOutput {
    pub address: Option<String>,
    pub address_n: Option<Vec<u32>>,
    pub amount: String,
    pub script_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BitcoinSignResponse {
    pub signatures: Vec<String>,  // Hex-encoded signatures for each input
    pub serialized_tx: String,    // Hex-encoded serialized transaction
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BitcoinSignMessageRequest {
    pub address_n: Vec<u32>,
    pub message: String,
    pub coin: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BitcoinSignMessageResponse {
    pub address: String,
    pub signature: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BitcoinVerifyMessageRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
    pub coin: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BitcoinVerifyMessageResponse {
    pub valid: bool,
}

