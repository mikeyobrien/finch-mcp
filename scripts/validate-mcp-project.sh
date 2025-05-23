#!/bin/bash

# MCP Project Validation Script using @modelcontextprotocol/inspector
# This script validates that finch-mcp can properly containerize and run MCP servers

set -e

PROJECT_PATH=${1:-"tests/fixtures/test-mcp-time"}
TIMEOUT=${2:-30}
FINCH_MCP_PATH=${3:-"./target/release/finch-mcp"}

echo "🔍 MCP Project Validation"
echo "========================="
echo "Project: $PROJECT_PATH"
echo "Timeout: ${TIMEOUT}s"
echo "finch-mcp: $FINCH_MCP_PATH"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run Inspector with timeout
run_inspector_with_timeout() {
    local method=$1
    local extra_args=$2
    local output_file=$(mktemp)
    
    echo -n "Testing $method... "
    
    # Run Inspector (without timeout for now, will implement proper timeout later)
    if npx @modelcontextprotocol/inspector --cli \
        "$FINCH_MCP_PATH" run "$PROJECT_PATH" \
        --method "$method" $extra_args > "$output_file" 2>&1; then
        echo -e "${GREEN}✅ PASS${NC}"
        if [[ "$method" == "tools/list" || "$method" == "resources/list" || "$method" == "prompts/list" ]]; then
            # Show available items
            local items=$(cat "$output_file" | jq -r ".$method | length" 2>/dev/null || echo "0")
            if [[ "$items" != "0" && "$items" != "null" ]]; then
                echo "   Found $items items"
            fi
        fi
        rm "$output_file"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        echo "   Error output:"
        sed 's/^/   /' "$output_file"
        rm "$output_file"
        return 1
    fi
}

# Check if finch-mcp binary exists
if [[ ! -f "$FINCH_MCP_PATH" ]]; then
    echo -e "${RED}❌ finch-mcp binary not found at: $FINCH_MCP_PATH${NC}"
    echo "   Build with: cargo build --release"
    exit 1
fi

# Check if project exists
if [[ ! -d "$PROJECT_PATH" ]]; then
    echo -e "${RED}❌ Project path not found: $PROJECT_PATH${NC}"
    exit 1
fi

echo -e "${BLUE}🚀 Starting MCP validation tests...${NC}"
echo ""

# Test 1: Basic server connectivity
echo "Test 1: Server Connectivity"
echo "----------------------------"
if run_inspector_with_timeout "resources/list"; then
    CONNECTIVITY_PASS=true
else
    CONNECTIVITY_PASS=false
fi
echo ""

# Test 2: Tools availability
echo "Test 2: Tools Discovery"
echo "-----------------------"
if run_inspector_with_timeout "tools/list"; then
    TOOLS_PASS=true
else
    TOOLS_PASS=false
fi
echo ""

# Test 3: Prompts availability  
echo "Test 3: Prompts Discovery"
echo "-------------------------"
if run_inspector_with_timeout "prompts/list"; then
    PROMPTS_PASS=true
else
    PROMPTS_PASS=false
fi
echo ""

# Test 4: Resource reading (if resources exist)
echo "Test 4: Resource Access"
echo "-----------------------"
# First get list of resources
RESOURCES_OUTPUT=$(mktemp)
if npx @modelcontextprotocol/inspector --cli \
    "$FINCH_MCP_PATH" run "$PROJECT_PATH" \
    --method "resources/list" > "$RESOURCES_OUTPUT" 2>/dev/null; then
    
    # Try to read first resource if any exist
    FIRST_RESOURCE=$(cat "$RESOURCES_OUTPUT" | jq -r '.resources[0].uri' 2>/dev/null || echo "null")
    if [[ "$FIRST_RESOURCE" != "null" && "$FIRST_RESOURCE" != "" ]]; then
        echo -n "Reading resource: $FIRST_RESOURCE... "
        if npx @modelcontextprotocol/inspector --cli \
            "$FINCH_MCP_PATH" run "$PROJECT_PATH" \
            --method "resources/read" --resource-uri "$FIRST_RESOURCE" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ PASS${NC}"
            RESOURCE_ACCESS_PASS=true
        else
            echo -e "${RED}❌ FAIL${NC}"
            RESOURCE_ACCESS_PASS=false
        fi
    else
        echo -e "${YELLOW}⚠️  No resources to test${NC}"
        RESOURCE_ACCESS_PASS=true
    fi
else
    echo -e "${RED}❌ Could not list resources${NC}"
    RESOURCE_ACCESS_PASS=false
fi
rm "$RESOURCES_OUTPUT"
echo ""

# Summary
echo "📊 Validation Summary"
echo "====================="
echo -n "Server Connectivity: "
if $CONNECTIVITY_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "Tools Discovery: "
if $TOOLS_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "Prompts Discovery: "
if $PROMPTS_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "Resource Access: "
if $RESOURCE_ACCESS_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo ""

# Overall result
if $CONNECTIVITY_PASS && $TOOLS_PASS && $PROMPTS_PASS && $RESOURCE_ACCESS_PASS; then
    echo -e "${GREEN}🎉 All tests passed! MCP server is working correctly.${NC}"
    exit 0
else
    echo -e "${RED}💥 Some tests failed. Check the output above for details.${NC}"
    exit 1
fi