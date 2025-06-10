use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use utoipa::ToSchema;
use std::fs;
use serde_json::Value;
use log;

use super::ServerState;
use crate::device_update;

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
    State(state): State<Arc<ServerState>>,
) -> Result<Json<DeviceStatus>, StatusCode> {
    let _device_manager = state.device_manager.lock().await;
    
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
    State(_state): State<Arc<ServerState>>,
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