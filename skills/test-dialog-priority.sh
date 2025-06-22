#!/bin/bash

echo "🧪 Testing Dialog Priority Handling"
echo "=================================="
echo ""
echo "This test verifies that:"
echo "1. Invalid state dialog shows when device times out"
echo "2. Invalid state dialog auto-closes when device is unplugged"
echo "3. PIN dialog takes priority over invalid state dialog"
echo "4. No overlapping dialogs occur"
echo ""

# Test endpoint
API_URL="http://127.0.0.1:1646"

echo "📋 Step 1: Getting current device list..."
curl -s "${API_URL}/api/devices" | jq '.'

echo ""
echo "📋 Step 2: Simulating device timeout (invalid state)..."
echo "This would normally be triggered by the device not responding"
echo ""

echo "⚡ Expected behavior:"
echo "  - Invalid state dialog should appear"
echo "  - Dialog should show reconnection instructions"
echo "  - Dialog should auto-close when device is unplugged"
echo "  - If PIN is needed after reconnect, PIN dialog should replace invalid state dialog"
echo ""

echo "✅ Test setup complete. Please:"
echo "  1. Wait for your device to timeout (or simulate an invalid state)"
echo "  2. Verify the invalid state dialog appears"
echo "  3. Unplug your device - dialog should auto-close"
echo "  4. Reconnect device - if PIN needed, it should show without overlap"
echo ""
echo "Press Ctrl+C to exit when done testing"

# Keep script running to observe behavior
while true; do
    sleep 1
done 