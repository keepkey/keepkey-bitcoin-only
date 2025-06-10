pub mod routes;

use axum::{
    Router,
    serve,
    routing::{get, post},
    response::sse::{Event, Sse},
    http::{StatusCode, HeaderMap, header},
    Json,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::time::Duration;
use serde_json::{json, Value};
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::usb_manager::DeviceManager;

pub struct ServerState {
    pub device_manager: Arc<Mutex<DeviceManager>>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health_check,
        routes::device_status,
        routes::list_devices,
        routes::registry_status,
        routes::firmware_releases,
    ),
    components(
        schemas(
            routes::HealthResponse,
            routes::DeviceStatus,
            routes::DeviceInfo,
            routes::KeepKeyInfo,
            routes::UsbDeviceInfo,
            routes::ApiResponse<routes::DeviceStatus>,
        )
    ),
    tags(
        (name = "system", description = "System health and status endpoints"),
        (name = "device", description = "KeepKey device management endpoints")
    ),
    info(
        title = "KeepKey Desktop API",
        version = "0.1.0",
        description = "REST API for KeepKey Desktop application with device management and MCP server functionality",
        contact(
            name = "KeepKey Support",
            url = "https://keepkey.com"
        )
    )
)]
struct ApiDoc;

pub async fn start_server(device_manager: Arc<Mutex<DeviceManager>>) -> anyhow::Result<()> {
    info!("Starting unified server with REST API and MCP on port 1646");
    
    let server_state = Arc::new(ServerState {
        device_manager,
    });

    // Create Swagger UI
    let swagger_ui = SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    let app = Router::new()
        // REST API endpoints
        .route("/api/health", get(routes::health_check))
        .route("/api/status", get(routes::device_status))
        .route("/api/devices", get(routes::list_devices))
        .route("/api/devices/debug", get(routes::debug_devices))
        .route("/api/devices/registry", get(routes::registry_status))
        .route("/api/firmware", get(routes::firmware_releases))
        // MCP endpoints
        .route("/mcp", post(mcp_json_rpc_handler))
        .route("/mcp", get(mcp_sse_handler))
        .with_state(server_state)
        // Swagger UI
        .merge(swagger_ui)
        .layer(CorsLayer::permissive());

    let addr = "127.0.0.1:1646";
    
    // Bind the listener first
    let listener = TcpListener::bind(addr).await?;
    
    // Log server info after successful binding
    info!("Server started successfully:");
    info!("  - REST API: http://{}/api", addr);
    info!("  - API Documentation: http://{}/docs", addr);
    info!("  - OpenAPI Spec: http://{}/api-docs/openapi.json", addr);
    info!("  - MCP JSON-RPC: http://{}/mcp", addr);
    info!("  - MCP SSE: http://{}/mcp (with Accept: text/event-stream)", addr);
    
    // Spawn the server on a separate tokio task so it doesn't block this function
    tokio::spawn(async move {
        if let Err(e) = serve(listener, app).await {
            info!("Server error: {}", e);
        }
    });
    
    // Return immediately, not waiting for the server to complete
    Ok(())
}

async fn mcp_json_rpc_handler(
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    info!("Received MCP JSON-RPC request: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
    
    let response = handle_mcp_request(request).await;
    
    info!("Sending MCP JSON-RPC response: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
    
    Ok(Json(response))
}

async fn mcp_sse_handler(
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // Check if client accepts SSE
    if let Some(accept) = headers.get(header::ACCEPT) {
        if accept.to_str().unwrap_or("").contains("text/event-stream") {
            info!("Starting MCP SSE connection");
            
            let stream = stream::unfold(0u64, move |counter| {
                async move {
                    if counter == 0 {
                        // Send initial connection event
                        let event = Event::default()
                            .data(json!({
                                "type": "connection",
                                "status": "connected",
                                "server": {
                                    "name": "KeepKey Desktop MCP Server",
                                    "version": "0.1.0",
                                    "protocol": "2024-11-05"
                                }
                            }).to_string());
                        
                        Some((Ok(event), counter + 1))
                    } else {
                        // Send periodic heartbeat
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        
                        let event = Event::default()
                            .event("heartbeat")
                            .data(json!({
                                "type": "heartbeat",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }).to_string());
                        
                        Some((Ok(event), counter + 1))
                    }
                }
            });
            
            return Ok(Sse::new(stream));
        }
    }
    
    Err(StatusCode::NOT_ACCEPTABLE)
}

async fn handle_mcp_request(request: Value) -> Value {
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = request.get("id").cloned();
    
    match method {
        "initialize" => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "resources": {}
                    },
                    "serverInfo": {
                        "name": "KeepKey Desktop MCP Server",
                        "version": "0.1.0"
                    }
                }
            })
        }
        "tools/list" => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "get_device_status",
                            "description": "Get KeepKey device status",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "get_device_features",
                            "description": "Get KeepKey device features and information",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "list_usb_devices",
                            "description": "List all USB devices connected to the system",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        }
                    ]
                }
            })
        }
        "tools/call" => {
            let tool_name = request.get("params")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            
            match tool_name {
                "get_device_status" => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": "Device status retrieved successfully"
                                }
                            ]
                        }
                    })
                }
                "get_device_features" => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": "Device features retrieved successfully"
                                }
                            ]
                        }
                    })
                }
                "list_usb_devices" => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": "USB devices listed successfully"
                                }
                            ]
                        }
                    })
                }
                _ => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": "Method not found"
                        }
                    })
                }
            }
        }
        _ => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            })
        }
    }
} 