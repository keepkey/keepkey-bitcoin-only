use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use tracing::{info, error};

use crate::server::ServerState;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub connected: bool,
    pub device: Option<DeviceInfo>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device_id: String,
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub is_keepkey: bool,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

// KeepKey SDK compatible Features structure
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Features {
    pub vendor: Option<String>,
    pub major_version: Option<u32>,
    pub minor_version: Option<u32>,
    pub patch_version: Option<u32>,
    pub bootloader_mode: Option<bool>,
    pub device_id: Option<String>,
    pub pin_protection: Option<bool>,
    pub passphrase_protection: Option<bool>,
    pub language: Option<String>,
    pub label: Option<String>,
    pub initialized: Option<bool>,
    pub revision: Option<String>,
    pub firmware_hash: Option<String>,
    pub bootloader_hash: Option<String>,
    pub imported: Option<bool>,
    pub pin_cached: Option<bool>,
    pub passphrase_cached: Option<bool>,
    pub wipe_code_protection: Option<bool>,
    pub auto_lock_delay_ms: Option<u32>,
    pub policies: Option<Vec<Policy>>,
    pub model: Option<String>,
    pub firmware_variant: Option<String>,
    pub no_backup: Option<bool>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Policy {
    pub policy_name: String,
    pub enabled: bool,
}

// Legacy KeepKeyFeatures for backward compatibility
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KeepKeyFeatures {
    pub vendor: Option<String>,
    pub major_version: u32,
    pub minor_version: u32,
    pub patch_version: u32,
    pub bootloader_mode: bool,
    pub device_id: Option<String>,
    pub pin_protection: bool,
    pub passphrase_protection: bool,
    pub language: Option<String>,
    pub label: Option<String>,
    pub initialized: bool,
    pub revision: Option<String>,
    pub bootloader_hash: Option<String>,
    pub imported: bool,
    pub unlocked: bool,
    pub firmware_present: bool,
    pub needs_backup: bool,
    pub flags: u32,
    pub model: Option<String>,
    pub fw_major: Option<u32>,
    pub fw_minor: Option<u32>,
    pub fw_patch: Option<u32>,
    pub fw_vendor: Option<String>,
    pub fw_vendor_keys: Option<String>,
    pub unfinished_backup: bool,
    pub no_backup: bool,
}

// Route handlers
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
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<DeviceStatus>, StatusCode> {
    match crate::server::get_device_status_impl().await {
        Ok(status) => Ok(Json(status)),
        Err(e) => {
            error!("Failed to get device status: {}", e);
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
    let devices = crate::server::list_devices();
    let mut device_infos = Vec::new();
    
    for (index, device) in devices.iter().enumerate() {
        if let Ok(device_desc) = device.device_descriptor() {
            let vendor_id = device_desc.vendor_id();
            let product_id = device_desc.product_id();
            
            // Try to get string descriptors
            let mut manufacturer = None;
            let mut product = None;
            let mut serial_number = None;
            
            if let Ok(handle) = device.open() {
                if let Ok(mfg) = handle.read_manufacturer_string_ascii(&device_desc) {
                    manufacturer = Some(mfg);
                }
                if let Ok(prod) = handle.read_product_string_ascii(&device_desc) {
                    product = Some(prod);
                }
                if let Ok(serial) = handle.read_serial_number_string_ascii(&device_desc) {
                    serial_number = Some(serial);
                }
            }
            
            device_infos.push(DeviceInfo {
                device_id: format!("keepkey-{}", index),
                name: format!("KeepKey #{}", index + 1),
                vendor_id,
                product_id,
                manufacturer,
                product,
                serial_number,
                is_keepkey: true,
            });
        }
    }
    
    info!("Found {} KeepKey device(s)", device_infos.len());
    Ok(Json(device_infos))
}

#[utoipa::path(
    get,
    path = "/api/devices/usb",
    responses(
        (status = 200, description = "List of all USB devices", body = Vec<UsbDeviceInfo>),
        (status = 500, description = "Internal server error")
    ),
    tag = "device"
)]
pub async fn list_usb_devices(
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<Vec<UsbDeviceInfo>>, StatusCode> {
    match crate::server::list_usb_devices_impl().await {
        Ok(devices) => {
            info!("Found {} USB device(s)", devices.len());
            Ok(Json(devices))
        }
        Err(e) => {
            error!("Failed to list USB devices: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Legacy endpoint for backward compatibility
#[utoipa::path(
    get,
    path = "/api/devices/features",
    responses(
        (status = 200, description = "KeepKey device features", body = KeepKeyFeatures),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "device"
)]
pub async fn get_device_features(
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<KeepKeyFeatures>, StatusCode> {
    match crate::server::get_device_features_impl().await {
        Ok(features) => {
            info!("Retrieved device features: version {}.{}.{}", 
                features.major_version, features.minor_version, features.patch_version);
            Ok(Json(features))
        }
        Err(e) => {
            error!("Failed to get device features: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

// Get device features (SDK compatible format)
#[utoipa::path(
    post,
    path = "/system/info/get-features",
    responses(
        (status = 200, description = "Device features retrieved successfully", body = Features),
        (status = 500, description = "Internal server error")
    ),
    tag = "device"
)]
pub async fn get_features_sdk_handler(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Features>, StatusCode> {
    match crate::server::get_features_sdk_compatible(&state.cache).await {
        Ok(features) => {
            info!("✅ Retrieved device features from cache");
            Ok(Json(features))
        }
        Err(e) => {
            error!("Failed to get device features: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
} 