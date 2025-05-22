#!/bin/bash

echo "Testing Claude Code with filesystem MCP server..."

# Start Claude Code with the filesystem configuration in the background
echo "Starting Claude Code with filesystem server..."

# Send a simple command to test the filesystem tools and then exit
echo "Please read the contents of the README.md file" | timeout 30 claude -p claude-filesystem-test.json --dangerously-skip-permissions

echo "Test completed."