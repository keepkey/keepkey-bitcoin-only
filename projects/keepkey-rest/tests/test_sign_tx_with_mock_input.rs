use keepkey_rest::bitcoin_router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::ServiceExt; // for .oneshot

#[tokio::test]
async fn test_sign_tx_with_mock_input() {
    let app = bitcoin_router();
    let payload = serde_json::json!({
        "coin": "Bitcoin",
        "inputs": [
            {
                "addressNList": [2147483732,2147483648,2147483648,0,0],
                "scriptType": "p2wpkh",
                "amount": "38544",
                "vout": 0,
                "txid": "a47f60ff416f17cc9b0543f8afe05ae8bad98c9c2c69c207ecc0d50799dc52f0",
                "hex": "0200000000010192b0bf35840f7846fa0fbad2586bc2dd81d1ae2d82f1691e8726bb49fd11648f0100000000fdffffff0490960000000000001600143b944081068951cc1b040818ad95ee1146cbef63204e0000000000001976a914ff655cc26f2ec3d0e34030bdcf5182e95907a17488acbb4f00000000000017a914d57d7d91f1843c5ab88c9aa48a183363bda3a5cc8706fc020000000000160014b66b3f985645441b52cc64151816f8af697516cd02483045022100d7f1754e6959dc0dfef9da47cb701018aaf457e8d42cfc5db6a37ade94ef9df10220083359151180a80e82a54bd98bfc750f21fdbbdb08e24aebcf1cba7536858cfb0121022340c6019b5bd42bc28c85d88fb5880403eedb701f10b65eaf90e0a4634d199100000000"
            },
            {
                "addressNList": [2147483692,2147483648,2147483650,0,0],
                "scriptType": "p2pkh",
                "amount": "20000",
                "vout": 1,
                "txid": "a47f60ff416f17cc9b0543f8afe05ae8bad98c9c2c69c207ecc0d50799dc52f0",
                "hex": "0200000000010192b0bf35840f7846fa0fbad2586bc2dd81d1ae2d82f1691e8726bb49fd11648f0100000000fdffffff0490960000000000001600143b944081068951cc1b040818ad95ee1146cbef63204e0000000000001976a914ff655cc26f2ec3d0e34030bdcf5182e95907a17488acbb4f00000000000017a914d57d7d91f1843c5ab88c9aa48a183363bda3a5cc8706fc020000000000160014b66b3f985645441b52cc64151816f8af697516cd02483045022100d7f1754e6959dc0dfef9da47cb701018aaf457e8d42cfc5db6a37ade94ef9df10220083359151180a80e82a54bd98bfc750f21fdbbdb08e24aebcf1cba7536858cfb0121022340c6019b5bd42bc28c85d88fb5880403eedb701f10b65eaf90e0a4634d199100000000"
            },
            {
                "addressNList": [2147483697,2147483648,2147483648,0,0],
                "scriptType": "p2sh-p2wpkh",
                "amount": "20411",
                "vout": 2,
                "txid": "a47f60ff416f17cc9b0543f8afe05ae8bad98c9c2c69c207ecc0d50799dc52f0",
                "hex": "0200000000010192b0bf35840f7846fa0fbad2586bc2dd81d1ae2d82f1691e8726bb49fd11648f0100000000fdffffff0490960000000000001600143b944081068951cc1b040818ad95ee1146cbef63204e0000000000001976a914ff655cc26f2ec3d0e34030bdcf5182e95907a17488acbb4f00000000000017a914d57d7d91f1843c5ab88c9aa48a183363bda3a5cc8706fc020000000000160014b66b3f985645441b52cc64151816f8af697516cd02483045022100d7f1754e6959dc0dfef9da47cb701018aaf457e8d42cfc5db6a37ade94ef9df10220083359151180a80e82a54bd98bfc750f21fdbbdb08e24aebcf1cba7536858cfb0121022340c6019b5bd42bc28c85d88fb5880403eedb701f10b65eaf90e0a4634d199100000000"
            }
        ],
        "outputs": [
            {
                "address": "bc1qu3ghkz8788ysk7gqcvke5l0mr7skhgvpuk6dk4",
                "amount": "73099",
                "addressType": "spend"
            }
        ],
        "version": 1,
        "locktime": 0
    });
    let resp = app.clone().oneshot(
        Request::post("/sign-tx")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "sign-tx failed for mock input");
    let body = to_bytes(resp.into_body(), 1024*1024).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(parsed.get("signature").is_some(), "Signature missing in sign-tx response");
}
