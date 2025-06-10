#!/bin/bash

# Fix KeepKey Device Context Issue

echo "🔧 KeepKey Device Context Fix Script"
echo "===================================="

# Device information from logs
DEVICE_ID="343737340F4736331F003B00"
ETH_ADDRESS="0x141d9959cae3853b035000490c03991eb70fc4ac"

echo "📱 Device ID: $DEVICE_ID"
echo "💰 ETH Address: $ETH_ADDRESS"
echo ""

# Check if server is running
echo "🔍 Checking if server is running on port 1646..."
if lsof -i :1646 > /dev/null 2>&1; then
    echo "✅ Server is running!"
    
    # Set device context
    echo ""
    echo "📡 Setting device context..."
    curl -X POST http://localhost:1646/api/context \
        -H "Content-Type: application/json" \
        -d "{\"device_id\": \"$DEVICE_ID\", \"eth_address\": \"$ETH_ADDRESS\", \"label\": null}" \
        -w "\n"
    
    if [ $? -eq 0 ]; then
        echo "✅ Device context set successfully!"
        
        # Verify context was set
        echo ""
        echo "🔍 Verifying context..."
        curl -s http://localhost:1646/api/context | jq .
    else
        echo "❌ Failed to set device context"
    fi
else
    echo "❌ Server is not running on port 1646"
    echo ""
    echo "To start the application, run:"
    echo "  cd projects/vault"
    echo "  npm run tauri dev"
    echo ""
    echo "Then run this script again."
fi

echo ""
echo "📝 Additional troubleshooting:"
echo "  1. Make sure your KeepKey is connected via USB"
echo "  2. Try unplugging and reconnecting the device"
echo "  3. Check device status: curl http://localhost:1646/api/devices"
echo "  4. View API docs: http://localhost:1646/docs" 