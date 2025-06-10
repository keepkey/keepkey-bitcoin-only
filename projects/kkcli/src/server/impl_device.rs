use anyhow::Result;
use tokio::time::timeout;
use axum::extract::State;
use std::sync::Arc;
use crate::server::ServerState;
use crate::messages::{self, Message, Features as ProtosFeatures, GetFeatures};
use crate::transport::ProtocolAdapter;
use tracing::{info, error, warn};
use rusb::{Device, GlobalContext};

use crate::transport::UsbTransport;
use crate::server::routes;
use crate::server::cache::DeviceCache;
use crate::server::{DEVICE_IDS, DEVICE_OPERATION_TIMEOUT, try_get_device};

// Device status implementation
pub(crate) async fn get_device_status_impl() -> Result<routes::DeviceStatus> {
    match try_get_device() {
        Ok(_device) => {
            Ok(routes::DeviceStatus {
                connected: true,
                device: Some(routes::DeviceInfo {
                    device_id: "kkcli-device".to_string(),
                    name: "KeepKey via CLI".to_string(),
                    vendor_id: 0x2B24,
                    product_id: 0x0001,
                    manufacturer: Some("ShapeShift".to_string()),
                    product: Some("KeepKey".to_string()),
                    serial_number: None,
                    is_keepkey: true,
                }),
            })
        }
        Err(_) => {
            Ok(routes::DeviceStatus {
                connected: false,
                device: None,
            })
        }
    }
}

// Device features implementation
pub(crate) async fn get_device_features_impl(State(server_state): State<Arc<ServerState>>) -> Result<routes::KeepKeyFeatures> {
    info!("Attempting to get device features using shared transport...");

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let get_features_msg = messages::GetFeatures {};
            let response = transport.with_standard_handler().handle(get_features_msg.into()).map_err(|e| {
                error!("Error sending GetFeatures: {:?}", e);
                anyhow::anyhow!("Failed to send GetFeatures: {}", e)
            })?;

            match response {
                Message::Features(features_msg) => {
                    info!("Successfully received Features from device.");
                    // Map protobuf Features to routes::KeepKeyFeatures
                    Ok(routes::KeepKeyFeatures {
                        vendor: features_msg.vendor.clone().or_else(|| Some("keepkey.com".to_string())),
                        major_version: features_msg.major_version.unwrap_or_default(),
                        minor_version: features_msg.minor_version.unwrap_or_default(),
                        patch_version: features_msg.patch_version.unwrap_or_default(),
                        bootloader_mode: features_msg.bootloader_mode.unwrap_or_default(),
                        device_id: features_msg.device_id,
                        pin_protection: features_msg.pin_protection.unwrap_or_default(),
                        passphrase_protection: features_msg.passphrase_protection.unwrap_or_default(),
                        language: features_msg.language,
                        label: features_msg.label,
                        initialized: features_msg.initialized.unwrap_or_default(),
                        revision: features_msg.revision.map(hex::encode),
                        bootloader_hash: features_msg.bootloader_hash.map(hex::encode),
                        imported: features_msg.imported.unwrap_or_default(),
                        unlocked: features_msg.pin_cached.unwrap_or(false) || features_msg.passphrase_cached.unwrap_or(false),
                        firmware_present: true, // If we got features, firmware is present
                        needs_backup: features_msg.no_backup.map(|nb| !nb).unwrap_or(true), // Assuming no_backup=false means needs_backup=true
                        // flags: features_msg.flags.unwrap_or_default(), // Not in protos::Features
                        flags: 0, // Default for legacy, not in protos::Features
                        model: features_msg.model,
                        // fw_major: features_msg.fw_major, // Not in protos::Features
                        fw_major: features_msg.major_version, // Use existing major_version
                        // fw_minor: features_msg.fw_minor, // Not in protos::Features
                        fw_minor: features_msg.minor_version, // Use existing minor_version
                        // fw_patch: features_msg.fw_patch, // Not in protos::Features
                        fw_patch: features_msg.patch_version, // Use existing patch_version
                        // fw_vendor: features_msg.fw_vendor, // Not in protos::Features, compiler suggested features_msg.vendor
                        fw_vendor: features_msg.vendor.clone(),
                        // fw_vendor_keys: features_msg.fw_vendor_keys.map(hex::encode), // Not in protos::Features
                        fw_vendor_keys: None, // Not in protos::Features
                        // unfinished_backup: features_msg.unfinished_backup.unwrap_or_default(), // Not in protos::Features
                        unfinished_backup: false, // Default for legacy, not in protos::Features
                        no_backup: features_msg.no_backup.unwrap_or_default(), // Retain original no_backup field as well
                    })
                }
                unexpected_msg => {
                    error!("Unexpected response to GetFeatures: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for GetFeatures.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(features)) => Ok(features),
        Ok(Err(e)) => {
            error!("Failed to get device features: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("Get device features timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

// SDK compatible features implementation - now using cache!
pub(crate) async fn get_features_sdk_compatible(cache: &DeviceCache) -> Result<routes::Features> {
    // First try to get from cache
    if let Some(cached_features) = cache.get_cached_features() {
        info!("✨ Returning cached device features");
        
        // Parse the features JSON back to our Features struct
        let features: routes::Features = serde_json::from_str(&cached_features.features_json)?;
        return Ok(features);
    }
    
    // This should not happen if frontloading worked correctly
    error!("❌ No cached features found - this should not happen!");
    Err(anyhow::anyhow!("Device features not found in cache"))
}

// List USB devices
pub(crate) async fn list_usb_devices_impl() -> Result<Vec<routes::UsbDeviceInfo>> {
    let devices = rusb::devices()?;
    let mut device_list = Vec::new();
    
    for device in devices.iter() {
        let device_desc = device.device_descriptor()?;
        let vendor_id = device_desc.vendor_id();
        let product_id = device_desc.product_id();
        
        let mut manufacturer = None;
        let mut product = None;
        let mut serial_number = None;
        
        // Try to get string descriptors
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
        
        device_list.push(routes::UsbDeviceInfo {
            vendor_id,
            product_id,
            manufacturer,
            product,
            serial_number,
        });
    }
    
    Ok(device_list)
}

// Safe device list function
pub(crate) fn list_devices() -> Box<[Device<GlobalContext>]> {
    match rusb::devices() {
        Ok(devices) => {
            devices.iter()
                .filter_map(|device| {
                    match device.device_descriptor() {
                        Ok(device_desc) => {
                            let vendor_id = device_desc.vendor_id();
                            let product_id = device_desc.product_id();
                            if DEVICE_IDS.contains(&(vendor_id, product_id)) {
                                Some(device)
                            } else {
                                None
                            }
                        }
                        Err(_) => None, // Skip devices we can't read
                    }
                })
                .collect()
        }
        Err(e) => {
            warn!("Failed to list USB devices: {}", e);
            vec![].into_boxed_slice()
        }
    }
} 