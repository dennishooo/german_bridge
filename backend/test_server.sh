#!/bin/bash

# Test script to verify server starts correctly
echo "Testing server startup..."

# Start the server in the background
cargo run --manifest-path backend/Cargo.toml &
SERVER_PID=$!

# Wait a moment for server to start
sleep 2

# Test health endpoint
echo "Testing health endpoint..."
curl -s http://localhost:8080/health

# Kill the server
kill $SERVER_PID 2>/dev/null

echo ""
echo "Server test complete!"
