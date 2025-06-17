use tauri::State;
use tauri::Manager;
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
    for device in &devices {
        println!("  - {} (VID: 0x{:04x}, PID: 0x{:04x}) - {} {}", 
                 device.unique_id, device.vid, device.pid,
                 device.manufacturer.as_deref().unwrap_or("Unknown"),
                 device.product.as_deref().unwrap_or("Unknown"));
    }
    Ok(devices)
}

#[tauri::command]
pub async fn debug_device_communication(device_id: String) -> Result<String, String> {
    println!("🔍 Debug: Testing communication with device {}", device_id);
    
    // Get device info
    let devices = keepkey_rust::features::list_connected_devices();
    let device = devices
        .iter()
        .find(|d| d.unique_id == device_id)
        .ok_or_else(|| format!("Device {} not found in enumeration", device_id))?;
    
    println!("🔍 Debug: Device info - VID: 0x{:04x}, PID: 0x{:04x}, IsKeepKey: {}", 
             device.vid, device.pid, device.is_keepkey);
    
    // Try to get features with detailed error reporting
    match keepkey_rust::features::get_device_features_with_fallback(device) {
        Ok(features) => {
            let result = format!("✅ SUCCESS: Device {} responded with firmware v{}, initialized: {}", 
                                device_id, features.version, features.initialized);
            println!("{}", result);
            Ok(result)
        }
        Err(e) => {
            let error_msg = format!("❌ FAILED: Device {} communication error: {}", device_id, e);
            println!("{}", error_msg);
            Ok(error_msg) // Return as Ok so frontend gets the debug info
        }
    }
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

/// Get connected devices (frontend expects this name, not list_connected_devices)
#[tauri::command]
pub async fn get_connected_devices() -> Result<Vec<serde_json::Value>, String> {
    let devices = keepkey_rust::features::list_connected_devices();
    
    // Convert to the structure the frontend expects
    // NOTE: We don't fetch features here to avoid crashes - features should be fetched separately
    let json_devices = devices.into_iter()
        .filter(|device| device.is_keepkey)
        .map(|device| {
            // Create structure that matches what the frontend expects
            // Features will be null initially - they should be fetched via separate calls
            serde_json::json!({
                "device": {
                    "unique_id": device.unique_id,
                    "name": device.name,
                    "vid": device.vid,
                    "pid": device.pid,
                    "manufacturer": device.manufacturer,
                    "product": device.product,
                    "serial_number": device.serial_number,
                    "is_keepkey": device.is_keepkey,
                },
                "features": null, // Don't fetch features during enumeration to avoid crashes
            })
        })
        .collect();
    
    Ok(json_devices)
}

/// Get queue status (needed by frontend)
#[tauri::command]
pub async fn get_queue_status() -> Result<serde_json::Value, String> {
    // Return empty queue status for now
    Ok(serde_json::json!({
        "total_queued": 0,
        "active_operations": 0,
        "status": "idle"
    }))
}

/// Get blocking actions (enhanced version)
#[tauri::command]
pub async fn get_blocking_actions() -> Result<Vec<serde_json::Value>, String> {
    // Return empty array for now - can be enhanced later
    Ok(vec![])
}

/// Get device features by device ID (safe individual fetch)
#[tauri::command]
pub async fn get_device_features_by_id(device_id: String) -> Result<Option<DeviceFeatures>, String> {
    println!("🔍 Getting features for device: {}", device_id);
    
    // Use the safe high-level API
    match keepkey_rust::features::get_device_features_by_id(&device_id) {
        Ok(features) => {
            println!("✅ Successfully got features for device {}: firmware v{}", device_id, features.version);
            Ok(Some(features))
        }
        Err(e) => {
            println!("⚠️ Failed to get features for device {}: {}", device_id, e);
            // Return None instead of error to avoid frontend crashes
            Ok(None)
        }
    }
}

/// Shutdown background tasks manually (useful for preventing conflicts)
#[tauri::command]
pub async fn shutdown_background_tasks(
    app: tauri::AppHandle,
) -> Result<(), String> {
    println!("🛑 Manual shutdown of background tasks requested");
    
    // Stop event controller if it exists in app state
    if let Some(controller_state) = app.try_state::<std::sync::Arc<std::sync::Mutex<crate::event_controller::EventController>>>() {
        if let Ok(mut controller) = controller_state.lock() {
            controller.stop();
            println!("✅ Event controller stopped manually");
            Ok(())
        } else {
            Err("Failed to lock event controller for shutdown".to_string())
        }
    } else {
        Err("Event controller not found in app state".to_string())
    }
} 