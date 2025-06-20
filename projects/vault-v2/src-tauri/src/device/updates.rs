use tauri::State;
use semver::Version;
use uuid;
use crate::logging::{log_device_request, log_device_response};
use crate::commands::DeviceQueueManager;
use crate::embedded_firmware;

/// Update device bootloader using the device queue
#[tauri::command]
pub async fn update_device_bootloader(
    device_id: String,
    target_version: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<bool, String> {
    println!("🔄 Starting bootloader update for device {}: target version {}", device_id, target_version);
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    let request_data = serde_json::json!({
        "device_id": device_id,
        "target_version": target_version,
        "operation": "update_device_bootloader"
    });
    
    if let Err(e) = log_device_request(&device_id, &request_id, "UpdateBootloader", &request_data).await {
        eprintln!("Failed to log bootloader update request: {}", e);
    }
    
    // Validate target version
    let _target_semver = Version::parse(&target_version)
        .map_err(|e| format!("Invalid target bootloader version: {}", e))?;
    
    // Load the bootloader binary from embedded firmware (bundled with app)
    println!("📦 Loading embedded bootloader binary for version {}", target_version);
    
    let bootloader_bytes = match embedded_firmware::get_bootloader_bytes(&target_version) {
        Some(bytes) => {
            println!("✅ Found embedded bootloader v{}: {} bytes", target_version, bytes.len());
            bytes.to_vec()
        }
        None => {
            let available_versions = embedded_firmware::get_available_bootloader_versions();
            let error_msg = format!(
                "Bootloader version {} not found in embedded firmware. Available versions: {}",
                target_version,
                available_versions.join(", ")
            );
            
            println!("❌ {}", error_msg);
            
            // Log the error response
            let response_data = serde_json::json!({
                "error": error_msg,
                "operation": "update_device_bootloader",
                "available_versions": available_versions
            });
            
            if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error_msg)).await {
                eprintln!("Failed to log bootloader update error response: {}", e);
            }
            
            return Err(error_msg);
        }
    };
    
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
                    let handle = keepkey_rust::device_queue::DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
                    manager.insert(device_id.clone(), handle.clone());
                    handle
                }
                None => {
                    let error = format!("Device {} not found", device_id);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "update_device_bootloader"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log bootloader update error response: {}", e);
                    }
                    
                    return Err(error);
                }
            }
        }
    };
    
    // Check device features to ensure it's in bootloader mode
    match queue_handle.get_features().await {
        Ok(features) => {
            if !features.bootloader_mode.unwrap_or(false) {
                let error = "Device is not in bootloader mode. Please hold the button while reconnecting to enter bootloader mode.".to_string();
                
                // Log the error response
                let response_data = serde_json::json!({
                    "error": error,
                    "operation": "update_device_bootloader"
                });
                
                if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                    eprintln!("Failed to log bootloader update error response: {}", e);
                }
                
                return Err(error);
            }
            println!("✅ Device confirmed in bootloader mode, firmware version: {}", format!(
                "{}.{}.{}",
                features.major_version.unwrap_or(0),
                features.minor_version.unwrap_or(0),
                features.patch_version.unwrap_or(0)
            ));
        }
        Err(e) => {
            let error_str = e.to_string();
            
            // Check if this looks like an OOB bootloader that doesn't understand GetFeatures
            if error_str.contains("Unknown message") || 
               error_str.contains("Failure: Unknown message") ||
               error_str.contains("Unexpected response") {
                
                println!("🔧 Device may be in OOB bootloader mode (can't get features), proceeding with update...");
                println!("    This is normal for devices with very old bootloaders that don't support GetFeatures");
                println!("    The update process will verify bootloader mode during the actual update");
            } else {
                let error = format!("Failed to get device features: {}", error_str);
                
                // Log the error response
                let response_data = serde_json::json!({
                    "error": error,
                    "operation": "update_device_bootloader"
                });
                
                if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                    eprintln!("Failed to log bootloader update error response: {}", e);
                }
                
                return Err(error);
            }
        }
    }
    
    println!("⚠️  IMPORTANT: Check your KeepKey device screen!");
    println!("    You may need to press the button to confirm the update.");
    println!("    The v1.0.3 bootloader requires manual confirmation.");
    println!("    If you see 'Upload' on the device screen, press and hold the button.");
    
    // Perform the bootloader update through the queue
    match queue_handle.update_bootloader(target_version.clone(), bootloader_bytes).await {
        Ok(success) => {
            println!("✅ Bootloader update successful for device {}", device_id);
            
            // Log the successful response
            let response_data = serde_json::json!({
                "success": success,
                "target_version": target_version,
                "operation": "update_device_bootloader"
            });
            
            if let Err(e) = log_device_response(&device_id, &request_id, true, &response_data, None).await {
                eprintln!("Failed to log bootloader update success response: {}", e);
            }
            
            Ok(success)
        }
        Err(e) => {
            let error_msg = e.to_string();
            println!("❌ Bootloader update failed for device {}: {}", device_id, error_msg);
            
            // Log the error response
            let response_data = serde_json::json!({
                "error": error_msg,
                "operation": "update_device_bootloader"
            });
            
            if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error_msg)).await {
                eprintln!("Failed to log bootloader update error response: {}", e);
            }
            
            Err(format!("Bootloader update failed: {}", error_msg))
        }
    }
}

/// Update device firmware using the device queue
#[tauri::command]
pub async fn update_device_firmware(
    device_id: String,
    target_version: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<bool, String> {
    println!("🔄 Starting firmware update for device {}: target version {}", device_id, target_version);
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    let request_data = serde_json::json!({
        "device_id": device_id,
        "target_version": target_version,
        "operation": "update_device_firmware"
    });
    
    if let Err(e) = log_device_request(&device_id, &request_id, "UpdateFirmware", &request_data).await {
        eprintln!("Failed to log firmware update request: {}", e);
    }
    
    // Validate target version
    let _target_semver = Version::parse(&target_version)
        .map_err(|e| format!("Invalid target firmware version: {}", e))?;
    
    // Load the firmware binary from embedded firmware (bundled with app)
    println!("📦 Loading embedded firmware binary for version {}", target_version);
    
    let firmware_bytes = match embedded_firmware::get_firmware_bytes(&target_version) {
        Some(bytes) => {
            println!("✅ Found embedded firmware v{}: {} bytes", target_version, bytes.len());
            bytes.to_vec()
        }
        None => {
            let available_versions = embedded_firmware::get_available_firmware_versions();
            let error_msg = format!(
                "Firmware version {} not found in embedded firmware. Available versions: {}",
                target_version,
                available_versions.join(", ")
            );
            
            println!("❌ {}", error_msg);
            
            // Log the error response
            let response_data = serde_json::json!({
                "error": error_msg,
                "operation": "update_device_firmware",
                "available_versions": available_versions
            });
            
            if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error_msg)).await {
                eprintln!("Failed to log firmware update error response: {}", e);
            }
            
            return Err(error_msg);
        }
    };
    
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
                    let handle = keepkey_rust::device_queue::DeviceQueueFactory::spawn_worker(device_id.clone(), device_info.clone());
                    manager.insert(device_id.clone(), handle.clone());
                    handle
                }
                None => {
                    let error = format!("Device {} not found", device_id);
                    
                    // Log the error response
                    let response_data = serde_json::json!({
                        "error": error,
                        "operation": "update_device_firmware"
                    });
                    
                    if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                        eprintln!("Failed to log firmware update error response: {}", e);
                    }
                    
                    return Err(error);
                }
            }
        }
    };
    
    // Check device features to ensure it's in bootloader mode (required for firmware updates)
    match queue_handle.get_features().await {
        Ok(features) => {
            if !features.bootloader_mode.unwrap_or(false) {
                let error = "Device must be in bootloader mode for firmware update. Please hold the button while reconnecting to enter bootloader mode.".to_string();
                
                // Log the error response
                let response_data = serde_json::json!({
                    "error": error,
                    "operation": "update_device_firmware"
                });
                
                if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                    eprintln!("Failed to log firmware update error response: {}", e);
                }
                
                return Err(error);
            }
            println!("✅ Device confirmed in bootloader mode, ready for firmware update. Current version: {}", format!(
                "{}.{}.{}",
                features.major_version.unwrap_or(0),
                features.minor_version.unwrap_or(0),
                features.patch_version.unwrap_or(0)
            ));
        }
        Err(e) => {
            let error_str = e.to_string();
            
            // Check if this looks like an OOB bootloader that doesn't understand GetFeatures
            if error_str.contains("Unknown message") || 
               error_str.contains("Failure: Unknown message") ||
               error_str.contains("Unexpected response") {
                
                println!("🔧 Device may be in OOB bootloader mode (can't get features), proceeding with firmware update...");
                println!("    This is normal for devices with very old bootloaders that don't support GetFeatures");
                println!("    The update process will verify bootloader mode during the actual update");
            } else {
                let error = format!("Failed to get device features: {}", error_str);
                
                // Log the error response
                let response_data = serde_json::json!({
                    "error": error,
                    "operation": "update_device_firmware"
                });
                
                if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error)).await {
                    eprintln!("Failed to log firmware update error response: {}", e);
                }
                
                return Err(error);
            }
        }
    }
    
    println!("⚠️  IMPORTANT: Check your KeepKey device screen!");
    println!("    You may need to press the button to confirm the firmware update.");
    println!("    If you see 'Upload' on the device screen, press and hold the button.");
    
    // Perform the firmware update through the queue  
    match queue_handle.update_firmware(target_version.clone(), firmware_bytes).await {
        Ok(success) => {
            println!("✅ Firmware update successful for device {}", device_id);
            
            // Log the successful response
            let response_data = serde_json::json!({
                "success": success,
                "target_version": target_version,
                "operation": "update_device_firmware"
            });
            
            if let Err(e) = log_device_response(&device_id, &request_id, true, &response_data, None).await {
                eprintln!("Failed to log firmware update success response: {}", e);
            }
            
            Ok(success)
        }
        Err(e) => {
            let error_msg = e.to_string();
            println!("❌ Firmware update failed for device {}: {}", device_id, error_msg);
            
            // Log the error response
            let response_data = serde_json::json!({
                "error": error_msg,
                "operation": "update_device_firmware"
            });
            
            if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error_msg)).await {
                eprintln!("Failed to log firmware update error response: {}", e);
            }
            
            Err(format!("Firmware update failed: {}", error_msg))
        }
    }
} 