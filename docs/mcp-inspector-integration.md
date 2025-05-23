# MCP Inspector Integration

This document describes how finch-mcp integrates with the `@modelcontextprotocol/inspector` for validation and testing.

## Overview

The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) is the official testing and debugging tool for MCP servers. We use it to validate that:

1. **npm MCP servers work correctly** when run directly
2. **finch-mcp containerization** preserves MCP functionality  
3. **End-to-end workflows** function properly

## Validation Scripts

### Quick npm Validation

```bash
./scripts/test-npm-validation.sh
```

This script:
- Tests npm MCP servers directly using Inspector CLI
- Validates finch-mcp can containerize npm projects
- Confirms basic MCP protocol functionality

### Comprehensive Validation  

```bash
./scripts/validate-mcp-inspector.sh [project-path] [finch-mcp-binary]
```

This script provides detailed testing of:
- Direct MCP server functionality
- finch-mcp UI vs MCP modes
- Container build and caching
- JSON-RPC protocol compliance

## Using Inspector for Development

### CLI Mode Testing

Test an npm MCP server directly:
```bash
cd tests/fixtures/test-mcp-filesystem
npx @modelcontextprotocol/inspector --cli npx -y @modelcontextprotocol/server-filesystem /tmp --method tools/list
```

Test finch-mcp containerized server:
```bash
# Note: Direct Inspector testing of finch-mcp is complex due to containerization
# Use our validation scripts instead
./scripts/test-npm-validation.sh
```

### UI Mode Testing

Launch Inspector UI for interactive testing:
```bash
cd tests/fixtures/test-mcp-filesystem
npx @modelcontextprotocol/inspector npx -y @modelcontextprotocol/server-filesystem /tmp
```

This opens a web interface at http://localhost:6274 for visual testing.

### Available Inspector Methods

- `tools/list` - List available tools
- `tools/call --tool-name <name> --tool-arg key=value` - Call a specific tool
- `resources/list` - List available resources (if supported)
- `resources/read --resource-uri <uri>` - Read a specific resource
- `prompts/list` - List available prompts (if supported)
- `prompts/get --prompt-name <name>` - Get a specific prompt

## npm Local Functionality Validation

Our test suite validates the complete npm workflow:

### 1. Direct Server Testing
```bash
# Tests that npm MCP servers work correctly
npx @modelcontextprotocol/inspector --cli npx -y @modelcontextprotocol/server-filesystem /tmp --method tools/list
```

### 2. Container Build Testing
```bash
# Tests that finch-mcp can build npm projects
finch-mcp run tests/fixtures/test-mcp-filesystem
```

### 3. Protocol Compliance
- JSON-RPC message format validation
- Tool schema validation  
- Error handling verification
- Connection lifecycle testing

## Test Projects

### test-mcp-filesystem
- **Type**: npm project with MCP server dependency
- **Package**: `@modelcontextprotocol/server-filesystem`
- **Purpose**: Tests npm package installation and binary execution
- **Tools**: File system operations (read, write, list, etc.)

### test-mcp-time  
- **Type**: npm project with Python MCP server (uvx)
- **Package**: `mcp-server-time` (Python package via uvx)
- **Purpose**: Tests cross-language package management
- **Tools**: Time and timezone operations

## CI/CD Integration

Add to your CI pipeline:

```yaml
- name: Validate MCP functionality
  run: |
    cargo build --release
    ./scripts/test-npm-validation.sh
```

This ensures:
- finch-mcp builds correctly
- npm MCP servers are functional
- Containerization preserves MCP protocol compliance

## Troubleshooting

### Inspector Connection Issues

If Inspector fails to connect:
1. **Check the MCP server path**: Ensure the executable/package exists
2. **Verify arguments**: Make sure paths and arguments are valid
3. **Test directly**: Try running the MCP server command manually
4. **Check logs**: Look for startup errors or missing dependencies

### finch-mcp Containerization Issues

If finch-mcp fails to containerize:
1. **Check Docker/Finch**: Ensure container runtime is running
2. **Verify project structure**: Ensure package.json and dependencies are valid
3. **Check build logs**: Look for npm install or build failures
4. **Test manual build**: Try `npm install` in the project directory

### Protocol Compliance Issues

If MCP protocol validation fails:
1. **Check JSON-RPC format**: Ensure server returns valid JSON-RPC responses
2. **Verify tool schemas**: Ensure tools match expected input/output formats
3. **Test capability negotiation**: Ensure server properly handles initialize/handshake
4. **Check STDIO cleanliness**: Ensure no debug output pollutes STDIO in MCP mode

## Best Practices

1. **Always test npm servers directly first** before testing through finch-mcp
2. **Use Inspector CLI for automated testing**, UI for interactive debugging
3. **Validate both cached and fresh container builds**
4. **Test with real-world MCP servers** from the ecosystem
5. **Include Inspector validation in your development workflow**

## Future Enhancements

- **Automated regression testing** against MCP specification updates
- **Performance benchmarking** comparing direct vs containerized execution
- **Integration with Claude Desktop** configuration validation
- **Support for additional MCP server types** (Python, Go, etc.)