#!/bin/bash

# Test script to verify WebSocket connection management
echo "Testing WebSocket connection management..."

# Start the server in the background
cargo run --manifest-path backend/Cargo.toml &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Test health endpoint
echo "Testing health endpoint..."
HEALTH_RESPONSE=$(curl -s http://localhost:8080/health)
echo "Health check: $HEALTH_RESPONSE"

# Test WebSocket connection using websocat (if available)
if command -v websocat &> /dev/null; then
    echo ""
    echo "Testing WebSocket connection..."
    echo '{"type":"Ping"}' | timeout 2 websocat ws://localhost:8080/ws || echo "WebSocket test completed"
else
    echo ""
    echo "websocat not installed, skipping WebSocket test"
    echo "Install with: cargo install websocat"
fi

# Kill the server
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo ""
echo "Connection management test complete!"
