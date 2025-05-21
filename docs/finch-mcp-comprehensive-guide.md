# Finch-MCP: Comprehensive Guide

## Overview

Finch-MCP is a specialized tool designed for containerizing and distributing MCP (Model Context Protocol) servers using [Finch](https://runfinch.com/), a lightweight container management solution. This tool streamlines the process of packaging AI/ML model servers that implement the Model Context Protocol, making them easy to distribute and run using either npm or uv package managers.

## What is Model Context Protocol (MCP)?

The Model Context Protocol is a standardized interface for AI/ML model servers that allows them to be easily integrated into applications. MCP servers expose a consistent API that enables communication between AI models and client applications, ensuring compatibility across different model implementations.

## Key Features

- **Automatic MCP Server Detection**: Automatically identifies MCP servers in your repository by analyzing dependencies, entry points, and code patterns.
- **Multi-Package Manager Support**: Works with both npm and uv package managers, detecting which one you're using and applying the appropriate configurations.
- **Finch Integration**: Leverages Finch for container management, providing a lighter alternative to Docker while maintaining compatibility.
- **Development Mode**: Offers a development environment with live reloading of changes for efficient iterative development.
- **Production-Ready Publishing**: Simplifies the process of publishing MCP servers to container registries for distribution.

## Prerequisites

- **Node.js**: Version 18.0.0 or higher
- **Finch**: The [Finch container runtime](https://runfinch.com/) must be installed and available
- **Package Manager**: Either npm or uv must be installed for dependency management

## Installation

Install Finch-MCP globally via npm:

```bash
npm install -g finch-mcp
```

Or with uv:

```bash
uvx install -g finch-mcp
```

## Core Commands

### 1. Create

Creates a containerized MCP server from a local repository:

```bash
finch-mcp create <path> [options]
```

**Arguments:**
- `<path>`: Path to the MCP server repository (required)

**Options:**
- `-t, --tag <tag>`: Tag for the container image (default: "latest")
- `-n, --name <name>`: Name for the container image (default: repository directory name)

**Examples:**
```bash
# Create a container from the current directory
finch-mcp create .

# Create a container with a specific name and tag
finch-mcp create ./my-mcp-server -n my-custom-name -t v1.0.0
```

### 2. Run

Runs an MCP server container:

```bash
finch-mcp run <path-or-image> [options]
```

**Arguments:**
- `<path-or-image>`: Path to the MCP server repository or image name (required)

**Options:**
- `-p, --port <port>`: Port to expose the MCP server on (default: "3000")
- `-e, --env <env...>`: Environment variables to pass to the container
- `-v, --volume <volume...>`: Mount volumes in the container (format: /host/path:/container/path)
- `--stdio`: Run in STDIO mode for direct pipe communication

**Examples:**
```bash
# Run a container from the current directory
finch-mcp run .

# Run a specific container image
finch-mcp run my-mcp-server:latest

# Run with a specific port and environment variables
finch-mcp run ./my-mcp-server -p 5000 -e NODE_ENV=production -e DEBUG=true
```

### 3. Dev

Runs an MCP server in development mode with live reloading:

```bash
finch-mcp dev <path> [options]
```

**Arguments:**
- `<path>`: Path to the MCP server repository (required)

**Options:**
- `-p, --port <port>`: Port to expose the MCP server on (default: "3000")
- `-e, --env <env...>`: Environment variables to pass to the container

**Examples:**
```bash
# Run development mode for the current directory
finch-mcp dev .

# Run development mode with a specific port
finch-mcp dev ./my-mcp-server -p 5000

# Run development mode with environment variables
finch-mcp dev . -e NODE_ENV=development -e DEBUG=true
```

### 4. Publish

Publishes an MCP server container to a registry:

```bash
finch-mcp publish <path> [options]
```

**Arguments:**
- `<path>`: Path to the MCP server repository (required)

**Options:**
- `-r, --registry <registry>`: Container registry to publish to
- `-t, --tag <tag>`: Tag for the container image (default: "latest")
- `-n, --name <name>`: Name for the container image (default: repository directory name)

**Examples:**
```bash
# Publish a container from the current directory (interactive prompt for registry)
finch-mcp publish .

# Publish to a specific registry
finch-mcp publish ./my-mcp-server -r docker.io/username

# Publish with a specific name and tag
finch-mcp publish . -r docker.io/username -n my-mcp-server -t v1.0.0
```

## Technical Architecture

Finch-MCP is built with a modular architecture consisting of several key components:

### 1. MCP Server Detection

The `McpDetector` class (`mcp-detector.ts`) is responsible for identifying MCP servers in repositories by:

- Analyzing `package.json` for MCP-related dependencies
- Locating the server entry point file
- Determining the server type (Express, Fastify, or custom)
- Detecting the port configuration

Supported server types:
- **Express**: Node.js web framework commonly used for MCP servers
- **Fastify**: High-performance web framework alternative to Express
- **Custom**: Other custom server implementations

### 2. Package Manager Support

The `package-manager.ts` module provides functionality for:

- Detecting the package manager used in a project (npm or uv)
- Checking for lock files to determine the appropriate configuration
- Generating appropriate install and run commands

Supported package managers:
- **npm**: The default Node.js package manager
- **uv**: A faster alternative to npm, with compatible commands and improved performance

### 3. Containerization Process

The containerization process involves:

1. **Detection**: Identifying the MCP server and its configuration
2. **Dockerfile Generation**: Creating a Dockerfile based on the server type and package manager
3. **Build Context Preparation**: Setting up the files needed for the container build
4. **Finch Integration**: Using Finch to build, run, and manage containers

### 4. Finch Client

The `FinchClient` class (`finch-client.ts`) provides a wrapper around the Finch CLI with methods for:

- **Building Images**: Creating container images with appropriate tags and options
- **Running Containers**: Executing containers with port mappings, environment variables, and volumes
- **Publishing Images**: Pushing images to container registries

## Development Workflow

### Standard Development Workflow

1. **Create your MCP server**: Develop a Node.js MCP server with the necessary dependencies
2. **Containerize the server**: Run `finch-mcp create .` in your server directory
3. **Test the container**: Use `finch-mcp run .` to ensure it works correctly
4. **Distribute the container**: Publish the container with `finch-mcp publish .`

### Development Mode Workflow

1. **Start development mode**: Run `finch-mcp dev .` in your server directory
2. **Make changes**: Edit your code, and the server will automatically reload
3. **Monitor logs**: View server output and debug information in the console
4. **Test**: Test your server via the exposed port

## Container Configuration

The tool generates appropriate Dockerfiles based on the detected package manager:

### npm-based Dockerfile

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

EXPOSE ${PORT}

CMD ["node", "${ENTRY_POINT}"]
```

### uv-based Dockerfile

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json uv-lock.json* ./
RUN npm install -g uv && uv install

COPY . .

EXPOSE ${PORT}

CMD ["node", "${ENTRY_POINT}"]
```

## Advanced Usage

### Using with npx/uvx

You can run Finch-MCP directly with npx or uvx without installing it globally:

```bash
npx finch-mcp create ./my-mcp-server
```

### CI/CD Integration

In CI/CD environments, use the non-interactive mode by providing all required options:

```bash
finch-mcp publish ./my-mcp-server -r ${REGISTRY_URL} -t ${VERSION} -n ${IMAGE_NAME}
```

### Custom Port Configuration

By default, Finch-MCP attempts to detect the port your MCP server uses by analyzing the code. If it cannot detect the port or you want to use a different port, specify it with the `-p` option:

```bash
finch-mcp run . -p 5000
```

### Environment Variables

Pass environment variables to your MCP server container:

```bash
finch-mcp run . -e API_KEY=my-api-key -e NODE_ENV=production
```

### Volume Mounts

When using the `run` command, you can mount host directories as volumes:

```bash
finch-mcp run . -v ./data:/app/data -v ./logs:/app/logs
```

## Troubleshooting

### Finch Not Installed

If you see an error indicating Finch is not installed, make sure to:
1. Install Finch from [https://runfinch.com/](https://runfinch.com/)
2. Ensure Finch is in your PATH
3. Verify Finch is working by running `finch version`

### Container Build Failures

Common causes for build failures:
1. Missing dependencies in your MCP server
2. Unsupported Node.js version
3. Issues with the repository structure

Solution: Check the error message and ensure your project has all necessary dependencies and is properly structured.

### Container Run Failures

If your container fails to run:
1. Check the server logs for error messages
2. Verify port configurations are correct
3. Ensure your MCP server starts properly outside of the container

### MCP Server Not Detected

If Finch-MCP fails to detect your MCP server:
1. Make sure your server implements the MCP protocol
2. Check that dependencies related to MCP are properly installed
3. Verify the server entry point exists and is correctly identified

## Best Practices

1. **Proper Dependencies**: Ensure all dependencies are listed in your package.json
2. **Port Configuration**: Use environment variables for port configuration (e.g., `process.env.PORT || 3000`)
3. **Clear Entry Point**: Have a clearly defined entry point for your MCP server
4. **Environment Variables**: Use environment variables for configuration rather than hardcoded values
5. **Version Control**: Tag your containers with meaningful versions that follow semantic versioning

## Conclusion

Finch-MCP provides a streamlined workflow for containerizing, developing, and distributing MCP servers. By leveraging Finch for container management and supporting both npm and uv package managers, it offers flexibility and efficiency for AI/ML model server deployment.

For further assistance, refer to the [documentation](./README.md) or open an issue on the GitHub repository.