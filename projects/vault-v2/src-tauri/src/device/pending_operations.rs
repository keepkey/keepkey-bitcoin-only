use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use keepkey_rust::messages::Message;

/// Stores operations that are waiting for passphrase input
#[derive(Debug, Clone)]
pub struct PendingOperation {
    pub device_id: String,
    pub request_id: String,
    pub operation: Message,
    pub timestamp: std::time::Instant,
}

// Global storage for pending operations
lazy_static::lazy_static! {
    pub static ref PENDING_OPERATIONS: Arc<RwLock<HashMap<String, PendingOperation>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

/// Store an operation that's waiting for passphrase
pub async fn store_pending_operation(
    device_id: String,
    request_id: String,
    operation: Message,
) {
    let mut pending = PENDING_OPERATIONS.write().await;
    let key = format!("{}_{}", device_id, request_id);
    
    let device_id_copy = device_id.clone();
    pending.insert(key, PendingOperation {
        device_id,
        request_id,
        operation,
        timestamp: std::time::Instant::now(),
    });
    
    log::info!("Stored pending operation for device: {}", device_id_copy);
}

/// Retrieve and remove a pending operation
pub async fn take_pending_operation(device_id: &str) -> Option<PendingOperation> {
    let mut pending = PENDING_OPERATIONS.write().await;
    
    // Find the most recent operation for this device
    let key = pending
        .keys()
        .find(|k| k.starts_with(&format!("{}_", device_id)))
        .cloned();
    
    if let Some(key) = key {
        pending.remove(&key)
    } else {
        None
    }
}

/// Clean up old pending operations (older than 5 minutes)
pub async fn cleanup_old_operations() {
    let mut pending = PENDING_OPERATIONS.write().await;
    let now = std::time::Instant::now();
    
    pending.retain(|_, op| {
        now.duration_since(op.timestamp).as_secs() < 300 // 5 minutes
    });
}