use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use tracing::{info, warn, error};
use anyhow::Result;
use crate::transport::ProtocolAdapter;
use hex;

/// Global context for the currently selected device
static DEVICE_CONTEXT: Lazy<Mutex<Option<DeviceContext>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceContext {
    /// The device ID of the currently selected device
    pub device_id: String,
    /// The Ethereum address - helps track wallet state even if device is wiped/reinitialized
    pub eth_address: String,
    /// Optional label for the device
    pub label: Option<String>,
    /// Timestamp when this context was set
    pub set_at: u64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextResponse {
    /// The current device context, if any
    pub context: Option<DeviceContext>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetContextRequest {
    /// The device ID to set as current context
    pub device_id: String,
    /// The Ethereum address - helps track wallet state even if device is wiped/reinitialized
    pub eth_address: String,
    /// Optional label for the device
    pub label: Option<String>,
}

/// Get the current device context
pub async fn get_context() -> axum::Json<ContextResponse> {
    let context = DEVICE_CONTEXT.lock().unwrap().clone();
    axum::Json(ContextResponse { context })
}

/// Set the current device context
pub async fn set_context(payload: axum::Json<SetContextRequest>) -> axum::http::StatusCode {
    let mut context = DEVICE_CONTEXT.lock().unwrap();
    *context = Some(DeviceContext {
        device_id: payload.device_id.clone(),
        eth_address: payload.eth_address.clone(),
        label: payload.label.clone(),
        set_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    });
    
    info!("Device context set: device_id={}, eth_address={}, label={:?}", 
          payload.device_id, payload.eth_address, payload.label);
    
    axum::http::StatusCode::NO_CONTENT
}

/// Clear the current device context
pub async fn clear_context() -> axum::http::StatusCode {
    let mut context = DEVICE_CONTEXT.lock().unwrap();
    *context = None;
    axum::http::StatusCode::NO_CONTENT
}

/// Get the current device ID from context
pub fn get_current_device_id() -> Option<String> {
    DEVICE_CONTEXT.lock().unwrap().as_ref().map(|c| c.device_id.clone())
}

/// Get the current Ethereum address from context
pub fn get_current_eth_address() -> Option<String> {
    DEVICE_CONTEXT.lock().unwrap().as_ref().map(|c| c.eth_address.clone())
}

/// Get both device ID and Ethereum address from context
pub fn get_current_context_info() -> Option<(String, String)> {
    DEVICE_CONTEXT.lock().unwrap().as_ref().map(|c| (c.device_id.clone(), c.eth_address.clone()))
}

/// Helper function to get Ethereum address from device cache
/// This looks for a cached Ethereum address for the device
pub async fn get_cached_eth_address_for_device(device_id: &str) -> Option<String> {
    // Try to get Ethereum address from cache
    if let Ok(cache) = crate::cache::device_cache::DeviceCache::open() {
        // First load the device to populate memory cache
        if let Ok(_) = cache.load_device(device_id).await {
            // Look for Ethereum address at standard path m/44'/60'/0'/0/0
            let eth_path = vec![44 + 0x80000000, 60 + 0x80000000, 0x80000000, 0, 0];
            
            if let Some(cached_address) = cache.get_cached_address("Ethereum", "ethereum", &eth_path) {
                info!("Found cached Ethereum address for device {}: {}", device_id, cached_address.address);
                return Some(cached_address.address);
            }
        }
        
        warn!("No cached Ethereum address found for device {}", device_id);
    } else {
        warn!("Failed to open device cache to get Ethereum address for device {}", device_id);
    }
    
    None
}

/// Get the actual Ethereum address directly from the device (not cache)
/// This is used to verify the device state and detect wallet changes
pub async fn get_real_eth_address_from_device(device_id: &str) -> Result<String> {
    info!("Getting real Ethereum address from device {}", device_id);
    
    // Constants for KeepKey device IDs
    const DEVICE_IDS: &[(u16, u16)] = &[(0x2b24, 0x0001), (0x2b24, 0x0002)];
    
    // Get the first available KeepKey device
    // TODO: In the future, use device_id to select the specific device
    let devices: Box<[rusb::Device<rusb::GlobalContext>]> = rusb::devices()
        .map_err(|e| anyhow::anyhow!("Failed to enumerate USB devices: {}", e))?
        .iter()
        .filter(|device| {
            device.device_descriptor()
                .map(|desc| DEVICE_IDS.contains(&(desc.vendor_id(), desc.product_id())))
                .unwrap_or(false)
        })
        .collect();
    
    let device = devices
        .iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No KeepKey device found"))?;
    
    // Create transport for the device
    let (mut transport, _, _) = crate::transport::UsbTransport::new(device, 0)
        .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;
    
    // Create GetAddress message for Ethereum at m/44'/60'/0'/0/0
    let mut msg = crate::messages::EthereumGetAddress::default();
    msg.address_n = vec![44 + 0x80000000, 60 + 0x80000000, 0x80000000, 0, 0];
    msg.show_display = Some(false); // Don't show on device display
    
    // Send message and get response
    let response = transport.with_standard_handler().handle(msg.into())
        .map_err(|e| anyhow::anyhow!("Failed to get Ethereum address from device: {}", e))?;
    
    match response {
        crate::messages::Message::EthereumAddress(addr_msg) => {
            if !addr_msg.address.is_empty() {
                // Convert bytes to hex string with 0x prefix
                let address = format!("0x{}", hex::encode(&addr_msg.address));
                info!("✅ Got real Ethereum address from device {}: {}", device_id, address);
                return Ok(address);
            }
            Err(anyhow::anyhow!("Device returned empty Ethereum address"))
        }
        _ => {
            error!("Unexpected response from device when getting Ethereum address: {:?}", response);
            Err(anyhow::anyhow!("Unexpected response from device"))
        }
    }
}

/// Validate device cache against actual device state
/// Returns true if cache is valid, false if it should be cleared for this device
pub async fn validate_device_cache(device_id: &str) -> bool {
    info!("Validating cache for device {}", device_id);
    
    // Get the real Ethereum address from the device
    let real_eth_address = match get_real_eth_address_from_device(device_id).await {
        Ok(addr) => addr,
        Err(e) => {
            warn!("Failed to get real Ethereum address from device {}: {}", device_id, e);
            return true; // Assume cache is valid if we can't check
        }
    };
    
    // Get the cached Ethereum address for comparison
    let cached_eth_address = get_cached_eth_address_for_device(device_id).await;
    
    match cached_eth_address {
        Some(cached_addr) => {
            if cached_addr.to_lowercase() == real_eth_address.to_lowercase() {
                info!("✅ Cache validation passed for device {}: addresses match", device_id);
                true
            } else {
                warn!("❌ Cache validation failed for device {}: cached={}, real={}", 
                      device_id, cached_addr, real_eth_address);
                false
            }
        }
        None => {
            info!("📝 No cached Ethereum address for device {}, treating as valid (will be populated)", device_id);
            true
        }
    }
}

/// Clear cache for a specific device if its state has changed
pub async fn clear_device_cache_if_invalid(device_id: &str) -> Result<bool> {
    if !validate_device_cache(device_id).await {
        info!("🧹 Clearing cache for device {} due to wallet state change", device_id);
        
        if let Ok(cache) = crate::cache::device_cache::DeviceCache::open() {
            // Clear all cached data for this specific device
            cache.clear_device(device_id).await
                .map_err(|e| anyhow::anyhow!("Failed to clear device cache: {}", e))?;
            
            info!("✅ Successfully cleared cache for device {}", device_id);
            return Ok(true);
        } else {
            error!("Failed to open cache to clear data for device {}", device_id);
            return Err(anyhow::anyhow!("Failed to open device cache"));
        }
    }
    
    Ok(false)
}

/// Helper function to set context with automatic Ethereum address lookup from device
/// This gets the REAL address from the device, not cache
/// ALSO saves device features to database to avoid foreign key constraint errors during address caching
pub async fn set_context_with_real_eth_address(device_id: String, label: Option<String>) -> axum::http::StatusCode {
    // Constants for KeepKey device IDs
    const DEVICE_IDS: &[(u16, u16)] = &[(0x2b24, 0x0001), (0x2b24, 0x0002)];
    
    // Get the first available KeepKey device
    let devices: Box<[rusb::Device<rusb::GlobalContext>]> = match rusb::devices() {
        Ok(devices) => devices
            .iter()
            .filter(|device| {
                device.device_descriptor()
                    .map(|desc| DEVICE_IDS.contains(&(desc.vendor_id(), desc.product_id())))
                    .unwrap_or(false)
            })
            .collect(),
        Err(e) => {
            warn!("Failed to enumerate USB devices for context setting: {}", e);
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    
    let device = match devices.iter().next() {
        Some(device) => device,
        None => {
            warn!("No KeepKey device found for context setting");
            return axum::http::StatusCode::NOT_FOUND;
        }
    };
    
    // Create transport for the device
    let (mut transport, _, _) = match crate::transport::UsbTransport::new(device, 0) {
        Ok(transport) => transport,
        Err(e) => {
            warn!("Failed to create transport for context setting: {}", e);
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    
    // STEP 1: Get device features and save to database to fix foreign key constraint
    info!("Getting device features to save to database for device {}", device_id);
    
    let features = match transport.with_standard_handler().handle(crate::messages::GetFeatures {}.into()) {
        Ok(crate::messages::Message::Features(features_msg)) => {
            info!("✅ Got device features for {}", device_id);
            features_msg
        }
        Ok(response) => {
            error!("Unexpected response when getting device features: {:?}", response);
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        }
        Err(e) => {
            warn!("Failed to get device features for context setting: {}", e);
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    
    // Save features to database to ensure device exists for address caching
    if let Ok(cache) = crate::cache::device_cache::DeviceCache::open() {
        if let Err(e) = cache.save_features(&features, &device_id).await {
            warn!("Failed to save device features to database: {}", e);
            // Continue anyway - context setting is more important than caching
        } else {
            info!("✅ Saved device features to database for {}", device_id);
        }
    } else {
        warn!("Failed to open device cache to save features");
    }
    
    // STEP 2: Get the real Ethereum address from the device
    info!("Getting real Ethereum address from device {}", device_id);
    
    let eth_address = {
        // Create GetAddress message for Ethereum at m/44'/60'/0'/0/0
        let mut msg = crate::messages::EthereumGetAddress::default();
        msg.address_n = vec![44 + 0x80000000, 60 + 0x80000000, 0x80000000, 0, 0];
        msg.show_display = Some(false); // Don't show on device display
        
        // Send message and get response
        match transport.with_standard_handler().handle(msg.into()) {
            Ok(crate::messages::Message::EthereumAddress(addr_msg)) => {
                if !addr_msg.address.is_empty() {
                    // Convert bytes to hex string with 0x prefix
                    let address = format!("0x{}", hex::encode(&addr_msg.address));
                    info!("✅ Got real Ethereum address from device {}: {}", device_id, address);
                    address
                } else {
                    warn!("Device returned empty Ethereum address for {}, using placeholder", device_id);
                    "0x0000000000000000000000000000000000000000".to_string()
                }
            }
            Ok(response) => {
                error!("Unexpected response when getting Ethereum address: {:?}", response);
                warn!("Failed to get real Ethereum address for device {}, using placeholder", device_id);
                "0x0000000000000000000000000000000000000000".to_string()
            }
            Err(e) => {
                warn!("Failed to get real Ethereum address for device {}: {}, using placeholder", device_id, e);
                "0x0000000000000000000000000000000000000000".to_string()
            }
        }
    };
    
    // STEP 3: Set the context in memory
    let request = SetContextRequest {
        device_id,
        eth_address,
        label,
    };
    
    set_context(axum::Json(request)).await
}

/// Helper function to set context with automatic Ethereum address lookup from cache
/// This is a convenience function that tries to get the Ethereum address from cache
pub async fn set_context_with_cached_eth_address(device_id: String, label: Option<String>) -> axum::http::StatusCode {
    // Try to get Ethereum address from cache
    let eth_address = get_cached_eth_address_for_device(&device_id).await
        .unwrap_or_else(|| {
            warn!("No Ethereum address found in cache for device {}, using placeholder", device_id);
            "0x0000000000000000000000000000000000000000".to_string()
        });
    
    let request = SetContextRequest {
        device_id,
        eth_address,
        label,
    };
    
    set_context(axum::Json(request)).await
}
