use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{info, error};
use axum::{
    routing::{get, post},
    Router,
};



use axum::middleware;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use rusb::{Device, GlobalContext};
use serde_json::{json, Value};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::net::SocketAddr;
use hex;

use crate::transport::{UsbTransport, ProtocolAdapter};
use crate::messages::{self, Message};
use super::cache::{DeviceCache, DeviceFrontloader};
use super::ServerState;
use super::try_get_device;
use super::v2_endpoints;

/// Attempt to cleanup and reset any stuck USB devices
async fn cleanup_stuck_usb_devices() {
    info!("🧹 Attempting to cleanup stuck USB devices...");
    
    // Try to find and reset any KeepKey devices that might be stuck
    match rusb::devices() {
        Ok(devices) => {
            for device in devices.iter() {
                if let Ok(device_desc) = device.device_descriptor() {
                    let vendor_id = device_desc.vendor_id();
                    let product_id = device_desc.product_id();
                    
                    // Check if this is a KeepKey device
                    if vendor_id == 0x2b24 && (product_id == 0x0001 || product_id == 0x0002) {
                        info!("🔧 Found KeepKey device (VID={:04x} PID={:04x}), attempting reset...", vendor_id, product_id);
                        
                        // Try to open and reset the device
                        match device.open() {
                            Ok(mut handle) => {
                                match handle.reset() {
                                    Ok(()) => {
                                        info!("✅ Successfully reset KeepKey device");
                                    }
                                    Err(e) => {
                                        info!("⚠️ Failed to reset device (this is often normal): {}", e);
                                    }
                                }
                                // Let handle drop naturally to release
                            }
                            Err(e) => {
                                info!("⚠️ Could not open device for reset (might not be stuck): {}", e);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            info!("⚠️ Could not enumerate USB devices for cleanup: {}", e);
        }
    }
    
    info!("✅ USB cleanup attempt completed");
}

/// Start the KeepKey CLI HTTP server
pub async fn start_server(port: u16) -> Result<()> {
    info!("🚀 Starting KeepKey CLI server initialization...");
    
    // 0. First, try to cleanup any stuck USB devices from previous sessions
    cleanup_stuck_usb_devices().await;
    
    // 1. Open device cache database
    let cache = DeviceCache::open()?;
    let cache_for_v2 = cache.clone(); // Clone for v2 endpoints
    info!("✅ Device cache database opened");

    // Prepare to hold features outside the match
    let mut features = None;
    
    // 2. Detect connected device and verify communication
    info!("🔍 Checking KeepKey device connectivity...");
        
    // Variables for device state
    let device_id: String;
    let shared_active_transport: Arc<Mutex<Option<UsbTransport<GlobalContext>>>>;
        
        // Try to connect to physical device
        let usb_device = match super::try_get_device() {
            Ok(device_obj) => device_obj,
            Err(e) => {
                error!("✖ No KeepKey device found: {}", e);
                return Err(anyhow::anyhow!("No KeepKey device found: {}", e));
            }
        };
        
        // 3. Test device communication BEFORE proceeding and establish the persistent transport
        info!("🧪 Testing device communication and establishing persistent transport...");
        let result = timeout(Duration::from_secs(5), async {
            let (mut transport, _config_descriptor, _handle) = UsbTransport::new(&usb_device, 0)?;
            let get_features_msg = messages::GetFeatures {};
            
            let response = transport.with_standard_handler().handle(get_features_msg.into())?;
            
            match response {
                Message::Features(features_msg) => {
                    let device_id_str = features_msg.device_id
                        .as_ref()
                        .map(|id| hex::encode(id))
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let label = features_msg.label.as_deref().unwrap_or("Unnamed KeepKey");
                    info!("✅ Device communication successful!");
                    info!("   Device ID: {}", device_id_str);
                    info!("   Label: {}", label);
                    
                    Ok((device_id_str, features_msg, transport)) // Return transport here
                }
                _ => Err(anyhow::anyhow!("Unexpected response from device"))
            }
        }).await;
        
        // Handle various timeouts and response scenarios
        match result {
            Ok(Ok((device_id_result, features_msg, mut transport))) => {
                device_id = device_id_result;
                
                // Set up the shared transport
                let transport_mutex = Arc::new(Mutex::new(Some(transport)));
                shared_active_transport = Arc::clone(&transport_mutex);
                
                // Convert protobuf Features to routes::Features and save to cache
                let routes_features = super::routes::Features {
                    vendor: features_msg.vendor.clone(),
                    major_version: features_msg.major_version,
                    minor_version: features_msg.minor_version,
                    patch_version: features_msg.patch_version,
                    bootloader_mode: features_msg.bootloader_mode,
                    device_id: features_msg.device_id.clone(),
                    pin_protection: features_msg.pin_protection,
                    passphrase_protection: features_msg.passphrase_protection,
                    label: features_msg.label.clone(),
                    initialized: features_msg.initialized,
                    // Convert byte arrays to hex strings for hash values
                    revision: features_msg.revision.as_ref().map(|bytes| hex::encode(bytes)),
                    bootloader_hash: features_msg.bootloader_hash.as_ref().map(|bytes| hex::encode(bytes)),
                    firmware_hash: features_msg.firmware_hash.as_ref().map(|bytes| hex::encode(bytes)),
                    model: features_msg.model.clone(),
                    firmware_variant: features_msg.firmware_variant.clone(),
                    // Additional required fields with default values
                    language: None,
                    imported: None,
                    pin_cached: None,
                    passphrase_cached: None,
                    wipe_code_protection: None,
                    auto_lock_delay_ms: None,
                    policies: None,
                    no_backup: None,
                };
                
                cache.save_features(&routes_features, &device_id).await?;
                features = Some(features_msg.clone());
            }
            Ok(Err(e)) => {
                // Communication error - provide helpful guidance
                error!("❌ Failed to communicate with device: {}", e);
                error!("❌ Cannot start server without working device communication");
                error!("🔌 SOLUTION: Please disconnect and reconnect your KeepKey device, then try again");
                error!("💡 This is normal - KeepKey devices need to be reconnected after communication timeouts");
                
                // Try to cleanup any partial USB state
                cleanup_stuck_usb_devices().await;
                
                return Err(anyhow::anyhow!("Device communication failed: {}. Please reconnect your KeepKey device and try again.", e));
            }
            Err(e) => {
                // Timeout - provide helpful guidance
                error!("❌ Device communication timed out: {}", e);
                error!("❌ Cannot start server without responsive device");
                error!("🔌 SOLUTION: Please disconnect and reconnect your KeepKey device, then try again");
                error!("💡 This is normal - KeepKey devices often need reconnection after timeouts");
                
                // Try to cleanup any partial USB state
                cleanup_stuck_usb_devices().await;
                
                return Err(anyhow::anyhow!("Device communication timed out: {}. Please reconnect your KeepKey device and try again.", e));
            }
        }
    
    // At this point, device communication has succeeded and features are available.
    let features = features.expect("Device features should be available after successful communication");
    
    // 4. Check if device is in cache AND has addresses
    info!("🔍 Checking cache for device ID: {}", device_id);
    let device_exists = cache.has_device(&device_id).await?;
    let has_addresses = if device_exists {
        cache.has_cached_addresses(&device_id).await?
    } else {
        false
    };
    
    if device_exists && has_addresses {
        info!("📂 Device found in cache with addresses - loading from cache...");
        if let Some(cached_features) = cache.load_device(&device_id).await? {
            let elapsed = chrono::Utc::now().timestamp() - cached_features.last_seen;
            info!("✅ Device data loaded from cache (last seen: {} seconds ago)", elapsed);
        }
    } else {
        // Frontload device information - either new device OR existing device with no addresses
        if device_exists {
            info!("🔄 Device exists but has no cached addresses - forcing frontload...");
        } else {
            info!("🔄 New device detected, frontloading key material...");
        }
        info!("⏳ This may take 30-60 seconds on first run...");
        
        // We need the device object for frontloading
        let device_obj_result = try_get_device();
        
        // Check if we got a device or need to handle error
        if let Err(e) = &device_obj_result {
            error!("❌ Failed to get device for frontloading: {}", e);
            return Err(anyhow::anyhow!("Failed to get device for frontloading: {}", e));
        }
        
        // Only continue with frontloading if we have a valid device
        if let Ok(device_obj) = device_obj_result {
            // Pass the shared transport to DeviceFrontloader
            let frontloader = DeviceFrontloader::new(cache.clone(), Arc::clone(&shared_active_transport), device_obj);
            
            // This blocks until all data is loaded - MUST succeed before starting server
            match frontloader.frontload_all().await {
                Ok(_) => {
                    info!("✅ Device data frontloaded and cached successfully");
                }
                Err(e) => {
                    error!("❌ Failed to frontload device data: {}", e);
                    error!("❌ Cannot start server without working device communication");
                    return Err(anyhow::anyhow!("Device frontloading failed: {}", e));
                }
            }
        }
    }
    
    // 5. Final device health check before starting server using the shared transport
    info!("🏥 Final device health check...");
    let health_check_result = timeout(Duration::from_secs(3), async {
        let mut transport_opt_guard = shared_active_transport.lock().await;
        if let Some(transport) = transport_opt_guard.as_mut() {
            let ping_msg = messages::Ping {
                message: Some("Health check".to_string()),
                button_protection: None,
                pin_protection: None,
                passphrase_protection: None,
                wipe_code_protection: None,
            };
            
            match transport.handle(ping_msg.into()) {
                Ok(response) => match response {
                    Message::Success(_) => Ok(()),
                    _ => Err(anyhow::anyhow!("Unexpected response to ping")),
                },
                Err(e) => Err(anyhow::anyhow!("Failed to ping device: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("Device transport not available for health check"))
        }
    }).await;
    
    match health_check_result {
        Ok(Ok(_)) => {
            info!("✅ Device is healthy and ready for requests");
        }
        Ok(Err(e)) => {
            error!("❌ Device health check failed: {}", e);
            error!("❌ Cannot start server without working device communication");
            return Err(anyhow::anyhow!("Device health check failed: {}", e));
        }
        Err(e) => {
            error!("❌ Device health check timed out: {}", e);
            error!("❌ Cannot start server without responsive device");
            return Err(anyhow::anyhow!("Device health check timed out: {}", e));
        }
    }
    
    // 6. ONLY NOW start the REST server with confirmed working device
    info!("🌐 Device confirmed working - starting REST API server on port {}", port);
    
    // Create API documentation
    #[derive(OpenApi)]
    #[openapi(
        paths(
            super::routes::list_devices,
            super::routes::list_usb_devices,
            super::routes::get_device_features,
            super::routes::system_get_features,
            super::routes::system_ping,
            super::routes::generate_utxo_address,

            
            
            
            
            super::routes::bitcoin::utxo_sign_transaction,
            
            
        ),
        components(schemas(
            super::routes::HealthResponse,
            super::routes::DeviceStatus,
            super::routes::DeviceInfo,
            super::routes::UsbDeviceInfo,
            super::routes::Features,
            super::routes::Policy,
            super::routes::PingRequest,
            super::routes::PingResponse,
            super::routes::UtxoAddressRequest,
            super::routes::UtxoAddressResponse,


            // Use only types that exist in the mayachain routes
            
            
            
            
            
            
            
            
            
        )),
        tags(
            (name = "system", description = "System health and status endpoints"),
            (name = "device", description = "Device management and information endpoints"),
            (name = "addresses", description = "Address generation endpoints"),
            

            
            
        ),
        info(
            title = "KeepKey CLI Server API",
            description = "REST API and MCP server for KeepKey device management",
            version = "0.2.3"
        )
    )]
    struct ApiDoc;

    // Create the router with cache state
    let state = ServerState {
        cache,
        device_mutex: Arc::new(Mutex::new(())),
        active_transport: shared_active_transport,
    };
    
    // Build the application with all routes
    let app = Router::new()
    // Health endpoint
    .route("/api/health", get(super::routes::health_check))
    
    // Health check and device status
    .route("/health", get(super::routes::health_check))
    .route("/api/status", get(super::routes::device_status))
    .route("/api/devices", get(super::routes::list_devices))
    .route("/api/usb-devices", get(super::routes::list_usb_devices))
        .route("/system/info/get-features", get(super::routes::system_get_features).post(super::routes::system_get_features)) // Added to match client expectation, now accepts POST
        .route("/api/v1/system/ping", post(super::routes::system_ping))
        
        // Auth endpoints
        .route("/auth/pair", get(super::routes::auth::auth_verify))
        .route("/auth/pair", post(super::routes::auth::auth_pair))
        
        // Address generation endpoints
        // Modern API endpoints
        .route("/api/v1/utxo/address", post(super::routes::generate_utxo_address))
        
        // Legacy address endpoints for backward compatibility
        .route("/addresses/utxo", post(super::routes::generate_utxo_address))
        
        // Bitcoin endpoints
        .route("/api/v1/bitcoin/tx", post(super::routes::bitcoin::bitcoin_sign_tx))
        .route("/api/v1/bitcoin/sign-message", post(super::routes::bitcoin::bitcoin_sign_message))
        .route("/api/v1/bitcoin/verify-message", post(super::routes::bitcoin::bitcoin_verify_message))
        .route("/api/v1/utxo/tx", post(super::routes::bitcoin::utxo_sign_transaction))
        .route("/utxo/sign-transaction", post(super::routes::bitcoin::utxo_sign_transaction))

        
        // System management endpoints
        .route("/system/info/apply-settings", post(super::routes::system_management::system_apply_settings))
        .route("/api/v1/system/apply-settings", post(super::routes::system_management::system_apply_settings))
        .route("/system/info/apply-policy", post(super::routes::system_management::system_apply_policy))
        .route("/api/v1/system/apply-policy", post(super::routes::system_management::system_apply_policy))
        .route("/system/info/change-pin", post(super::routes::system_management::system_change_pin))
        .route("/api/v1/system/change-pin", post(super::routes::system_management::system_change_pin))
        .route("/system/info/wipe-device", post(super::routes::system_management::system_wipe_device))
        .route("/api/v1/system/wipe-device", post(super::routes::system_management::system_wipe_device))
        .route("/system/info/recovery-device", post(super::routes::system_management::system_recovery_device))
        .route("/api/v1/system/recovery-device", post(super::routes::system_management::system_recovery_device))
        .route("/system/info/reset-device", post(super::routes::system_management::system_reset_device))
        .route("/api/v1/system/reset-device", post(super::routes::system_management::system_reset_device))
        .route("/system/info/load-device", post(super::routes::system_management::system_load_device))
        .route("/api/v1/system/load-device", post(super::routes::system_management::system_load_device))
        .route("/system/info/backup-device", post(super::routes::system_management::system_backup_device))
        .route("/api/v1/system/backup-device", post(super::routes::system_management::system_backup_device))
        .route("/system/info/firmware-erase", post(super::routes::system_management::system_firmware_erase))
        .route("/api/v1/system/firmware-erase", post(super::routes::system_management::system_firmware_erase))
        .route("/system/info/firmware-upload", post(super::routes::system_management::system_firmware_upload))
        .route("/api/v1/system/firmware-upload", post(super::routes::system_management::system_firmware_upload))
        
        // Debug endpoints
        .route("/system/debug/link-state", post(super::routes::debug::debug_link_state))
        .route("/api/v1/debug/link-state", get(super::routes::debug::debug_link_state))
        .route("/system/debug/fill-config", post(super::routes::debug::debug_fill_config))
        .route("/api/v1/debug/fill-config", post(super::routes::debug::debug_fill_config))
        
        // Manufacturing endpoints
        .route("/system/manufacturing/get-hash", post(super::routes::manufacturing::manufacturing_get_hash))
        .route("/api/v1/manufacturing/get-hash", get(super::routes::manufacturing::manufacturing_get_hash))
        .route("/system/manufacturing/model-prefix", post(super::routes::manufacturing::manufacturing_model_prefix))
        .route("/api/v1/manufacturing/model-prefix", get(super::routes::manufacturing::manufacturing_model_prefix))
        
        // Raw message endpoint
        .route("/api/v1/raw-message", post(super::routes::raw::raw_message))
        .route("/raw", post(super::routes::raw::raw_message))
        
        // Legacy Swagger compatibility route
        .route("/spec/swagger.json", get(super::get_swagger_spec))
        
        // Apply middlewares
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(super::log_request))
        .layer(
            CorsLayer::permissive()
        )
        .with_state(Arc::new(state))
        // Add OpenAPI docs
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()));
    
    // Start listening - bind to localhost only for security
    // Using 127.0.0.1 instead of 0.0.0.0 to avoid exposing wallet to the network
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("🚀 Starting KeepKey CLI server on {} (localhost only)", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server documentation and endpoints:");
    info!("  - REST API: http://localhost:{}/api", port);
    info!("  - API Documentation: http://localhost:{}/docs", port);
    info!("  - OpenAPI Spec: http://localhost:{}/api-docs/openapi.json", port);
    info!("  - Legacy Swagger: http://localhost:{}/spec/swagger.json", port);
    info!("  - Authentication: http://localhost:{}/auth/pair", port);
    
    // --- V2 API endpoints ---
    // Create API router for v2 endpoints using the unified device cache
    let v2_router = v2_endpoints::v2_router(Arc::new(cache_for_v2))
        // Apply middlewares to v2 router as well to ensure logging 
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(super::log_request))
        .layer(CorsLayer::permissive());
    
    // Add the v2_router under /v2
    let app = app.nest("/v2", v2_router);

    // Start the server
    axum::serve(listener, app).await?;
    
    Ok(())
}

