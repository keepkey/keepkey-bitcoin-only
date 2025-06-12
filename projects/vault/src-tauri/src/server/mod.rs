pub mod routes;
pub mod bitcoin;
pub mod context;

use axum::{
    Router,
    serve,
    routing::{get, post, delete},
};
use bitcoin::BitcoinState; // legacy, still used for device-related state
use crate::cache::device_cache::DeviceCache;

use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;
use std::sync::{Arc, Mutex};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::usb_manager::DeviceManager;

pub struct ServerState {
    pub device_manager: Arc<Mutex<DeviceManager>>,
}

// Unified app state that contains both server and bitcoin states
pub struct AppState {
    pub server_state: Arc<ServerState>,
    pub bitcoin_state: Arc<BitcoinState>,
    pub device_cache: Arc<DeviceCache>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health_check,
        routes::device_status,
        routes::list_devices,
        routes::registry_status,
        routes::firmware_releases,
        // === context endpoints ===
        routes::api_get_context,
        routes::api_set_context,
        routes::api_clear_context,
        // === auth endpoints ===
        routes::auth::auth_verify,
        routes::auth::auth_pair,
        // === frontload endpoint ===
        bitcoin::frontload,
        // === v1 endpoints (legacy protocol compatibility) ===
        routes::utxo_get_address,
        // === v2 endpoints ===
        routes::v2_endpoints::get_networks,
        routes::v2_endpoints::post_network,
        routes::v2_endpoints::get_paths,
        routes::v2_endpoints::get_path,
        routes::v2_endpoints::post_path,
        routes::v2_endpoints::put_path,
        routes::v2_endpoints::delete_path,
        routes::v2_endpoints::get_pubkeys,
        routes::v2_endpoints::get_balances,
        routes::v2_endpoints::post_portfolio_balances,
        routes::v2_endpoints::get_portfolio_summary,
    ),
    components(
        schemas(
            routes::HealthResponse,
            routes::DeviceStatus,
            routes::DeviceInfo,
            routes::KeepKeyInfo,
            routes::UsbDeviceInfo,
            routes::ApiResponse<routes::DeviceStatus>,
            // context schemas
            context::DeviceContext,
            context::ContextResponse,
            context::SetContextRequest,
            // auth schemas
            routes::auth::PairingInfo,
            routes::auth::AuthResponse,
            // v1 schemas (legacy protocol compatibility)
            routes::UtxoAddressRequest,
            routes::UtxoAddressResponse,
            // v2 schemas
            crate::cache::device_cache::Network,
            crate::cache::device_cache::Path,
            crate::cache::device_cache::CachedBalance,
            crate::cache::device_cache::PortfolioSummary,
            routes::v2_endpoints::NetworkInput,
            routes::v2_endpoints::PubkeyResponse,
            routes::v2_endpoints::GetPubkeysQuery,
            routes::v2_endpoints::GetBalancesQuery,
            routes::v2_endpoints::BalanceResponse,
            routes::v2_endpoints::PortfolioBalanceRequest,
        )
    ),
    tags(
        (name = "system", description = "System health and status endpoints"),
        (name = "device", description = "KeepKey device management endpoints"),
        (name = "auth", description = "Authentication and pairing endpoints"),
        (name = "Address", description = "V1 Address endpoints for protocol compatibility"),
        (name = "v2", description = "Unified v2 API endpoints (networks, paths, balances, etc.)")
    ),
    info(
        title = "KeepKey Desktop API",
        version = "0.1.0",
        description = "Bitcoin API with v1 (legacy protocol) and v2 (modern) endpoints. Supports real device communication and frontloading.",
        contact(
            name = "KeepKey Support",
            url = "https://keepkey.com"
        )
    )
)]
struct ApiDoc;

pub async fn start_server(device_manager: Arc<Mutex<DeviceManager>>) -> anyhow::Result<()> {
    info!("Starting simple Bitcoin-focused server on port 1646");
    
    let server_state = Arc::new(ServerState {
        device_manager: device_manager.clone(),
    });

    // Open IndexDb for frontloading data
    let indexdb = crate::index_db::IndexDb::open()
        .expect("Failed to open IndexDb")
        .into_arc_mutex();

    // Create Bitcoin state with frontloading capability
    let bitcoin_state = Arc::new(
        BitcoinState::new(device_manager, indexdb)
            .await
            .expect("Failed to initialize Bitcoin state")
    );

    // Create DeviceCache for v2 endpoints
    let device_cache = Arc::new(DeviceCache::open()?);

    // Create unified app state
    let app_state = Arc::new(AppState {
        server_state,
        bitcoin_state,
        device_cache: device_cache.clone(),
    });

    // Check if any devices are already connected and set context automatically
    // This ensures V1 API calls work even if frontloading hasn't run yet
    if let Ok(entries) = crate::device_registry::get_all_device_entries() {
        if let Some(first_device) = entries.first() {
            info!("Setting initial device context to first available device: {}", first_device.device.unique_id);
            
            // Use the helper function that gets real Ethereum address from device
            let _status = context::set_context_with_real_eth_address(
                first_device.device.unique_id.clone(),
                first_device.features.as_ref().and_then(|f| f.label.clone())
            ).await;
            info!("Initial device context set successfully for device {}", first_device.device.unique_id);
        } else {
            info!("No devices available for initial context setting");
        }
    } else {
        info!("Failed to get device registry for initial context setting");
    }

    // Create Swagger UI
    let swagger_ui = SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    let app = Router::new()
        // System endpoints
        .route("/api/health", get(routes::health_check))
        .route("/api/status", get(routes::device_status))
        .route("/api/devices", get(routes::list_devices))
        .route("/api/devices/debug", get(routes::debug_devices))
        .route("/api/devices/registry", get(routes::registry_status))
        .route("/api/firmware", get(routes::firmware_releases))
        // Context endpoints
        .route("/api/context", get(routes::api_get_context))
        .route("/api/context", post(routes::api_set_context))
        .route("/api/context", delete(routes::api_clear_context))
        // Auth endpoints
        .route("/auth/pair", get(routes::auth::auth_verify))
        .route("/auth/pair", post(routes::auth::auth_pair))
        // Frontload endpoint
        .route("/api/frontload", post(bitcoin::frontload))
        // Mount v1 API router (legacy protocol compatibility)
        .merge(routes::v1_router())
        // Mount v2 API router under /api/v2 (modular endpoints)
        .nest("/api/v2", routes::v2_endpoints::v2_router())
        // Add state and middleware
        .with_state(app_state)
        .merge(swagger_ui)
        .layer(CorsLayer::permissive());

    let addr = "127.0.0.1:1646";
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Server started successfully:");
    info!("  📋 REST API: http://{}/api", addr);
    info!("  📚 API Documentation: http://{}/docs", addr);
    info!("  🎯 Device Context: http://{}/api/context", addr);
    info!("  🔐 Authentication: http://{}/auth/pair", addr);
    info!("  🔑 V1 API (Legacy): http://{}/addresses/utxo", addr);
    info!("  🔑 V2 API (Modern): http://{}/api/v2", addr);
    info!("  💾 Frontload: POST http://{}/api/frontload", addr);
    
    // Spawn the server
    tokio::spawn(async move {
        if let Err(e) = serve(listener, app).await {
            info!("Server error: {}", e);
        }
    });
    
    Ok(())
} 