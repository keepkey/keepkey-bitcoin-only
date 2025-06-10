# KeepKey Desktop Device Controller Implementation Plan

## Current State Analysis

The current implementation has the following limitations:

1. Multiple device managers are created:
   - One in `lib.rs` via `usb_manager::init_device_manager`
   - A separate one for the server in `lib.rs` created with `Arc::new(tokio::sync::Mutex::new(usb_manager::DeviceManager::new(app_handle.clone())))`

2. The `DEVICE_FEATURES` global variable only stores features for a single device, not multiple devices.

3. The server endpoint for listing devices can't access the same device cache as the rest of the application.

## Implementation Plan

### 1. Create a Global Device Cache

Create a global static device registry that will store all connected devices and their features:

```rust
// In src-tauri/src/device_registry.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use crate::features::DeviceFeatures;
use crate::usb_manager::FriendlyUsbDevice;

// Define a struct to hold device data including features
pub struct DeviceEntry {
    pub device: FriendlyUsbDevice,
    pub features: Option<DeviceFeatures>,
    pub last_updated: std::time::SystemTime,
}

// Global device registry
pub static DEVICE_REGISTRY: Lazy<Arc<Mutex<HashMap<String, DeviceEntry>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

// Helper functions for working with the registry
pub fn add_or_update_device(device: FriendlyUsbDevice, features: Option<DeviceFeatures>) -> Result<(), String> {
    let mut registry = DEVICE_REGISTRY.lock().map_err(|e| e.to_string())?;
    
    registry.insert(device.unique_id.clone(), DeviceEntry {
        device,
        features,
        last_updated: std::time::SystemTime::now(),
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
```

### 2. Modify DeviceManager to Use the Global Registry

Update the `DeviceManager` class to use the global registry:

```rust
// In src-tauri/src/usb_manager.rs

impl DeviceManager {
    // Add a method to sync with the global registry
    pub fn sync_with_registry(&self) -> Result<()> {
        let devices = self.get_connected_devices();
        
        // For each device, ensure it's in the registry
        for device in devices {
            // If it's a KeepKey, fetch features
            let features = if device.is_keepkey {
                crate::features::get_device_features_for_device(&device).ok()
            } else {
                None
            };
            
            // Add or update in registry
            crate::device_registry::add_or_update_device(device, features)?;
        }
        
        Ok(())
    }
    
    // Update the check_features_and_emit method to use the registry
    pub fn check_features_and_emit(&self) {
        // First sync the current state with the registry
        if let Err(e) = self.sync_with_registry() {
            log::error!("Failed to sync with device registry: {}", e);
        }
        
        // Rest of the method remains the same...
    }
    
    // ... other existing methods
}
```

### 3. Modify the Server Implementation

Update the server to use the global registry:

```rust
// In src-tauri/src/server/routes.rs

use crate::device_registry;

#[tauri::command]
#[utoipa::path(
    get,
    path = "/api/devices",
    tag = "device",
    responses(
        (status = 200, description = "List of connected devices", body = Vec<DeviceInfo>)
    )
)]
pub async fn list_devices() -> ApiResponse<Vec<DeviceInfo>> {
    match device_registry::get_all_devices() {
        Ok(devices) => {
            let device_infos = devices.into_iter().map(|device| {
                DeviceInfo {
                    id: device.unique_id,
                    name: device.name,
                    is_keepkey: device.is_keepkey,
                    // Map other fields...
                }
            }).collect();
            
            ApiResponse::success(device_infos)
        },
        Err(e) => {
            ApiResponse::error(format!("Failed to get devices: {}", e))
        }
    }
}

// Similar updates for other device-related endpoints
```

### 4. Update the MCP JSON-RPC Handler

```rust
// In src-tauri/src/server/mod.rs

async fn handle_mcp_request(request: Value) -> Value {
    // ... existing code
    
    match method {
        // ... existing cases
        
        "tools/call" => {
            let tool_name = request.get("params")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            
            match tool_name {
                "get_device_status" => {
                    // Use the registry to get device status
                    let devices = match device_registry::get_all_devices() {
                        Ok(devices) => devices,
                        Err(e) => {
                            return json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32603,
                                    "message": format!("Internal error: {}", e)
                                }
                            });
                        }
                    };
                    
                    // Format response with device information
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Found {} devices", devices.len())
                                },
                                {
                                    "type": "json",
                                    "json": devices
                                }
                            ]
                        }
                    })
                },
                
                // Handle other tool calls similarly
                // ...
            }
        }
        
        // ... remaining code
    }
}
```

### 5. Update `lib.rs` to Use a Single Device Manager

```rust
pub fn run() {
    // ... existing setup code
    
    tauri::Builder::default()
        .plugin(log_plugin)
        .setup(|app| {
            // ... existing setup code
            
            // Initialize the USB watcher in a background thread
            spawn_usb_watcher(app.handle().clone());
            
            // Clone the app handle
            let app_handle = app.handle().clone();
            
            // Start the device manager and server initialization
            std::thread::spawn(move || {
                // Give the UI a moment to render
                std::thread::sleep(std::time::Duration::from_millis(100));
                
                // Initialize device manager
                let device_manager = usb_manager::init_device_manager(app_handle.clone());
                
                // Manage the device manager in the app state
                app_handle.manage(device_manager.clone());
                log::info!("{TAG} Device manager added to app state");
                
                // Use the same device manager for the server
                log::info!("{TAG} Starting REST API and MCP server...");
                
                // Use spawn instead of block_on
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = server::start_server(device_manager).await {
                        log::error!("{TAG} Failed to start REST API and MCP server: {}", e);
                    }
                });
            });
            
            Ok(())
        })
        
        // ... rest of code
}
```

### 6. Modify the `features` Module to Work with Multiple Devices

```rust
// In src-tauri/src/features/mod.rs

// Add a function to get features for a specific device
pub fn get_device_features_for_device(device: &usb_manager::FriendlyUsbDevice) -> Result<DeviceFeatures, anyhow::Error> {
    // Implementation to connect to the specific device and get its features
    // ...
}

// Keep the original implementation for backward compatibility
pub fn get_device_features_impl() -> Result<DeviceFeatures, anyhow::Error> {
    // This now becomes a convenience function that gets features for the first KeepKey device
    // ...
}
```

## Migration Steps

1. Create the new `device_registry.rs` file with the global registry implementation
2. Update the `usb_manager.rs` file to use the registry
3. Update the server implementation to use the registry instead of its own device manager
4. Modify `lib.rs` to use a single device manager instance
5. Update the features module to support multiple devices
6. Test all functionality to ensure compatibility with existing code

## Benefits

1. Single source of truth for device state
2. Improved performance by avoiding duplicate scanning
3. Consistent device information across all parts of the application
4. Better support for multiple devices
5. Simplified code with clear separation of concerns
