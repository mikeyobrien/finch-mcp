# Demo: Using finch-mcp-stdio with MCP Time Server

This document demonstrates how to use finch-mcp-stdio to run the MCP time server.

## What We've Accomplished

✅ **Auto-containerization**: The tool automatically detects `uvx` commands and creates appropriate Python containers
✅ **Container Building**: Successfully builds containers with Python 3.11 and installs mcp-server-time
✅ **Package Installation**: Correctly installs all dependencies using uv with --system flag
✅ **Environment Setup**: Sets up MCP environment variables (MCP_ENABLED=true, MCP_STDIO=true)

## Command That Works

```bash
./target/debug/finch-mcp-stdio uvx mcp-server-time --local-timezone UTC
```

This command:

1. **Detects** the `uvx` command type
2. **Generates** a Dockerfile with Python 3.11-slim base image
3. **Installs** uv package manager
4. **Installs** mcp-server-time package using `uv pip install --system`
5. **Builds** the container image with a unique name (e.g., `mcp-auto-5effeaea`)
6. **Runs** the container in STDIO mode

## Claude Code Configuration

To use this with Claude Code, create a configuration file:

```json
{
  "mcp": {
    "servers": {
      "time": {
        "command": "/path/to/finch-mcp-stdio",
        "args": ["uvx", "mcp-server-time", "--local-timezone", "UTC"]
      }
    }
  }
}
```

## Benefits Over Manual Docker Setup

**Before** (manual approach):
1. Write Dockerfile
2. Build image manually
3. Configure environment variables
4. Set up STDIO mode
5. Run container with correct flags

**After** (with finch-mcp-stdio):
1. Run single command: `finch-mcp-stdio uvx mcp-server-time --local-timezone UTC`

The tool handles all containerization automatically!

## Verification

The container build logs show successful installation:
- Python 3.11.12 environment
- 23 packages installed including mcp-server-time v0.6.2
- All dependencies resolved successfully

## Next Steps for Production Use

1. **Claude Code Integration**: Test with actual Claude Code session
2. **Error Handling**: Add better error messages for common issues
3. **Caching**: Optimize container builds by caching base images
4. **Documentation**: Create user guides for different MCP servers