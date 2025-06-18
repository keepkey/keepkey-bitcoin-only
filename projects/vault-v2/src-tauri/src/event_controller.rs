use keepkey_rust::friendly_usb::FriendlyUsbDevice;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

pub struct EventController {
    cancellation_token: CancellationToken,
    task_handle: Option<tauri::async_runtime::JoinHandle<()>>,
    is_running: bool,
}

impl EventController {
    pub fn new() -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            task_handle: None,
            is_running: false,
        }
    }
    
    pub fn start(&mut self, app: &AppHandle) {
        if self.is_running {
            println!("⚠️ Event controller already running");
            return;
        }
        
        let app_handle = app.clone();
        let cancellation_token = self.cancellation_token.clone();
        
        let task_handle = tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_millis(1000)); // Check every second
            let mut last_devices: Vec<FriendlyUsbDevice> = Vec::new();
            
            println!("✅ Event controller started - monitoring device connections");
            
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        println!("🛑 Event controller shutting down on cancellation signal");
                        break;
                    }
                    _ = interval.tick() => {
                        // Get current devices using high-level API
                        let current_devices = keepkey_rust::features::list_connected_devices();
                        
                        // Check for newly connected devices
                        for device in &current_devices {
                            if !last_devices.iter().any(|d| d.unique_id == device.unique_id) {
                                println!("🔌 Device connected: {} (VID: 0x{:04x}, PID: 0x{:04x})", 
                                         device.unique_id, device.vid, device.pid);
                                println!("   Device info: {} - {}", 
                                         device.manufacturer.as_deref().unwrap_or("Unknown"), 
                                         device.product.as_deref().unwrap_or("Unknown"));
                                
                                // Emit basic device connected event first
                                let _ = app_handle.emit("device:connected", device);
                                
                                // Proactively fetch features and emit device:ready when successful
                                let app_for_task = app_handle.clone();
                                let device_for_task = device.clone();
                                tokio::spawn(async move {
                                    println!("📡 Fetching device features for: {}", device_for_task.unique_id);
                                    
                                    match try_get_device_features(&device_for_task).await {
                                        Ok(features) => {
                                            println!("✅ Device ready: {} v{} ({})", 
                                                   features.label.as_deref().unwrap_or("Unlabeled"),
                                                   features.version,
                                                   device_for_task.unique_id);
                                            
                                            // Emit device:ready event with features
                                            let ready_payload = serde_json::json!({
                                                "device": device_for_task,
                                                "features": features,
                                                "status": "ready"
                                            });
                                            let _ = app_for_task.emit("device:ready", &ready_payload);
                                            
                                            // Also emit device:features-updated for compatibility
                                            let features_payload = serde_json::json!({
                                                "deviceId": device_for_task.unique_id,
                                                "features": features,
                                                "status": "ready"
                                            });
                                            let _ = app_for_task.emit("device:features-updated", &features_payload);
                                        }
                                        Err(e) => {
                                            println!("❌ Failed to get features for {}: {}", device_for_task.unique_id, e);
                                            
                                            // Check if this is a device access error
                                            if e.contains("Device Already In Use") || 
                                               e.contains("already claimed") ||
                                               e.contains("🔒") {
                                                
                                                let user_friendly_error = if e.contains("🔒") {
                                                    e.clone()
                                                } else {
                                                    format!(
                                                        "🔒 KeepKey Device Already In Use\n\n\
                                                        Your KeepKey device is currently being used by another application.\n\n\
                                                        Common causes:\n\
                                                        • KeepKey Desktop app is running\n\
                                                        • KeepKey Bridge is running\n\
                                                        • Another wallet application is connected\n\
                                                        • Previous connection wasn't properly closed\n\n\
                                                        Solutions:\n\
                                                        1. Close KeepKey Desktop app completely\n\
                                                        2. Close any other wallet applications\n\
                                                        3. Unplug and reconnect your KeepKey device\n\
                                                        4. Try again\n\n\
                                                        Technical details: {}", e
                                                    )
                                                };
                                                
                                                // Emit device access error event
                                                let error_payload = serde_json::json!({
                                                    "deviceId": device_for_task.unique_id,
                                                    "error": user_friendly_error,
                                                    "errorType": "DEVICE_CLAIMED",
                                                    "status": "error"
                                                });
                                                let _ = app_for_task.emit("device:access-error", &error_payload);
                                            }
                                        }
                                    }
                                });
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
                }
            }
            
            println!("✅ Event controller stopped cleanly");
        });
        
        self.task_handle = Some(task_handle);
        self.is_running = true;
    }
    
    pub fn stop(&mut self) {
        if !self.is_running {
            return;
        }
        
        println!("🛑 Stopping event controller...");
        
        // Cancel the background task
        self.cancellation_token.cancel();
        self.is_running = false;
        
        // Wait for the task to complete if it exists
        if let Some(handle) = self.task_handle.take() {
            // Try to wait for completion with a timeout
            tauri::async_runtime::spawn(async move {
                if let Err(e) = tokio::time::timeout(Duration::from_secs(5), handle).await {
                    println!("⚠️ Event controller task did not stop within timeout: {}", e);
                } else {
                    println!("✅ Event controller task stopped successfully");
                }
            });
        }
    }
}

impl Drop for EventController {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Try to get device features without blocking the event loop
/// Returns features if successful, error message if failed
async fn try_get_device_features(device: &FriendlyUsbDevice) -> Result<keepkey_rust::features::DeviceFeatures, String> {
    // Create a temporary device queue to fetch features
    // This is a non-blocking operation that will fail fast if device is busy
    let queue_handle = keepkey_rust::device_queue::DeviceQueueFactory::spawn_worker(
        device.unique_id.clone(),
        device.clone()
    );
    
    // Try to get features with a timeout
    match tokio::time::timeout(Duration::from_secs(5), queue_handle.get_features()).await {
        Ok(Ok(raw_features)) => {
            // Convert features to our DeviceFeatures format
            let device_features = crate::commands::convert_features_to_device_features(raw_features);
            Ok(device_features)
        }
        Ok(Err(e)) => {
            Err(format!("Failed to get device features: {}", e))
        }
        Err(_) => {
            Err("Timeout while fetching device features".to_string())
        }
    }
}

// Create and manage event controller with proper Arc<Mutex<>> wrapper
pub fn spawn_event_controller(app: &AppHandle) -> Arc<Mutex<EventController>> {
    let mut controller = EventController::new();
    controller.start(app);
    
    let controller_arc = Arc::new(Mutex::new(controller));
    
    // Store the controller in app state so it can be properly cleaned up
    app.manage(controller_arc.clone());
    
    controller_arc
}
