# Auto-Containerization for MCP Servers

finch-mcp now supports automatic containerization of MCP servers. This feature allows you to run any command that starts an MCP server, and finch-mcp will automatically:

1. Detect the type of command (Python/uvx, Node/npm, etc.)
2. Create an appropriate container environment
3. Run the server in STDIO mode within the container

## Usage

Instead of building your own Docker image, you can simply run a command like:

```bash
# Simple command format
finch-mcp-stdio uvx mcp-server-time --local-timezone UTC

# Quoted command format (recommended for complex commands)
finch-mcp-stdio "uvx mcp-server-time --local-timezone UTC"

# NPX commands with volume mounts
finch-mcp-stdio -v /host/path:/container/path "npx @package /container/path"
```

The tool will:
- Detect the command type (Python/uvx, Node.js/npx, etc.)
- Create an appropriate container image
- Install the necessary dependencies
- Run the command with the provided arguments in STDIO mode

## Supported Command Types

The auto-containerization feature currently supports:

- **Python Commands**:
  - `uvx` - For UVx package manager commands (uses Python 3.11-slim)
  - `pip` - For pip package manager commands

- **Node.js Commands**:
  - `npm` - For npm package manager commands (uses Node.js 20-slim)
  - `npx` - For npx commands with intelligent argument parsing

- **Generic Commands**:
  - Any other command type will use a generic Debian base image

## Quoted Command Support

For complex commands with flags and arguments, use quoted strings:

```bash
# Without quotes (simple)
finch-mcp-stdio uvx mcp-server-time

# With quotes (recommended for complex commands)
finch-mcp-stdio "npx -y @modelcontextprotocol/server-filesystem /workspace"
```

The quoted format is especially useful for:
- Commands with flags (like `-y` for NPX)
- Package names with special characters (like `@scope/package`)
- Multiple arguments that should stay together

## Example: Running the MCP Time Server

The MCP time server provides time-related functionality like getting the current time in different timezones or converting times between timezones.

Instead of manually creating a Docker image, you can run:

```bash
finch-mcp-stdio uvx mcp-server-time --local-timezone America/New_York
```

This will:
1. Create a Python container
2. Install the uv package manager
3. Install the mcp-server-time package
4. Run the server with the specified timezone
5. Connect the STDIO streams

## Example: Running the MCP Filesystem Server

The filesystem server provides file operations and requires volume mounting:

```bash
finch-mcp-stdio -v /local/project:/workspace "npx @modelcontextprotocol/server-filesystem /workspace"
```

This command:
1. Creates a Node.js 20 container
2. Installs the filesystem server package via NPX
3. Mounts your local project directory as `/workspace`
4. Runs the server with access to the mounted files

## Using with Claude Code

**Time Server Configuration:**
```json
{
  "mcp": {
    "servers": {
      "time": {
        "command": "/path/to/finch-mcp-stdio",
        "args": ["uvx mcp-server-time --local-timezone UTC"]
      }
    }
  }
}
```

**Filesystem Server Configuration:**
```json
{
  "mcp": {
    "servers": {
      "filesystem": {
        "command": "/path/to/finch-mcp-stdio",
        "args": [
          "-v", "${workspaceFolder}:/workspace",
          "npx @modelcontextprotocol/server-filesystem /workspace"
        ]
      }
    }
  }
}
```

## Environment Variables and Volumes

You can pass environment variables and mount volumes just like when running a regular container:

```bash
finch-mcp-stdio uvx mcp-server-time \
  --local-timezone UTC \
  -e CUSTOM_VAR=value \
  -v /host/path:/container/path
```

## Direct Container Mode

If you already have a Docker image and want to run it directly (without auto-containerization), use the `--direct` flag:

```bash
finch-mcp-stdio --direct my-custom-mcp-image:latest
```

This behaves the same as the original behavior, running the specified image directly.

## How It Works

1. The tool parses your command and arguments
2. It detects what type of environment is needed (Python, Node.js, etc.)
3. It creates a temporary Dockerfile tailored to that environment
4. It builds the Docker image using finch
5. It runs the container with the appropriate environment variables and mounts
6. STDIO is connected between your terminal and the containerized server

This approach dramatically simplifies the process of containerizing MCP servers, removing the need to manually create Dockerfiles or build images.