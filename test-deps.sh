#!/bin/bash

echo "🧪 Testing 32-bit dependencies installation..."

# Update package lists
sudo apt-get update

# Enable multiarch support for 32-bit packages
echo "📦 Adding i386 architecture..."
sudo dpkg --add-architecture i386
sudo apt-get update

# Install 32-bit dependencies
echo "📦 Installing 32-bit OpenSSL and multilib packages..."
sudo apt-get install -y \
  gcc-multilib \
  g++-multilib \
  libc6-dev-i386 \
  libssl-dev:i386 \
  libssl3:i386 \
  pkg-config \
  libusb-1.0-0-dev \
  libudev-dev \
  protobuf-compiler \
  libhidapi-dev \
  libhidapi-hidraw0 \
  libhidapi-libusb0 \
  libssl-dev

echo "✅ Dependencies installed successfully!"

# Verify 32-bit OpenSSL is available
echo "🔍 Verifying 32-bit OpenSSL installation..."
if dpkg -l | grep -q "libssl-dev:i386"; then
  echo "✅ libssl-dev:i386 is installed"
else
  echo "❌ libssl-dev:i386 is NOT installed"
  exit 1
fi

if dpkg -l | grep -q "libssl3:i386"; then
  echo "✅ libssl3:i386 is installed"
else
  echo "❌ libssl3:i386 is NOT installed"
  exit 1
fi

echo "✅ All 32-bit dependencies verified successfully!" 