#!/bin/bash

# Test script for firmware verification in OOB setup flow
# This script verifies that firmware verification cannot be skipped

echo "========================================="
echo "Firmware Verification Test Script"
echo "========================================="
echo ""
echo "This script tests the following scenarios:"
echo "1. Device with outdated firmware CANNOT skip to initialization"
echo "2. Device must be on latest firmware (7.10.0) to proceed"
echo "3. Verification step shows proper status indicators"
echo "4. Skip button is removed/disabled for security"
echo ""
echo "Expected behavior:"
echo "- Firmware step shows 'Verifying Firmware' with loading indicator"
echo "- Verification takes at least 1.5 seconds (forced delay)"
echo "- If not on latest version, user sees 'Update Required' badge"
echo "- Only 'Verify & Continue' button available after verification"
echo "- If versions don't match, error message is shown"
echo ""
echo "========================================="
echo "Manual Test Steps:"
echo "========================================="
echo ""
echo "1. Connect a KeepKey device with older firmware (< 7.10.0)"
echo "2. Start the vault application"
echo "3. Observe the Setup Wizard appears"
echo "4. Check that Step 2 is 'Check Firmware'"
echo "5. Verify that:"
echo "   - Loading screen shows 'Verifying Firmware'"
echo "   - 'This verification is mandatory for security' message appears"
echo "   - After verification, if not on 7.10.0:"
echo "     - 'Update Required' badge is shown"
echo "     - 'Update Firmware Now (Required)' button appears"
echo "     - NO skip button is available"
echo "   - If on 7.10.0:"
echo "     - '✅ Firmware verified' message appears"
echo "     - Auto-proceeds after 2 seconds"
echo ""
echo "6. Try to bypass verification (should fail):"
echo "   - Cannot proceed without updating to latest firmware"
echo "   - 'Verify & Continue' button shows error if versions don't match"
echo ""
echo "========================================="
echo "Console Logs to Watch For:"
echo "========================================="
echo ""
echo "Look for these log messages in browser console:"
echo "- '🔒 FIRMWARE VERIFICATION:' with version details"
echo "- '✅ Firmware verified at latest version' (if on 7.10.0)"
echo "- '⚠️ Backend says no update needed but versions don't match!' (mismatch warning)"
echo "- 'Cannot skip firmware update - not on latest version' (skip prevention)"
echo ""
echo "========================================="
echo "Building and Running the Application..."
echo "========================================="
echo ""

# Navigate to the vault-v2 directory
cd "$(dirname "$0")"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# Build and run the application
echo "Starting the application..."
echo "Please follow the manual test steps above."
echo ""
npm run tauri dev