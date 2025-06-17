use tauri::State;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use keepkey_rust::{
    device_queue::{DeviceQueueFactory, DeviceQueueHandle},
    features::DeviceFeatures,
};
use uuid;
use hex;
use crate::logging::{log_device_request, log_device_response, log_raw_device_message};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_id: String,
    pub connected: bool,
    pub features: Option<DeviceFeatures>,
    pub needs_bootloader_update: bool,
    pub needs_firmware_update: bool,
    pub needs_initialization: bool,
    pub bootloader_check: Option<BootloaderCheck>,
    pub firmware_check: Option<FirmwareCheck>,
    pub initialization_check: Option<InitializationCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootloaderCheck {
    pub current_version: String,
    pub latest_version: String,
    pub needs_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareCheck {
    pub current_version: String,
    pub latest_version: String,
    pub needs_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationCheck {
    pub initialized: bool,
    pub has_backup: bool,
    pub imported: bool,
    pub needs_setup: bool,
}

/// Unified device queue command - all device operations go through this
#[tauri::command]
pub async fn reset_device_queue(
    device_id: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<(), String> {
    println!("♻️  Resetting device queue for {}", device_id);
    let mut manager = queue_manager.lock().await;
    if let Some(handle) = manager.remove(&device_id) {
        let _ = handle.shutdown().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn add_to_device_queue(
    request: DeviceRequestWrapper,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<String, String> {
    println!("Adding to device queue: {:?}", request);
    
    // Log the incoming request
    let request_data = serde_json::json!({
        "request": request.request,
        "device_id": request.device_id,
        "request_id": request.request_id
    });
    
    let request_type = match &request.request {
        DeviceRequest::GetXpub { .. } => "GetXpub",
        DeviceRequest::GetAddress { .. } => "GetAddress", 
        DeviceRequest::GetFeatures => "GetFeatures",
        DeviceRequest::SendRaw { .. } => "SendRaw",
    };
    
    if let Err(e) = log_device_request(
        &request.device_id,
        &request.request_id,
        request_type,
        &request_data
    ).await {
        eprintln!("Failed to log device request: {}", e);
    }
    
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
        DeviceRequest::SendRaw { message_type, message_data } => {
            // Log the raw message being sent
            if let Err(e) = log_raw_device_message(
                &request.device_id,
                "SEND",
                &message_type,
                &message_data
            ).await {
                eprintln!("Failed to log raw device message: {}", e);
            }
            
            // For raw messages, we'd need to implement proper message parsing
            Err("Raw message sending not yet implemented".to_string())
        }
    };
    
    // Log the response
    match &result {
        Ok(response) => {
            println!("✅ Device operation completed: {}", response);
            
            let response_data = serde_json::json!({
                "response": response,
                "request_type": request_type
            });
            
            if let Err(e) = log_device_response(
                &request.device_id,
                &request.request_id,
                true,
                &response_data,
                None
            ).await {
                eprintln!("Failed to log device response: {}", e);
            }
            
            Ok(request.request_id)
        }
        Err(e) => {
            println!("❌ Device operation failed: {}", e);
            
            let error_data = serde_json::json!({
                "error": e,
                "request_type": request_type
            });
            
            if let Err(log_err) = log_device_response(
                &request.device_id,
                &request.request_id,
                false,
                &error_data,
                Some(e)
            ).await {
                eprintln!("Failed to log device error response: {}", log_err);
            }
            
            Err(e.clone())
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

/// Get device status including update needs and initialization status
#[tauri::command]
pub async fn get_device_status(
    device_id: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<Option<DeviceStatus>, String> {
    println!("Getting device status for: {}", device_id);
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    let request_data = serde_json::json!({
        "device_id": device_id,
        "operation": "get_device_status"
    });
    
    if let Err(e) = log_device_request(&device_id, &request_id, "GetDeviceStatus", &request_data).await {
        eprintln!("Failed to log get device status request: {}", e);
    }
    
    // Get connected devices to find the one we want
    let devices = keepkey_rust::features::list_connected_devices();
    let device_info = devices
        .iter()
        .find(|d| d.unique_id == device_id);
    
    if let Some(device_info) = device_info {
        // Get or create device queue handle
        let queue_handle = {
            let mut manager = queue_manager.lock().await;
            
            if let Some(handle) = manager.get(&device_id) {
                handle.clone()
            } else {
                // Spawn a new device worker
                let handle = DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
                manager.insert(device_id.clone(), handle.clone());
                handle
            }
        };
        
        // Fetch device features through the queue
        let features = match queue_handle.get_features().await {
            Ok(raw_features) => {
                // Convert from raw Features message to DeviceFeatures
                Some(convert_features_to_device_features(raw_features))
            }
            Err(e) => {
                println!("Failed to get features for device {}: {}", device_id, e);
                None
            }
        };
        
        // Evaluate device status
        let status = evaluate_device_status(device_id.clone(), features.as_ref());
        
        // Log the response
        let response_data = serde_json::json!({
            "status": status,
            "operation": "get_device_status"
        });
        
        if let Err(e) = log_device_response(&device_id, &request_id, true, &response_data, None).await {
            eprintln!("Failed to log get device status response: {}", e);
        }
        
        Ok(Some(status))
    } else {
        println!("Device {} not found", device_id);
        
        // Log the not found response
        let response_data = serde_json::json!({
            "error": "Device not found",
            "operation": "get_device_status"
        });
        
        if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some("Device not found")).await {
            eprintln!("Failed to log get device status error response: {}", e);
        }
        
        Ok(None)
    }
}

/// Get device info by ID (features only)
#[tauri::command]
pub async fn get_device_info_by_id(
    device_id: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<Option<DeviceFeatures>, String> {
    println!("Getting device info for: {}", device_id);
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    let request_data = serde_json::json!({
        "device_id": device_id,
        "operation": "get_device_info_by_id"
    });
    
    if let Err(e) = log_device_request(&device_id, &request_id, "GetDeviceInfo", &request_data).await {
        eprintln!("Failed to log get device info request: {}", e);
    }
    
    // Get or create device queue handle
    let queue_handle = {
        let mut manager = queue_manager.lock().await;
        
        if let Some(handle) = manager.get(&device_id) {
            handle.clone()
        } else {
            // Find the device by ID
            let devices = keepkey_rust::features::list_connected_devices();
            let device_info = devices
                .iter()
                .find(|d| d.unique_id == device_id);
                
            match device_info {
                Some(device_info) => {
                    // Spawn a new device worker
                    let handle = DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
                    manager.insert(device_id.clone(), handle.clone());
                    handle
                }
                None => {
                    let error = format!("Device {} not found", device_id);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "get_device_info_by_id"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log get device info error response: {}", e);
                    }
                    
                    return Err(error);
                }
            }
        }
    };
    
    // Fetch device features through the queue
    match queue_handle.get_features().await {
        Ok(raw_features) => {
            // Convert from raw Features message to DeviceFeatures
            let device_features = convert_features_to_device_features(raw_features);
            
            // Log the successful response
            let response_data = serde_json::json!({
                "features": device_features,
                "operation": "get_device_info_by_id"
            });
            
            if let Err(e) = log_device_response(&device_id, &request_id, true, &response_data, None).await {
                eprintln!("Failed to log get device info response: {}", e);
            }
            
            Ok(Some(device_features))
        }
        Err(e) => {
            println!("Failed to get features for device {}: {}", device_id, e);
            let error = format!("Failed to get device features: {}", e);
            
            // Log the error response
            let response_data = serde_json::json!({
                "error": error,
                "operation": "get_device_info_by_id"
            });
            
            if let Err(log_err) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                eprintln!("Failed to log get device info error response: {}", log_err);
            }
            
            Err(error)
        }
    }
}

/// Wipe device (factory reset)
#[tauri::command]
pub async fn wipe_device(
    device_id: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<(), String> {
    println!("Wiping device: {}", device_id);
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    let request_data = serde_json::json!({
        "device_id": device_id,
        "operation": "wipe_device"
    });
    
    if let Err(e) = log_device_request(&device_id, &request_id, "WipeDevice", &request_data).await {
        eprintln!("Failed to log wipe device request: {}", e);
    }
    
    // Get or create device queue handle
    let queue_handle = {
        let mut manager = queue_manager.lock().await;
        
        if let Some(handle) = manager.get(&device_id) {
            handle.clone()
        } else {
            // Find the device by ID
            let devices = keepkey_rust::features::list_connected_devices();
            let device_info = devices
                .iter()
                .find(|d| d.unique_id == device_id);
                
            match device_info {
                Some(device_info) => {
                    // Spawn a new device worker
                    let handle = DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
                    manager.insert(device_id.clone(), handle.clone());
                    handle
                }
                None => {
                    let error = format!("Device {} not found", device_id);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "wipe_device"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log wipe device error response: {}", e);
                    }
                    
                    return Err(error);
                }
            }
        }
    };
    
    // Create WipeDevice message
    let wipe_message = keepkey_rust::messages::Message::WipeDevice(
        keepkey_rust::messages::WipeDevice {}
    );
    
    // Log the raw message being sent
    let message_data = serde_json::json!({
        "message_type": "WipeDevice",
        "message": {}
    });
    
    if let Err(e) = log_raw_device_message(&device_id, "SEND", "WipeDevice", &message_data).await {
        eprintln!("Failed to log wipe device raw message: {}", e);
    }
    
    // Send wipe device command through queue
    match queue_handle.send_raw(wipe_message, true).await {
        Ok(response) => {
            // Log the raw response
            let response_message_data = serde_json::json!({
                "response": format!("{:?}", response)
            });
            
            if let Err(e) = log_raw_device_message(&device_id, "RECEIVE", "WipeDeviceResponse", &response_message_data).await {
                eprintln!("Failed to log wipe device raw response: {}", e);
            }
            
            match response {
                keepkey_rust::messages::Message::Success(_) => {
                    println!("✅ Device {} wiped successfully", device_id);
                    
                    // Log the successful response
                    let response_data = serde_json::json!({
                        "success": true,
                        "operation": "wipe_device"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, true, &response_data, None).await {
                        eprintln!("Failed to log wipe device response: {}", e);
                    }
                    
                    Ok(())
                }
                keepkey_rust::messages::Message::Failure(failure) => {
                    let error = format!("Device rejected wipe request: {}", failure.message.unwrap_or_default());
                    println!("❌ Failed to wipe device {}: {}", device_id, error);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "wipe_device"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log wipe device error response: {}", e);
                    }
                    
                    Err(error)
                }
                _ => {
                    let error = "Unexpected response from device".to_string();
                    println!("❌ Failed to wipe device {}: {}", device_id, error);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "wipe_device"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log wipe device error response: {}", e);
                    }
                    
                    Err(error)
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to wipe device {}: {}", device_id, e);
            let error = format!("Failed to wipe device: {}", e);
            
            // Log the error response
            let response_data = serde_json::json!({
                "error": error,
                "operation": "wipe_device"
            });
            
            if let Err(log_err) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                eprintln!("Failed to log wipe device error response: {}", log_err);
            }
            
            Err(error)
        }
    }
}

/// Set device label
#[tauri::command]
pub async fn set_device_label(
    device_id: String,
    label: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<(), String> {
    println!("Setting device label for {}: '{}'", device_id, label);
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    let request_data = serde_json::json!({
        "device_id": device_id,
        "label": label,
        "operation": "set_device_label"
    });
    
    if let Err(e) = log_device_request(&device_id, &request_id, "SetDeviceLabel", &request_data).await {
        eprintln!("Failed to log set device label request: {}", e);
    }
    
    // Validate label (max 32 chars for KeepKey)
    if label.len() > 32 {
        let error = "Label must be 32 characters or less".to_string();
        
        // Log the validation error
        let response_data = serde_json::json!({
            "error": error,
            "operation": "set_device_label"
        });
        
        if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
            eprintln!("Failed to log set device label validation error: {}", e);
        }
        
        return Err(error);
    }
    
    if !label.chars().all(|c| c.is_ascii() && !c.is_control()) {
        let error = "Label must contain only ASCII printable characters".to_string();
        
        // Log the validation error
        let response_data = serde_json::json!({
            "error": error,
            "operation": "set_device_label"
        });
        
        if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
            eprintln!("Failed to log set device label validation error: {}", e);
        }
        
        return Err(error);
    }
    
    // Get or create device queue handle
    let queue_handle = {
        let mut manager = queue_manager.lock().await;
        
        if let Some(handle) = manager.get(&device_id) {
            handle.clone()
        } else {
            // Find the device by ID
            let devices = keepkey_rust::features::list_connected_devices();
            let device_info = devices
                .iter()
                .find(|d| d.unique_id == device_id);
                
            match device_info {
                Some(device_info) => {
                    // Spawn a new device worker
                    let handle = DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
                    manager.insert(device_id.clone(), handle.clone());
                    handle
                }
                None => {
                    let error = format!("Device {} not found", device_id);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "set_device_label"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log set device label error response: {}", e);
                    }
                    
                    return Err(error);
                }
            }
        }
    };
    
    // Create ApplySettings message with the label
    let apply_settings = keepkey_rust::messages::Message::ApplySettings(
        keepkey_rust::messages::ApplySettings {
            language: None,
            label: Some(label.clone()),
            use_passphrase: None,
            auto_lock_delay_ms: None,
            u2f_counter: None,
        }
    );
    
    // Log the raw message being sent
    let message_data = serde_json::json!({
        "message_type": "ApplySettings",
        "message": {
            "label": label
        }
    });
    
    if let Err(e) = log_raw_device_message(&device_id, "SEND", "ApplySettings", &message_data).await {
        eprintln!("Failed to log apply settings raw message: {}", e);
    }
    
    // Send label update through queue
    match queue_handle.send_raw(apply_settings, true).await {
        Ok(response) => {
            // Log the raw response
            let response_message_data = serde_json::json!({
                "response": format!("{:?}", response)
            });
            
            if let Err(e) = log_raw_device_message(&device_id, "RECEIVE", "ApplySettingsResponse", &response_message_data).await {
                eprintln!("Failed to log apply settings raw response: {}", e);
            }
            
            match response {
                keepkey_rust::messages::Message::Success(_) => {
                    println!("✅ Device label set successfully for {}: '{}'", device_id, label);
                    
                    // Log the successful response
                    let response_data = serde_json::json!({
                        "success": true,
                        "label": label,
                        "operation": "set_device_label"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, true, &response_data, None).await {
                        eprintln!("Failed to log set device label response: {}", e);
                    }
                    
                    Ok(())
                }
                keepkey_rust::messages::Message::Failure(failure) => {
                    let error = format!("Device rejected label change: {}", failure.message.unwrap_or_default());
                    println!("❌ Failed to set device label for {}: {}", device_id, error);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "set_device_label"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log set device label error response: {}", e);
                    }
                    
                    Err(error)
                }
                _ => {
                    let error = "Unexpected response from device".to_string();
                    println!("❌ Failed to set device label for {}: {}", device_id, error);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "set_device_label"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log set device label error response: {}", e);
                    }
                    
                    Err(error)
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to set device label for {}: {}", device_id, e);
            let error = format!("Failed to set device label: {}", e);
            
            // Log the error response
            let response_data = serde_json::json!({
                "error": error,
                "operation": "set_device_label"
            });
            
            if let Err(log_err) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                eprintln!("Failed to log set device label error response: {}", log_err);
            }
            
            Err(error)
        }
    }
}

/// Enhanced get_connected_devices that fetches features through the queue
#[tauri::command]
pub async fn get_connected_devices_with_features(
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<Vec<serde_json::Value>, String> {
    let devices = keepkey_rust::features::list_connected_devices();
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    let request_data = serde_json::json!({
        "operation": "get_connected_devices_with_features",
        "device_count": devices.len()
    });
    
    if let Err(e) = log_device_request("all", &request_id, "GetConnectedDevicesWithFeatures", &request_data).await {
        eprintln!("Failed to log get connected devices request: {}", e);
    }
    
    // Process devices in parallel to fetch features
    let mut tasks = Vec::new();
    
    for device in devices.into_iter().filter(|device| device.is_keepkey) {
        let device_id = device.unique_id.clone();
        let queue_manager = queue_manager.inner().clone();
        
        let task = tokio::spawn(async move {
            // Log individual device feature request
            let device_request_id = uuid::Uuid::new_v4().to_string();
            let device_request_data = serde_json::json!({
                "device_id": device_id,
                "operation": "get_features_for_device"
            });
            
            if let Err(e) = log_device_request(&device_id, &device_request_id, "GetFeaturesForDevice", &device_request_data).await {
                eprintln!("Failed to log device features request: {}", e);
            }
            
            // Get or create device queue handle
            let queue_handle = {
                let mut manager = queue_manager.lock().await;
                
                if let Some(handle) = manager.get(&device_id) {
                    handle.clone()
                } else {
                    // Spawn a new device worker
                    let handle = DeviceQueueFactory::spawn_worker(device_id.clone(), device.clone());
                    manager.insert(device_id.clone(), handle.clone());
                    handle
                }
            };
            
            // Try to fetch features through the queue
            let features = match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                queue_handle.get_features()
            ).await {
                Ok(Ok(raw_features)) => {
                    // Convert from raw Features message to DeviceFeatures
                    let device_features = convert_features_to_device_features(raw_features);
                    
                    // Log successful feature retrieval
                    let device_response_data = serde_json::json!({
                        "features": device_features,
                        "operation": "get_features_for_device"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &device_request_id, true, &device_response_data, None).await {
                        eprintln!("Failed to log device features response: {}", e);
                    }
                    
                    Some(device_features)
                }
                Ok(Err(e)) => {
                    println!("Failed to get features for device {}: {}", device_id, e);
                    
                    // Log failed feature retrieval
                    let device_response_data = serde_json::json!({
                        "error": format!("Failed to get features: {}", e),
                        "operation": "get_features_for_device"
                    });
                    
                    if let Err(log_err) = log_device_response(&device_id, &device_request_id, false, &device_response_data, Some(&format!("Failed to get features: {}", e))).await {
                        eprintln!("Failed to log device features error response: {}", log_err);
                    }
                    
                    None
                }
                Err(_) => {
                    println!("Timeout getting features for device {}", device_id);
                    
                    // Log timeout
                    let device_response_data = serde_json::json!({
                        "error": "Timeout getting features",
                        "operation": "get_features_for_device"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &device_request_id, false, &device_response_data, Some("Timeout getting features")).await {
                        eprintln!("Failed to log device features timeout response: {}", e);
                    }
                    
                    None
                }
            };
            
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
                "features": features,
            })
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => {
                println!("Task failed: {}", e);
                // Continue with other devices
            }
        }
    }
    
    // Log the overall response
    let response_data = serde_json::json!({
        "devices": results,
        "operation": "get_connected_devices_with_features"
    });
    
    if let Err(e) = log_device_response("all", &request_id, true, &response_data, None).await {
        eprintln!("Failed to log get connected devices response: {}", e);
    }
    
    Ok(results)
}

/// Evaluate device status to determine what actions are needed
fn evaluate_device_status(device_id: String, features: Option<&DeviceFeatures>) -> DeviceStatus {
    let mut status = DeviceStatus {
        device_id: device_id.clone(),
        connected: true,
        features: features.cloned(),
        needs_bootloader_update: false,
        needs_firmware_update: false,
        needs_initialization: false,
        bootloader_check: None,
        firmware_check: None,
        initialization_check: None,
    };
    
    if let Some(features) = features {
        // Check bootloader version
        if let Some(bootloader_version) = &features.bootloader_version {
            let current_version = bootloader_version.clone();
            let latest_version = "2.1.5".to_string(); // Latest bootloader version
            let needs_update = current_version != latest_version && current_version != "2.1.4";
            
            status.bootloader_check = Some(BootloaderCheck {
                current_version: current_version.clone(),
                latest_version: latest_version.clone(),
                needs_update,
            });
            status.needs_bootloader_update = needs_update;
        }
        
        // Check firmware version
        let current_version = features.version.clone();
        let latest_version = "7.10.0".to_string(); // Latest firmware version
        let needs_update = !current_version.starts_with("7.10.") && !features.bootloader_mode;
        
        status.firmware_check = Some(FirmwareCheck {
            current_version: current_version.clone(),
            latest_version: latest_version.clone(),
            needs_update,
        });
        status.needs_firmware_update = needs_update;
        
        // Check initialization status
        let initialized = features.initialized;
        let has_backup = !features.no_backup;
        let imported = features.imported.unwrap_or(false);
        let needs_setup = !initialized && !features.bootloader_mode;
        
        status.initialization_check = Some(InitializationCheck {
            initialized,
            has_backup,
            imported,
            needs_setup,
        });
        status.needs_initialization = needs_setup;
    } else {
        // No features available - device not communicating
        status.connected = false;
    }
    
    status
}

/// Convert raw Features message to DeviceFeatures
fn convert_features_to_device_features(raw_features: keepkey_rust::messages::Features) -> DeviceFeatures {
    DeviceFeatures {
        label: raw_features.label,
        vendor: raw_features.vendor,
        model: raw_features.model,
        firmware_variant: raw_features.firmware_variant,
        device_id: raw_features.device_id,
        language: raw_features.language,
        bootloader_mode: raw_features.bootloader_mode.unwrap_or(false),
        version: format!(
            "{}.{}.{}",
            raw_features.major_version.unwrap_or(0),
            raw_features.minor_version.unwrap_or(0),
            raw_features.patch_version.unwrap_or(0)
        ),
        firmware_hash: raw_features.firmware_hash.map(hex::encode),
        bootloader_hash: raw_features.bootloader_hash.clone().map(hex::encode),
        bootloader_version: raw_features.bootloader_hash
            .map(hex::encode)
            .and_then(|hash| Some(hash)),
        initialized: raw_features.initialized.unwrap_or(false),
        imported: raw_features.imported,
        no_backup: raw_features.no_backup.unwrap_or(false),
        pin_protection: raw_features.pin_protection.unwrap_or(false),
        pin_cached: raw_features.pin_cached.unwrap_or(false),
        passphrase_protection: raw_features.passphrase_protection.unwrap_or(false),
        passphrase_cached: raw_features.passphrase_cached.unwrap_or(false),
        wipe_code_protection: raw_features.wipe_code_protection.unwrap_or(false),
        auto_lock_delay_ms: raw_features.auto_lock_delay_ms.map(|ms| ms as u64),
        policies: raw_features
            .policies
            .into_iter()
            .filter(|p| p.enabled())
            .map(|p| p.policy_name().to_string())
            .collect(),
    }
}

/// Get the path to today's device communication log file
#[tauri::command]
pub async fn get_device_log_path() -> Result<String, String> {
    let logger = crate::logging::get_device_logger();
    let log_path = logger.get_todays_log_path();
    
    Ok(log_path.to_string_lossy().to_string())
}

/// Get recent device communication log entries (last N entries)
#[tauri::command]
pub async fn get_recent_device_logs(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    let logger = crate::logging::get_device_logger();
    let log_path = logger.get_todays_log_path();
    let limit = limit.unwrap_or(50); // Default to last 50 entries
    
    if !log_path.exists() {
        return Ok(vec![]);
    }
    
    // Read the log file and parse JSON lines
    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("Failed to read log file: {}", e))?;
    
    let mut entries: Vec<serde_json::Value> = content
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            match serde_json::from_str(line) {
                Ok(json) => Some(json),
                Err(e) => {
                    eprintln!("Failed to parse log line: {} - Error: {}", line, e);
                    None
                }
            }
        })
        .collect();
    
    // Return the last N entries
    if entries.len() > limit {
        let skip_count = entries.len() - limit;
        entries = entries.into_iter().skip(skip_count).collect();
    }
    
    Ok(entries)
}

/// Clear old device communication logs (manually trigger cleanup)
#[tauri::command]
pub async fn cleanup_device_logs() -> Result<String, String> {
    let logger = crate::logging::get_device_logger();
    logger.cleanup_old_logs().await?;
    Ok("Old device logs cleaned up successfully".to_string())
} 