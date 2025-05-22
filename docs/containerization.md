# Containerization in Finch-MCP

This document explains how Finch-MCP containerizes different types of MCP servers and projects.

## Overview

Finch-MCP supports four containerization modes:

1. **Direct Container Mode**: Run existing Docker/OCI images
2. **Auto-Containerization**: Automatically containerize commands
3. **Local Directory Mode**: Containerize local projects
4. **Git Repository Mode**: Clone and containerize Git repos

## Auto-Containerization

### How It Works

When you run a command like `finch-mcp run uvx mcp-server-time`, Finch-MCP:

1. **Detects the command type** (uvx, npm, npx, pip, python)
2. **Generates an optimized Dockerfile** for that command type
3. **Builds the container** with proper dependencies
4. **Runs the command** inside the container with MCP environment

### Supported Commands

#### Python/UV Commands
```bash
# UV package runner
finch-mcp run uvx package-name

# Python modules
finch-mcp run python -m module_name

# Pip installations
finch-mcp run pip install package && python script.py
```

Generated Dockerfile example:
```dockerfile
FROM python:3.11-slim
RUN pip install --no-cache-dir uv
WORKDIR /app
ENV MCP_ENABLED=true
ENV MCP_STDIO=true
ENTRYPOINT ["uvx", "package-name"]
```

#### Node.js/npm Commands
```bash
# NPX runner
finch-mcp run npx @scope/package

# NPM scripts
finch-mcp run npm run start

# Direct Node execution
finch-mcp run node server.js
```

Generated Dockerfile example:
```dockerfile
FROM node:18-alpine
RUN apk add --no-cache dumb-init
WORKDIR /app
ENV MCP_ENABLED=true
ENV MCP_STDIO=true
ENTRYPOINT ["dumb-init", "--", "npx", "@scope/package"]
```

## Local Directory Containerization

### Project Detection

Finch-MCP automatically detects project types by looking for:

#### Node.js Projects
- `package.json` file
- Detects Node version from `engines` field
- Identifies entry point from `bin` or `main` fields
- Supports TypeScript (builds automatically)
- Detects package manager (npm, yarn, pnpm)

#### Python Projects
- `pyproject.toml` (Poetry/UV projects)
- `setup.py` (setuptools)
- `requirements.txt` (pip)
- Detects Python version requirements
- Identifies entry points

#### Rust Projects
- `Cargo.toml` file
- Builds in release mode
- Supports binary crates

### Generated Dockerfile Structure

#### Node.js Project Example
```dockerfile
FROM node:18-slim

WORKDIR /app

# Copy dependency files first (for layer caching)
COPY package*.json ./
RUN npm ci --only=production

# Copy source code
COPY . .

# Build TypeScript if needed
RUN npm run build

# Set MCP environment
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the server
CMD ["node", "dist/index.js"]
```

#### Python Project Example (Poetry)
```dockerfile
FROM python:3.11-slim

# Install Poetry
RUN pip install poetry

WORKDIR /app

# Copy dependency files
COPY pyproject.toml poetry.lock ./
RUN poetry config virtualenvs.create false && \
    poetry install --no-interaction --no-ansi

# Copy source code
COPY . .

# Set MCP environment
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the server
CMD ["poetry", "run", "python", "-m", "mcp_server"]
```

### Monorepo Support

Finch-MCP detects and handles monorepos:

- **npm workspaces**: Detects workspace configuration
- **yarn workspaces**: Full support for Yarn workspaces
- **pnpm workspaces**: Handles pnpm-workspace.yaml
- **Python monorepos**: Limited support

Example with npm workspaces:
```bash
finch-mcp run ./my-monorepo/packages/mcp-server
```

## Git Repository Containerization

### Supported Git URLs

```bash
# HTTPS
finch-mcp run https://github.com/user/repo

# SSH
finch-mcp run git@github.com:user/repo.git

# Specific branch
finch-mcp run https://github.com/user/repo#branch-name
```

### Process Flow

1. **Clone**: Repository cloned to temporary directory
2. **Analyze**: Same as local directory mode
3. **Build**: Generate and build Dockerfile
4. **Cache**: Image cached by repository content hash
5. **Run**: Execute with MCP environment

## Caching System

### Content-Based Caching

Finch-MCP uses SHA256 hashing to create unique cache keys:

- **Commands**: Hash of command + arguments
- **Local Directories**: Hash of all source files (excluding common directories)
- **Git Repos**: Hash of commit ID + uncommitted changes

### Cache Key Components

```
mcp-{mode}-{type}-{name}-{hash}
```

Examples:
- `mcp-cmd-uvx-mcp-server-time-a1b2c3d4`
- `mcp-local-nodejs-my-project-e5f6g7h8`
- `mcp-git-python-repo-name-i9j0k1l2`

### Ignored Directories

The following are excluded from cache calculations:
- `node_modules/`
- `__pycache__/`
- `.git/`
- `dist/`
- `build/`
- `target/`
- `.venv/`
- `venv/`

## Optimization Techniques

### Multi-Stage Builds

For compiled languages, Finch-MCP uses multi-stage builds:

```dockerfile
# Build stage
FROM node:18 AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Runtime stage
FROM node:18-slim
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY --from=builder /app/dist ./dist
CMD ["node", "dist/index.js"]
```

### Layer Caching

Dependencies are installed before copying source code:
1. Copy dependency files
2. Install dependencies (cached if unchanged)
3. Copy source code
4. Build application

### Security Best Practices

- Run as non-root user when possible
- Use `dumb-init` for proper signal handling
- Minimal base images (alpine, slim variants)
- No unnecessary build tools in runtime image

## Environment Variables

### Automatic MCP Variables

Finch-MCP automatically sets:
- `MCP_ENABLED=true`
- `MCP_STDIO=true`

### Custom Variables

Pass custom environment variables:
```bash
finch-mcp run -e API_KEY=secret -e DEBUG=true ./my-server
```

## Volume Mounts

Mount local directories into containers:
```bash
finch-mcp run -v /local/data:/app/data ./my-server
```

Common use cases:
- Configuration files
- Data directories
- Development source code

## Build Process Details

### Build Output

During build, you'll see:
```
🔍 Analyzing project...
  Detected: Node.js project (npm)
  Entry point: dist/index.js
  Node version: 18

📦 Building container...
  Generated Dockerfile (450 bytes)
  Building with Finch...
  
✅ Container built successfully
  Image: mcp-local-nodejs-my-server-a1b2c3d4
  Size: 125MB
  Build time: 45s
```

### Verbose Mode

Use `-V` for detailed output:
```bash
finch-mcp run -V ./my-project
```

Shows:
- Full Dockerfile content
- Build command executed
- Detailed build logs
- Cache hit/miss information

## Troubleshooting Containerization

### Common Issues

1. **Build Failures**
   - Check project dependencies are correct
   - Ensure build scripts work locally
   - Review build logs: `finch-mcp logs show`

2. **Wrong Project Type Detection**
   - Ensure project has standard structure
   - Check for required files (package.json, etc.)
   - Use verbose mode to see detection logic

3. **Cache Issues**
   - Clear cache: `finch-mcp cache clear`
   - Force rebuild with modified source
   - Check cache stats: `finch-mcp cache stats`

### Manual Dockerfile

If automatic detection fails, create a Dockerfile in your project:
```dockerfile
FROM node:18-slim
WORKDIR /app
COPY . .
RUN npm install
ENV MCP_STDIO=true
CMD ["node", "server.js"]
```

Then run with direct mode:
```bash
finch build -t my-mcp-server .
finch-mcp run --direct my-mcp-server
```