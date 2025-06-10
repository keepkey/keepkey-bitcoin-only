/*fn main() {
    protobuf_codegen::Codegen::new()
        // Use pure Rust parser.
        .protoc()
        .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
        // All inputs and imports from the inputs must reside in `includes` directories.
        .includes(&[
            protoc_bin_vendored::include_path().unwrap().to_str().unwrap(),
            "deps/device-protocol",
        ])
        // Inputs must reside in some of include paths.
        .input("deps/device-protocol/messages-binance.proto")
        .input("deps/device-protocol/messages-cosmos.proto")
        .input("deps/device-protocol/messages-eos.proto")
        .input("deps/device-protocol/messages-nano.proto")
        .input("deps/device-protocol/messages-osmosis.proto")
        .input("deps/device-protocol/messages-ripple.proto")
        .input("deps/device-protocol/messages-tendermint.proto")
        .input("deps/device-protocol/messages-thorchain.proto")
        .input("deps/device-protocol/messages.proto")
        // Specify output directory relative to Cargo output directory.
        .cargo_out_dir("device-protocol")
        .run_from_script();
}*/

fn main() -> std::io::Result<()> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    std::env::set_var(
        "PROTOC_INCLUDE",
        protoc_bin_vendored::include_path().unwrap(),
    );
    // config.
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
    config.compile_protos(
        &[
            "deps/device-protocol/types.proto",
            "deps/device-protocol/messages.proto",
            "deps/device-protocol/messages-binance.proto",
            "deps/device-protocol/messages-cosmos.proto",
            "deps/device-protocol/messages-eos.proto",
            "deps/device-protocol/messages-ethereum.proto",
            "deps/device-protocol/messages-mayachain.proto",
            "deps/device-protocol/messages-nano.proto",
            "deps/device-protocol/messages-osmosis.proto",
            "deps/device-protocol/messages-ripple.proto",
            "deps/device-protocol/messages-tendermint.proto",
            "deps/device-protocol/messages-thorchain.proto",
        ],
        &["deps/device-protocol/"],
    )?;
    Ok(())
}
