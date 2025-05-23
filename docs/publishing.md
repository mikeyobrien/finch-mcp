# Publishing MCP Servers

This guide explains how to publish and distribute your MCP servers using Finch-MCP.

## Overview

Publishing your MCP server involves:
1. Building an optimized container image
2. Tagging with appropriate versions
3. Pushing to a container registry
4. Sharing with users

## Preparing for Publication

### 1. Optimize Your MCP Server

Ensure your server is production-ready:

```bash
# Test thoroughly
finch-mcp run ./my-mcp-server

# Verify MCP compliance
MCP_STDIO=1 npx @modelcontextprotocol/inspector \
  --cli finch-mcp \
  -- run ./my-mcp-server
```

### 2. Create Production Dockerfile

While Finch-MCP auto-generates Dockerfiles, create a custom one for production:

```dockerfile
# Multi-stage build for smaller image
FROM node:18-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./
RUN npm ci --only=production

# Copy source
COPY . .

# Build TypeScript
RUN npm run build

# Production stage
FROM node:18-alpine

# Add dumb-init for proper signal handling
RUN apk add --no-cache dumb-init

# Create non-root user
RUN adduser -D -u 1000 mcp-user

WORKDIR /app

# Copy only necessary files
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

# Set MCP environment
ENV NODE_ENV=production
ENV MCP_STDIO=true

USER mcp-user

ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "dist/index.js"]
```

### 3. Version Management

Use semantic versioning in `package.json`:

```json
{
  "name": "@myorg/mcp-server-example",
  "version": "1.0.0",
  "description": "MCP server for example functionality"
}
```

## Building for Publication

### Build the Image

```bash
# Build with custom Dockerfile
finch build -t myorg/mcp-server-example:1.0.0 .

# Also tag as latest
finch tag myorg/mcp-server-example:1.0.0 myorg/mcp-server-example:latest
```

### Multi-Architecture Builds

Support multiple platforms:

```bash
# Build for multiple architectures
finch build \
  --platform linux/amd64,linux/arm64 \
  -t myorg/mcp-server-example:1.0.0 \
  .
```

### Optimize Image Size

Check and reduce image size:

```bash
# Check image size
finch images myorg/mcp-server-example

# Analyze layers
finch history myorg/mcp-server-example:1.0.0
```

Tips for smaller images:
- Use Alpine-based images
- Multi-stage builds
- Minimize layers
- Remove unnecessary files
- Use .dockerignore

## Publishing to Registries

### Docker Hub

```bash
# Login to Docker Hub
finch login -u yourusername

# Push to Docker Hub
finch push myorg/mcp-server-example:1.0.0
finch push myorg/mcp-server-example:latest
```

### GitHub Container Registry

```bash
# Login to GitHub
echo $GITHUB_TOKEN | finch login ghcr.io -u USERNAME --password-stdin

# Tag for GitHub
finch tag myorg/mcp-server-example:1.0.0 \
  ghcr.io/yourusername/mcp-server-example:1.0.0

# Push
finch push ghcr.io/yourusername/mcp-server-example:1.0.0
```

### AWS ECR

```bash
# Get login token
aws ecr get-login-password --region us-east-1 | \
  finch login --username AWS --password-stdin \
  123456789.dkr.ecr.us-east-1.amazonaws.com

# Tag for ECR
finch tag myorg/mcp-server-example:1.0.0 \
  123456789.dkr.ecr.us-east-1.amazonaws.com/mcp-server-example:1.0.0

# Push
finch push 123456789.dkr.ecr.us-east-1.amazonaws.com/mcp-server-example:1.0.0
```

### Private Registries

```bash
# Login to private registry
finch login registry.company.com

# Tag and push
finch tag myorg/mcp-server-example:1.0.0 \
  registry.company.com/mcp/server-example:1.0.0
finch push registry.company.com/mcp/server-example:1.0.0
```

## Creating Release Documentation

### 1. README for Users

Create clear instructions for using your published image:

```markdown
# MCP Server Example

## Quick Start

Run with Finch-MCP:
```bash
finch-mcp run --direct myorg/mcp-server-example:latest
```

## Configuration

Environment variables:
- `API_ENDPOINT`: API endpoint URL (required)
- `LOG_LEVEL`: Logging level (default: info)

## Usage with Claude Desktop

Add to your Claude configuration:
```json
{
  "mcpServers": {
    "example": {
      "command": "finch-mcp",
      "args": ["run", "--direct", "myorg/mcp-server-example:latest"],
      "env": {
        "API_ENDPOINT": "https://api.example.com"
      }
    }
  }
}
```
```

### 2. Changelog

Maintain a CHANGELOG.md:

```markdown
# Changelog

## [1.0.0] - 2024-01-15

### Added
- Initial release
- Support for X, Y, Z operations
- MCP tools: tool1, tool2, tool3

### Security
- Non-root user execution
- Minimal Alpine base image
```

### 3. MCP Manifest

Include MCP server details:

```json
{
  "name": "mcp-server-example",
  "version": "1.0.0",
  "description": "Example MCP server",
  "author": "Your Name",
  "license": "MIT",
  "mcp": {
    "tools": [
      {
        "name": "example_tool",
        "description": "Does something useful"
      }
    ],
    "resources": [],
    "prompts": []
  }
}
```

## Distribution Strategies

### 1. Container Registry Only

Simplest approach:
- Publish to Docker Hub or similar
- Users run with: `finch-mcp run --direct image:tag`

### 2. NPM Package + Container

Hybrid approach:
```json
{
  "name": "@myorg/mcp-server-example",
  "bin": {
    "mcp-server-example": "./dist/index.js"
  },
  "scripts": {
    "container:build": "finch build -t myorg/mcp-server-example .",
    "container:push": "finch push myorg/mcp-server-example"
  }
}
```

Users can choose:
- NPM: `npx @myorg/mcp-server-example`
- Container: `finch-mcp run --direct myorg/mcp-server-example`

### 3. GitHub Releases

Automated with GitHub Actions:

```yaml
name: Publish

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Set up Finch
        run: |
          brew install --cask finch
          finch vm init
          
      - name: Build image
        run: |
          finch build -t ghcr.io/${{ github.repository }}:${{ github.ref_name }} .
          
      - name: Push to GHCR
        run: |
          echo ${{ secrets.GITHUB_TOKEN }} | finch login ghcr.io -u ${{ github.actor }} --password-stdin
          finch push ghcr.io/${{ github.repository }}:${{ github.ref_name }}
```

## Testing Published Images

### Local Testing

Before publishing:

```bash
# Test the exact image users will run
finch-mcp run --direct myorg/mcp-server-example:1.0.0

# Test with different configurations
finch-mcp run --direct -e LOG_LEVEL=debug myorg/mcp-server-example:1.0.0

# Verify with MCP Inspector
MCP_STDIO=1 npx @modelcontextprotocol/inspector \
  --cli finch-mcp \
  -- run --direct myorg/mcp-server-example:1.0.0
```

### Integration Testing

Create test script:

```bash
#!/bin/bash
# test-published-image.sh

IMAGE="myorg/mcp-server-example:1.0.0"

echo "Testing $IMAGE..."

# Test 1: Basic functionality
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | \
  finch-mcp run --direct $IMAGE

# Test 2: With environment variables
finch-mcp run --direct -e API_ENDPOINT=https://test.api.com $IMAGE

# Test 3: Volume mounts
finch-mcp run --direct -v /tmp:/data:ro $IMAGE
```

## Versioning Strategy

### Semantic Versioning

Follow semver strictly:
- **MAJOR**: Breaking changes to MCP interface
- **MINOR**: New tools/features (backward compatible)
- **PATCH**: Bug fixes

### Tag Structure

```bash
# Version tags
finch tag image:latest image:1.0.0
finch tag image:latest image:1.0
finch tag image:latest image:1

# Feature tags
finch tag image:latest image:stable
finch tag image:dev image:beta
```

### Version Constraints

Document compatible versions:

```markdown
## Compatibility

- MCP Protocol: 1.0+
- Finch-MCP: 0.1.0+
- Claude Desktop: 1.5+
- Node.js: 18+ (container includes)
```

## Maintenance

### Security Updates

Regular update process:

```bash
# Update base image
finch pull node:18-alpine

# Rebuild
finch build -t myorg/mcp-server-example:1.0.1 .

# Push patch version
finch push myorg/mcp-server-example:1.0.1
finch push myorg/mcp-server-example:1.0
finch push myorg/mcp-server-example:latest
```

### Deprecation

When deprecating versions:

1. **Announce** in README and changelog
2. **Add warnings** to old versions
3. **Maintain** for grace period
4. **Remove** after notice period

### User Communication

Keep users informed:
- GitHub releases for announcements
- Changelog for details
- Issues for support
- Discussions for feedback

## Best Practices

1. **Reproducible Builds**
   - Lock dependencies
   - Specify exact base image versions
   - Document build environment

2. **Clear Documentation**
   - Installation instructions
   - Configuration options
   - Usage examples
   - Troubleshooting guide

3. **Consistent Naming**
   - Follow registry conventions
   - Use descriptive names
   - Include "mcp" in name

4. **Security First**
   - Regular updates
   - Vulnerability scanning
   - Non-root execution
   - Minimal permissions

5. **User Experience**
   - Fast startup times
   - Clear error messages
   - Sensible defaults
   - Easy configuration