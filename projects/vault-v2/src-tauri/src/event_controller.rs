use keepkey_rust::friendly_usb::FriendlyUsbDevice;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::time::interval;

pub fn spawn_event_controller(app: &AppHandle) {
    let app_handle = app.clone();
    
    // Spawn a background task to monitor for device changes using Tauri's async runtime
    tauri::async_runtime::spawn(async move {
        let mut interval = interval(Duration::from_millis(1000)); // Check every second
        let mut last_devices: Vec<FriendlyUsbDevice> = Vec::new();
        
        loop {
            interval.tick().await;
            
            // Get current devices using high-level API
            let current_devices = keepkey_rust::features::list_connected_devices();
            
            // Check for newly connected devices
            for device in &current_devices {
                if !last_devices.iter().any(|d| d.unique_id == device.unique_id) {
                    println!("🔌 Device connected: {}", device.unique_id);
                    let _ = app_handle.emit("device:connected", device);
                    
                    // Try to get device features and emit state change
                    match keepkey_rust::features::get_device_features_with_fallback(device) {
                        Ok(features) => {
                            let payload = serde_json::json!({
                                "device": device,
                                "features": features
                            });
                            let _ = app_handle.emit("device:state-changed", payload);
                        }
                        Err(e) => {
                            println!("⚠️ Failed to get features for device {}: {}", device.unique_id, e);
                        }
                    }
                }
            }
            
            // Check for disconnected devices
            for device in &last_devices {
                if !current_devices.iter().any(|d| d.unique_id == device.unique_id) {
                    println!("🔌❌ Device disconnected: {}", device.unique_id);
                    let _ = app_handle.emit("device:disconnected", &device.unique_id);
                }
            }
            
            last_devices = current_devices;
        }
    });
    
    println!("✅ Event controller started - monitoring device connections");
}
