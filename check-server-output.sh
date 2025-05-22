#!/bin/bash

echo "Checking what the MCP server outputs when started..."

# Start the server and capture its output for 10 seconds
{
    ./target/debug/finch-mcp-stdio uvx mcp-server-time --local-timezone UTC &
    SERVER_PID=$!
    
    # Wait for the server to start and capture output
    sleep 5
    
    # Send it an empty line to see what happens
    echo "" | nc localhost 8080 2>/dev/null || echo "No response from server"
    
    # Kill the server
    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
} 2>&1 | head -20

echo "Server output captured."