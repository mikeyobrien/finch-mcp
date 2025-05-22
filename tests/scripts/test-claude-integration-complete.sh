#!/bin/bash

# This script tests a complete integration with all features

echo "Claude Code Complete MCP Integration Test"
echo "-------------------------------"

# Get absolute path to the test volume directory
VOLUME_DIR="$(cd "$(dirname "$0")/test-volume" && pwd)"

# The prompt/query to send to the MCP server
PROMPT='{"query": "Complete integration test"}'

# Use our Rust binary with all features
echo $PROMPT | ./target/debug/finch-mcp-stdio test-mcp-stdio:latest \
  -e API_KEY=test12345 \
  -e DEBUG=true \
  -v "$VOLUME_DIR:/data" \
  -V

echo ""
echo "Test completed!"