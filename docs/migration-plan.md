# Migration Plan: REST API & UI Extraction

## 1. Extract REST API
- Create `keepkey-rest` Rust crate
- Move all endpoint handlers, models, and logic from `kkcli/src/server/` to `keepkey-rest/src/lib.rs`
- Expose `pub fn create_router(...) -> axum::Router`
- Update `kkcli` and `vault` to use this crate

## 2. Extract UI
- Create `vault-ui` Vite project
- Move all UI code there
- Build outputs static bundle
- kkcli: serve bundle as static files
- vault: import bundle into Tauri frontend

## 3. Monorepo Setup
- Add Makefile for unified builds
- Document structure and process in README and docs

## 4. Future Steps
- Add integration tests for REST API
- Add e2e tests for UI
