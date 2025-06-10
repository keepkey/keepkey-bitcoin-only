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

// Enhanced cosmos address generation with timeout and error handling - NO MOCKS
pub(crate) async fn generate_cosmos_address_impl(
    request: routes::CosmosAddressRequest,
    cache: &DeviceCache,
    device_mutex: Arc<Mutex<()>>,
) -> Result<String> {
    info!("🚀 Checking cache for Cosmos address: path={:?}", request.address_n);
    
    // Check cache first
    if let Some(cached_address) = cache.get_cached_address("Cosmos", "cosmos", &request.address_n) {
        info!("✨ Found cached Cosmos address: {}", cached_address.address);
        return Ok(cached_address.address);
    }
    
    // Not in cache - fetch from device with mutex protection
    info!("💫 Cosmos address not in cache, fetching from device...");
    
    // Acquire device mutex to prevent concurrent access
    let _lock = device_mutex.lock().await;
    info!("🔒 Device mutex acquired for Cosmos address generation");
    
    // Get the USB device - fail if not available
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
        // Create GetAddress message for Cosmos
        let get_address_msg = messages::GetAddress {
            coin_name: Some("Cosmos".to_string()),
            address_n: request.address_n.clone(),
            script_type: None,
            show_display: request.show_display,
            multisig: None,
        };
        
        info!("Sending GetAddress message to device for Cosmos with path: {:?}", request.address_n);
        
        // Send the message and wait for response
        let response = transport.with_standard_handler().handle(get_address_msg.into())?;
        
        // Extract the address from the response
        match response {
            Message::Address(addr_msg) => {
                if !addr_msg.address.is_empty() {
                    // Cache the address for future use
                    if let Some(device_id) = cache.get_device_id() {
                        if let Err(e) = cache.save_address(
                            &device_id,
                            "Cosmos",
                            "cosmos",
                            &request.address_n,
                            &addr_msg.address,
                            None,
                        ).await {
                            warn!("Failed to cache Cosmos address: {}", e);
                        } else {
                            info!("💾 Cached new Cosmos address for future use");
                        }
                    }
                    Ok(addr_msg.address)
                } else {
                    Err(anyhow::anyhow!("Device returned empty Cosmos address"))
                }
            }
            _ => {
                error!("Unexpected response type from device: {:?}", response);
                Err(anyhow::anyhow!("Unexpected response from device"))
            }
        }
    }).await;
    
    match result {
        Ok(Ok(address)) => {
            info!("✅ Got Cosmos address from device: {}", address);
            Ok(address)
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

// Enhanced mayachain address generation with timeout and error handling - NO MOCKS
pub(crate) async fn generate_mayachain_address_impl(
    request: routes::CosmosAddressRequest,
    cache: &DeviceCache,
    device_mutex: Arc<Mutex<()>>,
) -> Result<String> {
    info!("🚀 Checking cache for Mayachain address: path={:?}", request.address_n);
    
    // Check cache first
    if let Some(cached_address) = cache.get_cached_address("Mayachain", "mayachain", &request.address_n) {
        info!("✨ Found cached Mayachain address: {}", cached_address.address);
        return Ok(cached_address.address);
    }
    
    // Not in cache - fetch from device with mutex protection
    info!("💫 Mayachain address not in cache, fetching from device...");
    
    // Acquire device mutex to prevent concurrent access
    let _lock = device_mutex.lock().await;
    info!("🔒 Device mutex acquired for Mayachain address generation");
    
    // Get the USB device - fail if not available
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
        // Create MayachainGetAddress message for proper bech32 "maya" prefix address
        let mayachain_get_address_msg = messages::MayachainGetAddress {
            address_n: request.address_n.clone(),
            show_display: request.show_display,
            testnet: None, // None for mainnet, Some(true) for testnet
        };
        
        info!("Sending MayachainGetAddress message to device with path: {:?}", request.address_n);
        
        // Send the message and wait for response
        let response = transport.with_standard_handler().handle(mayachain_get_address_msg.into())?;
        
        // Extract the address from the response
        match response {
            Message::MayachainAddress(addr_msg) => {
                let address = addr_msg.address.ok_or_else(|| anyhow::anyhow!("No address in response"))?;
                info!("Received Mayachain address from device: {}", address);
                
                // Cache the address for future use
                if let Some(device_id) = cache.get_device_id() {
                    if let Err(e) = cache.save_address(
                        &device_id,
                        "Mayachain",
                        "mayachain",
                        &request.address_n,
                        &address,
                        None,
                    ).await {
                        warn!("Failed to cache Mayachain address: {}", e);
                    } else {
                        info!("💾 Cached new Mayachain address for future use");
                    }
                }
                
                Ok(address)
            }
            _ => {
                error!("Unexpected response type from device: {:?}", response);
                Err(anyhow::anyhow!("Unexpected response from device"))
            }
        }
    }).await;
    
    match result {
        Ok(Ok(address)) => {
            info!("✅ Got Mayachain address from device: {}", address);
            Ok(address)
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

// Enhanced ethereum address generation with timeout and error handling - NO MOCKS
pub(crate) async fn generate_eth_address_impl(
    request: routes::EthAddressRequest,
    cache: &DeviceCache,
    device_mutex: Arc<Mutex<()>>,
) -> Result<String> {
    info!("🚀 Checking cache for Ethereum address: path={:?}", request.address_n);
    
    // Check cache first
    if let Some(cached_address) = cache.get_cached_address("Ethereum", "ethereum", &request.address_n) {
        info!("✨ Found cached Ethereum address: {}", cached_address.address);
        return Ok(cached_address.address);
    }
    
    // Not in cache - fetch from device with mutex protection and retry logic
    info!("💫 Ethereum address not in cache, fetching from device...");
    
    // Acquire device mutex to prevent concurrent access
    let _lock = device_mutex.lock().await;
    info!("🔒 Device mutex acquired for Ethereum address generation");
    
    // Retry logic for device communication
    let max_retries = 3;
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        info!("🔄 Device communication attempt {}/{}", attempt, max_retries);
        
        match try_get_device_with_retry().await {
            Ok(device) => {
                // Wrap device communication in timeout
                let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
                    let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
                    
                    // Create EthereumGetAddress message for proper hex format
                    let ethereum_get_address_msg = messages::EthereumGetAddress {
                        address_n: request.address_n.clone(),
                        show_display: request.show_display,
                    };
                    
                    info!("Sending EthereumGetAddress message to device for Ethereum with path: {:?}", request.address_n);
                    
                    // Send the message and wait for response
                    let response = transport.with_standard_handler().handle(ethereum_get_address_msg.into())?;
                    
                    // Extract the address from the response
                    match response {
                        Message::EthereumAddress(addr_msg) => {
                            if !addr_msg.address.is_empty() {
                                // Convert bytes to hex string with 0x prefix
                                let address = format!("0x{}", hex::encode(&addr_msg.address));
                                
                                // Cache the address for future use
                                if let Some(device_id) = cache.get_device_id() {
                                    if let Err(e) = cache.save_address(
                                        &device_id,
                                        "Ethereum",
                                        "ethereum",
                                        &request.address_n,
                                        &address,
                                        None,
                                    ).await {
                                        warn!("Failed to cache Ethereum address: {}", e);
                                    } else {
                                        info!("💾 Cached new Ethereum address for future use");
                                    }
                                }
                                Ok(address)
                            } else {
                                Err(anyhow::anyhow!("Device returned empty Ethereum address"))
                            }
                        }
                        _ => {
                            error!("Unexpected response type from device: {:?}", response);
                            Err(anyhow::anyhow!("Unexpected response from device"))
                        }
                    }
                }).await;
                
                match result {
                    Ok(Ok(address)) => {
                        info!("✅ Got Ethereum address from device: {}", address);
                        return Ok(address);
                    }
                    Ok(Err(e)) => {
                        error!("Device communication failed on attempt {}: {}", attempt, e);
                        last_error = Some(e);
                        
                        if attempt < max_retries {
                            let delay_ms = (100 * attempt * attempt) as u64; // Exponential backoff
                            info!("⏳ Waiting {}ms before retry...", delay_ms);
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        }
                    }
                    Err(_) => {
                        error!("Device communication timed out on attempt {}", attempt);
                        last_error = Some(anyhow::anyhow!("Device operation timed out"));
                        
                        if attempt < max_retries {
                            let delay_ms = (200 * attempt) as u64; // Linear backoff for timeouts
                            info!("⏳ Waiting {}ms before retry...", delay_ms);
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to get device on attempt {}: {}", attempt, e);
                last_error = Some(e);
                
                if attempt < max_retries {
                    let delay_ms = (500 * attempt) as u64; // Longer delay for device detection issues
                    info!("⏳ Waiting {}ms before device retry...", delay_ms);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
    
    // All retries failed
    error!("❌ All {} device communication attempts failed", max_retries);
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Device communication failed after all retries")))
}

// Enhanced thorchain address generation
pub(crate) async fn generate_thorchain_address_impl(
    request: routes::CosmosAddressRequest,
    cache: &DeviceCache,
    device_mutex: Arc<Mutex<()>>,
) -> Result<String> {
    info!("🚀 Checking cache for Thorchain address: path={:?}", request.address_n);
    
    // Check cache first
    if let Some(cached_address) = cache.get_cached_address("Thorchain", "thorchain", &request.address_n) {
        info!("✨ Found cached Thorchain address: {}", cached_address.address);
        return Ok(cached_address.address);
    }
    
    // Not in cache - fetch from device with mutex protection
    info!("💫 Thorchain address not in cache, fetching from device...");
    
    // Acquire device mutex to prevent concurrent access
    let _lock = device_mutex.lock().await;
    info!("🔒 Device mutex acquired for Thorchain address generation");
    
    // Get the USB device - fail if not available
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
        // Create ThorchainGetAddress message for proper bech32 "thor" prefix address
        let thorchain_get_address_msg = messages::ThorchainGetAddress {
            address_n: request.address_n.clone(),
            show_display: request.show_display,
            testnet: None, // None for mainnet, Some(true) for testnet
        };
        
        info!("Sending ThorchainGetAddress message to device with path: {:?}", request.address_n);
        
        // Send the message and wait for response
        let response = transport.with_standard_handler().handle(thorchain_get_address_msg.into())?;
        
        // Extract the address from the response
        match response {
            Message::ThorchainAddress(addr_msg) => {
                let address = addr_msg.address.ok_or_else(|| anyhow::anyhow!("No address in response"))?;
                if !address.is_empty() {
                    // Cache the address for future use
                    if let Some(device_id) = cache.get_device_id() {
                        if let Err(e) = cache.save_address(
                            &device_id,
                            "Thorchain",
                            "thorchain",
                            &request.address_n,
                            &address,
                            None,
                        ).await {
                            warn!("Failed to cache Thorchain address: {}", e);
                        } else {
                            info!("💾 Cached new Thorchain address for future use");
                        }
                    }
                    Ok(address)
                } else {
                    Err(anyhow::anyhow!("Device returned empty Thorchain address"))
                }
            }
            _ => {
                error!("Unexpected response type from device: {:?}", response);
                Err(anyhow::anyhow!("Unexpected response from device"))
            }
        }
    }).await;
    
    match result {
        Ok(Ok(address)) => {
            info!("✅ Got Thorchain address from device: {}", address);
            Ok(address)
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

// Placeholder implementations for other chains
pub(crate) async fn generate_osmosis_address_impl(_request: routes::CosmosAddressRequest) -> Result<String> {
    error!("Osmosis address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn generate_bnb_address_impl(_request: routes::CosmosAddressRequest) -> Result<String> {
    error!("Binance Chain address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn generate_xrp_address_impl(_request: routes::RippleAddressRequest) -> Result<String> {
    error!("Ripple address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn generate_eos_address_impl(_request: routes::EosAddressRequest) -> Result<String> {
    error!("EOS address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn generate_nano_address_impl(_request: routes::NanoAddressRequest) -> Result<String> {
    error!("Nano address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
} 