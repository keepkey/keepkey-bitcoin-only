# KeepKey Monorepo

This monorepo contains all core components for the KeepKey Bitcoin-only ecosystem:

- **firmware/**: Bitcoin-only firmware for KeepKey hardware wallets.
- **kkcli/**: Command-line interface for Bitcoin-only KeepKey operations.
- **vault/**: Tauri desktop app for KeepKey.
- **vault-v2/**: Next-generation Tauri desktop app with clean architecture.

## Quick Start

### Windows Users (Recommended)
```cmd
# Simply double-click or run from command prompt:
vault.bat
```
**That's it!** The batch file handles all the complexity for you.

### Unix/Linux/macOS/WSL Users
```sh
make vault
```

## Development

### Prerequisites

- Rust (latest stable)
- Tauri prerequisites (see [docs/tauri.md](docs/tauri.md))
- Bun (preferred) or npm
- Platform-specific build tools:
  - **Unix/Linux/macOS**: GNU Make
  - **Windows**: PowerShell (built-in) or WSL with make

### Cross-Platform Build

The build system automatically detects your platform and uses the appropriate method:

#### Windows (Native PowerShell)
```cmd
# Method 1: Batch file (RECOMMENDED for Windows users)
vault.bat

# Method 2: Direct PowerShell (advanced users)
powershell -ExecutionPolicy Bypass -File "skills/build.ps1" -Debug

# Method 3: Clean build
powershell -ExecutionPolicy Bypass -File "skills/build.ps1" -Clean -Debug
```

**Why use vault.bat?**
- ✅ Automatically handles PowerShell execution policy
- ✅ Provides clear error messages and troubleshooting
- ✅ Works from any command prompt (CMD, PowerShell, Git Bash)
- ✅ Pauses for user input so you can see results
- ✅ Checks prerequisites and provides helpful error messages

#### Unix/Linux/macOS/WSL
```sh
make vault          # Build and run vault-v2 in development mode
make vault-build    # Build vault-v2 for production
make platform-info  # Show detected platform information
```

### Build Individual Projects

```sh
make firmware       # Build firmware
make kkcli         # Build command-line interface
make keepkey-rust  # Build core Rust library
make vault         # Build and run vault-v2
make clean         # Clean all build artifacts
```

### Platform-Specific Notes

#### Windows
- **PRIMARY METHOD**: Use `vault.bat` for the best Windows experience
- The build system automatically uses PowerShell script (`skills/build.ps1`) when Windows is detected
- WSL users can use the standard Unix commands (`make vault`)
- Git Bash users can use either method

#### Windows Troubleshooting
If `vault.bat` fails:
1. Ensure PowerShell is available (it's built into Windows 10/11)
2. Try running as Administrator if you get permission errors
3. Check that Rust and Bun/npm are installed and in your PATH
4. The script will show detailed error messages to guide you

#### All Platforms
- Dependencies are automatically built in the correct order
- The `keepkey-rust` library is built first as required by vault-v2
- Bun is preferred over npm for faster builds

## Project Goals

- **Single-source REST API**: All business logic and endpoints live in `keepkey-rest`.
- **Unified UI**: One Vite UI bundle for both web and desktop.
- **Easy builds**: One command to build everything, regardless of platform.
- **Clean Architecture**: Clear separation between UI, business logic, and hardware communication.

## Contributing

See [docs/contributing.md](docs/contributing.md).

## Architecture

See [docs/architecture.md](docs/architecture/architecture.md).

Bitcoin Only Stack




