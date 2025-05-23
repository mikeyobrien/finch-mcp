# Finch-MCP Documentation

Welcome to the comprehensive documentation for Finch-MCP, a specialized tool for running MCP (Model Context Protocol) servers in containers using [Finch](https://runfinch.com/).

## Table of Contents

### Getting Started
- [Getting Started Guide](./getting-started.md) - Quick start and basic usage
- [Architecture Overview](./architecture.md) - System design and components

### Core Documentation
- [Containerization](./containerization.md) - How auto-containerization works
- [Development Mode](./development.md) - Live reloading and debugging
- [CLI Commands](./commands.md) - Command reference
- [API Reference](./api-reference.md) - Complete API documentation

### Advanced Topics
- [Publishing MCP Servers](./publishing.md) - Distribution and deployment
- [Security Best Practices](./security.md) - Security considerations
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions

### Integration Guides
- [MCP Inspector Integration](./mcp-inspector-integration.md) - Testing with MCP Inspector
- [Auto-Containerization Details](./auto-containerization.md) - Deep dive into containerization

### Testing & Validation
- [Manual Verification](./manual_verification.md) - Testing procedures

## Overview

Finch-MCP is a Rust-based CLI tool that simplifies the process of containerizing and running MCP servers. It provides multiple execution modes to accommodate different use cases, from running pre-built container images to automatically containerizing local projects and commands.

### Key Features

- **Multiple Execution Modes**: Direct container, auto-containerization, local directory, and Git repository support
- **Intelligent Detection**: Automatically detects project types and generates appropriate Dockerfiles
- **Smart Caching**: Content-based caching system prevents unnecessary rebuilds
- **MCP Protocol Support**: Seamless integration with MCP clients like Claude Desktop
- **Security First**: Non-root execution, minimal images, proper signal handling
- **Developer Friendly**: Live reloading, debugging support, comprehensive logging

### Core Capabilities

1. **Auto-Containerization**: Transform commands like `uvx mcp-server-time` into containerized services
2. **Project Support**: Handle Node.js, Python, Rust, and other project types
3. **Git Integration**: Clone and containerize repositories directly
4. **Volume Management**: Mount local directories for development
5. **Environment Control**: Pass environment variables and configuration

## Quick Examples

```bash
# Run a Python MCP server
finch-mcp run uvx mcp-server-time

# Containerize a local Node.js project
finch-mcp run ./my-mcp-server

# Clone and run from Git
finch-mcp run https://github.com/user/mcp-server

# Use with Claude Desktop
finch-mcp run -v ~/docs:/workspace npx @modelcontextprotocol/server-filesystem /workspace
```

## Architecture Highlights

Finch-MCP uses a layered architecture:

1. **CLI Layer**: Command parsing and mode detection
2. **Core Modules**: Containerization logic for different scenarios
3. **Infrastructure**: Finch client, cache manager, logging system

See the [Architecture Overview](./architecture.md) for detailed information.

## Use Cases

### Development
- Rapid prototyping with auto-containerization
- Live reloading for iterative development
- Consistent environments across team members

### Testing
- Isolated test environments
- Integration with MCP Inspector
- Reproducible test scenarios

### Production
- Optimized container images
- Multi-architecture support
- Secure execution environment

### Distribution
- Easy publishing to registries
- Version management
- User-friendly deployment

## Getting Help

If you need assistance:

1. Start with the [Getting Started Guide](./getting-started.md)
2. Check the [Troubleshooting Guide](./troubleshooting.md) for common issues
3. Review the [API Reference](./api-reference.md) for detailed command information
4. Open an issue on [GitHub](https://github.com/mikeyobrien/finch-mcp/issues)
5. Join the community discussions

## Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/mikeyobrien/finch-mcp/blob/main/CONTRIBUTING.md) for details on:

- Setting up your development environment
- Running tests
- Submitting pull requests
- Code style guidelines

## License

Finch-MCP is released under the MIT License. See the [LICENSE](https://github.com/mikeyobrien/finch-mcp/blob/main/LICENSE) file for details.