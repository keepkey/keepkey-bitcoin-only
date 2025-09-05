#!/bin/bash

# Build script for creating Windows .exe using Docker
set -e

echo "🏗️  Building KeepKey Vault v2 for Windows..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist-windows
mkdir -p dist-windows

# Optional: Set Tauri signing keys (for code signing)
if [ -f ".env" ]; then
    echo "📝 Loading environment variables..."
    export $(cat .env | grep -v '^#' | xargs)
fi

# Build with Docker
echo "🐳 Starting Docker build..."
echo "This will take several minutes on first run..."

# Build the Docker image and run it
docker-compose -f docker-compose.windows.yml --progress=plain build
docker-compose -f docker-compose.windows.yml up

# Check if build was successful
if [ -f "dist-windows/vault-v2.exe" ]; then
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo ""
    echo "📦 Built files:"
    ls -lh dist-windows/
    
    # Calculate file sizes
    if [ -f "dist-windows/vault-v2.exe" ]; then
        SIZE=$(du -h "dist-windows/vault-v2.exe" | cut -f1)
        echo -e "${GREEN}   vault-v2.exe: $SIZE${NC}"
    fi
    
    if [ -d "dist-windows/installer" ] && [ "$(ls -A dist-windows/installer)" ]; then
        echo ""
        echo "📦 Installer files:"
        ls -lh dist-windows/installer/
    fi
else
    echo -e "${RED}❌ Build failed. Check the Docker logs above for errors.${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Windows build complete!${NC}"
echo "Files are available in: ./dist-windows/"
echo ""
echo "To test the .exe in Wine (Linux/macOS):"
echo "  wine dist-windows/vault-v2.exe"
echo ""
echo "To copy to a Windows machine, use the files in ./dist-windows/"