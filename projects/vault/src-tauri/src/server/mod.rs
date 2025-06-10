pub mod routes;
pub mod bitcoin;

use axum::{
    Router,
    serve,
    routing::get,
};
use bitcoin::BitcoinState;

use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;
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
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health_check,
        routes::device_status,
        routes::list_devices,
        routes::registry_status,
        routes::firmware_releases,
        bitcoin::health,
        bitcoin::networks,
        bitcoin::parse_path,
        bitcoin::pubkey,
        bitcoin::frontload,
    ),
    components(
        schemas(
            routes::HealthResponse,
            routes::DeviceStatus,
            routes::DeviceInfo,
            routes::KeepKeyInfo,
            routes::UsbDeviceInfo,
            routes::ApiResponse<routes::DeviceStatus>,
            bitcoin::NetworksResponse,
            bitcoin::ParsePathRequest,
            bitcoin::ParsePathResponse,
            bitcoin::PubkeyRequest,
            bitcoin::PubkeyResponse,
            bitcoin::HealthResponse,
            bitcoin::FrontloadResponse,
        )
    ),
    tags(
        (name = "system", description = "System health and status endpoints"),
        (name = "device", description = "KeepKey device management endpoints"),
        (name = "bitcoin", description = "Bitcoin REST endpoints with frontloading")
    ),
    info(
        title = "KeepKey Desktop API",
        version = "0.1.0",
        description = "Simple, focused Bitcoin API with real device communication and frontloading",
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

    // Create unified app state
    let app_state = Arc::new(AppState {
        server_state,
        bitcoin_state,
    });

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
        // Bitcoin endpoints
        .route("/api/bitcoin/health", get(bitcoin::health))
        .route("/api/bitcoin/networks", get(bitcoin::networks))
        .route("/api/bitcoin/parse-path", axum::routing::post(bitcoin::parse_path))
        .route("/api/bitcoin/pubkey", axum::routing::post(bitcoin::pubkey))
        .route("/api/bitcoin/frontload", axum::routing::post(bitcoin::frontload))
        // Add state and middleware
        .with_state(app_state)
        .merge(swagger_ui)
        .layer(CorsLayer::permissive());

    let addr = "127.0.0.1:1646";
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Server started successfully:");
    info!("  📋 REST API: http://{}/api", addr);
    info!("  📚 API Documentation: http://{}/docs", addr);
    info!("  🔑 Bitcoin API: http://{}/api/bitcoin", addr);
    info!("  💾 Frontload: POST http://{}/api/bitcoin/frontload", addr);
    
    // Spawn the server
    tokio::spawn(async move {
        if let Err(e) = serve(listener, app).await {
            info!("Server error: {}", e);
        }
    });
    
    Ok(())
} 