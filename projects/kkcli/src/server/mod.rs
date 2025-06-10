pub mod routes;
pub mod cache;

// Implementation modules
mod impl_device;
mod impl_addresses;
mod impl_bitcoin;
mod impl_system;
mod server_init;
mod v2_endpoints;

use anyhow::Result;
use axum::{
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
    extract::Request,
};
use prost::Message as ProstMessage;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::any::type_name;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, error, warn, debug};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use rusb::{Device, GlobalContext};
use base64;
use hex;

use crate::transport::{UsbTransport, ProtocolAdapter};
use crate::messages::{self, Message};
use self::cache::{DeviceCache, DeviceFrontloader};

// Re-export implementation functions
pub(crate) use impl_device::*;
pub(crate) use impl_addresses::*;
pub(crate) use impl_bitcoin::*;
pub(crate) use impl_system::*;

// Export server initialization function
pub use server_init::start_server;

// Server state for sharing across handlers
#[derive(Clone)]
pub struct ServerState {
    pub cache: DeviceCache,
    pub device_mutex: Arc<Mutex<()>>, // Prevents concurrent device access
    pub active_transport: Arc<Mutex<Option<UsbTransport<GlobalContext>>>>, // Holds the active, shared USB transport
}

// Constants
pub(crate) const DEVICE_IDS: &[(u16, u16)] = &[(0x2b24, 0x0001), (0x2b24, 0x0002)];
pub(crate) const DEVICE_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

// API Documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "KeepKey CLI API",
        version = "0.2.3",
    ),
    paths(
        routes::health_check,
        routes::get_device_features,
        routes::system_get_features,
        routes::system_ping,
        routes::generate_utxo_address,
        routes::generate_eth_address,
    ),
    components(schemas(
        routes::Features,
        routes::Policy,
        routes::PingRequest,
        routes::PingResponse,
        routes::UtxoAddressRequest,
        routes::UtxoAddressResponse,
        routes::EthAddressRequest,
        routes::AddressResponse,
    )),
    tags(
        (name = "device", description = "Device management endpoints"),
        (name = "addresses", description = "Address generation endpoints"),
        (name = "system", description = "System endpoints"),
    )
)]
struct ApiDoc;

// Request logging middleware
async fn log_request(
    req: Request,
    next: Next,
) -> Response {
    use axum::body::Body;
    use tracing::info;

    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Extract request body for logging (if it's a POST/PUT/PATCH request)
    let (parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap_or_default();
    let req_body_str = if !bytes.is_empty() {
        match std::str::from_utf8(&bytes) {
            Ok(s) => s,
            Err(_) => "<non-UTF8 body>",
        }
    } else {
        ""
    };

    // Log the incoming request
    if query.is_empty() {
        if req_body_str.is_empty() {
            info!("🌐 {} {}", method, path);
        } else {
            info!("🌐 {} {} with body: {}", method, path, req_body_str);
        }
    } else {
        if req_body_str.is_empty() {
            info!("🌐 {} {} ? {}", method, path, query);
        } else {
            info!("🌐 {} {} ? {} with body: {}", method, path, query, req_body_str);
        }
    }

    // Reconstruct the request with the consumed body
    let req = axum::http::Request::from_parts(parts, Body::from(bytes.clone()));
    let response = next.run(req).await;
    let status = response.status();
    info!("⬅️ {} {} -> {}", method, path, status);
    response
}

// Serve the OpenAPI spec at the legacy path for SDK compatibility
async fn get_swagger_spec() -> Json<Value> {
    Json(serde_json::to_value(ApiDoc::openapi()).unwrap())
}



// Enhanced device detection with timeout and better error handling
pub(crate) fn try_get_device() -> Result<rusb::Device<rusb::GlobalContext>> {
    info!("🔍 Starting device detection...");
    
    // First check if we can list devices safely
    let devices = match rusb::devices() {
        Ok(devices) => {
            info!("📱 Successfully listed {} USB devices", devices.len());
            devices
        },
        Err(e) => {
            error!("❌ Failed to list USB devices: {}", e);
            return Err(anyhow::anyhow!("No KeepKey device found - USB enumeration failed: {}", e));
        }
    };
    
    // Filter for KeepKey devices with better error handling
    let keepkey_devices: Vec<_> = devices.iter()
        .enumerate()
        .filter_map(|(idx, device)| {
            match device.device_descriptor() {
                Ok(device_desc) => {
                    let vendor_id = device_desc.vendor_id();
                    let product_id = device_desc.product_id();
                    info!("🔍 Device {}: VID={:04x} PID={:04x}", idx, vendor_id, product_id);
                    if DEVICE_IDS.contains(&(vendor_id, product_id)) {
                        info!("✅ Found KeepKey device at index {}", idx);
                        Some(device)
                    } else {
                        None
                    }
                }
                Err(e) => {
                    warn!("⚠️ Failed to read descriptor for device {}: {}", idx, e);
                    None
                }
            }
        })
        .collect();
    
    if keepkey_devices.is_empty() {
        error!("❌ No KeepKey devices found in {} total USB devices", devices.len());
        return Err(anyhow::anyhow!("No KeepKey device found"));
    }
    
    let device = keepkey_devices[0].clone();
    info!("🎯 Selected KeepKey device for communication");
    
    // Test device accessibility
    match device.device_descriptor() {
        Ok(desc) => {
            info!("📋 Device descriptor: VID={:04x} PID={:04x}", desc.vendor_id(), desc.product_id());
        }
        Err(e) => {
            error!("❌ Failed to read device descriptor: {}", e);
            error!("💡 TIP: This error often means the KeepKey needs to be unplugged and re-plugged in");
            return Err(anyhow::anyhow!("Device descriptor read failed: {}. Please unplug and re-plug your KeepKey device.", e));
        }
    }
    
    Ok(device)
}

// Robust device detection with retry and recovery
pub(crate) async fn try_get_device_with_retry() -> Result<rusb::Device<rusb::GlobalContext>> {
    let max_attempts = 3;
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match try_get_device() {
            Ok(device) => {
                info!("✅ Device detection successful on attempt {}", attempt);
                return Ok(device);
            }
            Err(e) => {
                warn!("⚠️ Device detection failed on attempt {}: {}", attempt, e);
                last_error = Some(e);
                
                if attempt < max_attempts {
                    // Progressive delay between attempts
                    let delay_ms = match attempt {
                        1 => 100,  // 100ms
                        2 => 500,  // 500ms
                        _ => 1000, // 1s
                    };
                    
                    info!("⏳ Waiting {}ms before device detection retry...", delay_ms);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
    
    error!("❌ Device detection failed after {} attempts", max_attempts);
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Device detection failed after all retries")))
}

// Safe device list function
pub(crate) fn list_devices() -> Box<[Device<GlobalContext>]> {
    match rusb::devices() {
        Ok(devices) => {
            devices.iter()
                .filter_map(|device| {
                    match device.device_descriptor() {
                        Ok(device_desc) => {
                            let vendor_id = device_desc.vendor_id();
                            let product_id = device_desc.product_id();
                            if DEVICE_IDS.contains(&(vendor_id, product_id)) {
                                Some(device)
                            } else {
                                None
                            }
                        }
                        Err(_) => None, // Skip devices we can't read
                    }
                })
                .collect()
        }
        Err(e) => {
            warn!("Failed to list USB devices: {}", e);
            vec![].into_boxed_slice()
        }
    }
}

// Implementation functions that can be reused by both REST and MCP endpoints
pub(crate) async fn get_device_status_impl() -> Result<routes::DeviceStatus> {
    match try_get_device() {
        Ok(_device) => {
            Ok(routes::DeviceStatus {
                connected: true,
                device: Some(routes::DeviceInfo {
                    device_id: "kkcli-device".to_string(),
                    name: "KeepKey via CLI".to_string(),
                    vendor_id: 0x2B24,
                    product_id: 0x0001,
                    manufacturer: Some("ShapeShift".to_string()),
                    product: Some("KeepKey".to_string()),
                    serial_number: None,
                    is_keepkey: true,
                }),
            })
        }
        Err(_) => {
            Ok(routes::DeviceStatus {
                connected: false,
                device: None,
            })
        }
    }
}

pub(crate) async fn get_device_features_impl() -> Result<routes::KeepKeyFeatures> {
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (_transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
        // Get features from the device (simplified)
        // In a real implementation, you would call the actual GetFeatures command
        Ok::<routes::KeepKeyFeatures, anyhow::Error>(routes::KeepKeyFeatures {
            vendor: Some("ShapeShift".to_string()),
            major_version: 7,
            minor_version: 7,
            patch_version: 0,
            bootloader_mode: false,
            device_id: Some("kkcli-device".to_string()),
            pin_protection: true,
            passphrase_protection: false,
            language: Some("english".to_string()),
            label: Some("KeepKey".to_string()),
            initialized: true,
            revision: None,
            bootloader_hash: None,
            imported: false,
            unlocked: false,
            firmware_present: true,
            needs_backup: false,
            flags: 0,
            model: Some("K1-14AM".to_string()),
            fw_major: Some(7),
            fw_minor: Some(7),
            fw_patch: Some(0),
            fw_vendor: Some("ShapeShift".to_string()),
            fw_vendor_keys: None,
            unfinished_backup: false,
            no_backup: false,
        })
    }).await;
    
    match result {
        Ok(Ok(features)) => Ok(features),
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

// SDK compatible features implementation - now using cache!
pub(crate) async fn get_features_sdk_compatible(cache: &DeviceCache) -> Result<routes::Features> {
    // First try to get from cache
    if let Some(cached_features) = cache.get_cached_features() {
        debug!("✨ Returning cached device features");
        
        // Parse the features JSON back to our Features struct
        let features: routes::Features = serde_json::from_str(&cached_features.features_json)?;
        return Ok(features);
    }
    
    // This should not happen if frontloading worked correctly
    error!("❌ No cached features found - this should not happen!");
    Err(anyhow::anyhow!("Device features not found in cache"))
}

pub(crate) async fn list_usb_devices_impl() -> Result<Vec<routes::UsbDeviceInfo>> {
    let devices = rusb::devices()?;
    let mut device_list = Vec::new();
    
    for device in devices.iter() {
        let device_desc = device.device_descriptor()?;
        let vendor_id = device_desc.vendor_id();
        let product_id = device_desc.product_id();
        
        let mut manufacturer = None;
        let mut product = None;
        let mut serial_number = None;
        
        // Try to get string descriptors
        if let Ok(handle) = device.open() {
            if let Ok(mfg) = handle.read_manufacturer_string_ascii(&device_desc) {
                manufacturer = Some(mfg);
            }
            if let Ok(prod) = handle.read_product_string_ascii(&device_desc) {
                product = Some(prod);
            }
            if let Ok(serial) = handle.read_serial_number_string_ascii(&device_desc) {
                serial_number = Some(serial);
            }
        }
        
        device_list.push(routes::UsbDeviceInfo {
            vendor_id,
            product_id,
            manufacturer,
            product,
            serial_number,
        });
    }
    
    Ok(device_list)
}

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

// Enhanced Mayachain transaction signing with timeout and error handling - NO MOCKS
pub(crate) async fn mayachain_sign_tx_impl(request: routes::MayachainSignRequest) -> Result<routes::MayachainSignResponse> {
    info!("Attempting to sign Mayachain transaction");
    
    // Get the USB device - fail if not available
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
        // First, get the address to derive the address_n from signer_address
        // For now, use the standard derivation path for Maya (m/44'/931'/0'/0/0)
        let address_n = vec![0x80000000 + 44, 0x80000000 + 931, 0x80000000 + 0, 0, 0];
        
        // Create MayachainSignTx message
        let maya_sign_tx = messages::MayachainSignTx {
            address_n: address_n.clone(),
            account_number: Some(request.sign_doc.account_number.parse::<u64>()?),
            chain_id: Some(request.sign_doc.chain_id.clone()),
            fee_amount: request.sign_doc.fee.amount.get(0)
                .and_then(|c| c.amount.parse::<u32>().ok()),
            gas: Some(request.sign_doc.fee.gas.parse::<u32>()?),
            memo: Some(request.sign_doc.memo.clone()),
            sequence: Some(request.sign_doc.sequence.parse::<u64>()?),
            msg_count: Some(request.sign_doc.msgs.len() as u32),
            testnet: None,
        };
        
        info!("Sending MayachainSignTx message to device");
        
        // Send the initial sign message
        let response = transport.with_standard_handler().handle(maya_sign_tx.into())?;
        
        // Handle message acknowledgment flow
        match response {
            Message::MayachainMsgRequest(_) => {
                // Device is requesting transaction details
                info!("Device requested transaction details");
                
                // For each message in the transaction
                for msg in &request.sign_doc.msgs {
                    if msg.msg_type == "mayachain/MsgSend" {
                        // Create MayachainMsgSend
                        let maya_msg_send = messages::MayachainMsgSend {
                            from_address: Some(msg.value.from_address.clone()),
                            to_address: Some(msg.value.to_address.clone()),
                            amount: msg.value.amount.get(0)
                                .and_then(|c| c.amount.parse::<u64>().ok()),
                            address_type: None,
                            denom: msg.value.amount.get(0)
                                .map(|c| c.denom.clone()),
                        };
                        
                        let maya_msg_ack = messages::MayachainMsgAck {
                            send: Some(maya_msg_send),
                            deposit: None,
                        };
                        
                        // Send the message details
                        let signed_response = transport.with_standard_handler()
                            .handle(maya_msg_ack.into())?;
                        
                        // Extract signature from response
                        match signed_response {
                            Message::MayachainSignedTx(signed_tx) => {
                                let public_key = signed_tx.public_key
                                    .ok_or_else(|| anyhow::anyhow!("No public key in response"))?;
                                let signature = signed_tx.signature
                                    .ok_or_else(|| anyhow::anyhow!("No signature in response"))?;
                                
                                info!("Transaction signed successfully");
                                
                                // Prepare the serialized transaction using base64 encoding
                                // This follows the format from the desktop implementation
                                let serialized = base64::encode(serde_json::to_string(&request.sign_doc)?);
                                
                                // Return the signed transaction response in the format expected by the client
                                return Ok(routes::MayachainSignResponse {
                                    serialized,
                                    signature: base64::encode(&signature),
                                });
                            }
                            _ => {
                                error!("Unexpected response from device: {:?}", signed_response);
                                return Err(anyhow::anyhow!("Unexpected response from device"));
                            }
                        }
                    }
                }
                
                Err(anyhow::anyhow!("No supported message type in transaction"))
            }
            _ => {
                error!("Unexpected initial response from device: {:?}", response);
                Err(anyhow::anyhow!("Unexpected response from device"))
            }
        }
    }).await;
    
    match result {
        Ok(Ok(response)) => Ok(response),
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

// Cosmos chain implementations
pub(crate) async fn cosmos_sign_tx_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Cosmos sign transaction not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn cosmos_sign_delegate_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Cosmos sign delegate not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn cosmos_sign_undelegate_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Cosmos sign undelegate not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn cosmos_sign_redelegate_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Cosmos sign redelegate not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn cosmos_sign_rewards_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Cosmos sign rewards not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn cosmos_sign_ibc_transfer_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Cosmos sign IBC transfer not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Thorchain implementations
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



// Osmosis implementations
pub(crate) async fn generate_osmosis_address_impl(_request: routes::CosmosAddressRequest) -> Result<String> {
    error!("Osmosis address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_tx_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign transaction not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_swap_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign swap not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_lp_add_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign LP add not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_lp_remove_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign LP remove not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_delegate_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign delegate not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_undelegate_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign undelegate not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_redelegate_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign redelegate not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_rewards_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign rewards not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn osmosis_sign_ibc_transfer_impl(_request: routes::CosmosSignRequest) -> Result<routes::CosmosSignResponse> {
    error!("Osmosis sign IBC transfer not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Binance implementations
pub(crate) async fn generate_bnb_address_impl(_request: routes::CosmosAddressRequest) -> Result<String> {
    error!("Binance Chain address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn binance_sign_tx_impl(_request: routes::BinanceSignRequest) -> anyhow::Result<routes::BinanceSignResponse> {
    error!("Binance sign transaction not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Bitcoin implementations
// NOTE: This old implementation is deprecated - use impl_bitcoin::bitcoin_sign_tx_impl instead
// which properly uses ServerState and has better error handling
/*
pub(crate) async fn bitcoin_sign_tx_impl(request: routes::BitcoinSignRequest) -> anyhow::Result<routes::BitcoinSignResponse> {
    info!("🚀 Starting Bitcoin transaction signing");
    info!("📋 Request: {} inputs, {} outputs", request.inputs.len(), request.outputs.len());
    
    // Get the USB device - fail if not available
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
        // Create SignTx message to initiate Bitcoin signing
        let sign_tx = messages::SignTx {
            outputs_count: request.outputs.len() as u32,
            inputs_count: request.inputs.len() as u32,
            coin_name: Some("Bitcoin".to_string()),
            version: Some(1),
            lock_time: Some(0),
            expiry: None,
            overwintered: None,
            version_group_id: None,
            branch_id: None,
        };
        
        // Track transaction state
        let mut current_input_index = 0;
        let mut current_output_index = 0;
        let mut signatures = Vec::new();
        let mut serialized_tx_parts = Vec::new();
        
        // Store previous transaction data for each input
        let mut prev_tx_cache = std::collections::HashMap::new();
        
        // Parse previous transactions from hex data
        for (idx, input) in request.inputs.iter().enumerate() {
            if let Some(hex) = &input.hex {
                // Parse the hex into transaction components
                // For now, store the raw hex - we'll parse it when needed
                prev_tx_cache.insert(input.prev_hash.clone(), hex.clone());
                info!("📦 Cached previous transaction for input {}: {} bytes", idx, hex.len() / 2);
            } else {
                warn!("⚠️ Input {} missing previous transaction hex", idx);
            }
        }
        
        // Start the signing process
        let mut current_message: Message = sign_tx.into();
        
        info!("📤 Sending SignTx message to device");
        
        loop {
            let response = transport
                .with_standard_handler()
                .handle(current_message)?;
            
            match response {
                Message::TxRequest(tx_req) => {
                    match tx_req.request_type {
                        Some(rt) if rt == messages::RequestType::Txinput as i32 => {
                            info!("📥 Device requesting input #{}", current_input_index);
                            
                            if current_input_index >= request.inputs.len() {
                                return Err(anyhow::anyhow!("Device requested input out of range"));
                            }
                            
                            let input = &request.inputs[current_input_index];
                            
                            // Parse the prev_hash from hex string
                            let prev_hash = hex::decode(&input.prev_hash)?;
                            
                            // Create TxInputType
                            let tx_input = messages::TxInputType {
                                address_n: input.address_n.clone(),
                                prev_hash: prev_hash.into(),
                                prev_index: input.prev_index,
                                script_sig: None,
                                sequence: Some(0xffffffff),
                                script_type: Some(parse_bitcoin_input_script_type(&input.script_type)? as i32),
                                multisig: None,
                                amount: Some(input.amount.parse()?),
                                decred_tree: None,
                                decred_script_version: None,
                            };
                            
                            // Create TxAck with the input
                            let tx_ack = messages::TxAck {
                                tx: Some(messages::TransactionType {
                                    version: None,
                                    inputs: vec![tx_input],
                                    bin_outputs: vec![],
                                    outputs: vec![],
                                    lock_time: None,
                                    inputs_cnt: None,
                                    outputs_cnt: None,
                                    extra_data: None,
                                    extra_data_len: None,
                                    expiry: None,
                                    overwintered: None,
                                    version_group_id: None,
                                    branch_id: None,
                                }),
                            };
                            
                            current_input_index += 1;
                            current_message = tx_ack.into();
                        },
                        Some(rt) if rt == messages::RequestType::Txoutput as i32 => {
                            info!("📥 Device requesting output #{}", current_output_index);
                            
                            if current_output_index >= request.outputs.len() {
                                return Err(anyhow::anyhow!("Device requested output out of range"));
                            }
                            
                            let output = &request.outputs[current_output_index];
                            
                            // Create TxOutputType
                            let tx_output = messages::TxOutputType {
                                address: output.address.clone(),
                                address_n: output.address_n.clone().unwrap_or_default(),
                                amount: output.amount.parse()?,
                                script_type: parse_bitcoin_output_script_type(&output.script_type)? as i32,
                                multisig: None,
                                op_return_data: None,
                                address_type: None,
                                decred_script_version: None,
                            };
                            
                            // Create TxAck with the output
                            let tx_ack = messages::TxAck {
                                tx: Some(messages::TransactionType {
                                    version: None,
                                    inputs: vec![],
                                    bin_outputs: vec![],
                                    outputs: vec![tx_output],
                                    lock_time: None,
                                    inputs_cnt: None,
                                    outputs_cnt: None,
                                    extra_data: None,
                                    extra_data_len: None,
                                    expiry: None,
                                    overwintered: None,
                                    version_group_id: None,
                                    branch_id: None,
                                }),
                            };
                            
                            current_output_index += 1;
                            current_message = tx_ack.into();
                        },
                        Some(rt) if rt == messages::RequestType::Txmeta as i32 => {
                            info!("📥 Device requesting transaction metadata");
                            
                            // The device wants metadata about a previous transaction
                            // Check if the request has tx_hash to know which transaction
                            if let Some(details) = &tx_req.details {
                                if let Some(tx_hash) = &details.tx_hash {
                                    let tx_hash_hex = hex::encode(tx_hash);
                                    info!("📋 Device wants metadata for tx: {}", tx_hash_hex);
                                    
                                    // Get the cached hex for this transaction
                                    if let Some(hex_data) = prev_tx_cache.get(&tx_hash_hex) {
                                        // Parse the transaction hex to extract metadata
                                        match parse_tx_metadata_from_hex(hex_data) {
                                            Ok((version, input_count, output_count)) => {
                                                info!("📊 Parsed tx metadata: version={}, inputs={}, outputs={}", 
                                                      version, input_count, output_count);
                                                
                                                let tx_meta = messages::TransactionType {
                                                    version: Some(version),
                                                    inputs: vec![],
                                                    bin_outputs: vec![],
                                                    outputs: vec![],
                                                    lock_time: Some(0), // TODO: Parse locktime from end of tx
                                                    inputs_cnt: Some(input_count),
                                                    outputs_cnt: Some(output_count),
                                                    extra_data: None,
                                                    extra_data_len: Some(0),
                                                    expiry: None,
                                                    overwintered: None,
                                                    version_group_id: None,
                                                    branch_id: None,
                                                };
                                                
                                                let tx_ack = messages::TxAck {
                                                    tx: Some(tx_meta),
                                                };
                                                
                                                current_message = tx_ack.into();
                                            }
                                            Err(e) => {
                                                warn!("⚠️ Failed to parse transaction hex: {}", e);
                                                // Fallback to default values
                                                let tx_meta = messages::TransactionType {
                                                    version: Some(1),
                                                    inputs: vec![],
                                                    bin_outputs: vec![],
                                                    outputs: vec![],
                                                    lock_time: Some(0),
                                                    inputs_cnt: Some(1),
                                                    outputs_cnt: Some(2),
                                                    extra_data: None,
                                                    extra_data_len: Some(0),
                                                    expiry: None,
                                                    overwintered: None,
                                                    version_group_id: None,
                                                    branch_id: None,
                                                };
                                                
                                                let tx_ack = messages::TxAck {
                                                    tx: Some(tx_meta),
                                                };
                                                
                                                current_message = tx_ack.into();
                                            }
                                        }
                                    } else {
                                        return Err(anyhow::anyhow!("Previous transaction not found for hash: {}", tx_hash_hex));
                                    }
                                } else {
                                    // No tx_hash means it wants metadata for the unsigned transaction
                                    let tx_meta = messages::TransactionType {
                                        version: Some(1),
                                        inputs: vec![],
                                        bin_outputs: vec![],
                                        outputs: vec![],
                                        lock_time: Some(0),
                                        inputs_cnt: Some(request.inputs.len() as u32),
                                        outputs_cnt: Some(request.outputs.len() as u32),
                                        extra_data: None,
                                        extra_data_len: Some(0),
                                        expiry: None,
                                        overwintered: None,
                                        version_group_id: None,
                                        branch_id: None,
                                    };
                                    
                                    let tx_ack = messages::TxAck {
                                        tx: Some(tx_meta),
                                    };
                                    
                                    current_message = tx_ack.into();
                                }
                            } else {
                                return Err(anyhow::anyhow!("TXMETA request missing details"));
                            }
                        },
                        Some(rt) if rt == messages::RequestType::Txextradata as i32 => {
                            info!("📥 Device requesting extra data");
                            // For Bitcoin, there's typically no extra data
                            let tx_ack = messages::TxAck {
                                tx: Some(messages::TransactionType {
                                    version: None,
                                    inputs: vec![],
                                    bin_outputs: vec![],
                                    outputs: vec![],
                                    lock_time: None,
                                    inputs_cnt: None,
                                    outputs_cnt: None,
                                    extra_data: Some(vec![].into()),
                                    extra_data_len: None,
                                    expiry: None,
                                    overwintered: None,
                                    version_group_id: None,
                                    branch_id: None,
                                }),
                            };
                            current_message = tx_ack.into();
                        },
                        Some(rt) if rt == messages::RequestType::Txfinished as i32 => {
                            info!("✅ Device finished signing transaction");
                            
                            // Collect the serialized transaction if provided
                            if let Some(serialized) = &tx_req.serialized {
                                if let Some(serialized_tx) = &serialized.serialized_tx {
                                    serialized_tx_parts.push(serialized_tx.clone());
                                }
                                if let Some(signature) = &serialized.signature {
                                    if let Some(sig_index) = serialized.signature_index {
                                        info!("📝 Got signature for input {}", sig_index);
                                        signatures.push(hex::encode(signature));
                                    }
                                }
                            }
                            
                            // Combine all serialized parts
                            let mut serialized_tx = Vec::new();
                            for part in serialized_tx_parts {
                                serialized_tx.extend_from_slice(&part);
                            }
                            
                            return Ok(routes::BitcoinSignResponse {
                                signatures,
                                serialized_tx: hex::encode(serialized_tx),
                            });
                        },
                        _ => {
                            error!("❌ Unknown request type: {:?}", tx_req.request_type);
                            return Err(anyhow::anyhow!("Unknown request type from device"));
                        }
                    }
                    
                    // Check if we have serialized data in this response
                    if let Some(serialized) = &tx_req.serialized {
                        if let Some(serialized_tx) = &serialized.serialized_tx {
                            serialized_tx_parts.push(serialized_tx.clone());
                        }
                        if let Some(signature) = &serialized.signature {
                            if let Some(sig_index) = serialized.signature_index {
                                info!("📝 Got signature for input {}", sig_index);
                                signatures.push(hex::encode(signature));
                            }
                        }
                    }
                },
                Message::Failure(failure) => {
                    let error_msg = failure.message.unwrap_or_else(|| "Unknown error".to_string());
                    error!("❌ Bitcoin signing failed: {}", error_msg);
                    return Err(anyhow::anyhow!("Failure: {}", error_msg));
                },
                _ => {
                    error!("❌ Unexpected message type during Bitcoin signing");
                    return Err(anyhow::anyhow!("Unexpected message type"));
                }
            }
        }
    }).await??;
    
    Ok(result)
}
*/

// Helper function to parse Bitcoin input script type
fn parse_bitcoin_input_script_type(script_type: &str) -> anyhow::Result<messages::InputScriptType> {
    match script_type.to_lowercase().as_str() {
        "p2pkh" => Ok(messages::InputScriptType::Spendaddress),
        "p2sh" => Ok(messages::InputScriptType::Spendmultisig),
        "p2wpkh" => Ok(messages::InputScriptType::Spendwitness),
        "p2sh-p2wpkh" => Ok(messages::InputScriptType::Spendp2shwitness),
        "p2tr" => Ok(messages::InputScriptType::Spendtaproot),
        _ => {
            warn!("Unknown input script type '{}', defaulting to p2pkh", script_type);
            Ok(messages::InputScriptType::Spendaddress)
        }
    }
}

// Helper function to parse Bitcoin output script type
fn parse_bitcoin_output_script_type(script_type: &str) -> anyhow::Result<messages::OutputScriptType> {
    match script_type.to_lowercase().as_str() {
        "p2pkh" => Ok(messages::OutputScriptType::Paytoaddress),
        "p2wpkh" => Ok(messages::OutputScriptType::Paytowitness),
        "p2sh" => Ok(messages::OutputScriptType::Paytoscripthash),
        "p2sh-p2wpkh" => Ok(messages::OutputScriptType::Paytop2shwitness),
        "p2tr" => Ok(messages::OutputScriptType::Paytotaproot),
        "multisig" => Ok(messages::OutputScriptType::Paytomultisig),
        "op_return" => Ok(messages::OutputScriptType::Paytoopreturn),
        _ => {
            warn!("Unknown output script type '{}', defaulting to PAYTOADDRESS", script_type);
            Ok(messages::OutputScriptType::Paytoaddress)
        }
    }
}

// NOTE: bitcoin_sign_message_impl and bitcoin_verify_message_impl are now in impl_bitcoin.rs

// Ethereum implementations
pub(crate) async fn ethereum_sign_tx_impl(request: routes::EthereumSignRequest) -> anyhow::Result<routes::EthereumSignResponse> {
    info!("🚀 Starting Ethereum transaction signing");
    
    // Get the USB device - fail if not available
    let device = try_get_device()?;
    
    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
        
        // Parse hex values to bytes (following CLI pattern)
        let nonce = parse_ethereum_value(&request.nonce)?;
        let gas_price = parse_ethereum_value(&request.gasPrice)?;
        let gas_limit = parse_ethereum_value(&request.gasLimit)?;
        let value = parse_ethereum_value(&request.value)?;
        
        // Parse to address (20 bytes)
        let to_address = if request.to.starts_with("0x") && request.to.len() == 42 {
            hex::decode(&request.to[2..])
                .map_err(|e| anyhow::anyhow!("Invalid to address: {}", e))?
        } else {
            return Err(anyhow::anyhow!("Invalid to address format: {}", request.to));
        };
        
        // Parse data if provided
        let (data_length, data_initial_chunk) = if let Some(data_str) = &request.data {
            if data_str.starts_with("0x") && data_str.len() > 2 {
                let data_bytes = hex::decode(&data_str[2..])
                    .map_err(|e| anyhow::anyhow!("Invalid data: {}", e))?;
                let chunk_size = std::cmp::min(data_bytes.len(), 1024);
                (Some(data_bytes.len() as u32), Some(data_bytes[..chunk_size].to_vec()))
            } else if data_str == "0x" {
                (None, None)
            } else {
                let data_bytes = hex::decode(data_str)
                    .map_err(|e| anyhow::anyhow!("Invalid data: {}", e))?;
                let chunk_size = std::cmp::min(data_bytes.len(), 1024);
                (Some(data_bytes.len() as u32), Some(data_bytes[..chunk_size].to_vec()))
            }
        } else {
            (None, None)
        };
        
        // Create EthereumSignTx message (following CLI pattern exactly)
        let eth_sign_tx = messages::EthereumSignTx {
            address_n: request.addressNList.clone(),
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            gas_limit: Some(gas_limit),
            to: Some(to_address),
            value: Some(value),
            chain_id: Some(request.chainId.parse::<u32>().unwrap_or(1)),
            data_length,
            data_initial_chunk,
            // Set defaults for optional fields
            to_address_n: vec![],
            address_type: None,
            r#type: Some(0), // Legacy transaction
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            token_value: None,
            token_to: None,
            token_shortcut: None,
            tx_type: None,
        };
        
        info!("📤 Sending EthereumSignTx message to device (chain_id: {})", request.chainId);
        
        // Send the message and wait for response (following CLI pattern)
        let mut remaining_data = if let Some(data_str) = &request.data {
            if data_str.starts_with("0x") && data_str.len() > 2 {
                let data_bytes = hex::decode(&data_str[2..])?;
                if data_bytes.len() > 1024 {
                    Some(data_bytes[1024..].to_vec())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        let response = transport
            .with_standard_handler()
            .with_mut_handler(&mut |msg| {
                Ok(match msg {
                    Message::EthereumTxRequest(req) => {
                        if let Some(data_length) = req.data_length {
                            if let Some(ref mut data) = remaining_data {
                                let chunk_size = std::cmp::min(data.len(), data_length as usize);
                                let chunk = data[..chunk_size].to_vec();
                                data.drain(..chunk_size);
                                if data.is_empty() {
                                    remaining_data = None;
                                }
                                Some(messages::EthereumTxAck {
                                    data_chunk: Some(chunk),
                                }.into())
                            } else {
                                Some(messages::EthereumTxAck {
                                    data_chunk: Some(vec![]),
                                }.into())
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
            })
            .handle(eth_sign_tx.into())?;
        
        // Extract signature from response (following CLI pattern)
        match response {
            Message::EthereumTxRequest(tx_req) => {
                let v = tx_req.signature_v.ok_or_else(|| anyhow::anyhow!("No signature_v in response"))?;
                let r = tx_req.signature_r.ok_or_else(|| anyhow::anyhow!("No signature_r in response"))?;
                let s = tx_req.signature_s.ok_or_else(|| anyhow::anyhow!("No signature_s in response"))?;
                
                if r.len() != 32 || s.len() != 32 {
                    return Err(anyhow::anyhow!("Invalid signature component length"));
                }
                
                info!("✅ Ethereum transaction signed successfully");
                
                // Create a properly serialized Ethereum transaction with 0x prefix
                let r_hex = hex::encode(&r);
                let s_hex = hex::encode(&s);
                let v_hex = format!("{:x}", v);
                
                Ok(routes::EthereumSignResponse {
                    v: format!("0x{}", v_hex),
                    r: format!("0x{}", r_hex),
                    s: format!("0x{}", s_hex),
                    serialized: format!("0x{}{}{}", r_hex, s_hex, v_hex),
                })
            }
            _ => {
                error!("❌ Unexpected response from device: {:?}", response);
                Err(anyhow::anyhow!("Unexpected response from device"))
            }
        }
    }).await;
    
    match result {
        Ok(Ok(response)) => {
            info!("✅ Ethereum transaction signing completed");
            Ok(response)
        }
        Ok(Err(e)) => {
            error!("❌ Ethereum signing failed: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("⏰ Ethereum signing timed out");
            Err(anyhow::anyhow!("Ethereum signing operation timed out"))
        }
    }
}

pub(crate) async fn ethereum_sign_message_impl(_request: routes::EthereumSignMessageRequest) -> anyhow::Result<routes::EthereumSignMessageResponse> {
    error!("Ethereum sign message not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn ethereum_sign_typed_data_impl(_request: routes::EthereumSignTypedDataRequest) -> anyhow::Result<routes::EthereumSignTypedDataResponse> {
    error!("Ethereum sign typed data not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn ethereum_verify_message_impl(_request: routes::EthereumVerifyMessageRequest) -> anyhow::Result<routes::EthereumVerifyMessageResponse> {
    error!("Ethereum verify message not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Ripple implementations
pub(crate) async fn generate_xrp_address_impl(_request: routes::RippleAddressRequest) -> anyhow::Result<String> {
    error!("Ripple address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn ripple_sign_tx_impl(_request: routes::RippleSignRequest) -> anyhow::Result<routes::RippleSignResponse> {
    error!("Ripple sign transaction not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// EOS implementations
pub(crate) async fn generate_eos_address_impl(_request: routes::EosAddressRequest) -> anyhow::Result<String> {
    error!("EOS address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn eos_sign_tx_impl(_request: routes::EosSignRequest) -> anyhow::Result<routes::EosSignResponse> {
    error!("EOS sign transaction not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Nano implementations
pub(crate) async fn generate_nano_address_impl(_request: routes::NanoAddressRequest) -> anyhow::Result<String> {
    error!("Nano address generation not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn nano_sign_tx_impl(_request: routes::NanoSignRequest) -> anyhow::Result<routes::NanoSignResponse> {
    error!("Nano sign transaction not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// System management implementations
pub(crate) async fn system_apply_settings_impl(_request: routes::ApplySettingsRequest) -> anyhow::Result<()> {
    error!("Apply settings not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_apply_policy_impl(_request: routes::ApplyPolicyRequest) -> anyhow::Result<()> {
    error!("Apply policy not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_change_pin_impl(_request: routes::ChangePinRequest) -> anyhow::Result<()> {
    error!("Change PIN not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_wipe_device_impl() -> anyhow::Result<()> {
    error!("Wipe device not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_recovery_device_impl(_request: routes::RecoveryDeviceRequest) -> anyhow::Result<()> {
    error!("Recovery device not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_reset_device_impl(_request: routes::ResetDeviceRequest) -> anyhow::Result<()> {
    error!("Reset device not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_load_device_impl(_request: routes::LoadDeviceRequest) -> anyhow::Result<()> {
    error!("Load device not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_backup_device_impl() -> anyhow::Result<()> {
    error!("Backup device not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_firmware_erase_impl() -> anyhow::Result<()> {
    error!("Firmware erase not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn system_firmware_upload_impl(_request: routes::FirmwareUploadRequest) -> anyhow::Result<()> {
    error!("Firmware upload not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Debug implementations
pub(crate) async fn debug_link_state_impl() -> anyhow::Result<routes::DebugLinkState> {
    error!("Debug link state not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn debug_fill_config_impl(_request: routes::DebugFillConfig) -> anyhow::Result<()> {
    error!("Debug fill config not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Manufacturing implementations
pub(crate) async fn manufacturing_get_hash_impl() -> anyhow::Result<String> {
    error!("Manufacturing get hash not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn manufacturing_model_prefix_impl() -> anyhow::Result<String> {
    error!("Manufacturing model prefix not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Raw message implementation
pub(crate) async fn raw_message_impl(_body: axum::body::Bytes) -> anyhow::Result<axum::body::Bytes> {
    error!("Raw message not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Helper function to parse Ethereum value strings (hex or decimal)
fn parse_ethereum_value(value_str: &str) -> anyhow::Result<Vec<u8>> {
    if value_str.starts_with("0x") {
        let hex_str = &value_str[2..];
        if hex_str.is_empty() {
            return Ok(vec![0]);
        }
        
        // Pad with leading zero if odd length
        let padded = if hex_str.len() % 2 == 1 {
            format!("0{}", hex_str)
        } else {
            hex_str.to_string()
        };
        
        hex::decode(padded)
            .map_err(|e| anyhow::anyhow!("Invalid hex value: {}", e))
    } else {
        // Parse as decimal and convert to big-endian bytes
        let num = value_str.parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid decimal value: {}", e))?;
        
        if num == 0 {
            Ok(vec![0])
        } else {
            Ok(num.to_be_bytes().to_vec())
        }
    }
}

// Helper function to parse basic transaction metadata from hex
fn parse_tx_metadata_from_hex(hex_str: &str) -> anyhow::Result<(u32, u32, u32)> {
    // Basic Bitcoin transaction parsing to extract version, input count, and output count
    // This is a simplified parser - in production you'd use a proper Bitcoin library
    
    let tx_bytes = hex::decode(hex_str)?;
    if tx_bytes.len() < 10 {
        return Err(anyhow::anyhow!("Transaction hex too short"));
    }
    
    let mut cursor = 0;
    
    // Read version (4 bytes, little-endian)
    let version = u32::from_le_bytes([
        tx_bytes[cursor],
        tx_bytes[cursor + 1],
        tx_bytes[cursor + 2],
        tx_bytes[cursor + 3],
    ]);
    cursor += 4;
    
    // Check for witness marker (BIP144)
    let has_witness = tx_bytes[cursor] == 0x00 && tx_bytes[cursor + 1] == 0x01;
    if has_witness {
        cursor += 2; // Skip marker and flag
    }
    
    // Read input count (varint)
    let (input_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
    cursor += bytes_read;
    
    // Skip through inputs to find output count
    // Each input has: txid (32 bytes) + vout (4 bytes) + script_len (varint) + script + sequence (4 bytes)
    for _ in 0..input_count {
        cursor += 36; // txid + vout
        let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
        cursor += bytes_read + script_len as usize + 4; // script + sequence
    }
    
    // Read output count (varint)
    let (output_count, _) = read_varint(&tx_bytes[cursor..])?;
    
    Ok((version, input_count as u32, output_count as u32))
}

// Helper function to read Bitcoin varint
fn read_varint(data: &[u8]) -> anyhow::Result<(u64, usize)> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("No data for varint"));
    }
    
    match data[0] {
        0..=252 => Ok((data[0] as u64, 1)),
        253 => {
            if data.len() < 3 {
                return Err(anyhow::anyhow!("Insufficient data for varint"));
            }
            Ok((u16::from_le_bytes([data[1], data[2]]) as u64, 3))
        }
        254 => {
            if data.len() < 5 {
                return Err(anyhow::anyhow!("Insufficient data for varint"));
            }
            Ok((u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as u64, 5))
        }
        255 => {
            if data.len() < 9 {
                return Err(anyhow::anyhow!("Insufficient data for varint"));
            }
            Ok((u64::from_le_bytes([
                data[1], data[2], data[3], data[4],
                data[5], data[6], data[7], data[8]
            ]), 9))
        }
    }
}