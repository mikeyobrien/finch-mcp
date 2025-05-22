#!/bin/bash
set -e

# Build the Docker image for the MCP time server
echo "Building Docker image for MCP time server..."
cd "$(dirname "$0")"
docker build -t mcp-time-server .

# Define example timezone - replace with desired timezone
TIMEZONE=${1:-UTC}
echo "Using timezone: $TIMEZONE"

# Run the time server with finch-mcp
echo "Running MCP time server with finch-mcp..."
# Uncomment below line when the Rust implementation is actually used
# cargo run -- mcp-time-server -e "LOCAL_TIMEZONE=$TIMEZONE"

# For now, use direct finch command to demonstrate the concept
finch run --rm -i \
  -e MCP_ENABLED=true \
  -e MCP_STDIO=true \
  -e "LOCAL_TIMEZONE=$TIMEZONE" \
  mcp-time-server