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
                                
                                // Always emit device connected event, even if we can't get features
                                let _ = app_handle.emit("device:connected", device);
                                
                                // Emit basic device state without features to avoid crashes
                                // Frontend can fetch features separately using get_device_features_by_id
                                let payload = serde_json::json!({
                                    "device": device,
                                    "features": null,
                                    "status": "connected"
                                });
                                let _ = app_handle.emit("device:state-changed", payload);
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

// Create and manage event controller with proper Arc<Mutex<>> wrapper
pub fn spawn_event_controller(app: &AppHandle) -> Arc<Mutex<EventController>> {
    let mut controller = EventController::new();
    controller.start(app);
    
    let controller_arc = Arc::new(Mutex::new(controller));
    
    // Store the controller in app state so it can be properly cleaned up
    app.manage(controller_arc.clone());
    
    controller_arc
}
