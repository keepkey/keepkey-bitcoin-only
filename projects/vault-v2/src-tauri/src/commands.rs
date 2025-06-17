use tauri::State;
use std::sync::Arc;
use keepkey_rust::{
    device_queue::{DeviceQueueFactory, DeviceQueueHandle},
    friendly_usb::FriendlyUsbDevice,
    features::DeviceFeatures,
};

type DeviceQueueManager = Arc<tokio::sync::Mutex<std::collections::HashMap<String, DeviceQueueHandle>>>;

#[tauri::command]
pub async fn list_connected_devices() -> Result<Vec<FriendlyUsbDevice>, String> {
    let devices = keepkey_rust::features::list_connected_devices();
    println!("Found {} connected devices", devices.len());
    Ok(devices)
}

#[tauri::command]
pub async fn get_device_features(
    device_id: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<DeviceFeatures, String> {
    println!("Getting features for device: {}", device_id);
    
    // Get or create device queue handle
    let queue_handle = {
        let mut manager = queue_manager.lock().await;
        
        if let Some(handle) = manager.get(&device_id) {
            handle.clone()
        } else {
            // Find the device by ID using high-level API
            let devices = keepkey_rust::features::list_connected_devices();
            let device_info = devices
                .iter()
                .find(|d| d.unique_id == device_id)
                .ok_or_else(|| format!("Device {} not found", device_id))?;
                
            // Spawn a new device worker
            let handle = DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
            manager.insert(device_id.clone(), handle.clone());
            handle
        }
    };
    
    // Get features using the real device queue
    let features = queue_handle
        .get_features()
        .await
        .map_err(|e| format!("Failed to get device features: {}", e))?;
        
    // Convert keepkey_rust Features to DeviceFeatures
    let device_features = DeviceFeatures {
        label: features.label,
        vendor: features.vendor,
        model: features.model,
        firmware_variant: features.firmware_variant,
        device_id: features.device_id,
        language: features.language,
        bootloader_mode: features.bootloader_mode.unwrap_or(false),
        version: format!(
            "{}.{}.{}",
            features.major_version.unwrap_or(0),
            features.minor_version.unwrap_or(0),
            features.patch_version.unwrap_or(0)
        ),
        firmware_hash: features.firmware_hash.map(|bytes| hex::encode(bytes)),
        bootloader_hash: features.bootloader_hash.clone().map(|bytes| hex::encode(bytes)),
        bootloader_version: features.bootloader_hash.map(|bytes| hex::encode(bytes)),
        initialized: features.initialized.unwrap_or(false),
        imported: features.imported,
        no_backup: features.no_backup.unwrap_or(false),
        pin_protection: features.pin_protection.unwrap_or(false),
        pin_cached: features.pin_cached.unwrap_or(false),
        passphrase_protection: features.passphrase_protection.unwrap_or(false),
        passphrase_cached: features.passphrase_cached.unwrap_or(false),
        wipe_code_protection: features.wipe_code_protection.unwrap_or(false),
        auto_lock_delay_ms: features.auto_lock_delay_ms.map(|ms| ms as u64),
        policies: features
            .policies
            .into_iter()
            .filter(|p| p.enabled())
            .map(|p| p.policy_name().to_string())
            .collect(),
    };
    
    Ok(device_features)
}

#[tauri::command]
pub async fn get_device_address(
    device_id: String,
    path: Vec<u32>,
    coin_name: String,
    script_type: Option<i32>,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<String, String> {
    println!("Getting address for device: {}, path: {:?}", device_id, path);
    
    // Get or create device queue handle
    let queue_handle = {
        let mut manager = queue_manager.lock().await;
        
        if let Some(handle) = manager.get(&device_id) {
            handle.clone()
        } else {
            // Find the device by ID using high-level API
            let devices = keepkey_rust::features::list_connected_devices();
            let device_info = devices
                .iter()
                .find(|d| d.unique_id == device_id)
                .ok_or_else(|| format!("Device {} not found", device_id))?;
                
            // Spawn a new device worker
            let handle = DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
            manager.insert(device_id.clone(), handle.clone());
            handle
        }
    };
    
    // Get address using the real device queue
    queue_handle
        .get_address(path, coin_name, script_type)
        .await
        .map_err(|e| format!("Failed to get device address: {}", e))
} 