#!/bin/bash

# KeepKey Vault v2 - Notarization Helper Script
# This script helps with Apple notarization after building

set -e

echo "🍎 KeepKey Vault v2 - Notarization Helper"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
TEAM_ID="DR57X8Z394"
BUNDLE_ID="com.vault-v2.app"

# Check for credentials
if [ -z "$APPLE_ID" ]; then
    echo -e "${RED}❌ APPLE_ID environment variable not set${NC}"
    echo ""
    echo "Please set your Apple ID:"
    echo "  export APPLE_ID='your-apple-id@example.com'"
    exit 1
fi

if [ -z "$APPLE_PASSWORD" ]; then
    echo -e "${RED}❌ APPLE_PASSWORD environment variable not set${NC}"
    echo ""
    echo "Please set your app-specific password:"
    echo "  export APPLE_PASSWORD='your-app-specific-password'"
    echo ""
    echo "To create an app-specific password:"
    echo "  1. Go to https://appleid.apple.com/account/manage"
    echo "  2. Sign in and go to Security"
    echo "  3. Under App-Specific Passwords, click Generate Password"
    echo "  4. Use 'KeepKey Vault Notarization' as the label"
    exit 1
fi

echo -e "${GREEN}✅ Credentials configured${NC}"
echo "  Apple ID: $APPLE_ID"
echo "  Team ID: $TEAM_ID"
echo ""

# Find DMG files
DMG_PATH="target/universal-apple-darwin/release/bundle/dmg"
APP_PATH="target/universal-apple-darwin/release/bundle/macos"

if [ ! -d "$DMG_PATH" ]; then
    DMG_PATH="src-tauri/target/universal-apple-darwin/release/bundle/dmg"
    APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos"
fi

if [ ! -d "$DMG_PATH" ]; then
    DMG_PATH="src-tauri/target/release/bundle/dmg"
    APP_PATH="src-tauri/target/release/bundle/macos"
fi

echo "🔍 Looking for built artifacts..."
echo ""

# Function to notarize a file
notarize_file() {
    local FILE="$1"
    local FILENAME=$(basename "$FILE")
    
    echo -e "${BLUE}📝 Notarizing: $FILENAME${NC}"
    echo "This may take 5-10 minutes..."
    
    # Submit for notarization
    echo "Submitting to Apple..."
    SUBMISSION_ID=$(xcrun notarytool submit "$FILE" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_PASSWORD" \
        --team-id "$TEAM_ID" \
        --output-format json \
        --wait 2>&1 | tee /tmp/notarization.log | grep -o '"id":"[^"]*"' | cut -d'"' -f4 | head -1)
    
    if [ -z "$SUBMISSION_ID" ]; then
        echo -e "${RED}❌ Failed to submit for notarization${NC}"
        echo "Check /tmp/notarization.log for details"
        return 1
    fi
    
    echo "Submission ID: $SUBMISSION_ID"
    
    # Check status
    echo "Checking notarization status..."
    xcrun notarytool info "$SUBMISSION_ID" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_PASSWORD" \
        --team-id "$TEAM_ID"
    
    # Get the log if needed
    echo ""
    echo "Getting notarization log..."
    xcrun notarytool log "$SUBMISSION_ID" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_PASSWORD" \
        --team-id "$TEAM_ID" \
        /tmp/notarization-log.json
    
    # Check if successful
    if grep -q '"status": "Accepted"' /tmp/notarization-log.json 2>/dev/null || grep -q '"status":"Accepted"' /tmp/notarization-log.json 2>/dev/null; then
        echo -e "${GREEN}✅ Notarization successful!${NC}"
        
        # Staple the ticket
        echo "Stapling notarization ticket..."
        if [[ "$FILE" == *.dmg ]]; then
            xcrun stapler staple "$FILE"
            echo -e "${GREEN}✅ Ticket stapled to $FILENAME${NC}"
        elif [[ "$FILE" == *.app ]]; then
            xcrun stapler staple "$FILE"
            echo -e "${GREEN}✅ Ticket stapled to $FILENAME${NC}"
        fi
        
        # Verify
        echo "Verifying..."
        xcrun stapler validate "$FILE"
        
        return 0
    else
        echo -e "${YELLOW}⚠️  Notarization may have issues${NC}"
        echo "Check the log at: /tmp/notarization-log.json"
        return 1
    fi
}

# Process DMG files
DMG_COUNT=0
if [ -d "$DMG_PATH" ]; then
    for dmg in "$DMG_PATH"/*.dmg; do
        if [ -f "$dmg" ]; then
            DMG_COUNT=$((DMG_COUNT + 1))
            notarize_file "$dmg"
            echo ""
        fi
    done
fi

# Process App bundles (if no DMG)
if [ $DMG_COUNT -eq 0 ] && [ -d "$APP_PATH" ]; then
    echo -e "${YELLOW}No DMG files found, notarizing app bundles...${NC}"
    for app in "$APP_PATH"/*.app; do
        if [ -d "$app" ]; then
            # Create a temporary DMG for notarization
            APP_NAME=$(basename "$app" .app)
            TEMP_DMG="/tmp/${APP_NAME}-temp.dmg"
            
            echo "Creating temporary DMG for notarization..."
            hdiutil create -volname "$APP_NAME" -srcfolder "$app" -ov -format UDBZ "$TEMP_DMG"
            
            notarize_file "$TEMP_DMG"
            
            # Staple to the original app
            echo "Stapling to original app..."
            xcrun stapler staple "$app"
            
            rm -f "$TEMP_DMG"
            echo ""
        fi
    done
fi

if [ $DMG_COUNT -eq 0 ] && [ ! -d "$APP_PATH" ]; then
    echo -e "${RED}❌ No DMG files or app bundles found${NC}"
    echo ""
    echo "Please build the app first:"
    echo "  ./build-signed.sh"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}🎉 Notarization process complete!${NC}"
echo ""
echo "Your app is now ready for distribution."
echo ""
echo "To verify notarization status:"
echo "  xcrun stapler validate <your-file.dmg>"
echo ""
echo "To check notarization history:"
echo "  xcrun notarytool history --apple-id $APPLE_ID --team-id $TEAM_ID"