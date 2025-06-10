use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use tracing::{info, error, warn};
use serde_json;
use hex;
use anyhow;

use crate::server::ServerState;
use super::common::ApiError;

// Helper type to handle amounts that can be either strings or numbers
#[derive(Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum AmountValue {
    String(String),
    Number(u64),
}

impl AmountValue {
    pub fn as_string(&self) -> String {
        match self {
            AmountValue::String(s) => s.clone(),
            AmountValue::Number(n) => n.to_string(),
        }
    }
}

// Bitcoin transaction structures
#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct BitcoinSignRequest {
    pub tx_hex: String,
    pub inputs: Vec<BitcoinInput>,
    pub outputs: Vec<BitcoinOutput>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct BitcoinInput {
    pub address_n: Vec<u32>,
    pub prev_hash: String,
    pub prev_index: u32,
    pub amount: String,
    pub script_type: String,
    pub hex: Option<String>, // Optional previous transaction hex
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct BitcoinOutput {
    pub address: Option<String>,
    pub address_n: Option<Vec<u32>>,
    pub amount: String,
    pub script_type: String,
}

#[derive(Serialize, ToSchema)]
pub struct BitcoinSignResponse {
    pub signatures: Vec<String>,  // Hex-encoded signatures for each input
    pub serialized_tx: String,    // Hex-encoded serialized transaction
}

// Bitcoin message signing
#[derive(Deserialize, ToSchema)]
pub struct BitcoinSignMessageRequest {
    pub address_n: Vec<u32>,
    pub message: String,
    pub coin: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BitcoinSignMessageResponse {
    pub address: String,
    pub signature: String,
}

// Bitcoin message verification
#[derive(Deserialize, ToSchema)]
pub struct BitcoinVerifyMessageRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
    pub coin: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BitcoinVerifyMessageResponse {
    pub valid: bool,
}

// UTXO transaction structures (SDK compatible)
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoSignTransactionRequest {
    pub coin: String,
    pub inputs: Vec<UtxoInput>,
    pub outputs: Vec<UtxoOutput>,
    pub version: Option<u32>,
    pub locktime: Option<u32>,
    pub op_return_data: Option<String>,
    pub vault_address: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoInput {
    pub address_n_list: Vec<u32>,
    pub txid: String,
    pub vout: u32,
    pub amount: AmountValue, // Can be string or number
    pub script_type: String,
    pub hex: Option<String>, // Full previous transaction hex
    pub tx: Option<PrevTransaction>, // Previous transaction data from SDK
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrevTransaction {
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<PrevTransactionInput>,
    pub vout: Vec<PrevTransactionOutput>,
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrevTransactionInput {
    pub txid: String,
    pub vout: u32,
    pub script_sig: ScriptSig,
    pub sequence: u32,
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScriptSig {
    pub hex: String,
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrevTransactionOutput {
    pub value: String,
    pub script_pub_key: ScriptPubKey,
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPubKey {
    pub hex: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoOutput {
    pub address: String,
    pub amount: AmountValue,  // Can be string or number
    pub address_type: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoSignTransactionResponse {
    pub serialized_tx: String,    // Hex-encoded serialized transaction
}

// Route handlers for Bitcoin
#[utoipa::path(
    post,
    path = "/bitcoin/sign-tx",
    request_body = BitcoinSignRequest,
    responses(
        (status = 200, description = "Transaction signed successfully", body = BitcoinSignResponse),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "bitcoin"
)]
pub async fn bitcoin_sign_tx(
    State(_state): State<Arc<ServerState>>, // Don't use the shared state anymore
    Json(request): Json<BitcoinSignRequest>,
) -> Result<Json<BitcoinSignResponse>, StatusCode> {
    info!("Bitcoin transaction signing request");
    info!("🔄 Using FRESH connection approach for better reliability");
    
    // Use the FRESH implementation that creates a new connection for each request
    match crate::server::impl_bitcoin::bitcoin_sign_tx_fresh_impl(request).await {
        Ok(response) => {
            info!("Transaction signed successfully with fresh connection");
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to sign transaction: {}", e);
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
    path = "/bitcoin/sign-message",
    request_body = BitcoinSignMessageRequest,
    responses(
        (status = 200, description = "Message signed successfully", body = BitcoinSignMessageResponse),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "bitcoin"
)]
pub async fn bitcoin_sign_message(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<BitcoinSignMessageRequest>,
) -> Result<Json<BitcoinSignMessageResponse>, StatusCode> {
    info!("Bitcoin message signing request");
    
    match crate::server::impl_bitcoin::bitcoin_sign_message_impl(request).await {
        Ok(response) => {
            info!("Message signed successfully");
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to sign message: {}", e);
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
    path = "/bitcoin/verify-message",
    request_body = BitcoinVerifyMessageRequest,
    responses(
        (status = 200, description = "Message verified", body = BitcoinVerifyMessageResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "bitcoin"
)]
pub async fn bitcoin_verify_message(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<BitcoinVerifyMessageRequest>,
) -> Result<Json<BitcoinVerifyMessageResponse>, StatusCode> {
    info!("Bitcoin message verification request");
    
    match crate::server::impl_bitcoin::bitcoin_verify_message_impl(request).await {
        Ok(response) => {
            info!("Message verification completed: valid={}", response.valid);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to verify message: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// UTXO sign transaction endpoint (SDK compatible)
#[utoipa::path(
    post,
    path = "/utxo/sign-transaction",
    request_body = UtxoSignTransactionRequest,
    responses(
        (status = 200, description = "Transaction signed successfully", body = UtxoSignTransactionResponse),
        (status = 404, description = "No KeepKey device found"),
        (status = 422, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "utxo"
)]
pub async fn utxo_sign_transaction(
    State(_state): State<Arc<ServerState>>, // Don't use the shared state anymore
    Json(request): Json<UtxoSignTransactionRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    info!("UTXO transaction signing request for {}", request.coin);
    info!("🔄 Using FRESH connection approach for better reliability");
    
    // Convert SDK format to our internal format
    let mut inputs = Vec::new();
    for (idx, input) in request.inputs.iter().enumerate() {
        // Get the hex for the previous transaction
        let prev_tx_hex = if let Some(hex) = &input.hex {
            // Direct hex provided
            hex.clone()
        } else if let Some(tx) = &input.tx {
            // Convert tx object to hex
            match serialize_prev_transaction(tx) {
                Ok(hex) => hex,
                Err(e) => {
                    error!("Failed to serialize previous transaction at input {}: {}", idx, e);
                    return Err(ApiError::unprocessable_entity(
                        format!("Failed to serialize previous transaction at input {}: {}", idx, e)
                    ));
                }
            }
        } else {
            error!("Input {} missing both hex and tx fields", idx);
            return Err(ApiError::unprocessable_entity(
                format!("Input {} missing both hex and tx fields", idx)
            ));
        };
        
        inputs.push(crate::server::routes::bitcoin::BitcoinInput {
            address_n: input.address_n_list.clone(),
            prev_hash: input.txid.clone(),
            prev_index: input.vout,
            amount: input.amount.as_string(),
            script_type: input.script_type.clone(),
            hex: Some(prev_tx_hex),
        });
    }
    
    let mut outputs = Vec::new();
    for output in request.outputs {
        // Detect script type based on address format
        let script_type = if output.address.starts_with("bc1q") {
            "p2wpkh".to_string()  // Native SegWit (bech32)
        } else if output.address.starts_with("bc1p") {
            "p2tr".to_string()    // Taproot (bech32m)
        } else if output.address.starts_with("3") {
            "p2sh".to_string()    // P2SH (could be p2sh-p2wpkh)
        } else if output.address.starts_with("1") {
            "p2pkh".to_string()   // Legacy P2PKH
        } else {
            // Default to p2pkh for unknown formats
            warn!("Unknown address format for {}, defaulting to p2pkh", output.address);
            "p2pkh".to_string()
        };
        
        outputs.push(crate::server::routes::bitcoin::BitcoinOutput {
            address: Some(output.address),
            address_n: None,  // SDK sends addresses, not derivation paths for outputs
            amount: output.amount.as_string(),
            script_type,
        });
    }
    
    let bitcoin_request = crate::server::routes::bitcoin::BitcoinSignRequest {
        tx_hex: "".to_string(), // Not used in our implementation
        inputs,
        outputs,
    };

    // Log the request as pretty JSON for debugging
    match serde_json::to_string_pretty(&bitcoin_request) {
        Ok(json) => info!("🔍 Bitcoin request body:\n{}", json),
        Err(_) => info!("🔍 Bitcoin request: {:?}", bitcoin_request),
    }
    
    // Use the FRESH implementation that creates a new connection for each request
    match crate::server::impl_bitcoin::bitcoin_sign_tx_fresh_impl(bitcoin_request).await {
        Ok(response) => {
            info!("Transaction signed successfully with fresh connection");
            Ok(Json(UtxoSignTransactionResponse {
                serialized_tx: response.serialized_tx,
            }))
        }
        Err(e) => {
            error!("Failed to sign transaction: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(ApiError::not_found("No KeepKey device found"))
            } else {
                Err(ApiError::internal_error(
                    format!("Failed to sign transaction: {}", e)
                ))
            }
        }
    }
}


// Helper function to serialize a previous transaction to hex
fn serialize_prev_transaction(tx: &PrevTransaction) -> Result<String, anyhow::Error> {
    use std::io::Write;
    let mut buffer = Vec::new();
    
    // Version (4 bytes, little-endian)
    buffer.write_all(&tx.version.to_le_bytes())?;
    
    // Input count (varint)
    write_varint(&mut buffer, tx.vin.len() as u64)?;
    
    // Inputs
    for input in &tx.vin {
        // Previous output hash (32 bytes, reversed)
        let txid_bytes = hex::decode(&input.txid)?;
        buffer.write_all(&txid_bytes.iter().rev().cloned().collect::<Vec<u8>>())?;
        
        // Previous output index (4 bytes, little-endian)
        buffer.write_all(&input.vout.to_le_bytes())?;
        
        // Script length (varint)
        let script_bytes = hex::decode(&input.script_sig.hex)?;
        write_varint(&mut buffer, script_bytes.len() as u64)?;
        
        // Script
        buffer.write_all(&script_bytes)?;
        
        // Sequence (4 bytes, little-endian)
        buffer.write_all(&input.sequence.to_le_bytes())?;
    }
    
    // Output count (varint)
    write_varint(&mut buffer, tx.vout.len() as u64)?;
    
    // Outputs
    for output in &tx.vout {
        // Value (8 bytes, little-endian)
        let value: u64 = output.value.parse()?;
        buffer.write_all(&value.to_le_bytes())?;
        
        // Script length (varint)
        let script_bytes = hex::decode(&output.script_pub_key.hex)?;
        write_varint(&mut buffer, script_bytes.len() as u64)?;
        
        // Script
        buffer.write_all(&script_bytes)?;
    }
    
    // Locktime (4 bytes, little-endian)
    buffer.write_all(&tx.locktime.to_le_bytes())?;
    
    Ok(hex::encode(buffer))
}

// Helper function to write a varint
fn write_varint(buffer: &mut Vec<u8>, n: u64) -> std::io::Result<()> {
    use std::io::Write;
    if n < 0xfd {
        buffer.write_all(&[n as u8])?;
    } else if n <= 0xffff {
        buffer.write_all(&[0xfd])?;
        buffer.write_all(&(n as u16).to_le_bytes())?;
    } else if n <= 0xffffffff {
        buffer.write_all(&[0xfe])?;
        buffer.write_all(&(n as u32).to_le_bytes())?;
    } else {
        buffer.write_all(&[0xff])?;
        buffer.write_all(&n.to_le_bytes())?;
    }
    Ok(())
} 