use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

// Common response structures
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub service: String,
    pub version: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressResponse {
    /// The generated address
    pub address: String,
}

#[derive(Serialize, ToSchema)]
pub struct PublicKeyResponse {
    pub xpub: String,
}

#[derive(Serialize, ToSchema)]
pub struct PingResponse {
    pub message: String,
}

// Coin table entry
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Coin {
    pub coin_name: Option<String>,
    pub coin_shortcut: Option<String>,
    pub address_type: Option<u32>,
    pub maxfee_kb: Option<u64>,
    pub address_type_p2sh: Option<u32>,
    pub signed_message_header: Option<String>,
    pub bip44_account_path: Option<u32>,
    pub forkid: Option<u32>,
    pub decimals: Option<u32>,
    pub xpub_magic: Option<u32>,
    pub segwit: Option<bool>,
    pub force_bip143: Option<bool>,
    pub cashaddr_prefix: Option<String>,
    pub bech32_prefix: Option<String>,
    pub decred: Option<bool>,
    pub xpub_magic_segwit_p2sh: Option<u32>,
    pub xpub_magic_segwit_native: Option<u32>,
}

// Common request structures
#[derive(Deserialize, ToSchema)]
pub struct EntropyRequest {
    pub size: u32,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct PingRequest {
    pub button_protection: Option<bool>,
    pub pin_protection: Option<bool>,
    pub passphrase_protection: Option<bool>,
    pub wipe_code_protection: Option<bool>,
    pub message: Option<String>,
}

// Common error response structure
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// Error response helper
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

// AppError is an alias to ApiError for better naming in API handlers
pub type AppError = ApiError;

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            details: None,
        }
    }
    
    // Add convenience method for JSON errors
    pub fn new_json(status: StatusCode, error_json: serde_json::Value) -> Self {
        let message = match error_json.get("message") {
            Some(serde_json::Value::String(msg)) => msg.clone(),
            _ => format!("Error: {:?}", error_json),
        };
        
        Self {
            status,
            message,
            details: Some(error_json),
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: match self.status {
                StatusCode::UNPROCESSABLE_ENTITY => "unprocessable_entity",
                StatusCode::NOT_FOUND => "not_found",
                StatusCode::INTERNAL_SERVER_ERROR => "internal_server_error",
                _ => "error",
            }.to_string(),
            message: self.message,
            details: self.details,
        };

        (self.status, Json(body)).into_response()
    }
} 