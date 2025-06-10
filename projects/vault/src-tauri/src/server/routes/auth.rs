use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use tracing::info;
use chrono::Utc;
use uuid::Uuid;

use super::super::AppState;

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PairingInfo {
    /// Application name requesting pairing
    pub name: String,
    /// Application URL or identifier  
    pub url: String,
    /// Application icon URL
    pub image_url: String,
    /// When this pairing was added (optional)
    pub added_on: Option<u64>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub api_key: String,
}

#[utoipa::path(
    get,
    path = "/auth/pair",
    responses(
        (status = 200, description = "API key verification successful", body = PairingInfo),
        (status = 403, description = "Invalid API key")
    ),
    tag = "auth",
    security(("apiKey" = []))
)]
pub async fn auth_verify(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<PairingInfo>, StatusCode> {
    // For now, accept any request as valid verification
    // In a real implementation, you would check the Authorization header
    info!("Auth verification request received");
    
    Ok(Json(PairingInfo {
        name: "KeepKey Vault".to_string(),
        url: "http://localhost:1646".to_string(),
        image_url: "https://github.com/BitHighlander/keepkey-desktop/raw/master/electron/icon.png".to_string(),
        added_on: Some(Utc::now().timestamp() as u64),
    }))
}

#[utoipa::path(
    post,
    path = "/auth/pair",
    request_body = PairingInfo,
    responses(
        (status = 200, description = "Pairing successful", body = AuthResponse),
        (status = 403, description = "Pairing request rejected")
    ),
    tag = "auth"
)]
pub async fn auth_pair(
    State(_state): State<Arc<AppState>>,
    Json(pairing_info): Json<PairingInfo>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("Pairing request from: {} ({})", pairing_info.name, pairing_info.url);
    
    // Generate a new API key for this pairing
    let api_key = Uuid::new_v4().to_string();
    
    info!("Generated new API key for {}", pairing_info.name);
    
    // In a real implementation, you would:
    // 1. Show a pairing prompt on the device
    // 2. Wait for user confirmation
    // 3. Store the pairing info and API key
    // 4. Only return success if user approved
    
    Ok(Json(AuthResponse { api_key }))
} 