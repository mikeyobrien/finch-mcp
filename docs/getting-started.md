# Getting Started with Finch-MCP

This guide will help you get up and running with Finch-MCP to containerize and run MCP servers.

## Prerequisites

Before you begin, ensure you have:

1. **Finch** installed on your system
   - Download from [runfinch.com](https://runfinch.com/)
   - Finch-MCP will automatically initialize the VM if needed

2. **An MCP server** to run (or a command that starts one)
   - Examples: `uvx mcp-server-time`, `npx @modelcontextprotocol/server-filesystem`
   - Or a local MCP project directory

## Installation

### Quick Install (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/mikeyobrien/finch-mcp/main/install.sh | bash
```

### Manual Installation

Download the appropriate binary for your platform from the [releases page](https://github.com/mikeyobrien/finch-mcp/releases).

For macOS (Apple Silicon):
```bash
curl -L -o finch-mcp.tar.gz https://github.com/mikeyobrien/finch-mcp/releases/latest/download/finch-mcp-macos-aarch64.tar.gz
tar -xzf finch-mcp.tar.gz
chmod +x finch-mcp
sudo mv finch-mcp /usr/local/bin/
```

## Quick Start

### 1. Running a Simple MCP Server

The easiest way to start is with a public MCP server like the time server:

```bash
finch-mcp run uvx mcp-server-time
```

This will:
- Detect that `uvx` is a Python package runner
- Create an appropriate container environment
- Run the MCP server in STDIO mode
- Connect your terminal to the server

### 2. Running with Arguments

Pass arguments to your MCP server:

```bash
finch-mcp run uvx mcp-server-time --local-timezone America/New_York
```

### 3. Running a Local Project

If you have a local MCP server project:

```bash
finch-mcp run ./my-mcp-server
```

Finch-MCP will:
- Detect the project type (Node.js, Python, etc.)
- Generate an appropriate Dockerfile
- Build and run the container

### 4. Running from a Git Repository

```bash
finch-mcp run https://github.com/username/mcp-server-example
```

## Common Use Cases

### Node.js MCP Server with npm

```bash
finch-mcp run npx @modelcontextprotocol/server-filesystem /workspace
```

### Python MCP Server with uv

```bash
finch-mcp run uvx mcp-server-sqlite database.db
```

### Local TypeScript Project

```bash
finch-mcp run ./my-typescript-mcp-server
```

### With Environment Variables

```bash
finch-mcp run -e API_KEY=secret123 -e DEBUG=true uvx my-mcp-server
```

### With Volume Mounts

```bash
finch-mcp run -v /local/data:/container/data npx my-mcp-server
```

## Integration with Claude Desktop

To use Finch-MCP with Claude Desktop, add to your Claude configuration:

```json
{
  "mcpServers": {
    "time": {
      "command": "finch-mcp",
      "args": ["run", "uvx", "mcp-server-time"]
    },
    "filesystem": {
      "command": "finch-mcp",
      "args": [
        "run",
        "-v", "${HOME}/Documents:/workspace",
        "npx", "@modelcontextprotocol/server-filesystem", "/workspace"
      ]
    }
  }
}
```

## Understanding Output

When running an MCP server, you'll see:

1. **Build Output** (first run only):
   ```
   🔍 Analyzing project...
   📦 Building container...
   ✅ Container built successfully
   ```

2. **Execution Output**:
   ```
   Connecting to MCP Server...
   ```

3. **MCP Communication**: The server is now ready for STDIO communication

## Troubleshooting First Run

### Finch VM Initialization

On first use, Finch-MCP will initialize the Finch VM:

```
🚀 Initializing Finch VM for first-time use...
This may take a few minutes to download and set up the VM.
```

This is normal and only happens once.

### Cache Behavior

Finch-MCP caches built containers. Subsequent runs of the same command/project will be much faster:

```
✅ Cache hit - using existing container
```

To force a rebuild, use the cache management commands (see [Commands](./commands.md)).

## Next Steps

- Learn about [all available commands](./commands.md)
- Understand [how containerization works](./containerization.md)
- Set up [development mode](./development.md) for your MCP server
- Learn about [publishing](./publishing.md) your MCP server

## Getting Help

If you run into issues:

1. Run with verbose mode: `finch-mcp run -V uvx mcp-server-time`
2. Check the [troubleshooting guide](./troubleshooting.md)
3. View build logs: `finch-mcp logs show`
4. Open an issue on [GitHub](https://github.com/mikeyobrien/finch-mcp/issues)