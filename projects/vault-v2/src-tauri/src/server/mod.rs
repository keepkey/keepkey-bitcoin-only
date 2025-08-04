pub mod routes;
pub mod context;
pub mod proxy;

use axum::{
    Router,
    serve,
    routing::{get, post},
    response::Json,
};

use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, debug};
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
        
        // Add compatibility route for Pioneer SDK kkapi detection
        .route("/spec/swagger.json", get(|| async move {
            Json(ApiDoc::openapi())
        }))
        
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
        .layer(
            CorsLayer::new()
                // Allow any origin with wildcard (includes localhost:8080 proxy)
                .allow_origin(tower_http::cors::Any)
                // Allow all methods
                .allow_methods(tower_http::cors::Any)
                // Allow all headers including X-Requested-With for AJAX
                .allow_headers(tower_http::cors::Any)
                // Max age for preflight caching
                .max_age(std::time::Duration::from_secs(3600))
                // Note: credentials cannot be used with wildcard origin
                .allow_credentials(false)
        );
    
    let addr = "127.0.0.1:1646";
    let listener = TcpListener::bind(addr).await?;
    
    // Start the proxy server on port 8080
    let proxy_addr = "127.0.0.1:8080";
    let proxy_app = proxy::create_proxy_router();
    let proxy_listener = TcpListener::bind(proxy_addr).await?;
    
    info!("🚀 Starting servers:");
    info!("  📋 REST API: http://{}/api", addr);
    info!("  🌍 Proxy: http://{} -> keepkey.com", proxy_addr);
    info!("  📚 API Documentation: http://{}/docs", addr);
    debug!("  🔌 Device Management: http://{}/api/devices", addr);
    debug!("  🤖 MCP Endpoint: http://{}/mcp", addr);
    debug!("  📄 Swagger JSON: http://{}/spec/swagger.json", addr);
    
    // Start the proxy server in a separate task
    let proxy_handle = tokio::spawn(async move {
        serve(proxy_listener, proxy_app).await
    });
    
    // Small delay to let proxy server start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    info!("✅ Both servers started successfully and are ready");
    
    // Run both servers concurrently
    tokio::select! {
        result = serve(listener, app) => {
            if let Err(e) = result {
                tracing::error!("API server error: {}", e);
            }
        }
        result = proxy_handle => {
            if let Err(e) = result {
                tracing::error!("Proxy server error: {}", e);
            }
        }
    }
    
    Ok(())
} 