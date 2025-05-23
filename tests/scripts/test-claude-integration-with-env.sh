#!/bin/bash

# This script tests using environment variables with the MCP server

echo "Claude Code MCP Integration Test with Environment Variables"
echo "-------------------------------"

# The prompt/query to send to the MCP server
PROMPT='{"query": "Tell me about the API key you received."}'

# Use our Rust binary with environment variables
echo $PROMPT | ./target/debug/finch-mcp-stdio test-mcp-stdio:latest -e API_KEY=test12345 -e DEBUG=true

echo ""
echo "Test completed!"