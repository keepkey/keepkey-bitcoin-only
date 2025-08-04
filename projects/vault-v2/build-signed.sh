#!/bin/bash

# KeepKey Vault v2 - Local Signed Build Script
# This script builds and signs the Tauri app for macOS

set -e

echo "🔐 KeepKey Vault v2 - Signed Build Script"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SIGNING_IDENTITY="Developer ID Application: KEY HODLERS LLC (DR57X8Z394)"
APPLE_ID="${APPLE_ID:-}" # Set via environment variable
APPLE_PASSWORD="${APPLE_PASSWORD:-}" # App-specific password
TEAM_ID="DR57X8Z394"

echo "📋 Build Configuration:"
echo "  - Signing Identity: $SIGNING_IDENTITY"
echo "  - Team ID: $TEAM_ID"
echo ""

# Check for signing identity
echo "🔍 Checking signing identity..."
if security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
    echo -e "${GREEN}✓ Signing identity found${NC}"
else
    echo -e "${RED}✗ Signing identity not found!${NC}"
    echo "Available identities:"
    security find-identity -v -p codesigning
    exit 1
fi

# Check for Rust and Tauri
echo ""
echo "🔍 Checking dependencies..."

if ! command -v rustc &> /dev/null; then
    echo -e "${RED}✗ Rust is not installed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Rust found: $(rustc --version)${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Cargo is not installed${NC}"
    exit 1
fi

if ! command -v bun &> /dev/null; then
    echo -e "${RED}✗ Bun is not installed${NC}"
    echo "Install with: curl -fsSL https://bun.sh/install | bash"
    exit 1
fi
echo -e "${GREEN}✓ Bun found: $(bun --version)${NC}"

# Install dependencies
echo ""
echo "📦 Installing dependencies..."
bun install

# Build the app
echo ""
echo "🔨 Building Tauri app..."
echo "This may take several minutes..."

# Set environment variables for signing
export TAURI_SIGNING_IDENTITY="$SIGNING_IDENTITY"
export TAURI_APPLE_TEAM_ID="$TEAM_ID"

# Build for universal macOS (both Intel and Apple Silicon)
bun tauri build --target universal-apple-darwin

echo ""
echo -e "${GREEN}✅ Build completed successfully!${NC}"
echo ""

# Show build output location
DMG_PATH="src-tauri/target/universal-apple-darwin/release/bundle/dmg"
APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos"

echo "📦 Build outputs:"
if [ -d "$DMG_PATH" ]; then
    echo "  - DMG files: $DMG_PATH"
    ls -la "$DMG_PATH"/*.dmg 2>/dev/null || echo "    No DMG files found"
fi

if [ -d "$APP_PATH" ]; then
    echo "  - App bundles: $APP_PATH"
    ls -la "$APP_PATH"/*.app 2>/dev/null || echo "    No app bundles found"
fi

# Verify signature
echo ""
echo "🔍 Verifying signatures..."
for app in "$APP_PATH"/*.app; do
    if [ -d "$app" ]; then
        echo "Checking: $(basename "$app")"
        if codesign -dv --verbose=4 "$app" 2>&1 | grep -q "$TEAM_ID"; then
            echo -e "${GREEN}✓ Signature valid${NC}"
        else
            echo -e "${YELLOW}⚠ Signature verification needs review${NC}"
        fi
    fi
done

# Optional: Notarization (requires Apple ID credentials)
if [ -n "$APPLE_ID" ] && [ -n "$APPLE_PASSWORD" ]; then
    echo ""
    echo "📝 Starting notarization process..."
    echo "This may take 5-10 minutes..."
    
    for dmg in "$DMG_PATH"/*.dmg; do
        if [ -f "$dmg" ]; then
            echo "Notarizing: $(basename "$dmg")"
            
            # Submit for notarization
            xcrun notarytool submit "$dmg" \
                --apple-id "$APPLE_ID" \
                --password "$APPLE_PASSWORD" \
                --team-id "$TEAM_ID" \
                --wait
            
            # Staple the notarization ticket
            xcrun stapler staple "$dmg"
            echo -e "${GREEN}✓ Notarization complete for $(basename "$dmg")${NC}"
        fi
    done
else
    echo ""
    echo -e "${YELLOW}ℹ️  Notarization skipped (no Apple ID credentials provided)${NC}"
    echo "To enable notarization, set these environment variables:"
    echo "  export APPLE_ID='your-apple-id@example.com'"
    echo "  export APPLE_PASSWORD='your-app-specific-password'"
fi

echo ""
echo "🎉 Build and signing complete!"
echo ""
echo "Next steps:"
echo "1. Test the signed app from: $APP_PATH"
echo "2. Distribute the DMG from: $DMG_PATH"
echo "3. If notarization is needed, run with APPLE_ID and APPLE_PASSWORD set"