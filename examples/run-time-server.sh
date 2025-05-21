#!/bin/bash
set -e

# Change to the project root directory
cd "$(dirname "$0")/.."

# Build the finch-mcp tool
echo "Building finch-mcp-stdio..."
cargo build

# Define the timezone to use (default to UTC if not provided)
TIMEZONE=${1:-UTC}
echo "Using timezone: $TIMEZONE"

# Run the MCP time server with finch-mcp-stdio
# This command will:
# 1. Detect that we're running a 'uvx' command
# 2. Create a container with the appropriate Python environment
# 3. Install the mcp-server-time package
# 4. Run the server in STDIO mode

echo "Starting MCP time server in timezone $TIMEZONE..."
./target/debug/finch-mcp-stdio uvx mcp-server-time \
  --local-timezone "$TIMEZONE" \
  -e "MCP_ENABLED=true" \
  -e "MCP_STDIO=true"

# Note: You don't actually need to specify MCP_ENABLED and MCP_STDIO
# because the auto-containerization will set these for you, but
# they're included here for clarity.

# The server should be running now and waiting for input.
# You can try sending some MCP requests to it like:
#
# {"name": "get_current_time", "arguments": {"timezone": "Europe/Warsaw"}}
#
# Or
#
# {"name": "convert_time", "arguments": {"source_timezone": "America/New_York", "time": "16:30", "target_timezone": "Asia/Tokyo"}}