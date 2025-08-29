#!/bin/bash

# Simple test script for the reverse proxy functionality

echo "Starting test backend server on port 8080..."
# Start a simple HTTP server on port 8080 to act as the backend
python3 -m http.server 8080 &
BACKEND_PID=$!

# Give the backend server time to start
sleep 2

echo "Starting OpenSOVD gateway with proxy..."
# Start the OpenSOVD gateway
cargo run --bin osovd-gateway -- --bind 127.0.0.1:7690 &
GATEWAY_PID=$!

# Give the gateway time to start
sleep 3

echo "Testing proxy endpoint..."
# Test the proxy
echo "GET /proxy/ (should proxy to http://localhost:8080/):"
curl -i http://localhost:7690/proxy/

echo ""
echo "Testing proxy with path:"
curl -i http://localhost:7690/proxy/README.md

# Cleanup
echo ""
echo "Cleaning up..."
kill $BACKEND_PID $GATEWAY_PID 2>/dev/null
wait $BACKEND_PID $GATEWAY_PID 2>/dev/null

echo "Test complete!"