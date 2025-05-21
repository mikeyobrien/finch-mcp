# Auto-Containerization for MCP Servers

finch-mcp now supports automatic containerization of MCP servers. This feature allows you to run any command that starts an MCP server, and finch-mcp will automatically:

1. Detect the type of command (Python/uvx, Node/npm, etc.)
2. Create an appropriate container environment
3. Run the server in STDIO mode within the container

## Usage

Instead of building your own Docker image, you can simply run a command like:

```bash
finch-mcp-stdio uvx mcp-server-time --local-timezone UTC
```

The tool will:
- Detect that you're running a Python/uvx command
- Create a Python container image
- Install the necessary dependencies
- Run the command with the provided arguments in STDIO mode

## Supported Command Types

The auto-containerization feature currently supports:

- **Python Commands**:
  - `uvx` - For UVx package manager commands (recommended for MCP time server)
  - `pip` - For pip package manager commands

- **Node.js Commands**:
  - `npm` - For npm package manager commands
  - `npx` - For npx commands

- **Generic Commands**:
  - Any other command type will use a generic Debian base image

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

## Using with Claude Code

To use this with Claude Code, create a configuration like:

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