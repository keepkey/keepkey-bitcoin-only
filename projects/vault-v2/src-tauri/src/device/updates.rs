use tauri::State;
use std::fs;
use std::path::PathBuf;
use semver::Version;
use uuid;
use crate::logging::{log_device_request, log_device_response};
use crate::commands::DeviceQueueManager;

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
    
    // Load the bootloader binary from the firmware directory
    let bootloader_filename = format!("bl_v{}", target_version);
    let firmware_path = PathBuf::from("../firmware").join(&bootloader_filename).join("blupdater.bin");
    
    let bootloader_bytes = if firmware_path.exists() {
        println!("📂 Loading bootloader from: {}", firmware_path.display());
        fs::read(&firmware_path)
            .map_err(|e| format!("Failed to read bootloader file {}: {}", firmware_path.display(), e))?
    } else {
        // Check available bootloader versions
        let firmware_dir = PathBuf::from("../firmware");
        let available_versions = if firmware_dir.exists() {
            match fs::read_dir(&firmware_dir) {
                Ok(entries) => {
                    let versions: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .filter(|name| name.starts_with("bl_v"))
                        .collect();
                    format!("Available bootloader versions: {}", versions.join(", "))
                }
                Err(_) => "Could not list firmware directory".to_string()
            }
        } else {
            "Firmware directory not found".to_string()
        };
        
        let error_msg = format!(
            "Bootloader file not found: {}. {}. Target version was: {}",
            firmware_path.display(),
            available_versions,
            target_version
        );
        
        // Log the error response
        let response_data = serde_json::json!({
            "error": error_msg,
            "operation": "update_device_bootloader"
        });
        
        if let Err(e) = log_device_response(&device_id, &request_id, false, &response_data, Some(&error_msg)).await {
            eprintln!("Failed to log bootloader update error response: {}", e);
        }
        
        return Err(error_msg);
    };
    
    println!("📦 Loaded bootloader binary: {} bytes", bootloader_bytes.len());
    
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

// TODO: Firmware update functionality not yet implemented in the device queue system.
// The DeviceQueueHandle currently only supports bootloader updates.
// Firmware updates will need to be added to the keepkey-rust device queue implementation. 