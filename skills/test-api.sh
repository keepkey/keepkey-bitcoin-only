#!/bin/bash

# Test KeepKey API endpoints

echo "🧪 Testing KeepKey API"
echo "====================="

# Check if server is running
if ! lsof -i :1646 > /dev/null 2>&1; then
    echo "❌ Server is not running on port 1646"
    echo "Please start the application first:"
    echo "  cd projects/vault"
    echo "  npm run tauri dev"
    exit 1
fi

echo "✅ Server is running on port 1646"
echo ""

# Test 1: Check device status
echo "1️⃣ Checking device status..."
curl -s http://localhost:1646/api/status | jq . || echo "Failed to get device status"
echo ""

# Test 2: List devices
echo "2️⃣ Listing devices..."
curl -s http://localhost:1646/api/devices | jq . || echo "Failed to list devices"
echo ""

# Test 3: Check context
echo "3️⃣ Checking current context..."
CONTEXT=$(curl -s http://localhost:1646/api/context)
echo "$CONTEXT" | jq .

# Check if context is set
if echo "$CONTEXT" | jq -e '.context != null' > /dev/null 2>&1; then
    echo "✅ Context is set!"
    echo ""
    
    # Test 4: Try to get a Bitcoin address
    echo "4️⃣ Testing Bitcoin address generation..."
    echo "Getting Legacy Bitcoin address at m/49'/0'/0'/0/0..."
    
    curl -X POST http://localhost:1646/addresses/utxo \
        -H "Content-Type: application/json" \
        -d '{
            "coin": "Bitcoin",
            "script_type": "p2pkh",
            "address_n": [2147483697, 2147483648, 2147483648, 0, 0],
            "show_display": false
        }' | jq .
else
    echo "❌ No context set! Run ./fix-device-context.sh first"
fi

echo ""
echo "📝 For more endpoints, visit: http://localhost:1646/docs" 