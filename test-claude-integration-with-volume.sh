#!/bin/bash

# This script tests using volume mounts with the MCP server

echo "Claude Code MCP Integration Test with Volume Mounts"
echo "-------------------------------"

# Get absolute path to the test volume directory
VOLUME_DIR="$(cd "$(dirname "$0")/test-volume" && pwd)"

# The prompt/query to send to the MCP server
PROMPT='{"query": "Access file in mounted volume"}'

# Use our Rust binary with volume mount
echo $PROMPT | ./target/debug/finch-mcp-stdio test-mcp-stdio:latest -v "$VOLUME_DIR:/data"

echo ""
echo "Test completed!"