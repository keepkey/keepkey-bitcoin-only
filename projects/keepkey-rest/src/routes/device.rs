use crate::device::{DeviceComm, Features};
use axum::{
    extract::{Extension, Json, Path, Query},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Request/response models
#[derive(Debug, Deserialize)]
pub struct PingRequest {
    pub message: String,
    pub button_protection: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct EntropyRequest {
    pub size: u32,
}

#[derive(Debug, Serialize)]
pub struct EntropyResponse {
    pub entropy: String, // Base64 encoded
}

#[derive(Debug, Deserialize)]
pub struct ChangePinRequest {
    pub remove: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ChangePinResponse {
    pub success: bool,
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
pub async fn get_features(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
) -> Result<Json<Features>, ApiError> {
    match device.get_features() {
        Ok(features) => Ok(Json(features)),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}

pub async fn ping(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
    Json(req): Json<PingRequest>,
) -> Result<Json<PingResponse>, ApiError> {
    let button_protection = req.button_protection.unwrap_or(false);
    
    match device.ping(&req.message, button_protection) {
        Ok(message) => Ok(Json(PingResponse { message })),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}

pub async fn clear_session(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    match device.clear_session() {
        Ok(_) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}

pub async fn get_entropy(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
    Json(req): Json<EntropyRequest>,
) -> Result<Json<EntropyResponse>, ApiError> {
    match device.get_entropy(req.size) {
        Ok(entropy) => Ok(Json(EntropyResponse { 
            entropy: base64::encode(entropy)
        })),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}

pub async fn change_pin(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
    Json(req): Json<ChangePinRequest>,
) -> Result<Json<ChangePinResponse>, ApiError> {
    let remove = req.remove.unwrap_or(false);
    
    match device.change_pin(remove) {
        Ok(_) => Ok(Json(ChangePinResponse { success: true })),
        Err(e) => Err(ApiError { error: e.to_string() }),
    }
}
