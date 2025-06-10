use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use crate::features::DeviceFeatures;
use crate::usb_manager::FriendlyUsbDevice;

// Define a struct to hold device data including features
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeviceEntry {
    pub device: FriendlyUsbDevice,
    pub features: Option<DeviceFeatures>,
    pub last_updated: u64, // Unix timestamp in seconds
}

// Global device registry - this replaces the single DEVICE_FEATURES
pub static DEVICE_REGISTRY: Lazy<Arc<Mutex<HashMap<String, DeviceEntry>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

// Helper functions for working with the registry
pub fn add_or_update_device(device: FriendlyUsbDevice, features: Option<DeviceFeatures>) -> Result<(), String> {
    let mut registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    
    registry.insert(device.unique_id.clone(), DeviceEntry {
        device,
        features,
        last_updated: timestamp,
    });
    
    Ok(())
}

pub fn remove_device(device_id: &str) -> Result<bool, String> {
    let mut registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    Ok(registry.remove(device_id).is_some())
}

pub fn get_all_devices() -> Result<Vec<FriendlyUsbDevice>, String> {
    let registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    Ok(registry.values().map(|entry| entry.device.clone()).collect())
}

pub fn get_device_features(device_id: &str) -> Result<Option<DeviceFeatures>, String> {
    let registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    Ok(registry.get(device_id).and_then(|entry| entry.features.clone()))
}

pub fn get_all_device_entries() -> Result<Vec<DeviceEntry>, String> {
    let registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    Ok(registry.values().cloned().collect())
}

// Get the first connected device's features (for backward compatibility)
pub fn get_first_device_features() -> Result<Option<DeviceFeatures>, String> {
    let registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    Ok(registry.values()
        .find(|entry| entry.device.is_keepkey)
        .and_then(|entry| entry.features.clone()))
}

// Clear all devices from the registry
pub fn clear_registry() -> Result<(), String> {
    let mut registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    registry.clear();
    Ok(())
} 