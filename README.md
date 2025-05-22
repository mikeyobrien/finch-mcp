# Finch-MCP

[![CI](https://github.com/mikeyobrien/finch-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/mikeyobrien/finch-mcp/actions/workflows/ci.yml)
[![Release](https://github.com/mikeyobrien/finch-mcp/actions/workflows/release.yml/badge.svg)](https://github.com/mikeyobrien/finch-mcp/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/mikeyobrien/finch-mcp)](https://github.com/mikeyobrien/finch-mcp/releases/latest)

A specialized tool for running MCP (Model Context Protocol) servers in STDIO mode using [Finch](https://runfinch.com/).

## Features

- Run MCP servers in STDIO mode for direct communication via standard input/output
- **NEW: Auto-containerization** - Run commands directly without building Docker images
- Support for custom environment variables and volume mounts
- Graceful container termination with Ctrl+C
- Built with Rust for performance and reliability

## Prerequisites

- [Finch](https://runfinch.com/) (for container management)
  - **First-time users**: finch-mcp will automatically run `finch vm init` if needed
  - **Manual setup**: You can also run `finch vm init` manually if you prefer
- Either:
  - An MCP server image with STDIO mode support, OR
  - A command that runs an MCP server (e.g., `uvx mcp-server-time`)

**Note**: If building from source, you'll also need [Rust](https://www.rust-lang.org/tools/install) 1.70 or later.

## Installation

### Pre-built Binaries (Recommended)

#### Quick Install (Unix)
```bash
curl -sSL https://raw.githubusercontent.com/mikeyobrien/finch-mcp/main/install.sh | bash
```

#### Manual Download
Download the latest pre-built binary for your platform from the [Releases page](https://github.com/mikeyobrien/finch-mcp/releases):

#### macOS
```bash
# Intel Macs
curl -L -o finch-mcp-stdio.tar.gz https://github.com/mikeyobrien/finch-mcp/releases/latest/download/finch-mcp-stdio-macos-x86_64.tar.gz
tar -xzf finch-mcp-stdio.tar.gz
chmod +x finch-mcp-stdio
sudo mv finch-mcp-stdio /usr/local/bin/

# Apple Silicon (M1/M2/M3)
curl -L -o finch-mcp-stdio.tar.gz https://github.com/mikeyobrien/finch-mcp/releases/latest/download/finch-mcp-stdio-macos-aarch64.tar.gz
tar -xzf finch-mcp-stdio.tar.gz
chmod +x finch-mcp-stdio
sudo mv finch-mcp-stdio /usr/local/bin/
```

#### Linux
```bash
# x86_64
curl -L -o finch-mcp-stdio.tar.gz https://github.com/mikeyobrien/finch-mcp/releases/latest/download/finch-mcp-stdio-linux-x86_64.tar.gz
tar -xzf finch-mcp-stdio.tar.gz
chmod +x finch-mcp-stdio
sudo mv finch-mcp-stdio /usr/local/bin/

# ARM64
curl -L -o finch-mcp-stdio.tar.gz https://github.com/mikeyobrien/finch-mcp/releases/latest/download/finch-mcp-stdio-linux-aarch64.tar.gz
tar -xzf finch-mcp-stdio.tar.gz
chmod +x finch-mcp-stdio
sudo mv finch-mcp-stdio /usr/local/bin/
```

#### Windows
1. Download `finch-mcp-stdio-windows-x86_64.exe.zip` from the [Releases page](https://github.com/mikeyobrien/finch-mcp/releases)
2. Extract the zip file
3. Move `finch-mcp-stdio.exe` to a directory in your PATH

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

### Local Directory Mode (NEW!)

Containerize and run an MCP server from a local directory:

```bash
finch-mcp-stdio ./my-mcp-project
finch-mcp-stdio /absolute/path/to/project
```

This will automatically:
1. Detect the project type (Node.js, Python, etc.) from files like `package.json` or `pyproject.toml`
2. Create an appropriate Dockerfile based on the project structure
3. Build and run the container in STDIO mode

Supports various project types:
- **Node.js**: Projects with `package.json` (including monorepos with workspaces)
- **Python**: Projects with `pyproject.toml` (Poetry/UV), `setup.py`, or `requirements.txt`
- **TypeScript**: Automatically compiled during build

### Git Repository Mode (NEW!)

Clone and containerize an MCP server directly from a git repository:

```bash
finch-mcp-stdio https://github.com/user/mcp-server-repo
```

This will automatically:
1. Clone the repository to a temporary directory
2. Detect the project type and dependencies
3. Build and run the container in STDIO mode

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
    <COMMAND>       MCP server image, command, git repository URL, or local directory to run
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
# Simple command
finch-mcp-stdio uvx mcp-server-time --local-timezone America/New_York

# Quoted command (recommended for complex commands)
finch-mcp-stdio "uvx mcp-server-time --local-timezone America/New_York"
```

### Running the MCP Filesystem Server

The filesystem server provides file operations with volume mounting:

```bash
# NPX with volume mount
finch-mcp-stdio -v /local/path:/workspace "npx @modelcontextprotocol/server-filesystem /workspace"

# With NPX flags
finch-mcp-stdio -v ./:/workspace "npx -y @modelcontextprotocol/server-filesystem /workspace"
```

### Using with Claude Code

Configure Claude Code to use finch-mcp-stdio:

**Time Server:**
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

**Filesystem Server:**
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

## Troubleshooting

### First-time Finch Setup

If you're using Finch for the first time, `finch-mcp-stdio` will automatically initialize the VM:

```bash
finch-mcp-stdio uvx mcp-server-time
# Output:
# 🚀 Initializing Finch VM for first-time use...
# This may take a few minutes to download and set up the VM.
# [VM initialization progress...]
# ✅ Finch VM initialized successfully!
# 🔄 Starting Finch VM...
# ✅ Finch VM started successfully
```

### Manual Finch Setup

If you prefer to set up Finch manually:

```bash
# Initialize the VM (first-time only)
finch vm init

# Start the VM
finch vm start

# Check VM status
finch vm status
```

### Common Issues

1. **"Finch is not installed"**: Install Finch from https://runfinch.com/
2. **VM initialization takes a long time**: This is normal for first-time setup as it downloads VM images
3. **Permission errors**: On some systems, you may need to run with appropriate permissions or add your user to the docker group equivalent

For more troubleshooting, see the [Finch documentation](https://runfinch.com/docs/).

## Validation and Testing

### MCP Inspector Integration

finch-mcp integrates with the official [`@modelcontextprotocol/inspector`](https://github.com/modelcontextprotocol/inspector) for validation and testing. This ensures that containerized MCP servers maintain full protocol compliance.

#### Quick Validation

Test npm local functionality:
```bash
./scripts/test-npm-validation.sh
```

This validates:
- ✅ npm MCP servers work correctly with Inspector
- ✅ finch-mcp can containerize npm projects  
- ✅ MCP protocol compliance is maintained

#### Comprehensive Testing

For detailed validation:
```bash
./scripts/validate-mcp-inspector.sh [project-path] [finch-mcp-binary]
```

This tests:
- Direct MCP server functionality
- finch-mcp UI vs MCP modes
- Container build and caching
- JSON-RPC protocol compliance

#### Manual Testing with Inspector

Test an npm MCP server directly:
```bash
cd tests/fixtures/test-mcp-filesystem
npx @modelcontextprotocol/inspector --cli npx -y @modelcontextprotocol/server-filesystem /tmp --method tools/list
```

Launch Inspector UI for interactive testing:
```bash
npx @modelcontextprotocol/inspector npx -y @modelcontextprotocol/server-filesystem /tmp
# Opens web interface at http://localhost:6274
```

For detailed documentation, see [MCP Inspector Integration](./docs/mcp-inspector-integration.md).

## License

MIT License - See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.