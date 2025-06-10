use axum::{
    extract::State,
    http::StatusCode,
    body::Bytes,
};
use std::sync::Arc;
use tracing::{info, error};

use crate::server::ServerState;

// Route handlers for Raw communication
#[utoipa::path(
    post,
    path = "/raw",
    request_body(
        content = Bytes,
        description = "Raw protobuf message",
        content_type = "application/octet-stream"
    ),
    responses(
        (status = 200, description = "Raw protobuf response", content_type = "application/octet-stream"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "raw"
)]
pub async fn raw_message(
    State(_state): State<Arc<ServerState>>,
    body: Bytes,
) -> Result<Bytes, StatusCode> {
    info!("Raw protobuf message request: {} bytes", body.len());
    
    match crate::server::raw_message_impl(body).await {
        Ok(response) => {
            info!("Raw message processed successfully: {} bytes", response.len());
            Ok(response)
        }
        Err(e) => {
            error!("Failed to process raw message: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
} 