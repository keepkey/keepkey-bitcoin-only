#!/bin/bash

# Start Local Pioneer Server for Testing
# Usage: ./scripts/start-pioneer-local.sh

echo "🚀 Starting Local Pioneer Server..."
echo "==================================="

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Check if port 9001 is already in use
if lsof -i :9001 > /dev/null 2>&1; then
    echo "⚠️  Port 9001 is already in use."
    echo "Checking what's running on port 9001..."
    lsof -i :9001
    echo ""
    echo "Would you like to stop the existing service? (y/N)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        echo "Stopping services on port 9001..."
        lsof -ti :9001 | xargs kill -9
        sleep 2
    else
        echo "Exiting. Please stop the service manually or use a different port."
        exit 1
    fi
fi

# Try to pull latest Pioneer image
echo "📦 Pulling latest Pioneer image..."
if ! docker pull pioneer:latest 2>/dev/null; then
    echo "⚠️  Could not pull pioneer:latest image."
    echo "This is expected if you're using a local build."
    echo "Continuing with existing local image..."
fi

# Start Pioneer container
echo "🏃 Starting Pioneer container on port 9001..."
CONTAINER_ID=$(docker run -d -p 9001:9001 pioneer:latest)

if [ $? -eq 0 ]; then
    echo "✅ Pioneer container started successfully!"
    echo "   Container ID: $CONTAINER_ID"
    echo "   Local URL: http://127.0.0.1:9001"
    echo ""
    
    # Wait for service to be ready
    echo "⏳ Waiting for Pioneer service to be ready..."
    for i in {1..30}; do
        if curl -s http://127.0.0.1:9001/health > /dev/null 2>&1; then
            echo "✅ Pioneer service is ready!"
            echo ""
            echo "📋 PIONEER SERVER INFO:"
            echo "   Health: http://127.0.0.1:9001/health"
            echo "   API Docs: http://127.0.0.1:9001/docs"
            echo "   Fee Rates: http://127.0.0.1:9001/api/v1/GetFeeRate/bip122%3A000000000019d6689c085ae165831e93"
            echo ""
            echo "🧪 RUN TESTS:"
            echo "   node tests/test-pioneer-only.js"
            echo "   node tests/test-pioneer-live.js (after starting vault)"
            echo ""
            echo "🛑 TO STOP:"
            echo "   docker stop $CONTAINER_ID"
            echo "   docker rm $CONTAINER_ID"
            exit 0
        fi
        echo "   Attempt $i/30: Service not ready yet..."
        sleep 2
    done
    
    echo "❌ Pioneer service failed to start properly."
    echo "Checking container logs..."
    docker logs "$CONTAINER_ID"
    echo ""
    echo "Stopping container..."
    docker stop "$CONTAINER_ID"
    docker rm "$CONTAINER_ID"
    exit 1
else
    echo "❌ Failed to start Pioneer container."
    echo "Check Docker status and try again."
    exit 1
fi 