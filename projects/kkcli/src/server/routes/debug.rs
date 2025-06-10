use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use tracing::{info, error};

use crate::server::ServerState;

// Debug structures
#[derive(Serialize, ToSchema)]
pub struct DebugLinkState {
    pub layout: Option<String>,
    pub pin: Option<String>,
    pub matrix: Option<String>,
    pub mnemonic: Option<String>,
    pub node: Option<String>,
    pub passphrase_protection: bool,
    pub reset_word: Option<String>,
    pub reset_entropy: Option<String>,
    pub recovery_fake_word: Option<String>,
    pub recovery_word_pos: Option<u32>,
}

#[derive(Deserialize, ToSchema)]
pub struct DebugFillConfig {
    pub config: serde_json::Value,
}

// Route handlers for Debug
#[utoipa::path(
    post,
    path = "/system/debug/link-state",
    responses(
        (status = 200, description = "Debug link state", body = DebugLinkState),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "debug"
)]
pub async fn debug_link_state(
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<DebugLinkState>, StatusCode> {
    info!("Debug link state request");
    
    match crate::server::debug_link_state_impl().await {
        Ok(state) => {
            info!("Retrieved debug link state");
            Ok(Json(state))
        }
        Err(e) => {
            error!("Failed to get debug link state: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/system/debug/fill-config",
    request_body = DebugFillConfig,
    responses(
        (status = 200, description = "Config filled successfully"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "debug"
)]
pub async fn debug_fill_config(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<DebugFillConfig>,
) -> Result<StatusCode, StatusCode> {
    info!("Debug fill config request");
    
    match crate::server::debug_fill_config_impl(request).await {
        Ok(_) => {
            info!("Config filled successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to fill config: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
} 