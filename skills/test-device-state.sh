#!/bin/bash

echo "🔍 Testing Device State and Events..."
echo "=================================="

# Check if device is connected
echo "1. Checking connected devices..."
curl -s http://127.0.0.1:1646/api/devices | jq '.'

echo -e "\n2. Getting device features..."
# First, get the device path from the devices list
DEVICE_PATH=$(curl -s http://127.0.0.1:1646/api/devices | jq -r '.[0].path')

if [ -z "$DEVICE_PATH" ] || [ "$DEVICE_PATH" = "null" ]; then
    echo "❌ No device found!"
    exit 1
fi

echo "Device path: $DEVICE_PATH"

# Get features
curl -X POST http://127.0.0.1:1646/system/info/get-features \
  -H "Content-Type: application/json" \
  -d "{\"device_path\": \"$DEVICE_PATH\"}" | jq '.'

echo -e "\n3. Checking server health..."
curl -s http://127.0.0.1:1646/api/health | jq '.'

echo -e "\n✅ Test complete!"
echo "=================================="
echo "If device is connected and features show initialized=true and bootloader_mode=false,"
echo "then the device should be ready for the VaultInterface." 