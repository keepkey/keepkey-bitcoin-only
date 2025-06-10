use keepkey_rest::bitcoin_router;
use keepkey_rest::bitcoin_router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::ServiceExt; // for .oneshot
use serde_json::json;

const BTC_MAINNET: &str = "bitcoin-mainnet";
const BTC_PATHS: [(&str, &str); 3] = [
    ("legacy", "m/44'/0'/0'"),
    ("segwit", "m/49'/0'/0'"),
    ("native_segwit", "m/84'/0'/0'"),
];

#[tokio::test]
async fn test_parse_paths_for_all_script_types() {
    let app = bitcoin_router();
    for (script, path) in BTC_PATHS.iter() {
        let payload = json!({"path": path});
        let resp = app.clone().oneshot(Request::post("/parse-path")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string())).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "parse-path failed for {script}");
        let body = to_bytes(resp.into_body(), 1024*1024).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed["valid"], true, "Path not valid for {script}");
        assert_eq!(parsed["normalized"], *path, "Path not normalized for {script}");
    }
}

#[tokio::test]
async fn test_pubkey_for_all_script_types() {
    let app = bitcoin_router();
    for (script, path) in BTC_PATHS.iter() {
        let payload = json!({"path": path, "network": BTC_MAINNET});
        let resp = app.clone().oneshot(Request::post("/pubkey")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string())).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "pubkey failed for {script}");
        let body = to_bytes(resp.into_body(), 1024*1024).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(parsed["xpub"].as_str().unwrap().contains(path), "XPUB missing path for {script}");
        assert!(parsed["pubkey"].as_str().unwrap().contains(path), "Pubkey missing path for {script}");
    }
}

#[tokio::test]
async fn test_balance_for_all_script_types() {
    let app = bitcoin_router();
    // Use a mock address for each script type
    let addresses = [
        "1MockLegacyBTCAddress1111111111114T1an2",
        "3MockSegwitBTCAddress1111111111114T1an2",
        "bc1qmocknativebtcaddress11111111111p7n4j",
    ];
    for (i, (script, _)) in BTC_PATHS.iter().enumerate() {
        let payload = json!({"address": addresses[i], "network": BTC_MAINNET});
        let resp = app.clone().oneshot(Request::post("/balance")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string())).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "balance failed for {script}");
        let body = to_bytes(resp.into_body(), 1024*1024).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed["balance"], 123456789, "Mock balance wrong for {script}");
    }
}
