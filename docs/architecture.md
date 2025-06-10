# Architecture Overview

## Monorepo Structure

- `firmware/`: Bitcoin-only KeepKey firmware
- `kkcli/`: CLI for KeepKey Bitcoin-only operations
- `keepkey-rest/`: Shared REST API crate (Rust, used by both kkcli and Vault)
- `vault-ui/`: Vite-based UI, shared between kkcli (web) and Vault (Tauri)
- `vault/`: Tauri desktop app

## Shared REST API
- All endpoint handlers and business logic are in `keepkey-rest`.
- Both `kkcli` and `vault` depend on this crate and mount its router.

## Shared UI
- The Vite UI is developed in `vault-ui`.
- Built bundle is served by kkcli (via REST API) and imported into Vault (Tauri frontend).

## Build/Deploy Flow
- Single Makefile for unified builds.
- Each project can be built/tested independently or all at once.
