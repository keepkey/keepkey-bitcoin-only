pub mod routes;
pub mod context;

use axum::{
    Router,
    serve,
    routing::{get, post, delete},
};

use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct ServerState {
    pub device_queue_manager: crate::commands::DeviceQueueManager,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health_check,
        // Context endpoints - commented out until full device interaction is implemented
        // routes::api_get_context,
        // routes::api_set_context,
        // routes::api_clear_context,
        routes::api_list_devices,
        routes::api_get_features,
        routes::mcp_handle,
    ),
    components(
        schemas(
            routes::HealthResponse,
            routes::DeviceInfo,
            routes::KeepKeyInfo,
            routes::Features,
            // Context schemas - commented out until needed
            // context::DeviceContext,
            // context::ContextResponse,
            // context::SetContextRequest,
        )
    ),
    tags(
        (name = "system", description = "System health and status endpoints"),
        (name = "device", description = "Device management endpoints"),
        (name = "mcp", description = "Model Context Protocol endpoints")
    ),
    info(
        title = "KeepKey Vault API",
        description = "REST API and MCP server for KeepKey device management (Bitcoin-only)",
        version = "2.0.0"
    )
)]
struct ApiDoc;

pub async fn start_server(device_queue_manager: crate::commands::DeviceQueueManager) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing if not already done
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "vault_v2=info,axum=info");
    }
    
    // Try to initialize tracing, ignore if already initialized
    let _ = tracing_subscriber::fmt::try_init();
    
    // Create server state
    let server_state = Arc::new(ServerState {
        device_queue_manager,
    });
    
    // Create Swagger UI
    let swagger_ui = SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi());
    
    // Build the router
    let app = Router::new()
        // System endpoints
        .route("/api/health", get(routes::health_check))
        
        // Context endpoints - commented out until full device interaction is implemented
        // .route("/api/context", get(routes::api_get_context))
        // .route("/api/context", post(routes::api_set_context))
        // .route("/api/context", delete(routes::api_clear_context))
        
        // Device management endpoints
        .route("/api/devices", get(routes::api_list_devices))
        .route("/system/info/get-features", post(routes::api_get_features))
        
        // MCP endpoint - Model Context Protocol
        .route("/mcp", post(routes::mcp_handle))
        
        // Merge swagger UI first
        .merge(swagger_ui)
        // Then add state and middleware
        .with_state(server_state)
        .layer(CorsLayer::permissive());
    
    let addr = "127.0.0.1:1646";
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Server started successfully:");
    info!("  📋 REST API: http://{}/api", addr);
    info!("  📚 API Documentation: http://{}/docs", addr);
    info!("  🔌 Device Management: http://{}/api/devices", addr);
    info!("  🤖 MCP Endpoint: http://{}/mcp", addr);
    
    // Spawn the server
    serve(listener, app).await?;
    
    Ok(())
} 