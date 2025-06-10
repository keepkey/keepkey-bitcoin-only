use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Serialize;
use utoipa::ToSchema;
use tracing::{info, error};

use crate::server::ServerState;

// Manufacturing structures
#[derive(Serialize, ToSchema)]
pub struct ManufacturingHash {
    pub hash: String,
}

#[derive(Serialize, ToSchema)]
pub struct ModelPrefix {
    pub prefix: String,
}

// Route handlers for Manufacturing
#[utoipa::path(
    post,
    path = "/system/manufacturing/get-hash",
    responses(
        (status = 200, description = "Manufacturing hash", body = ManufacturingHash),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "manufacturing"
)]
pub async fn manufacturing_get_hash(
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<ManufacturingHash>, StatusCode> {
    info!("Manufacturing get hash request");
    
    match crate::server::manufacturing_get_hash_impl().await {
        Ok(hash) => {
            info!("Retrieved manufacturing hash");
            Ok(Json(ManufacturingHash { hash }))
        }
        Err(e) => {
            error!("Failed to get manufacturing hash: {}", e);
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
    path = "/system/manufacturing/model-prefix",
    responses(
        (status = 200, description = "Model prefix", body = ModelPrefix),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "manufacturing"
)]
pub async fn manufacturing_model_prefix(
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<ModelPrefix>, StatusCode> {
    info!("Manufacturing model prefix request");
    
    match crate::server::manufacturing_model_prefix_impl().await {
        Ok(prefix) => {
            info!("Retrieved model prefix");
            Ok(Json(ModelPrefix { prefix }))
        }
        Err(e) => {
            error!("Failed to get model prefix: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
} 