use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{info, error, warn};
use hex;

use crate::transport::{UsbTransport, ProtocolAdapter};
use crate::messages::{self, Message};
use crate::server::routes;
use crate::server::cache::DeviceCache;
use crate::server::{DEVICE_OPERATION_TIMEOUT, try_get_device, try_get_device_with_retry};

// Enhanced UTXO address generation - using cache!
pub(crate) async fn generate_utxo_address_impl(
    request: routes::UtxoAddressRequest,
    cache: &DeviceCache,
    device_mutex: Arc<Mutex<()>>,
) -> Result<routes::UtxoAddressResponse> {
    info!("🚀 Checking cache for UTXO address: coin={}, script_type={:?}, path={:?}", 
        request.coin, request.script_type, request.address_n);
    
    // Map script type to our internal format
    let script_type = request.script_type.as_deref().unwrap_or("p2pkh");
    
    // Check cache first
    if let Some(cached_address) = cache.get_cached_address(&request.coin, script_type, &request.address_n) {
        info!("✨ Found cached address: {}", cached_address.address);
        return Ok(routes::UtxoAddressResponse {
            address: cached_address.address,
            address_n: request.address_n,
        });
    }
    
    // Not in cache - fetch from device with mutex protection
    info!("💫 Address not in cache, fetching from device...");
    
    // Acquire device mutex to prevent concurrent access
    let _lock = device_mutex.lock().await;
    info!("🔒 Device mutex acquired for UTXO address generation");
    
    // Get the USB device
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
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
            _ => msg.script_type = Some(messages::InputScriptType::Spendaddress as i32), // Default to p2pkh
        }
        
        info!("Sending GetAddress message to device for {} with path: {:?}", request.coin, request.address_n);
        
        // Send the message and wait for response
        let response = transport.with_standard_handler().handle(msg.into())?;
        
        // Extract the address from the response
        match response {
            Message::Address(addr_msg) => {
                if !addr_msg.address.is_empty() {
                    // Cache the address for future use
                    if let Some(device_id) = cache.get_device_id() {
                        if let Err(e) = cache.save_address(
                            &device_id,
                            &request.coin,
                            script_type,
                            &request.address_n,
                            &addr_msg.address,
                            None,
                        ).await {
                            warn!("Failed to cache address: {}", e);
                        } else {
                            info!("💾 Cached new address for future use");
                        }
                    }
                    
                    Ok(routes::UtxoAddressResponse {
                        address: addr_msg.address,
                        address_n: request.address_n,
                    })
                } else {
                    Err(anyhow::anyhow!("Device returned empty address"))
                }
            }
            _ => {
                error!("Unexpected response type from device: {:?}", response);
                Err(anyhow::anyhow!("Unexpected response from device"))
            }
        }
    }).await;
    
    match result {
        Ok(Ok(response)) => {
            info!("✅ Got address from device: {}", response.address);
            Ok(response)
        }
        Ok(Err(e)) => {
            error!("Device communication failed: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("Device communication timed out");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}
