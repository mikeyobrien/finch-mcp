# Finch-MCP - MCP Server Distribution Tool

Finch-MCP is a tool for containerizing and distributing MCP (Model Context Protocol) servers using [Finch](https://runfinch.com/). It makes it easy to package and run MCP servers using npm/uv and npx/uvx.

## Features

- Automatic detection of MCP servers in local repositories
- Support for npm and uv package managers
- Container creation with Finch
- Easy distribution and consumption with npx/uvx
- Development mode with live reloading

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18.0.0
- [Finch](https://runfinch.com/) (for container management)
- npm or uv (for package management)

## Installation

```bash
npm install -g finch-mcp
```

## Usage

### Create an MCP server container

```bash
finch-mcp create ./path/to/mcp-server
```

### Run an MCP server

```bash
finch-mcp run ./path/to/mcp-server
```

### Develop with live reloading

```bash
finch-mcp dev ./path/to/mcp-server
```

### Distribute an MCP server

```bash
finch-mcp publish ./path/to/mcp-server
```

## Documentation

For more detailed documentation, see the [docs](./docs) directory.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.