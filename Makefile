# KeepKey Bitcoin-Only Vault Build System
#
# Main targets:
#   make vault        - Build and run vault-v2 in development mode (with keepkey-rust)
#   make vault-build  - Build vault-v2 for production 
#   make vault-dev    - Quick development build (skips dependency checks)
#   make keepkey-rust - Build keepkey-rust library only
#   make clean        - Clean all build artifacts
#   make rebuild      - Clean and rebuild everything
#   make check-deps   - Verify keepkey-rust dependency linking
#   make test-keepkey-rust - Run keepkey-rust tests
#
# Dependencies:
#   - Rust/Cargo (for keepkey-rust and Tauri backend)
#   - Bun (for frontend dependencies)
#   - jq (for dependency verification)

# Platform detection
ifeq ($(OS),Windows_NT)
    DETECTED_OS := Windows
    # Check if we're in PowerShell or WSL
    ifdef WSLENV
        DETECTED_OS := WSL
    endif
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        DETECTED_OS := Linux
    endif
    ifeq ($(UNAME_S),Darwin)
        DETECTED_OS := macOS
    endif
endif

.PHONY: all firmware kkcli rest vault-ui vault test test-rest clean keepkey-rust vault-build rebuild check-deps help platform-info

# Display help information
help:
	@echo "KeepKey Bitcoin-Only Vault Build System"
	@echo ""
	@echo "Detected Platform: $(DETECTED_OS)"
	@echo ""
	@echo "Main targets:"
	@echo "  vault         - Build and run vault-v2 in development mode (with keepkey-rust)"
	@echo "  vault-build   - Build vault-v2 for production"
	@echo "  vault-dev     - Quick development build (skips dependency checks)"
	@echo "  keepkey-rust  - Build keepkey-rust library only"
	@echo "  clean         - Clean all build artifacts"
	@echo "  rebuild       - Clean and rebuild everything"
	@echo "  check-deps    - Verify keepkey-rust dependency linking"
	@echo "  test-keepkey-rust - Run keepkey-rust tests"
	@echo "  platform-info - Show detected platform information"
	@echo ""
	@echo "Dependencies:"
	@echo "  - Rust/Cargo (for keepkey-rust and Tauri backend)"
	@echo "  - Bun (for frontend dependencies)"
	@echo "  - jq (for dependency verification - Unix only)"

# Show platform information
platform-info:
	@echo "🖥️  Platform Detection:"
	@echo "   Detected OS: $(DETECTED_OS)"
ifeq ($(DETECTED_OS),Windows)
	@echo "   Build Method: PowerShell script (skills/build.ps1)"
	@echo "   Make Alternative: Consider using 'powershell -ExecutionPolicy Bypass -File skills/build.ps1'"
else
	@echo "   Build Method: Traditional Makefile"
endif

all: firmware kkcli rest vault-ui vault

test:
	cargo test --manifest-path projects/keepkey-rest/Cargo.toml --all-features

# Test keepkey-rust specifically
test-keepkey-rust:
	@echo "🧪 Testing keepkey-rust library..."
	cd projects/keepkey-rust && cargo test --all-features
	@echo "✅ keepkey-rust tests passed"

firmware:
	$(MAKE) -C firmware

kkcli:
	cd projects/kkcli && cargo build && target/debug/kkcli server

# Build keepkey-rust separately to ensure it's up to date
keepkey-rust:
	@echo "🔧 Building keepkey-rust library..."
	cd projects/keepkey-rust && cargo check --all-features
	cd projects/keepkey-rust && cargo build --release
	@echo "✅ keepkey-rust built successfully"

# Check dependencies and linking - Platform aware
check-deps:
	@echo "🔍 Checking keepkey-rust dependency in vault-v2..."
ifeq ($(DETECTED_OS),Windows)
	@cd projects/vault-v2/src-tauri && cargo tree | findstr /C:"keepkey_rust" || echo "⚠️  keepkey-rust not found in dependency tree"
	@echo "🔍 Verifying path dependency (Windows - basic check)..."
	@cd projects/vault-v2/src-tauri && cargo metadata --format-version 1 > metadata.json && type metadata.json | findstr /C:"keepkey_rust" > nul && echo "✅ keepkey-rust dependency found" || echo "⚠️  Could not verify keepkey-rust path"
	@del projects/vault-v2/src-tauri/metadata.json 2>nul || echo ""
else
	@cd projects/vault-v2/src-tauri && cargo tree | grep -E "(keepkey_rust|keepkey-rust)" || echo "⚠️  keepkey-rust not found in dependency tree"
	@echo "🔍 Verifying path dependency..."
	@cd projects/vault-v2/src-tauri && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "keepkey_rust") | .manifest_path' || echo "⚠️  Could not verify keepkey-rust path"
endif

# Platform-aware vault target
vault: keepkey-rust check-deps
ifeq ($(DETECTED_OS),Windows)
	@echo "🖥️  Windows detected - using PowerShell build script..."
	@echo "🔧 Building vault-v2 with latest keepkey-rust (Windows)..."
	@cd projects/vault-v2 && powershell -ExecutionPolicy Bypass -File "../../skills/build.ps1" -Debug
else
	@echo "🔧 Building vault-v2 with latest keepkey-rust (Unix)..."
	cd projects/vault-v2 && bun i && bun run tauri:dev
endif

# Platform-aware vault-build target
vault-build: keepkey-rust check-deps
ifeq ($(DETECTED_OS),Windows)
	@echo "🖥️  Windows detected - using PowerShell build script..."
	@echo "🔧 Building vault-v2 for production with latest keepkey-rust (Windows)..."
	@cd projects/vault-v2 && powershell -ExecutionPolicy Bypass -File "../../skills/build.ps1"
else
	@echo "🔧 Building vault-v2 for production with latest keepkey-rust (Unix)..."
	cd projects/vault-v2 && bun i && bun run tauri:build
endif

# Platform-aware clean target
clean:
	@echo "🧹 Cleaning all build artifacts..."
	cd projects/keepkey-rust && cargo clean
	cd projects/vault-v2 && cargo clean
ifeq ($(DETECTED_OS),Windows)
	@if exist "projects\vault-v2\node_modules" rmdir /s /q "projects\vault-v2\node_modules"
	@if exist "projects\vault-v2\dist" rmdir /s /q "projects\vault-v2\dist"
	@if exist "projects\vault-v2\src-tauri\target" rmdir /s /q "projects\vault-v2\src-tauri\target"
else
	rm -rf projects/vault-v2/node_modules
	rm -rf projects/vault-v2/dist
	rm -rf projects/vault-v2/src-tauri/target
endif
	@echo "✅ All build artifacts cleaned"

# Force rebuild everything
rebuild: clean all

# Quick development build (skips some checks) - Platform aware
vault-dev:
ifeq ($(DETECTED_OS),Windows)
	@echo "🚀 Quick vault-v2 development build (Windows)..."
	@cd projects/vault-v2 && powershell -ExecutionPolicy Bypass -File "../../skills/build.ps1" -Debug
else
	@echo "🚀 Quick vault-v2 development build (Unix)..."
	cd projects/vault-v2 && bun i && bun run tauri:dev
endif
