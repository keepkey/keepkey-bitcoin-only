use std::path::PathBuf;

fn main() {
    // Reuse the protocol definitions vendored in the original vault project.
    let proto_dir: PathBuf = ["..", "vault", "src-tauri", "deps", "device-protocol"].iter().collect();

    // Collect all *.proto files in the directory.
    let protos: Vec<PathBuf> = std::fs::read_dir(&proto_dir)
        .expect("read proto dir")
        .filter_map(|e| {
            let p = e.ok()?.path();
            if p.extension().and_then(|s| s.to_str()) == Some("proto") {
                Some(p)
            } else {
                None
            }
        })
        .collect();

    prost_build::Config::new()
        .compile_protos(&protos, &[proto_dir])
        .expect("compile protos");
}
