#!/bin/bash

# Quick test script to verify signing setup

echo "🔐 KeepKey Vault v2 - Signing Setup Test"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

SIGNING_IDENTITY="Developer ID Application: KEY HODLERS LLC (DR57X8Z394)"
TEAM_ID="DR57X8Z394"

echo "1️⃣  Checking signing identity..."
if security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
    echo -e "${GREEN}✅ Signing identity found${NC}"
    echo "   Identity: $SIGNING_IDENTITY"
else
    echo -e "${RED}❌ Signing identity not found${NC}"
    echo "Available identities:"
    security find-identity -v -p codesigning | grep "Developer ID"
    exit 1
fi

echo ""
echo "2️⃣  Checking Tauri configuration..."
if [ -f "src-tauri/tauri.conf.json" ]; then
    if grep -q "KEY HODLERS LLC" src-tauri/tauri.conf.json; then
        echo -e "${GREEN}✅ Tauri config has signing identity${NC}"
    else
        echo -e "${RED}❌ Tauri config missing signing identity${NC}"
    fi
else
    echo -e "${RED}❌ tauri.conf.json not found${NC}"
fi

echo ""
echo "3️⃣  Checking entitlements..."
if [ -f "src-tauri/entitlements.plist" ]; then
    echo -e "${GREEN}✅ Entitlements file found${NC}"
    echo "   USB access: $(grep -q "com.apple.security.device.usb" src-tauri/entitlements.plist && echo "✓" || echo "✗")"
    echo "   Network access: $(grep -q "com.apple.security.network.client" src-tauri/entitlements.plist && echo "✓" || echo "✗")"
else
    echo -e "${RED}❌ entitlements.plist not found${NC}"
fi

echo ""
echo "4️⃣  Checking build tools..."
echo -n "   Rust: "
if command -v rustc &> /dev/null; then
    echo -e "${GREEN}✅ $(rustc --version)${NC}"
else
    echo -e "${RED}❌ Not installed${NC}"
fi

echo -n "   Bun: "
if command -v bun &> /dev/null; then
    echo -e "${GREEN}✅ $(bun --version)${NC}"
else
    echo -e "${RED}❌ Not installed${NC}"
fi

echo -n "   Cargo: "
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✅ $(cargo --version)${NC}"
else
    echo -e "${RED}❌ Not installed${NC}"
fi

echo ""
echo "5️⃣  Testing codesign..."
# Create a test executable
echo '#!/bin/bash' > /tmp/test-sign.sh
echo 'echo "test"' >> /tmp/test-sign.sh
chmod +x /tmp/test-sign.sh

if codesign -s "$SIGNING_IDENTITY" /tmp/test-sign.sh 2>/dev/null; then
    echo -e "${GREEN}✅ Codesign test successful${NC}"
    codesign -dv /tmp/test-sign.sh 2>&1 | grep "TeamIdentifier=$TEAM_ID" > /dev/null && \
        echo "   Team ID verified: $TEAM_ID" || \
        echo -e "${YELLOW}⚠️  Team ID mismatch${NC}"
else
    echo -e "${RED}❌ Codesign test failed${NC}"
fi
rm -f /tmp/test-sign.sh

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Summary:"
echo ""
echo "To build a signed version, run:"
echo -e "${GREEN}./build-signed.sh${NC}"
echo ""
echo "For notarization, set environment variables:"
echo "export APPLE_ID='your-apple-id@example.com'"
echo "export APPLE_PASSWORD='your-app-specific-password'"
echo ""
echo "Then run the build script again."