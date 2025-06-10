use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, warn, error};
use base64::{Engine as _, engine::general_purpose};

use crate::server::ServerState;

/// Create mapping of CAIP-19 identifiers to icon file paths
fn get_caip19_to_icon_mapping() -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    
    // Bitcoin family
    mapping.insert(
        "bip122:000000000019d6689c085ae165831e93/slip44:0".to_string(),
        "assets/icons/bitcoin.png".to_string()
    );
    mapping.insert(
        "bip122:000000000000000000651ef99cb9fcbe/slip44:145".to_string(),
        "assets/icons/bitcoin-cash.png".to_string()
    );
    
    // Ethereum
    mapping.insert(
        "eip155:1/slip44:60".to_string(),
        "assets/icons/ethereum.png".to_string()
    );
    
    // Other UTXO coins
    mapping.insert(
        "bip122:12a765e31ffd4059bada1e25190f6e98/slip44:2".to_string(),
        "assets/icons/litecoin.png".to_string()
    );
    mapping.insert(
        "bip122:1a91e3dace36e2be3bf030a28c99f6e5/slip44:3".to_string(),
        "assets/icons/dogecoin.png".to_string()
    );
    mapping.insert(
        "bip122:00000ffd590b1485b3caadc19b22e637/slip44:5".to_string(),
        "assets/icons/dash.png".to_string()
    );
    mapping.insert(
        "bip122:7497ea1b465eb39f1c8f507bc877078f/slip44:20".to_string(),
        "assets/icons/digibyte.png".to_string()
    );
    mapping.insert(
        "bip122:00040fe8ec8471911baa1db1266ea15d/slip44:133".to_string(),
        "assets/icons/zcash.png".to_string()
    );
    
    // Cosmos ecosystem
    mapping.insert(
        "cosmos:cosmoshub-4/slip44:118".to_string(),
        "assets/icons/cosmos.png".to_string()
    );
    mapping.insert(
        "cosmos:thorchain-mainnet-v1/slip44:931".to_string(),
        "assets/icons/thorchain.png".to_string()
    );
    mapping.insert(
        "cosmos:mayachain-mainnet-v1/slip44:931".to_string(),
        "assets/icons/mayachain.png".to_string()
    );
    mapping.insert(
        "cosmos:osmosis-1/slip44:118".to_string(),
        "assets/icons/osmosis.png".to_string()
    );
    
    // Other supported chains  
    mapping.insert(
        "eip155:137/slip44:966".to_string(),
        "assets/icons/polygon.png".to_string()
    );
    mapping.insert(
        "eip155:43114/slip44:9005".to_string(),
        "assets/icons/avalanche.png".to_string()
    );
    mapping.insert(
        "ripple:mainnet/slip44:144".to_string(),
        "assets/icons/ripple.png".to_string()
    );
    
    mapping
}

/// Serve coin icons for primary assets using CAIP-19 base64 encoded identifiers
/// GET /icons/{base64_caip19}.png
#[utoipa::path(
    get,
    path = "/icons/{base64_caip19}.png",
    tag = "icons",
    summary = "Get coin icon",
    description = "Retrieve PNG icon for a cryptocurrency using base64-encoded CAIP-19 identifier. Only primary assets on supported networks are available.",
    params(
        ("base64_caip19" = String, Path, description = "Base64-encoded CAIP-19 identifier")
    ),
    responses(
        (status = 200, description = "Icon file", content_type = "image/png"),
        (status = 404, description = "Icon not found"),
        (status = 400, description = "Invalid base64 or CAIP-19 format"),
        (status = 500, description = "Server error")
    )
)]
pub async fn get_coin_icon(
    State(_state): State<Arc<ServerState>>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let base64_caip19 = filename.trim_end_matches(".png");
    
    info!("🖼️ Icon request for base64 CAIP-19: {}", base64_caip19);
    
    // Decode base64 to get CAIP-19 identifier
    let caip19 = match general_purpose::STANDARD.decode(base64_caip19) {
        Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
            Ok(caip19_str) => caip19_str,
            Err(e) => {
                warn!("🚫 Invalid UTF-8 in decoded CAIP-19: {}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    "Invalid UTF-8 in CAIP-19 identifier".to_string()
                ).into_response();
            }
        },
        Err(e) => {
            warn!("🚫 Invalid base64 encoding: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                "Invalid base64 encoding".to_string()
            ).into_response();
        }
    };
    
    info!("🔍 Decoded CAIP-19: {}", caip19);
    
    // Get the mapping and look up the icon path
    let mapping = get_caip19_to_icon_mapping();
    let icon_path = match mapping.get(&caip19) {
        Some(path) => path,
        None => {
            warn!("🚫 Icon not found for CAIP-19: {}", caip19);
            return (
                StatusCode::NOT_FOUND,
                format!("Icon not found for CAIP-19: {}", caip19)
            ).into_response();
        }
    };
    
    // Try to read the icon file
    match fs::read(icon_path).await {
        Ok(content) => {
            info!("✅ Served icon for CAIP-19: {}", caip19);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/png")
                .header(header::CACHE_CONTROL, "public, max-age=86400") // Cache for 24 hours
                .body(content.into())
                .unwrap()
                .into_response()
        }
        Err(e) => {
            error!("❌ Failed to read icon file {}: {}", icon_path, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read icon file: {}", e)
            ).into_response()
        }
    }
}

/// List available coin icons with their CAIP-19 base64 encoded names
/// GET /icons
#[utoipa::path(
    get,
    path = "/icons",
    tag = "icons", 
    summary = "List available icons",
    description = "Get a list of all available coin icons for primary assets with CAIP-19 base64 encoded names",
    responses(
        (status = 200, description = "Map of CAIP-19 identifiers to base64 encoded filenames", body = std::collections::HashMap<String, String>),
    )
)]
pub async fn list_coin_icons() -> impl IntoResponse {
    let mapping = get_caip19_to_icon_mapping();
    let mut available_icons = std::collections::HashMap::new();
    
    // Convert CAIP-19 identifiers to base64 encoded filenames
    for (caip19, _path) in mapping.iter() {
        let base64_name = general_purpose::STANDARD.encode(caip19.as_bytes());
        let filename = format!("{}.png", base64_name);
        available_icons.insert(caip19.clone(), filename);
    }
    
    info!("📋 Listed {} available icons with CAIP-19 base64 encoding", available_icons.len());
    axum::Json(available_icons).into_response()
} 