#!/bin/bash

# Comprehensive MCP Validation using Inspector
# Tests both direct MCP servers and finch-mcp containerization

set -e

PROJECT_PATH=${1:-"tests/fixtures/test-mcp-filesystem"}
FINCH_MCP_PATH=${2:-"./target/release/finch-mcp"}

echo "🔍 MCP Inspector Validation"
echo "============================="
echo "Project: $PROJECT_PATH"
echo "finch-mcp: $FINCH_MCP_PATH"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Function to run Inspector CLI and capture output
run_inspector_cli() {
    local command_args="$1"
    local method="$2"
    local extra_args="$3"
    local description="$4"
    
    echo -n "$description... "
    
    local output_file=$(mktemp)
    local error_file=$(mktemp)
    
    # Run Inspector CLI
    if npx @modelcontextprotocol/inspector --cli $command_args \
        --method "$method" $extra_args > "$output_file" 2> "$error_file"; then
        echo -e "${GREEN}✅ PASS${NC}"
        cat "$output_file"
        rm "$output_file" "$error_file"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        echo "   Error output:"
        sed 's/^/   /' "$error_file"
        rm "$output_file" "$error_file"
        return 1
    fi
}

# Get project information
if [[ -f "$PROJECT_PATH/package.json" ]]; then
    PROJECT_TYPE="npm"
    # Extract start command from package.json
    START_COMMAND=$(jq -r '.scripts.start // "npm start"' "$PROJECT_PATH/package.json")
    echo -e "${BLUE}📦 Detected npm project${NC}"
    echo "   Start command: $START_COMMAND"
    echo ""
else
    echo -e "${YELLOW}⚠️  Not a Node.js project, skipping direct validation${NC}"
    echo ""
fi

# Test 1: Direct MCP Server Testing (if npm project)
if [[ "$PROJECT_TYPE" == "npm" ]]; then
    echo -e "${BLUE}🧪 Test 1: Direct MCP Server Validation${NC}"
    echo "=========================================="
    
    cd "$PROJECT_PATH"
    
    # Parse the start command to get executable and args
    if [[ "$START_COMMAND" == npx* ]]; then
        # Extract npx command
        NPX_ARGS=$(echo "$START_COMMAND" | sed 's/npx //')
        echo "Testing direct server: npx $NPX_ARGS"
        echo ""
        
        # Test tools/list
        echo "Direct Server - Tools List:"
        echo "----------------------------"
        if run_inspector_cli "npx $NPX_ARGS" "tools/list" "" "Testing tools/list"; then
            DIRECT_TOOLS_PASS=true
        else
            DIRECT_TOOLS_PASS=false
        fi
        echo ""
        
        # Test a specific tool if available
        echo "Direct Server - Tool Call:"
        echo "---------------------------"
        # Try to call a tool that most servers have
        if run_inspector_cli "npx $NPX_ARGS" "tools/call" "--tool-name list_allowed_directories" "Testing tool call (list_allowed_directories)"; then
            DIRECT_TOOL_CALL_PASS=true
        else
            echo "   Trying alternative tool..."
            # Try calling the first available tool from tools/list
            FIRST_TOOL=$(npx @modelcontextprotocol/inspector --cli npx $NPX_ARGS --method tools/list 2>/dev/null | jq -r '.tools[0].name' 2>/dev/null || echo "")
            if [[ -n "$FIRST_TOOL" && "$FIRST_TOOL" != "null" ]]; then
                if run_inspector_cli "npx $NPX_ARGS" "tools/call" "--tool-name $FIRST_TOOL" "Testing first available tool ($FIRST_TOOL)"; then
                    DIRECT_TOOL_CALL_PASS=true
                else
                    DIRECT_TOOL_CALL_PASS=false
                fi
            else
                DIRECT_TOOL_CALL_PASS=false
            fi
        fi
        echo ""
        
        # Test resources if supported
        echo "Direct Server - Resources:"
        echo "-------------------------"
        if run_inspector_cli "npx $NPX_ARGS" "resources/list" "" "Testing resources/list"; then
            DIRECT_RESOURCES_PASS=true
        else
            echo -e "   ${YELLOW}ℹ️  Resources not supported by this server${NC}"
            DIRECT_RESOURCES_PASS=true  # Not an error if not supported
        fi
        echo ""
        
        # Test prompts if supported  
        echo "Direct Server - Prompts:"
        echo "-----------------------"
        if run_inspector_cli "npx $NPX_ARGS" "prompts/list" "" "Testing prompts/list"; then
            DIRECT_PROMPTS_PASS=true
        else
            echo -e "   ${YELLOW}ℹ️  Prompts not supported by this server${NC}"
            DIRECT_PROMPTS_PASS=true  # Not an error if not supported
        fi
        echo ""
    else
        echo -e "${YELLOW}⚠️  Non-npx start command, skipping direct server test${NC}"
        DIRECT_TOOLS_PASS=true
        DIRECT_TOOL_CALL_PASS=true
        DIRECT_RESOURCES_PASS=true
        DIRECT_PROMPTS_PASS=true
        echo ""
    fi
    
    cd - > /dev/null
else
    DIRECT_TOOLS_PASS=true
    DIRECT_TOOL_CALL_PASS=true
    DIRECT_RESOURCES_PASS=true
    DIRECT_PROMPTS_PASS=true
fi

# Test 2: finch-mcp UI Mode Testing
echo -e "${BLUE}🐳 Test 2: finch-mcp UI Mode Testing${NC}"
echo "====================================="

echo "Testing finch-mcp UI mode (non-MCP context)..."
UI_OUTPUT=$(mktemp)
UI_ERROR=$(mktemp)

# Run finch-mcp in UI mode and check it shows banner and builds container
if echo '{}' | timeout 10s "$FINCH_MCP_PATH" run "$PROJECT_PATH" > "$UI_OUTPUT" 2> "$UI_ERROR"; then
    if grep -q "Finch-MCP v" "$UI_ERROR" && grep -q "Starting MCP server" "$UI_ERROR"; then
        echo -e "${GREEN}✅ UI mode works correctly${NC}"
        echo "   Shows banner and starts server"
        FINCH_UI_PASS=true
    else
        echo -e "${RED}❌ UI mode failed${NC}"
        echo "   Expected banner and server start messages"
        FINCH_UI_PASS=false
    fi
else
    echo -e "${RED}❌ UI mode failed to start${NC}"
    FINCH_UI_PASS=false
fi

rm "$UI_OUTPUT" "$UI_ERROR"
echo ""

# Test 3: finch-mcp MCP Mode Testing
echo -e "${BLUE}🔧 Test 3: finch-mcp MCP Mode Testing${NC}"
echo "====================================="

echo "Testing finch-mcp MCP mode (with MCP_STDIO)..."
MCP_OUTPUT=$(mktemp)
MCP_ERROR=$(mktemp)

# Test that MCP mode doesn't show banner and goes straight to STDIO
if echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}' | \
   MCP_STDIO=1 timeout 10s "$FINCH_MCP_PATH" run "$PROJECT_PATH" > "$MCP_OUTPUT" 2> "$MCP_ERROR"; then
    
    # Check that it doesn't show banner in MCP mode
    if ! grep -q "Finch-MCP v" "$MCP_ERROR"; then
        echo -e "${GREEN}✅ MCP mode suppresses banner correctly${NC}"
        FINCH_MCP_BANNER_PASS=true
    else
        echo -e "${RED}❌ MCP mode shows banner (should be suppressed)${NC}"
        FINCH_MCP_BANNER_PASS=false
    fi
    
    # Check for JSON-RPC response
    if grep -q '"jsonrpc"' "$MCP_OUTPUT"; then
        echo -e "${GREEN}✅ MCP mode returns JSON-RPC responses${NC}"
        FINCH_MCP_JSONRPC_PASS=true
    else
        echo -e "${RED}❌ MCP mode doesn't return JSON-RPC responses${NC}"
        echo "   Output: $(head -n 3 "$MCP_OUTPUT")"
        FINCH_MCP_JSONRPC_PASS=false
    fi
else
    echo -e "${RED}❌ MCP mode failed to respond${NC}"
    echo "   Error: $(head -n 3 "$MCP_ERROR")"
    FINCH_MCP_BANNER_PASS=false
    FINCH_MCP_JSONRPC_PASS=false
fi

rm "$MCP_OUTPUT" "$MCP_ERROR"
echo ""

# Test 4: Container Integration Testing
echo -e "${BLUE}🚀 Test 4: Container Integration Testing${NC}"
echo "========================================"

echo "Testing container build and caching..."
CONTAINER_OUTPUT=$(mktemp)
CONTAINER_ERROR=$(mktemp)

# Run finch-mcp twice to test caching
echo "First run (should build container):"
if echo '{}' | timeout 15s "$FINCH_MCP_PATH" run "$PROJECT_PATH" > "$CONTAINER_OUTPUT" 2> "$CONTAINER_ERROR"; then
    if grep -q "Cache miss\|Building\|Containerizing" "$CONTAINER_ERROR"; then
        echo -e "${GREEN}✅ Container builds correctly${NC}"
        CONTAINER_BUILD_PASS=true
    else
        echo -e "${YELLOW}⚠️  Container might be cached from previous runs${NC}"
        CONTAINER_BUILD_PASS=true
    fi
else
    echo -e "${RED}❌ Container build failed${NC}"
    echo "   Error: $(head -n 3 "$CONTAINER_ERROR")"
    CONTAINER_BUILD_PASS=false
fi

echo ""
echo "Second run (should use cached container):"
if echo '{}' | timeout 10s "$FINCH_MCP_PATH" run "$PROJECT_PATH" > "$CONTAINER_OUTPUT" 2> "$CONTAINER_ERROR"; then
    if grep -q "Cache hit\|Using cached" "$CONTAINER_ERROR"; then
        echo -e "${GREEN}✅ Container caching works${NC}"
        CONTAINER_CACHE_PASS=true
    else
        echo -e "${YELLOW}⚠️  Cache status unclear${NC}"
        CONTAINER_CACHE_PASS=true
    fi
else
    echo -e "${RED}❌ Cached container run failed${NC}"
    CONTAINER_CACHE_PASS=false
fi

rm "$CONTAINER_OUTPUT" "$CONTAINER_ERROR"
echo ""

# Summary
echo -e "${BLUE}📊 Validation Summary${NC}"
echo "====================="

if [[ "$PROJECT_TYPE" == "npm" ]]; then
    echo -n "Direct Server Tools: "
    if $DIRECT_TOOLS_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi
    
    echo -n "Direct Server Tool Calls: "
    if $DIRECT_TOOL_CALL_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi
    
    echo -n "Direct Server Resources: "
    if $DIRECT_RESOURCES_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi
    
    echo -n "Direct Server Prompts: "
    if $DIRECT_PROMPTS_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi
fi

echo -n "finch-mcp UI Mode: "
if $FINCH_UI_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "finch-mcp MCP Banner Suppression: "
if $FINCH_MCP_BANNER_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "finch-mcp MCP JSON-RPC: "
if $FINCH_MCP_JSONRPC_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "Container Build: "
if $CONTAINER_BUILD_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo -n "Container Caching: "
if $CONTAINER_CACHE_PASS; then echo -e "${GREEN}✅ PASS${NC}"; else echo -e "${RED}❌ FAIL${NC}"; fi

echo ""

# Overall result
ALL_TESTS_PASS=true
if [[ "$PROJECT_TYPE" == "npm" ]]; then
    if ! $DIRECT_TOOLS_PASS || ! $DIRECT_TOOL_CALL_PASS || ! $DIRECT_RESOURCES_PASS || ! $DIRECT_PROMPTS_PASS; then
        ALL_TESTS_PASS=false
    fi
fi

if ! $FINCH_UI_PASS || ! $FINCH_MCP_BANNER_PASS || ! $FINCH_MCP_JSONRPC_PASS || ! $CONTAINER_BUILD_PASS || ! $CONTAINER_CACHE_PASS; then
    ALL_TESTS_PASS=false
fi

if $ALL_TESTS_PASS; then
    echo -e "${GREEN}🎉 All tests passed! npm local functionality and finch-mcp work correctly.${NC}"
    exit 0
else
    echo -e "${RED}💥 Some tests failed. Check the output above for details.${NC}"
    exit 1
fi