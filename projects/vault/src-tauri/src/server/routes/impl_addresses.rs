use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, warn};

use crate::server::routes;
use crate::cache::device_cache::DeviceCache;
use crate::messages::{self, Message};
use crate::transport::{UsbTransport, ProtocolAdapter};

// Constants for KeepKey device IDs
const DEVICE_IDS: &[(u16, u16)] = &[(0x2b24, 0x0001), (0x2b24, 0x0002)];

// Simplified UTXO address generation - cache-first with device fallback
#[derive(Debug, thiserror::Error)]
pub enum UtxoAddressError {
    #[error("Address not cached and device not available")]
    NotCached,
    #[error("No device context set. Please select a device first.")]
    NoContext,
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub(crate) async fn generate_utxo_address_impl(
    request: routes::UtxoAddressRequest,
    cache: &DeviceCache,
    _device_mutex: Arc<Mutex<()>>,
) -> Result<routes::UtxoAddressResponse, UtxoAddressError> {
    info!("🚀 V1 API: UTXO address request - coin={}, script_type={:?}, path={:?}", 
        request.coin, request.script_type, request.address_n);
    
    // Map script type to our internal format
    let script_type = request.script_type.as_deref().unwrap_or("p2pkh");
    
    // Check cache first, but ALWAYS go to device if show_display is true
    if !request.show_display.unwrap_or(false) {
        if let Some(cached_address) = cache.get_cached_address(&request.coin, script_type, &request.address_n) {
            info!("✨ Found cached address: {}", cached_address.address);
            return Ok(routes::UtxoAddressResponse {
                address: cached_address.address,
                address_n: request.address_n,
            });
        }
    } else {
        info!("🔍 show_display=true, bypassing cache and going to device");
    }
    
    // Address not in cache - try to get from device
    info!("📱 Address not cached, attempting to fetch from device...");
    
    // Get current device context
    let device_id = crate::server::context::get_current_device_id()
        .ok_or(UtxoAddressError::NoContext)?;
    
    info!("🎯 Using device context: {}", device_id);
    
    // Get the first available KeepKey device
    // Note: In the future, we would use the device_id to select the specific device
    // For now, we just get the first available device
    let devices: Box<[rusb::Device<rusb::GlobalContext>]> = rusb::devices()
        .map_err(|e| {
            error!("Failed to enumerate USB devices: {}", e);
            UtxoAddressError::Other(anyhow::anyhow!("Failed to enumerate USB devices: {}", e))
        })?
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
        .ok_or_else(|| {
            error!("No KeepKey device found");
            UtxoAddressError::DeviceNotFound("No KeepKey device connected".to_string())
        })?;
    
    // Create transport for the device
    let (mut transport, _, _) = UsbTransport::new(device, 0)
        .map_err(|e| {
            error!("Failed to create transport: {}", e);
            UtxoAddressError::Other(anyhow::anyhow!("Failed to create transport: {}", e))
        })?;
    
    // Create GetAddress message
    let mut msg = messages::GetAddress::default();
    msg.address_n = request.address_n.clone();
    msg.coin_name = Some(request.coin.clone());
    msg.show_display = request.show_display;
    
    // Set script type
    match script_type {
        "p2pkh" => msg.script_type = Some(messages::InputScriptType::Spendaddress as i32),
        "p2wpkh" => msg.script_type = Some(messages::InputScriptType::Spendwitness as i32),
        "p2sh-p2wpkh" => msg.script_type = Some(messages::InputScriptType::Spendp2shwitness as i32),
        _ => {
            error!("Unknown script type: {}", script_type);
            return Err(UtxoAddressError::Other(anyhow::anyhow!("Unknown script type: {}", script_type)));
        }
    }
    
    // Send message and get response
    let response = transport.with_standard_handler().handle(msg.into())
        .map_err(|e| {
            error!("Failed to get address from device: {}", e);
            UtxoAddressError::Other(anyhow::anyhow!("Device communication failed: {}", e))
        })?;
    
    match response {
        Message::Address(addr_msg) => {
            if !addr_msg.address.is_empty() {
                info!("✅ Got address from device: {}", addr_msg.address);
                
                // Save to cache for future use
                if let Err(e) = cache.save_address(
                    &device_id,
                    &request.coin,
                    script_type,
                    &request.address_n,
                    &addr_msg.address,
                    None, // We're not storing pubkeys for now
                ).await {
                    warn!("Failed to cache address: {}", e);
                    // Continue anyway - we have the address
                }
                
                Ok(routes::UtxoAddressResponse {
                    address: addr_msg.address,
                    address_n: request.address_n,
                })
            } else {
                error!("Device returned empty address");
                Err(UtxoAddressError::Other(anyhow::anyhow!("Device returned empty address")))
            }
        }
        _ => {
            error!("Unexpected response from device: {:?}", response);
            Err(UtxoAddressError::Other(anyhow::anyhow!("Unexpected response from device")))
        }
    }
}
