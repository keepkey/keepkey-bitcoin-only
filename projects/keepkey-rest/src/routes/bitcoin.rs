// --- Bitcoin-only REST endpoints for Vault/kkcli sharing ---
use axum::{Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};

// --- Networks ---
#[derive(Debug, Serialize)]
pub struct NetworksResponse {
    pub networks: Vec<&'static str>,
}

pub async fn list_networks() -> Json<NetworksResponse> {
    Json(NetworksResponse {
        networks: vec!["bitcoin-mainnet", "bitcoin-testnet"],
    })
}

// --- Parse Path ---
#[derive(Debug, Deserialize)]
pub struct ParsePathRequest {
    pub path: String,
}
#[derive(Debug, Serialize)]
pub struct ParsePathResponse {
    pub valid: bool,
    pub normalized: Option<String>,
    pub error: Option<String>,
}

pub async fn parse_path(Json(req): Json<ParsePathRequest>) -> Json<ParsePathResponse> {
    // Minimal mock: accept m/44'/0'/0'/0/0, reject others
    if req.path.starts_with("m/44'/0'") {
        Json(ParsePathResponse {
            valid: true,
            normalized: Some(req.path.clone()),
            error: None,
        })
    } else {
        Json(ParsePathResponse {
            valid: false,
            normalized: None,
            error: Some("Invalid or unsupported path".to_string()),
        })
    }
}

// --- Pubkey ---
#[derive(Debug, Deserialize)]
pub struct PubkeyRequest {
    pub path: String,
    pub network: String,
}
#[derive(Debug, Serialize)]
pub struct PubkeyResponse {
    pub xpub: String,
    pub pubkey: String,
}

pub async fn get_pubkey(Json(req): Json<PubkeyRequest>) -> Json<PubkeyResponse> {
    Json(PubkeyResponse {
        xpub: format!("MOCK_XPUB_{}_{}", req.network, req.path),
        pubkey: format!("MOCK_PUBKEY_{}_{}", req.network, req.path),
    })
}

// --- Balance ---
#[derive(Debug, Deserialize)]
pub struct BalanceRequest {
    pub address: String,
    pub network: String,
}
#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub balance: u64,
}

pub async fn get_balance(Json(_req): Json<BalanceRequest>) -> Json<BalanceResponse> {
    Json(BalanceResponse { balance: 123456789 }) // mock sats
}

// --- Sign Tx ---
#[derive(Debug, Deserialize)]
pub struct SignTxRequest {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub path: String,
    pub network: String,
}
#[derive(Debug, Serialize)]
pub struct SignTxResponse {
    pub signature: String,
}

pub async fn sign_tx(Json(req): Json<SignTxRequest>) -> Json<SignTxResponse> {
    Json(SignTxResponse {
        signature: format!("MOCK_SIG_TX_{}_{}", req.network, req.path),
    })
}

// --- Sign Message ---
#[derive(Debug, Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub path: String,
    pub network: String,
}
#[derive(Debug, Serialize)]
pub struct SignMessageResponse {
    pub signature: String,
}

pub async fn sign_message(Json(req): Json<SignMessageRequest>) -> Json<SignMessageResponse> {
    Json(SignMessageResponse {
        signature: format!("MOCK_SIG_MSG_{}_{}", req.network, req.path),
    })
}

// --- Router for Vault ---
pub fn bitcoin_router() -> Router {
    Router::new()
        .route("/networks", get(list_networks))
        .route("/parse-path", post(parse_path))
        .route("/pubkey", post(get_pubkey))
        .route("/balance", post(get_balance))
        .route("/sign-tx", post(sign_tx))
        .route("/sign-message", post(sign_message))
}

// --- TODO: Add robust tests for all endpoints ---
