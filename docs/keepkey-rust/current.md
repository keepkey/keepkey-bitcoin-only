# Vault v1 — File catalogue (projects/vault/src-tauri)

The table below lists **every first-party source file** that lives inside `projects/vault/src-tauri`. 3rd-party vendored code under `deps/` and generated assets under `gen/`, `icons/`, `capabilities/`, `target/` are omitted.

> Sizes shown are from the working tree when this document was generated.

## Root
- `.gitignore` – ignores build artefacts and IDE clutter.
- `Cargo.toml` (2 KB) – workspace manifest; enables `vault_server`, `tauri`, `hid`, `usb` feature flags.
- `Cargo.lock` (163 KB) – exact crate versions.
- `build.rs` (1.5 KB) – builds protobufs, embeds icons.
- `tauri.conf.json` (664 B) – desktop-app settings for Tauri.

## src/
- `lib.rs` (43 KB) – crate root; sets up panic hooks, tracing, and re-exports all sub-modules.
- `main.rs` (186 B) – minimal Tauri entry point.
- `blocking_actions.rs` (7.2 KB) – helper to off-load CPU-heavy tasks to a Rayon thread-pool.
- `commands.rs` (119 KB) – **huge** list of `#[tauri::command]` IPC handlers wrapping device / cache logic.
- `default-paths.json` (3.3 KB) – OS-specific data paths.
- `device_controller.rs` (19 KB) – high-level, synchronous façade over a `DeviceQueue` (sign, get_xpub, etc.).
- `device_controller_ext.rs` (2.7 KB) – macro blanket impls so controller methods can be called from JS.
- `device_queue.rs` (19 KB) – per-device inbox/outbox, runs an actor thread and handles reconnect and faults.
- `device_registry.rs` (5.5 KB) – global HashMap<serial, DeviceHandle>, emits events on hot-plug.
- `device_update.rs` (12.9 KB) – DFU / firmware-update orchestrator.
- `error.rs` (1.3 KB) – common error enum.
- `features/mod.rs` (16.8 KB) – compile-time coin-support gates (`kk_only`, `litecoin`, …).
- `index_db.rs` (27 KB) – sqlite address gap-scan indexer.
- `updates.rs` (24 KB) – bundled firmware release metadata & fetch helpers.
- `usb_manager.rs` (33 KB) – libusb hot-plug monitor; creates/destroys transports.
- `utils.rs` (2 KB) – misc helpers.
- `vault.rs` (6.4 KB) – high-level API exported to Tauri / REST.

### src/cache/
- `mod.rs` (9.4 KB) – builder & error types.
- `device_cache.rs` (58 KB) – rusqlite wrapper around `schema.sql`; persists device metadata.
- `frontload.rs` (96 KB) – background task that walks every supported coin path to pre-fetch XPUBs.
- `schema.sql` (5.4 KB) – DB schema & migrations.

### src/messages/
- `encoding.rs` (2.1 KB) – protobuf encode/decode helpers.
- `macros.rs` (2.3 KB) – proc-macros for message generation.
- `mod.rs` (3 KB) – re-exports & message traits.
- `protos.rs` (94 B) – includes generated protobufs.
- `timeouts.rs` (2.2 KB) – per-command recommended timeouts.

### src/server/
- `bitcoin.rs` (12 KB) – bitcoin-specific helpers & newtype conversions.
- `context.rs` (20 KB) – `AppContext` (registry + cache) shared across routes.
- `mod.rs` (8 KB) – feature gates; chooses Axum vs Tide.
- `routes.rs` (27 KB) – registers all versioned REST endpoints.

#### src/server/routes/
- `auth.rs` (2.5 KB) – rudimentary JWT login.
- `impl_addresses.rs` (5.8 KB) – address-index endpoints.
- `impl_bitcoin.rs` (61 KB) – bitcoin Tx, PSBT, fee, and signing endpoints.
- `impl_device.rs` (9.1 KB) – device management endpoints (wipe, backup, etc.).
- `impl_system.rs` (28 KB) – system and settings endpoints.
- `v2_db.rs` (655 B) – placeholder for new DB layer.
- `v2_endpoints.rs` (88 KB) – WIP second-gen REST surface.

### src/transport/
- `mod.rs` (10 KB) – `Transport` trait and helpers.
- `usb.rs` (7.6 KB) – libusb bulk transport implementation.
- `hid.rs` (12 KB) – hidapi transport fallback.
- `protocol_adapter.rs` (1.8 KB) – wraps protobuf envelopes into packets.

--------------------------------------------------------------------------------
Generated on 2025-06-16.  Total first-party source files listed: **53**.
