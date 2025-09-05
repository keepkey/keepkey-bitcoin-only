#!/bin/bash

# Fast Windows build script using simplified Docker
set -e

echo "🚀 Fast Windows .exe build (simplified Docker)"

# Check Docker
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Clean and create output dir
rm -rf dist-windows
mkdir -p dist-windows

echo "🏗️  Building Windows executable..."
echo "This may take 5-10 minutes on first run..."

# Build and extract in one command
docker build \
    -f Dockerfile.windows-simple \
    -t vault-windows-builder \
    --build-arg BUILDKIT_PROGRESS=plain \
    ../.. 

# Run container to copy files
docker run --rm \
    -v "$(pwd)/dist-windows:/host-output" \
    vault-windows-builder \
    sh -c "cp -v /output/* /host-output/ 2>/dev/null || echo 'Build may have failed'"

# Check result
if [ -f "dist-windows/vault-v2.exe" ]; then
    echo "✅ Build successful!"
    echo "📦 Output:"
    ls -lh dist-windows/
else
    echo "❌ Build failed. Checking logs..."
    docker run --rm vault-windows-builder ls -la /workspace/projects/vault-v2/src-tauri/target/
fi