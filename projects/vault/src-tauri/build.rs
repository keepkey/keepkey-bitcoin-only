fn main() -> std::io::Result<()> {
    // Set up protoc environment variables
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    std::env::set_var(
        "PROTOC_INCLUDE",
        protoc_bin_vendored::include_path().unwrap(),
    );
    
    // Configure protobuf generation
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[::serde_with::serde_as]");
    config.type_attribute(".", "#[::serde_with::skip_serializing_none]");
    config.type_attribute(".", "#[derive(::serde::Serialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    config.field_attribute(
        ".CoinType.contract_address",
        "#[serde_as(as = \"Option<::serde_with::hex::Hex>\")]",
    );
    config.btree_map(["."]);  
    
    // Compile the protos
    config.compile_protos(
        &[
            "deps/device-protocol/messages-binance.proto",
            "deps/device-protocol/messages-cosmos.proto",
            "deps/device-protocol/messages-eos.proto",
            "deps/device-protocol/messages-nano.proto",
            "deps/device-protocol/messages-osmosis.proto",
            "deps/device-protocol/messages-ripple.proto",
            "deps/device-protocol/messages-tendermint.proto",
            "deps/device-protocol/messages-thorchain.proto",
            "deps/device-protocol/messages.proto",
        ],
        &["deps/device-protocol/"],
    )?;
    
    // Build the Tauri app
    tauri_build::build();
    
    Ok(())
}
