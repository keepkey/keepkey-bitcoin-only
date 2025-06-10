# KeepKey Monorepo

This monorepo contains all core components for the KeepKey Bitcoin-only ecosystem:

- **firmware/**: Bitcoin-only firmware for KeepKey hardware wallets.
- **kkcli/**: Command-line interface for Bitcoin-only KeepKey operations.
- **keepkey-rest/**: Shared REST API crate (Rust) used by both kkcli and Vault.
- **vault-ui/**: Vite-based UI, shared between kkcli (web) and Vault (Tauri).
- **vault/**: Tauri desktop app for KeepKey.

## Development

### Prerequisites

- Rust (latest stable)
- Node.js (for `vault-ui`)
- Tauri prerequisites (see [docs/tauri.md](docs/tauri.md))
- GNU Make

### Build Everything

```sh
make all
```

### Build Individual Projects

```sh
make firmware
make kkcli
make rest
make vault-ui
make vault
```

## Project Goals

- **Single-source REST API**: All business logic and endpoints live in `keepkey-rest`.
- **Unified UI**: One Vite UI bundle for both web and desktop.
- **Easy builds**: One command to build everything.

## Contributing

See [docs/contributing.md](docs/contributing.md).

## Architecture

See [docs/architecture.md](docs/architecture.md).
 Bitcoin Only Stack




