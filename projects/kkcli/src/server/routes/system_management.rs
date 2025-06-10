use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Deserialize;
use utoipa::ToSchema;
use tracing::{info, error};

use crate::server::ServerState;

// System management structures
#[derive(Deserialize, ToSchema)]
pub struct ApplySettingsRequest {
    pub language: Option<String>,
    pub label: Option<String>,
    pub use_passphrase: Option<bool>,
    pub auto_lock_delay_ms: Option<u32>,
}

#[derive(Deserialize, ToSchema)]
pub struct ApplyPolicyRequest {
    pub enabled: bool,
    pub policy_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangePinRequest {
    pub remove: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct RecoveryDeviceRequest {
    pub word_count: u32,
    pub passphrase_protection: Option<bool>,
    pub pin_protection: Option<bool>,
    pub language: Option<String>,
    pub label: Option<String>,
    pub enforce_wordlist: Option<bool>,
    pub dry_run: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetDeviceRequest {
    pub display_random: bool,
    pub strength: Option<u32>,
    pub passphrase_protection: Option<bool>,
    pub pin_protection: Option<bool>,
    pub language: Option<String>,
    pub label: Option<String>,
    pub no_backup: Option<bool>,
    pub auto_lock_delay_ms: Option<u32>,
}

#[derive(Deserialize, ToSchema)]
pub struct LoadDeviceRequest {
    pub mnemonic: String,
    pub passphrase: Option<String>,
    pub pin: Option<String>,
    pub language: Option<String>,
    pub label: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct FirmwareUploadRequest {
    pub firmware: Vec<u8>,
}

// Route handlers for System Management
#[utoipa::path(
    post,
    path = "/system/info/apply-settings",
    request_body = ApplySettingsRequest,
    responses(
        (status = 200, description = "Settings applied successfully"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_apply_settings(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<ApplySettingsRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("Apply settings request: label={:?}", request.label);
    
    match crate::server::system_apply_settings_impl(request).await {
        Ok(_) => {
            info!("Settings applied successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to apply settings: {}", e);
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
    path = "/system/info/apply-policy",
    request_body = ApplyPolicyRequest,
    responses(
        (status = 200, description = "Policy applied successfully"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_apply_policy(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<ApplyPolicyRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("Apply policy request: {}", request.policy_name);
    
    match crate::server::system_apply_policy_impl(request).await {
        Ok(_) => {
            info!("Policy applied successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to apply policy: {}", e);
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
    path = "/system/info/change-pin",
    request_body = ChangePinRequest,
    responses(
        (status = 200, description = "PIN change initiated"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_change_pin(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<ChangePinRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("Change PIN request: remove={:?}", request.remove);
    
    match crate::server::system_change_pin_impl(request).await {
        Ok(_) => {
            info!("PIN change initiated");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to change PIN: {}", e);
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
    path = "/system/info/wipe-device",
    responses(
        (status = 200, description = "Device wiped successfully"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_wipe_device(
    State(_state): State<Arc<ServerState>>,
) -> Result<StatusCode, StatusCode> {
    info!("Wipe device request");
    
    match crate::server::system_wipe_device_impl().await {
        Ok(_) => {
            info!("Device wiped successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to wipe device: {}", e);
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
    path = "/system/info/recovery-device",
    request_body = RecoveryDeviceRequest,
    responses(
        (status = 200, description = "Recovery initiated"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_recovery_device(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<RecoveryDeviceRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("Recovery device request: word_count={}", request.word_count);
    
    match crate::server::system_recovery_device_impl(request).await {
        Ok(_) => {
            info!("Recovery initiated");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to initiate recovery: {}", e);
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
    path = "/system/info/reset-device",
    request_body = ResetDeviceRequest,
    responses(
        (status = 200, description = "Device reset initiated"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_reset_device(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<ResetDeviceRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("Reset device request");
    
    match crate::server::system_reset_device_impl(request).await {
        Ok(_) => {
            info!("Device reset initiated");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to reset device: {}", e);
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
    path = "/system/info/load-device",
    request_body = LoadDeviceRequest,
    responses(
        (status = 200, description = "Device loaded successfully"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_load_device(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<LoadDeviceRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("Load device request");
    
    match crate::server::system_load_device_impl(request).await {
        Ok(_) => {
            info!("Device loaded successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to load device: {}", e);
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
    path = "/system/info/backup-device",
    responses(
        (status = 200, description = "Backup initiated"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_backup_device(
    State(_state): State<Arc<ServerState>>,
) -> Result<StatusCode, StatusCode> {
    info!("Backup device request");
    
    match crate::server::system_backup_device_impl().await {
        Ok(_) => {
            info!("Backup initiated");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to initiate backup: {}", e);
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
    path = "/system/info/firmware-erase",
    responses(
        (status = 200, description = "Firmware erased successfully"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_firmware_erase(
    State(_state): State<Arc<ServerState>>,
) -> Result<StatusCode, StatusCode> {
    info!("Firmware erase request");
    
    match crate::server::system_firmware_erase_impl().await {
        Ok(_) => {
            info!("Firmware erased successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to erase firmware: {}", e);
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
    path = "/system/info/firmware-upload",
    request_body = FirmwareUploadRequest,
    responses(
        (status = 200, description = "Firmware uploaded successfully"),
        (status = 404, description = "No KeepKey device found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn system_firmware_upload(
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<FirmwareUploadRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("Firmware upload request: {} bytes", request.firmware.len());
    
    match crate::server::system_firmware_upload_impl(request).await {
        Ok(_) => {
            info!("Firmware uploaded successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to upload firmware: {}", e);
            if e.to_string().contains("No KeepKey device found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
} 