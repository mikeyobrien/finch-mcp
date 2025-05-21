# Finch-MCP

A specialized tool for running MCP (Model Context Protocol) servers in STDIO mode using [Finch](https://runfinch.com/).

## Features

- Run MCP servers in STDIO mode for direct communication via standard input/output
- **NEW: Auto-containerization** - Run commands directly without building Docker images
- Support for custom environment variables and volume mounts
- Graceful container termination with Ctrl+C
- Built with Rust for performance and reliability

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later
- [Finch](https://runfinch.com/) (for container management)
- Either:
  - An MCP server image with STDIO mode support, OR
  - A command that runs an MCP server (e.g., `uvx mcp-server-time`)

## Installation

### From Source

Clone the repository and build the project:

```bash
git clone https://github.com/mikeyobrien/finch-mcp.git
cd finch-mcp
cargo build --release
```

The compiled binary will be available at `target/release/finch-mcp-stdio`.

### Using Cargo

```bash
cargo install --git https://github.com/mikeyobrien/finch-mcp.git
```

## Usage

### Direct Container Mode

Run an existing MCP server Docker image:

```bash
finch-mcp-stdio --direct my-mcp-image:latest
```

### Auto-Containerization Mode (NEW!)

Run an MCP server command directly:

```bash
finch-mcp-stdio uvx mcp-server-time --local-timezone UTC
```

This will automatically:
1. Detect the command type (Python/uvx in this case)
2. Create an appropriate container environment
3. Run the server in STDIO mode

### With Environment Variables

```bash
finch-mcp-stdio uvx mcp-server-time -e API_KEY=xyz123 -e DEBUG=true
```

### With Volume Mounts

```bash
finch-mcp-stdio uvx mcp-server-time -v /host/path:/container/path
```

### Full Options

```bash
USAGE:
    finch-mcp-stdio [OPTIONS] <COMMAND> [ARGS]...

ARGS:
    <COMMAND>       MCP server image or command to run
    [ARGS]...       Arguments for the command

OPTIONS:
    -e, --env <KEY=VALUE>...                Environment variables to pass to the container
    -v, --volume <HOST_PATH:CONTAINER_PATH>...    Mount volumes in the container
    --direct                               Skip auto-containerization (treat command as Docker image)
    -h, --help                             Print help information
    -V, --verbose                          Enable verbose logging (repeat for more verbosity)
```

## Examples

### Running the MCP Time Server

The MCP time server provides time-related functionality:

```bash
finch-mcp-stdio uvx mcp-server-time --local-timezone America/New_York
```

### Using with Claude Code

Configure Claude Code to use finch-mcp-stdio:

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

## Development

### Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests (requires Finch)
cargo test -- --ignored
```

### Building Documentation

```bash
cargo doc --no-deps --open
```

## How It Works

### Direct Container Mode

1. The tool validates that Finch is installed and available
2. It ensures the Finch VM is running
3. It executes the specified container with STDIO mode enabled (`MCP_STDIO=true`)
4. Standard input/output streams are piped between your application and the container
5. The tool handles signal interrupts for graceful termination

### Auto-Containerization Mode

1. The tool parses your command and arguments
2. It detects what type of environment is needed (Python, Node.js, etc.)
3. It creates a temporary Dockerfile tailored to that environment
4. It builds the Docker image using finch
5. It runs the container with the appropriate environment variables and mounts
6. STDIO is connected between your terminal and the containerized server

For more details, see [Auto-Containerization Documentation](./docs/auto-containerization.md).

## License

MIT License - See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.