#!/bin/bash
# Wrapper script for finch-mcp that handles build-then-run in MCP mode

# Check if we're in MCP mode
if [ -n "$MCP_STDIO" ]; then
    # Extract the target (last argument that doesn't start with -)
    TARGET=""
    for arg in "$@"; do
        case $arg in
            -*) ;;
            *) TARGET="$arg" ;;
        esac
    done
    
    # First, build the image without MCP_STDIO
    unset MCP_STDIO
    OUTPUT=$(finch-mcp "$@" 2>&1)
    BUILD_EXIT=$?
    
    if [ $BUILD_EXIT -ne 0 ]; then
        echo "Build failed" >&2
        exit $BUILD_EXIT
    fi
    
    # Extract the image name from the output
    IMAGE=$(echo "$OUTPUT" | grep "mcp-" | grep -o 'mcp-[^ ]*' | tail -1)
    
    if [ -z "$IMAGE" ]; then
        echo "Could not determine image name" >&2
        exit 1
    fi
    
    # Now run with MCP_STDIO and --direct
    export MCP_STDIO=1
    exec finch-mcp run "$IMAGE" --direct
else
    # Not in MCP mode, just run normally
    exec finch-mcp "$@"
fi