# CLI Commands

Finch-MCP provides a set of CLI commands for creating, running, and publishing MCP server containers. This page documents all available commands and their options.

## Global Options

These options apply to all commands:

- `--version`: Display the version number
- `--help`: Display help information

## Commands

### Create

Creates an MCP server container from a local repository.

```bash
finch-mcp create <path> [options]
```

#### Arguments

- `<path>`: Path to the MCP server repository (required)

#### Options

- `-t, --tag <tag>`: Tag for the container image (default: "latest")
- `-n, --name <name>`: Name for the container image (default: repository directory name)

#### Examples

```bash
# Create a container from the current directory
finch-mcp create .

# Create a container with a specific name and tag
finch-mcp create ./my-mcp-server -n my-custom-name -t v1.0.0
```

### Run

Runs an MCP server container.

```bash
finch-mcp run <path-or-image> [options]
```

#### Arguments

- `<path-or-image>`: Path to the MCP server repository or image name (required)

#### Options

- `-p, --port <port>`: Port to expose the MCP server on (default: "3000")
- `-e, --env <env...>`: Environment variables to pass to the container

#### Examples

```bash
# Run a container from the current directory
finch-mcp run .

# Run a specific container image
finch-mcp run my-mcp-server:latest

# Run with a specific port and environment variables
finch-mcp run ./my-mcp-server -p 5000 -e NODE_ENV=production -e DEBUG=true
```

### Dev

Runs an MCP server in development mode with live reloading.

```bash
finch-mcp dev <path> [options]
```

#### Arguments

- `<path>`: Path to the MCP server repository (required)

#### Options

- `-p, --port <port>`: Port to expose the MCP server on (default: "3000")
- `-e, --env <env...>`: Environment variables to pass to the container

#### Examples

```bash
# Run development mode for the current directory
finch-mcp dev .

# Run development mode with a specific port
finch-mcp dev ./my-mcp-server -p 5000

# Run development mode with environment variables
finch-mcp dev . -e NODE_ENV=development -e DEBUG=true
```

### Publish

Publishes an MCP server container to a registry.

```bash
finch-mcp publish <path> [options]
```

#### Arguments

- `<path>`: Path to the MCP server repository (required)

#### Options

- `-r, --registry <registry>`: Container registry to publish to
- `-t, --tag <tag>`: Tag for the container image (default: "latest")
- `-n, --name <name>`: Name for the container image (default: repository directory name)

#### Examples

```bash
# Publish a container from the current directory (interactive prompt for registry)
finch-mcp publish .

# Publish to a specific registry
finch-mcp publish ./my-mcp-server -r docker.io/username

# Publish with a specific name and tag
finch-mcp publish . -r docker.io/username -n my-mcp-server -t v1.0.0
```

## Advanced Usage

### Using with npx/uvx

You can run Finch-MCP directly with npx or uvx without installing it globally:

```bash
npx finch-mcp create ./my-mcp-server
```

### Using with CI/CD

In CI/CD environments, you can use the non-interactive mode by providing all required options:

```bash
finch-mcp publish ./my-mcp-server -r ${REGISTRY_URL} -t ${VERSION} -n ${IMAGE_NAME}
```