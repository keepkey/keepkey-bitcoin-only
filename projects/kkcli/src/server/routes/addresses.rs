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

// Cosmos-based chains
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct CosmosAddressRequest {
    /// BIP-32 path as array of numbers
    pub address_n: Vec<u32>,
    /// Whether to show on device display
    pub show_display: Option<bool>,
}

// EVM chains
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct EthAddressRequest {
    /// BIP-32 path as array of numbers
    pub address_n: Vec<u32>,
    /// Whether to show on device display
    pub show_display: Option<bool>,
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

// Cosmos address generation
#[utoipa::path(
    post,
    path = "/addresses/cosmos",
    request_body = CosmosAddressRequest,
    responses(
        (status = 200, description = "Cosmos address generated", body = AddressResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "addresses"
)]
pub async fn generate_cosmos_address(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<CosmosAddressRequest>,
) -> Result<Json<AddressResponse>, StatusCode> {
    info!("Cosmos address generation request: path={:?}", request.address_n);
    
    match crate::server::generate_cosmos_address_impl(request, &state.cache, state.device_mutex.clone()).await {
        Ok(address) => {
            info!("Generated Cosmos address: {}", address);
            Ok(Json(AddressResponse { address }))
        }
        Err(e) => {
            error!("Failed to generate Cosmos address: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Ethereum address generation
#[utoipa::path(
    post,
    path = "/addresses/eth",
    request_body = EthAddressRequest,
    responses(
        (status = 200, description = "Ethereum address generated", body = AddressResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "addresses"
)]
pub async fn generate_eth_address(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<EthAddressRequest>,
) -> Result<Json<AddressResponse>, StatusCode> {
    info!("Ethereum address generation request: path={:?}", request.address_n);
    
    match crate::server::generate_eth_address_impl(request, &state.cache, state.device_mutex.clone()).await {
        Ok(address) => {
            info!("Generated Ethereum address: {}", address);
            Ok(Json(AddressResponse { address }))
        }
        Err(e) => {
            error!("Failed to generate Ethereum address: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
} 