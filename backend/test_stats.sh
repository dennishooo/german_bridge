#!/bin/bash

# Test the /stats endpoint

echo "Testing /stats endpoint..."
echo ""

# Make a request to the stats endpoint
curl -s http://localhost:8080/stats | jq .

echo ""
echo "Stats endpoint test complete"
