# KeepKey Monorepo

This monorepo contains all core components for the KeepKey Bitcoin-only ecosystem:

- **firmware/**: Bitcoin-only firmware for KeepKey hardware wallets.
- **kkcli/**: Command-line interface for Bitcoin-only KeepKey operations.
- **vault/**: Tauri desktop app for KeepKey.

## Development

### Prerequisites

- Rust (latest stable)
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
make vault
```

## Project Goals

- **Single-source REST API**: All business logic and endpoints live in `keepkey-rest`.
- **Unified UI**: One Vite UI bundle for both web and desktop.
- **Easy builds**: One command to build everything.

## Release Process

### Creating a Release

The project uses a Makefile-based release workflow for versioning and branch management:

#### Quick Release Commands

```bash
# Create release branch with specific version
make release-branch VERSION=2.2.7

# Auto-increment versions and create branch
make release-patch-branch  # Bumps X.Y.Z -> X.Y.(Z+1)
make release-minor-branch  # Bumps X.Y.Z -> X.(Y+1).0
make release-major-branch  # Bumps X.Y.Z -> (X+1).0.0
```

#### Manual Release Steps

1. **Create release branch and update version:**
   ```bash
   make release-branch VERSION=2.2.7
   ```

2. **Build and test locally with signing:**
   ```bash
   cd projects/vault-v2
   ./build-signed.sh
   ```

3. **Commit changes:**
   ```bash
   git add .
   git commit -m "chore: release v2.2.7"
   ```

4. **Push branch:**
   ```bash
   git push -u origin release-2.2.7
   ```

5. **Create Pull Request to master**

6. **After merge, create and push tag:**
   ```bash
   git tag -a v2.2.7 -m "Release v2.2.7"
   git push --tags
   ```

### CI/CD Signing

The GitHub Actions workflow automatically signs and notarizes macOS builds when the following repository secrets are configured:

- `APPLE_ID` - Apple developer account email
- `APPLE_ID_PASSWORD` - App-specific password for notarization
- `APPLE_TEAM_ID` - Apple developer team ID (e.g., DR57X8Z394)
- `APPLE_CERTIFICATE` - Base64-encoded Developer ID certificate
- `CERT_PW` - Certificate password

### Local Signing (macOS)

For local signed builds:

```bash
cd projects/vault-v2

# Build and sign
./build-signed.sh

# Notarize (requires Apple credentials)
APPLE_ID="your-email@example.com" \
APPLE_PASSWORD="app-specific-password" \
./notarize.sh
```

## Contributing

See [docs/contributing.md](docs/contributing.md).

## Architecture

See [docs/architecture.md](docs/architecture.md).




