// --- Bitcoin-only REST endpoints for Vault/kkcli sharing ---
use axum::{Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};

// --- Networks ---
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NetworksResponse {
    pub networks: Vec<&'static str>,
}

#[utoipa::path(
    get,
    path = "/api/bitcoin/networks",
    responses(
        (status = 200, description = "List supported bitcoin networks", body = NetworksResponse)
    ),
    tag = "bitcoin"
)]
pub async fn list_networks() -> Json<NetworksResponse> {
    Json(NetworksResponse {
        networks: vec!["bitcoin-mainnet", "bitcoin-testnet"],
    })
}

// --- Parse Path ---
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ParsePathRequest {
    pub path: String,
}
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ParsePathResponse {
    pub valid: bool,
    pub normalized: Option<String>,
    pub error: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/bitcoin/parse-path",
    request_body = ParsePathRequest,
    responses(
        (status = 200, description = "Parse a BIP32 path", body = ParsePathResponse)
    ),
    tag = "bitcoin"
)]
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
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PubkeyRequest {
    pub path: String,
    pub network: String,
}
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PubkeyResponse {
    pub xpub: String,
    pub pubkey: String,
}

#[utoipa::path(
    post,
    path = "/api/bitcoin/pubkey",
    request_body = PubkeyRequest,
    responses(
        (status = 200, description = "Get public key for a path", body = PubkeyResponse)
    ),
    tag = "bitcoin"
)]
pub async fn get_pubkey(Json(req): Json<PubkeyRequest>) -> Json<PubkeyResponse> {
    Json(PubkeyResponse {
        xpub: format!("MOCK_XPUB_{}_{}", req.network, req.path),
        pubkey: format!("MOCK_PUBKEY_{}_{}", req.network, req.path),
    })
}

// --- Balance ---
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BalanceRequest {
    pub address: String,
    pub network: String,
}
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BalanceResponse {
    pub balance: u64,
}

#[utoipa::path(
    post,
    path = "/api/bitcoin/balance",
    request_body = BalanceRequest,
    responses(
        (status = 200, description = "Get balance for an address", body = BalanceResponse)
    ),
    tag = "bitcoin"
)]
pub async fn get_balance(Json(_req): Json<BalanceRequest>) -> Json<BalanceResponse> {
    Json(BalanceResponse { balance: 123456789 }) // mock sats
}

// --- Sign Tx ---
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SignTxRequest {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub path: String,
    pub network: String,
}
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SignTxResponse {
    pub signature: String,
}

#[utoipa::path(
    post,
    path = "/api/bitcoin/sign-tx",
    request_body = SignTxRequest,
    responses(
        (status = 200, description = "Sign a Bitcoin transaction", body = SignTxResponse)
    ),
    tag = "bitcoin"
)]
pub async fn sign_tx(Json(req): Json<SignTxRequest>) -> Json<SignTxResponse> {
    Json(SignTxResponse {
        signature: format!("MOCK_SIG_TX_{}_{}", req.network, req.path),
    })
}

// --- Sign Message ---
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SignMessageRequest {
    pub message: String,
    pub path: String,
    pub network: String,
}
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SignMessageResponse {
    pub signature: String,
}

#[utoipa::path(
    post,
    path = "/api/bitcoin/sign-message",
    request_body = SignMessageRequest,
    responses(
        (status = 200, description = "Sign a Bitcoin message", body = SignMessageResponse)
    ),
    tag = "bitcoin"
)]
pub async fn sign_message(Json(req): Json<SignMessageRequest>) -> Json<SignMessageResponse> {
    Json(SignMessageResponse {
        signature: format!("MOCK_SIG_MSG_{}_{}", req.network, req.path),
    })
}

// --- Router for Vault ---
use axum::extract::State;

/// Accepts any Axum state type S. Vault should call bitcoin_router(server_state.clone()).
pub fn bitcoin_router<S: Clone + Send + Sync + 'static>(state: S) -> Router<S> {
    Router::new()
        .route("/networks", axum::routing::get(list_networks))
        .route("/parse-path", axum::routing::post(parse_path))
        .route("/pubkey", axum::routing::post(get_pubkey))
        .route("/balance", axum::routing::post(get_balance))
        .route("/sign-tx", axum::routing::post(sign_tx))
        .route("/sign-message", axum::routing::post(sign_message))
        .with_state(state)
}

// Update all handlers to accept State<Arc<ServerState>> if needed.

// --- TODO: Add robust tests for all endpoints ---
