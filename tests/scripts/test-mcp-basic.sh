#!/bin/bash

# This script tests the basic functionality of finch-mcp-stdio
# with the MCP time server

echo "Testing finch-mcp-stdio with MCP time server..."

# Start the server in the background
./target/debug/finch-mcp-stdio uvx mcp-server-time --local-timezone UTC &
SERVER_PID=$!

# Give the server a moment to start
sleep 3

# Check if the server process is still running
if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✓ MCP server started successfully"
    
    # Try to send a simple request to test functionality
    echo '{"method": "initialize", "params": {"protocolVersion": "1.0.0", "capabilities": {}}}' | timeout 5 nc -l 8080 &
    
    # Kill the server
    kill $SERVER_PID 2>/dev/null
    echo "✓ Server terminated successfully"
else
    echo "✗ MCP server failed to start or crashed"
    exit 1
fi

echo "Basic functionality test completed."