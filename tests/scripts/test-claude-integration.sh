#!/bin/bash

# This script simulates how Claude Code might interact with our Rust tool
# Instead of using npx or uvx, it directly uses the compiled binary

echo "Claude Code MCP Integration Test"
echo "-------------------------------"

# The prompt/query to send to the MCP server
PROMPT='{"query": "What is the capital of France?"}'

# Use our Rust binary to send the prompt to the MCP server
echo $PROMPT | ./target/debug/finch-mcp-stdio test-mcp-stdio:latest

echo ""
echo "Test completed!"