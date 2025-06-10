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
use super::common::AddressResponse;

// UTXO Address types
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct UtxoAddressRequest {
    /// BIP-32 path as array of numbers
    pub address_n: Vec<u32>,
    /// Coin name (e.g., "Bitcoin")
    pub coin: String,
    /// Script type (e.g., "p2sh-p2wpkh")
    pub script_type: Option<String>,
    /// Whether to show on device display
    pub show_display: Option<bool>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoAddressResponse {
    /// The generated address
    pub address: String,
    /// The derivation path used
    pub address_n: Vec<u32>,
}



// Route handlers
#[utoipa::path(
    post,
    path = "/addresses/utxo",
    request_body = UtxoAddressRequest,
    responses(
        (status = 200, description = "Address generated successfully", body = UtxoAddressResponse),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "addresses"
)]
pub async fn generate_utxo_address(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<UtxoAddressRequest>,
) -> Result<Json<UtxoAddressResponse>, StatusCode> {
    info!("UTXO address generation request: coin={}, script_type={:?}, path={:?}", 
        request.coin, request.script_type, request.address_n);
    
    match crate::server::generate_utxo_address_impl(request, &state.cache, state.device_mutex.clone()).await {
        Ok(response) => {
            info!("Generated address: {}", response.address);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to generate address: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
} 


