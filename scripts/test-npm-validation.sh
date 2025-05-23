#!/bin/bash

# Quick npm MCP validation test
# Tests npm local functionality using Inspector

set -e

echo "🔍 npm MCP Validation Test"
echo "=========================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PROJECT_PATH="tests/fixtures/test-mcp-filesystem"
FINCH_MCP_PATH="./target/release/finch-mcp"

echo -e "${BLUE}🧪 Test 1: Direct npm MCP Server${NC}"
echo "================================="

cd "$PROJECT_PATH"
echo "Testing: npx -y @modelcontextprotocol/server-filesystem /tmp"
echo ""

# Test tools/list
echo -n "Testing tools/list... "
if npx @modelcontextprotocol/inspector --cli npx -y @modelcontextprotocol/server-filesystem /tmp --method tools/list > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
    DIRECT_TOOLS_PASS=true
else
    echo -e "${RED}❌ FAIL${NC}"
    DIRECT_TOOLS_PASS=false
fi

# Test tool call
echo -n "Testing tool call... "
if npx @modelcontextprotocol/inspector --cli npx -y @modelcontextprotocol/server-filesystem /tmp --method tools/call --tool-name list_allowed_directories > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
    DIRECT_TOOL_CALL_PASS=true
else
    echo -e "${RED}❌ FAIL${NC}"
    DIRECT_TOOL_CALL_PASS=false
fi

cd - > /dev/null
echo ""

echo -e "${BLUE}🐳 Test 2: finch-mcp with npm project${NC}"
echo "===================================="

# Test finch-mcp containerization
echo -n "Testing finch-mcp containerization... "
if echo '{}' | "$FINCH_MCP_PATH" run "$PROJECT_PATH" 2>&1 | grep -q "Starting MCP server"; then
    echo -e "${GREEN}✅ PASS${NC}"
    FINCH_CONTAINER_PASS=true
else
    echo -e "${RED}❌ FAIL${NC}"
    FINCH_CONTAINER_PASS=false
fi

echo ""

echo -e "${BLUE}📊 Summary${NC}"
echo "=========="
echo -n "Direct npm MCP server tools: "
if $DIRECT_TOOLS_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "Direct npm MCP server tool calls: "
if $DIRECT_TOOL_CALL_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "finch-mcp npm containerization: "
if $FINCH_CONTAINER_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo ""

if $DIRECT_TOOLS_PASS && $DIRECT_TOOL_CALL_PASS && $FINCH_CONTAINER_PASS; then
    echo -e "${GREEN}🎉 All npm functionality tests passed!${NC}"
    echo ""
    echo "✅ npm MCP servers work correctly with Inspector"
    echo "✅ finch-mcp can containerize npm projects" 
    echo "✅ npm local functionality is validated"
    exit 0
else
    echo -e "${RED}💥 Some tests failed.${NC}"
    exit 1
fi