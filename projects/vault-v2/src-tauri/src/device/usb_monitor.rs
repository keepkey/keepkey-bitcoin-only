use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;

use crate::device::{
    interaction_state::{DeviceInteractionState, DEVICE_SESSIONS},
    events::{DeviceEvent, emit_device_event},
};

pub struct UsbMonitor {
    app: AppHandle,
    last_devices: Arc<RwLock<Vec<String>>>,
}

impl UsbMonitor {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            last_devices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(self: Arc<Self>) {
        // Use platform-specific USB monitoring
        #[cfg(target_os = "linux")]
        self.start_udev_monitor().await;
        
        #[cfg(target_os = "macos")]
        self.start_iokit_monitor().await;
        
        #[cfg(target_os = "windows")]
        self.start_windows_monitor().await;

        // Fallback to polling if platform-specific monitoring not available
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        self.start_polling_monitor().await;
    }

    async fn check_device_changes(&self) {
        let current_devices = keepkey_rust::features::list_connected_devices();
        let current_ids: Vec<String> = current_devices
            .iter()
            .filter(|d| d.is_keepkey)
            .map(|d| d.unique_id.clone())
            .collect();

        let mut last = self.last_devices.write().await;
        
        // Check for disconnections
        for old_id in last.iter() {
            if !current_ids.contains(old_id) {
                log::info!("Device disconnected: {}", old_id);
                
                // Update session state
                {
                    let mut sessions = DEVICE_SESSIONS.write().await;
                    if let Some(session) = sessions.get_mut(old_id) {
                        if matches!(session.state, DeviceInteractionState::NeedsReconnect { .. }) {
                            session.transition(DeviceInteractionState::WaitingForReconnect).ok();
                        }
                    }
                }

                // Emit event
                emit_device_event(&self.app, DeviceEvent::DeviceDisconnected {
                    device_id: old_id.clone(),
                }).await.ok();
            }
        }

        // Check for connections
        for new_id in &current_ids {
            if !last.contains(new_id) {
                log::info!("Device connected: {}", new_id);
                
                // Handle reconnection
                {
                    let mut sessions = DEVICE_SESSIONS.write().await;
                    if let Some(session) = sessions.get_mut(new_id) {
                        if session.state == DeviceInteractionState::WaitingForReconnect {
                            session.transition(DeviceInteractionState::Reinitializing).ok();
                            
                            // Spawn reinitialization task
                            let device_id = new_id.clone();
                            let app = self.app.clone();
                            tokio::spawn(async move {
                                reinitialize_device(device_id, app).await;
                            });
                        }
                    }
                }

                // Emit event
                emit_device_event(&self.app, DeviceEvent::DeviceConnected {
                    device_id: new_id.clone(),
                }).await.ok();
            }
        }

        *last = current_ids;
    }

    #[cfg(target_os = "linux")]
    async fn start_udev_monitor(self: Arc<Self>) {
        // Use udev crate for Linux USB monitoring
        // This is more efficient than polling
        log::info!("Starting udev USB monitor for Linux");
        // For now, fall back to polling
        self.start_polling_monitor().await;
    }

    #[cfg(target_os = "macos")]
    async fn start_iokit_monitor(self: Arc<Self>) {
        // Use IOKit for macOS USB monitoring
        log::info!("Starting IOKit USB monitor for macOS");
        // For now, fall back to polling but with reduced frequency
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
            loop {
                interval.tick().await;
                self.check_device_changes().await;
            }
        });
    }

    #[cfg(target_os = "windows")]
    async fn start_windows_monitor(self: Arc<Self>) {
        // Use Windows device notifications
        log::info!("Starting Windows USB monitor");
        // For now, fall back to polling
        self.start_polling_monitor().await;
    }

    // Fallback polling monitor for when platform-specific monitoring is not available
    async fn start_polling_monitor(self: Arc<Self>) {
        log::info!("Starting polling USB monitor (fallback mode)");
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                self.check_device_changes().await;
            }
        });
    }
}

async fn reinitialize_device(device_id: String, app: AppHandle) -> Result<(), String> {
    use crate::commands::DeviceQueueManager;
    
    log::info!("Reinitializing device: {}", device_id);
    
    // Find the device
    let devices = keepkey_rust::features::list_connected_devices();
    let device = devices
        .iter()
        .find(|d| d.unique_id == device_id)
        .ok_or_else(|| "Device not found after reconnection".to_string())?;
    
    // Create or get device queue handle
    // Note: In a real implementation, we'd need access to the queue manager
    // For now, just update the session state
    
    // Update state
    {
        let mut sessions = DEVICE_SESSIONS.write().await;
        if let Some(session) = sessions.get_mut(&device_id) {
            session.transition(DeviceInteractionState::Idle)?;
            session.passphrase_cached = false;
            session.passphrase_cache_expiry = None;
        }
    }

    // Update device state
    crate::device::state::set_device_needs_reset(&device_id, false).await;
    
    log::info!("Device {} reinitialized successfully", device_id);
    Ok(())
}