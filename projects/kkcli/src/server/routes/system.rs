use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Deserialize;
use utoipa::ToSchema;
use tracing::{info, error};
use chrono::Utc;
use tokio::time::timeout;
use hex;

use crate::server::{ServerState, try_get_device, DEVICE_OPERATION_TIMEOUT};
use crate::transport::{UsbTransport, ProtocolAdapter};
use crate::messages::{self, Message};
use super::common::{HealthResponse, PublicKeyResponse, Coin, PingRequest, PingResponse, EntropyRequest};
use super::device::Features;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct PublicKeyRequest {
    pub address_n: Vec<u32>,
    pub ecdsa_curve_name: Option<String>,
    pub show_display: Option<bool>,
    pub coin_name: Option<String>,
    pub script_type: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse)
    ),
    tag = "system"
)]
pub async fn health_check() -> Json<HealthResponse> {
    // Version matches Cargo.toml
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        service: "KeepKey CLI API".to_string(),
        version: "0.2.3".to_string(),
    })
}

// KeepKey SDK compatible endpoints
#[utoipa::path(
    post,
    path = "/system/info/get-features",
    responses(
        (status = 200, description = "Device features", body = Features),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_get_features(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Features>, StatusCode> {
    match crate::server::get_features_sdk_compatible(&state.cache).await {
        Ok(features) => {
            info!("Retrieved device features (SDK compatible)");
            Ok(Json(features))
        }
        Err(e) => {
            error!("Failed to get device features: {}", e);
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
    path = "/system/info/get-entropy",
    request_body = EntropyRequest,
    responses(
        (status = 200, description = "Entropy data", content_type = "application/octet-stream"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_get_entropy(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<EntropyRequest>,
) -> Result<Vec<u8>, StatusCode> {
    info!("🎲 Generating {} bytes of entropy from device", request.size);

    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
        
        // Create GetEntropy message
        let get_entropy_msg = messages::GetEntropy {
            size: request.size,
        };
        
        info!("Sending GetEntropy message to device for {} bytes", request.size);
        
        // Send the message and wait for response
        let response = transport.with_standard_handler().handle(get_entropy_msg.into())?;
        
        // Extract the entropy from the response
        match response {
            Message::Entropy(entropy_msg) => {
                info!("✅ Received {} bytes of entropy from device", entropy_msg.entropy.len());
                Ok(entropy_msg.entropy)
            }
            unexpected_msg => {
                error!("Unexpected response to GetEntropy: {:?}", unexpected_msg);
                Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
            }
        }
        } else {
            error!("Device transport not available for GetEntropy.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(entropy)) => Ok(entropy),
        Ok(Err(e)) => {
            error!("Device communication failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            error!("Device communication timed out");
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

#[utoipa::path(
    post,
    path = "/system/info/get-public-key",
    request_body = PublicKeyRequest,
    responses(
        (status = 200, description = "Public key", body = PublicKeyResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_get_public_key(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<PublicKeyRequest>,
) -> Result<Json<PublicKeyResponse>, StatusCode> {
    info!("🔑 Getting public key for path: {:?}", request.address_n);

    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
        
        // Create GetPublicKey message
        let mut get_public_key_msg = messages::GetPublicKey::default();
        get_public_key_msg.address_n = request.address_n.clone();
        get_public_key_msg.ecdsa_curve_name = request.ecdsa_curve_name;
        get_public_key_msg.show_display = request.show_display;
        get_public_key_msg.coin_name = request.coin_name;

        // Set script type if provided
        if let Some(script_type) = request.script_type {
            match script_type.as_str() {
                "p2pkh" => get_public_key_msg.script_type = Some(messages::InputScriptType::Spendaddress as i32),
                "p2wpkh" => get_public_key_msg.script_type = Some(messages::InputScriptType::Spendwitness as i32),
                "p2sh-p2wpkh" => get_public_key_msg.script_type = Some(messages::InputScriptType::Spendp2shwitness as i32),
                _ => get_public_key_msg.script_type = Some(messages::InputScriptType::Spendaddress as i32),
            }
        }
        
        info!("Sending GetPublicKey message to device");
        
        // Send the message and wait for response
        let response = transport.with_standard_handler().handle(get_public_key_msg.into())?;
        
        // Extract the public key from the response
        match response {
            Message::PublicKey(public_key_msg) => {
                let xpub = public_key_msg.xpub.unwrap_or_else(|| "".to_string());
                info!("✅ Received public key from device");
                
                Ok(PublicKeyResponse {
                    xpub,
                })
            }
            unexpected_msg => {
                error!("Unexpected response to GetPublicKey: {:?}", unexpected_msg);
                Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
            }
        }
        } else {
            error!("Device transport not available for GetPublicKey.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(public_key_response)) => Ok(Json(public_key_response)),
        Ok(Err(e)) => {
            error!("Device communication failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            error!("Device communication timed out");
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

#[utoipa::path(
    post,
    path = "/system/info/list-coins",
    responses(
        (status = 200, description = "Supported coins", body = Vec<Coin>),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_list_coins(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<Coin>>, StatusCode> {
    info!("🪙 Listing supported coins from device");

    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
        
        // Get initial coin table info
        let get_coin_table_msg = messages::GetCoinTable {
            start: None,
            end: None,
        };
        
        info!("Sending GetCoinTable message to device");
        
        let response = transport.handle(get_coin_table_msg.into())?;
        
        match response {
            Message::CoinTable(coin_table_msg) => {
                let num_coins = coin_table_msg.num_coins.unwrap_or(0);
                let chunk_size = coin_table_msg.chunk_size.unwrap_or(1);
                
                info!("Device reports {} coins available", num_coins);
                
                // Collect all coins in chunks
                let mut all_coins = Vec::new();
                
                for start in (0..num_coins).step_by(chunk_size as usize) {
                    let end = std::cmp::min(num_coins, start + chunk_size);
                    
                    let chunk_request = messages::GetCoinTable {
                        start: Some(start),
                        end: Some(end),
                    };
                    
                    let chunk_response = transport.handle(chunk_request.into())?;
                    
                    if let Message::CoinTable(chunk_msg) = chunk_response {
                        for coin_type in chunk_msg.table {
                            all_coins.push(Coin {
                                coin_name: coin_type.coin_name,
                                coin_shortcut: coin_type.coin_shortcut,
                                address_type: coin_type.address_type,
                                maxfee_kb: coin_type.maxfee_kb,
                                address_type_p2sh: coin_type.address_type_p2sh,
                                signed_message_header: coin_type.signed_message_header,
                                bip44_account_path: coin_type.bip44_account_path,
                                forkid: coin_type.forkid,
                                decimals: coin_type.decimals,
                                contract_address: coin_type.contract_address.map(|bytes| hex::encode(bytes)),
                                xpub_magic: coin_type.xpub_magic,
                                segwit: coin_type.segwit,
                                force_bip143: coin_type.force_bip143,
                                curve_name: coin_type.curve_name,
                                cashaddr_prefix: coin_type.cashaddr_prefix,
                                bech32_prefix: coin_type.bech32_prefix,
                                decred: coin_type.decred,
                                xpub_magic_segwit_p2sh: coin_type.xpub_magic_segwit_p2sh,
                                xpub_magic_segwit_native: coin_type.xpub_magic_segwit_native,
                                nanoaddr_prefix: coin_type.nanoaddr_prefix,
                            });
                        }
                    }
                }
                
                info!("✅ Retrieved {} coins from device", all_coins.len());
                Ok(all_coins)
            }
            unexpected_msg => {
                error!("Unexpected initial response to GetCoinTable: {:?}", unexpected_msg);
                Err(anyhow::anyhow!("Unexpected initial response type from device: {:?}", unexpected_msg.message_type()))
            }
        }
        } else {
            error!("Device transport not available for GetCoinTable.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(coins)) => Ok(Json(coins)),
        Ok(Err(e)) => {
            error!("Device communication failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            error!("Device communication timed out");
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

#[utoipa::path(
    post,
    path = "/system/info/ping",
    request_body = PingRequest,
    responses(
        (status = 200, description = "Ping response", body = PingResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_ping(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<PingRequest>,
) -> Result<Json<PingResponse>, StatusCode> {
    info!("🏓 Ping request: {:?}", request.message);

    // Wrap device communication in timeout
    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
        
        // Create Ping message
        let ping_msg = messages::Ping {
            message: request.message.clone(),
            button_protection: request.button_protection,
            pin_protection: request.pin_protection,
            passphrase_protection: request.passphrase_protection,
            wipe_code_protection: None,
        };
        
        info!("Sending Ping message to device");
        
        // Send the message and wait for response
        let response = transport.with_standard_handler().handle(ping_msg.into())?;
        
        // Extract the response
        match response {
            Message::Success(success_msg) => {
                let message = success_msg.message.unwrap_or_else(|| "pong".to_string());
                info!("✅ Ping successful: {}", message);
                Ok(PingResponse { message })
            }
            unexpected_msg => {
                error!("Unexpected response to Ping: {:?}", unexpected_msg);
                Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
            }
        }
        } else {
            error!("Device transport not available for Ping.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(ping_response)) => Ok(Json(ping_response)),
        Ok(Err(e)) => {
            error!("Device communication failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            error!("Device communication timed out");
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
} 