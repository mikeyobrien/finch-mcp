# Finch-MCP Documentation

Welcome to the documentation for the Finch-MCP tool. This tool helps you containerize and distribute MCP (Model Context Protocol) servers using [Finch](https://runfinch.com/).

## Table of Contents

- [Installation](./installation.md)
- [Getting Started](./getting-started.md)
- [CLI Commands](./commands.md)
- [Containerization Process](./containerization.md)
- [Development Mode](./development.md)
- [Publishing MCP Servers](./publishing.md)
- [Security](./security.md)
- [API Reference](./api-reference.md)
- [Troubleshooting](./troubleshooting.md)
- [Contributing](./contributing.md)

## Overview

Finch-MCP is a tool designed to make it easy to containerize, run, and distribute MCP servers. It provides a simple CLI interface for creating containers from local repositories, running them in development mode with live reloading, and publishing them to container registries.

### Key Features

- **Automatic MCP Server Detection**: Finch-MCP can automatically detect MCP servers in your repository, including the server type, entry point, and port.
- **Package Manager Support**: Works with both npm and uv package managers.
- **Finch Integration**: Uses Finch for containerization, providing a lightweight and open-source alternative to Docker.
- **Development Mode**: Supports development with live reloading by mounting your local repository as a volume.
- **Easy Distribution**: Publish your MCP server containers to container registries for easy sharing and deployment.

## Use Cases

1. **Development**: Quickly containerize and run MCP servers during development without having to write Dockerfiles manually.

2. **Testing**: Test MCP servers in isolated environments with consistent dependencies.

3. **Distribution**: Package and distribute MCP servers as containers for easy deployment in various environments.

4. **CI/CD**: Integrate MCP server containerization into your CI/CD pipelines for automated building and testing.

## Getting Help

If you encounter any issues or have questions, please:

1. Check the [Troubleshooting](./troubleshooting.md) guide
2. Open an issue on the [GitHub repository](https://github.com/yourusername/finch-mcp)
3. Join our community on [Discord](https://discord.gg/finch-mcp)