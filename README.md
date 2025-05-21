# Finch-MCP

A specialized tool for running MCP (Model Context Protocol) servers in STDIO mode using [Finch](https://runfinch.com/).

## Features

- Run MCP servers in STDIO mode for direct communication via standard input/output
- Support for custom environment variables and volume mounts
- Graceful container termination with Ctrl+C
- Built with Rust for performance and reliability

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later
- [Finch](https://runfinch.com/) (for container management)
- An MCP server image with STDIO mode support

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

### Basic Usage

Run an MCP server in STDIO mode:

```bash
finch-mcp-stdio my-mcp-image:latest
```

### With Environment Variables

```bash
finch-mcp-stdio my-mcp-image:latest -e API_KEY=xyz123 -e DEBUG=true
```

### With Volume Mounts

```bash
finch-mcp-stdio my-mcp-image:latest -v /host/path:/container/path
```

### Full Options

```bash
USAGE:
    finch-mcp-stdio [OPTIONS] <IMAGE>

ARGS:
    <IMAGE>    MCP server image to run

OPTIONS:
    -e, --env <KEY=VALUE>...                Environment variables to pass to the container
    -v, --volume <HOST_PATH:CONTAINER_PATH>...    Mount volumes in the container
    -h, --help                              Print help information
    -V, --version                           Print version information
    -v, --verbose                           Enable verbose logging (repeat for more verbosity)
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

1. The tool validates that Finch is installed and available
2. It ensures the Finch VM is running
3. It executes the specified container with STDIO mode enabled (`MCP_STDIO=true`)
4. Standard input/output streams are piped between your application and the container
5. The tool handles signal interrupts for graceful termination

## License

MIT License - See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.