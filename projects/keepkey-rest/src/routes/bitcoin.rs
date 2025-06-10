use crate::device::{DeviceComm, Address, TxInput, TxOutput, SignedTx, SignedMessage};
use axum::{
    extract::{Extension, Json, Path, Query},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

// Request/response models
#[derive(Debug, Deserialize)]
pub struct GetAddressRequest {
    pub coin_name: String,
    pub address_n: Vec<u32>,
    pub show_display: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct GetAddressResponse {
    pub address: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SignTxRequest {
    pub coin_name: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Debug, Serialize)]
pub struct SignTxResponse {
    pub signatures: Vec<String>,
    pub serialized_tx: String,
}

#[derive(Debug, Deserialize)]
pub struct SignMessageRequest {
    pub coin_name: String,
    pub address_n: Vec<u32>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SignMessageResponse {
    pub address: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyMessageRequest {
    pub coin_name: String,
    pub address: String,
    pub signature: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyMessageResponse {
    pub verified: bool,
}

// Error handling
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

// API Routes
pub async fn get_address(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
    Json(req): Json<GetAddressRequest>,
) -> Result<Json<GetAddressResponse>, ApiError> {
    let show_display = req.show_display.unwrap_or(false);
    
    match device.get_address(&req.coin_name, &req.address_n, show_display) {
        Ok(address) => Ok(Json(GetAddressResponse {
            address: address.address,
            path: address.path,
        })),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}

pub async fn sign_tx(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
    Json(req): Json<SignTxRequest>,
) -> Result<Json<SignTxResponse>, ApiError> {
    match device.sign_tx(&req.coin_name, &req.inputs, &req.outputs) {
        Ok(result) => Ok(Json(SignTxResponse {
            signatures: result.signatures,
            serialized_tx: result.serialized_tx,
        })),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}

pub async fn sign_message(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
    Json(req): Json<SignMessageRequest>,
) -> Result<Json<SignMessageResponse>, ApiError> {
    match device.sign_message(&req.coin_name, &req.address_n, &req.message) {
        Ok(result) => Ok(Json(SignMessageResponse {
            address: result.address,
            signature: result.signature,
        })),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}

pub async fn verify_message(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
    Json(req): Json<VerifyMessageRequest>,
) -> Result<Json<VerifyMessageResponse>, ApiError> {
    match device.verify_message(&req.coin_name, &req.address, &req.signature, &req.message) {
        Ok(verified) => Ok(Json(VerifyMessageResponse { verified })),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}
