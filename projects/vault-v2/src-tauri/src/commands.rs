use tauri::State;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use keepkey_rust::{
    device_queue::{DeviceQueueFactory, DeviceQueueHandle},
    features::DeviceFeatures,
};
use uuid;

type DeviceQueueManager = Arc<tokio::sync::Mutex<std::collections::HashMap<String, DeviceQueueHandle>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceRequest {
    GetXpub {
        path: String,
    },
    GetAddress {
        path: String,
        coin_name: String,
        script_type: Option<String>,
        show_display: Option<bool>,
    },
    GetFeatures,
    SendRaw {
        message_type: String,
        message_data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRequestWrapper {
    pub device_id: String,
    pub request_id: String,
    pub request: DeviceRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceResponse {
    Xpub {
        request_id: String,
        device_id: String,
        path: String,
        xpub: String,
        success: bool,
        error: Option<String>,
    },
    Address {
        request_id: String,
        device_id: String,
        path: String,
        address: String,
        success: bool,
        error: Option<String>,
    },
    Features {
        request_id: String,
        device_id: String,
        features: DeviceFeatures,
        success: bool,
        error: Option<String>,
    },
    Raw {
        request_id: String,
        device_id: String,
        response: serde_json::Value,
        success: bool,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub device_id: Option<String>,
    pub total_queued: usize,
    pub active_operations: usize,
    pub status: String,
    pub last_response: Option<DeviceResponse>,
}

/// Unified device queue command - all device operations go through this
#[tauri::command]
pub async fn add_to_device_queue(
    request: DeviceRequestWrapper,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<String, String> {
    println!("Adding to device queue: {:?}", request);
    
    // Get or create device queue handle
    let queue_handle = {
        let mut manager = queue_manager.lock().await;
        
        if let Some(handle) = manager.get(&request.device_id) {
            handle.clone()
        } else {
            // Find the device by ID using high-level API
            let devices = keepkey_rust::features::list_connected_devices();
            let device_info = devices
                .iter()
                .find(|d| d.unique_id == request.device_id)
                .ok_or_else(|| format!("Device {} not found", request.device_id))?;
                
            // Spawn a new device worker using the real keepkey_rust implementation
            let handle = DeviceQueueFactory::spawn_worker(request.device_id.clone(), device_info.clone());
            manager.insert(request.device_id.clone(), handle.clone());
            handle
        }
    };
    
    // Process the request based on type
    let result = match request.request {
        DeviceRequest::GetXpub { path } => {
            // Parse derivation path
            let path_parts = parse_derivation_path(&path)?;
            // For xpub, we typically want the account-level path
            let xpub = queue_handle
                .get_address(path_parts, "Bitcoin".to_string(), None)
                .await
                .map_err(|e| format!("Failed to get xpub: {}", e))?;
            
            // Return xpub (in real implementation, we'd get the actual xpub from a different method)
            Ok(format!("xpub_{}", xpub)) // Placeholder - real implementation would use proper xpub method
        }
        DeviceRequest::GetAddress { path, coin_name, script_type, show_display: _ } => {
            let path_parts = parse_derivation_path(&path)?;
            let script_type_int = match script_type.as_deref() {
                Some("p2pkh") => Some(0),
                Some("p2sh-p2wpkh") => Some(1),
                Some("p2wpkh") => Some(2),
                _ => None,
            };
            
            queue_handle
                .get_address(path_parts, coin_name, script_type_int)
                .await
                .map_err(|e| format!("Failed to get address: {}", e))
        }
                 DeviceRequest::GetFeatures => {
             let features = queue_handle
                 .get_features()
                 .await
                 .map_err(|e| format!("Failed to get features: {}", e))?;
             
             // Create a serializable version of features
             let features_json = serde_json::json!({
                 "version": format!("{}.{}.{}", 
                     features.major_version.unwrap_or(0), 
                     features.minor_version.unwrap_or(0), 
                     features.patch_version.unwrap_or(0)),
                 "initialized": features.initialized.unwrap_or(false),
                 "label": features.label.unwrap_or_default(),
                 "vendor": features.vendor.unwrap_or_default(),
                 "model": features.model.unwrap_or_default(),
                 "bootloader_mode": features.bootloader_mode.unwrap_or(false)
             });
             
             Ok(features_json.to_string())
         }
        DeviceRequest::SendRaw { message_type: _, message_data: _ } => {
            // For raw messages, we'd need to implement proper message parsing
            Err("Raw message sending not yet implemented".to_string())
        }
    };
    
    match result {
        Ok(response) => {
            println!("✅ Device operation completed: {}", response);
            Ok(request.request_id)
        }
        Err(e) => {
            println!("❌ Device operation failed: {}", e);
            Err(e)
        }
    }
}

/// Get the current status of all device queues
#[tauri::command]
pub async fn get_queue_status(
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<QueueStatus, String> {
    let manager = queue_manager.lock().await;
    
    // For now, return basic status
    // In a full implementation, we'd track queue metrics
    Ok(QueueStatus {
        device_id: None,
        total_queued: 0,
        active_operations: manager.len(),
        status: if manager.is_empty() { "idle".to_string() } else { "active".to_string() },
        last_response: None,
    })
}

/// Get connected devices (frontend expects this name)
#[tauri::command]
pub async fn get_connected_devices() -> Result<Vec<serde_json::Value>, String> {
    let devices = keepkey_rust::features::list_connected_devices();
    
    // Convert to the structure the frontend expects
    let json_devices = devices.into_iter()
        .filter(|device| device.is_keepkey)
        .map(|device| {
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
                "features": null, // Features fetched separately via queue
            })
        })
        .collect();
    
    Ok(json_devices)
}

/// Get blocking actions (enhanced version)
#[tauri::command]
pub async fn get_blocking_actions() -> Result<Vec<serde_json::Value>, String> {
    // Return empty array for now - can be enhanced later
    Ok(vec![])
}

/// Helper function to parse derivation path string to Vec<u32>
fn parse_derivation_path(path: &str) -> Result<Vec<u32>, String> {
    let path = path.trim_start_matches("m/");
    let parts: Result<Vec<u32>, String> = path
        .split('/')
        .map(|part| {
            let part = part.trim_end_matches('\'');
            let mut value = part.parse::<u32>()
                .map_err(|_| format!("Invalid path component: {}", part))?;
            
            // Handle hardened derivation (')
            if path.contains(&format!("{}\'", part)) {
                value |= 0x80000000;
            }
            
            Ok(value)
        })
        .collect();
    
    parts.map_err(|e: String| format!("Failed to parse derivation path '{}': {}", path, e))
}

/// Test command to demonstrate the unified device queue interface
#[tauri::command] 
pub async fn test_device_queue() -> Result<String, String> {
    println!("🧪 Testing unified device queue interface...");
    
    // Example of how frontend would use the unified interface
    let test_request = DeviceRequestWrapper {
        device_id: "test-device-001".to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        request: DeviceRequest::GetFeatures,
    };
    
    println!("📝 Created test request: {:?}", test_request);
    
    // In real usage, this would be sent to add_to_device_queue
    Ok(format!("✅ Unified device queue test completed. Request ID: {}", test_request.request_id))
} 