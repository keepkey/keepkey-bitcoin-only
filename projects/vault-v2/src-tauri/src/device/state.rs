use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

/// Device state that tracks current status and needs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceState {
    pub device_id: String,
    
    // Authentication states
    pub is_unlocked: bool,        // Device is unlocked (PIN entered)
    pub needs_passphrase: bool,   // Device needs passphrase
    pub needs_reset: bool,         // Device needs reconnect/reset
    
    // Operation states
    pub is_busy: bool,             // Device is currently processing
    pub current_operation: Option<String>, // What operation is running
    
    // Additional metadata
    pub last_update: u64,          // Unix timestamp
    pub pin_cached: bool,          // PIN is cached in device
    pub passphrase_cached: bool,   // Passphrase is cached in device
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            is_unlocked: false,
            needs_passphrase: false,
            needs_reset: false,
            is_busy: false,
            current_operation: None,
            last_update: 0,
            pin_cached: false,
            passphrase_cached: false,
        }
    }
}

/// Global device state tracker
lazy_static::lazy_static! {
    pub static ref DEVICE_STATE_TRACKER: Arc<RwLock<HashMap<String, DeviceState>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

/// Update device state based on features
pub async fn update_device_state_from_features(
    device_id: &str, 
    features: &keepkey_rust::messages::Features
) {
    let mut states = DEVICE_STATE_TRACKER.write().await;
    
    let state = states.entry(device_id.to_string()).or_insert_with(|| {
        DeviceState {
            device_id: device_id.to_string(),
            ..Default::default()
        }
    });
    
    // Update based on features
    state.is_unlocked = !features.pin_protection.unwrap_or(false) || 
                       features.pin_cached.unwrap_or(false);
    state.pin_cached = features.pin_cached.unwrap_or(false);
    state.passphrase_cached = features.passphrase_cached.unwrap_or(false);
    state.needs_passphrase = features.passphrase_protection.unwrap_or(false) && 
                             !features.passphrase_cached.unwrap_or(false);
    state.last_update = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Clear reset flag if we got features successfully
    state.needs_reset = false;
    
    println!("📊 Updated device state for {}: unlocked={}, needs_passphrase={}, busy={}", 
             device_id, state.is_unlocked, state.needs_passphrase, state.is_busy);
}

/// Mark device as busy with an operation
pub async fn set_device_busy(device_id: &str, operation: Option<String>) {
    let mut states = DEVICE_STATE_TRACKER.write().await;
    
    if let Some(state) = states.get_mut(device_id) {
        state.is_busy = operation.is_some();
        state.current_operation = operation;
        state.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        println!("⚙️ Device {} busy state: {} ({})", 
                 device_id, 
                 state.is_busy, 
                 state.current_operation.as_ref().unwrap_or(&"idle".to_string()));
    }
}

/// Mark device as needing reset
pub async fn set_device_needs_reset(device_id: &str, needs_reset: bool) {
    let mut states = DEVICE_STATE_TRACKER.write().await;
    
    if let Some(state) = states.get_mut(device_id) {
        state.needs_reset = needs_reset;
        state.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        if needs_reset {
            // Clear other states when device needs reset
            state.is_unlocked = false;
            state.is_busy = false;
            state.current_operation = None;
        }
        
        println!("🔄 Device {} needs_reset: {}", device_id, needs_reset);
    }
}

/// Get current device state
pub async fn get_device_state(device_id: &str) -> Option<DeviceState> {
    let states = DEVICE_STATE_TRACKER.read().await;
    states.get(device_id).cloned()
}

/// Get all device states
pub async fn get_all_device_states() -> HashMap<String, DeviceState> {
    let states = DEVICE_STATE_TRACKER.read().await;
    states.clone()
}

/// Update PIN state
pub async fn set_device_pin_state(device_id: &str, is_unlocked: bool, pin_cached: bool) {
    let mut states = DEVICE_STATE_TRACKER.write().await;
    
    if let Some(state) = states.get_mut(device_id) {
        state.is_unlocked = is_unlocked;
        state.pin_cached = pin_cached;
        state.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        println!("🔐 Device {} PIN state: unlocked={}, cached={}", 
                 device_id, is_unlocked, pin_cached);
    }
}

/// Update passphrase state
pub async fn set_device_passphrase_state(device_id: &str, needs_passphrase: bool, passphrase_cached: bool) {
    let mut states = DEVICE_STATE_TRACKER.write().await;
    
    if let Some(state) = states.get_mut(device_id) {
        state.needs_passphrase = needs_passphrase;
        state.passphrase_cached = passphrase_cached;
        state.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        println!("🔑 Device {} passphrase state: needs={}, cached={}", 
                 device_id, needs_passphrase, passphrase_cached);
    }
}