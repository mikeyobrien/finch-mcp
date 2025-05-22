# Development Mode

This guide covers using Finch-MCP for MCP server development, including live reloading, debugging, and testing.

## Overview

Development mode allows you to:
- Run MCP servers with live code reloading
- Mount local source code into containers
- Debug containerized MCP servers
- Test with different environments

## Basic Development Setup

### Running with Source Code Mounted

Mount your local development directory:

```bash
finch-mcp run -v $(pwd):/app ./my-mcp-server
```

This allows you to edit code locally and see changes reflected in the container.

### Live Reloading

#### Node.js with nodemon

```bash
# Install nodemon in your project
npm install --save-dev nodemon

# Add to package.json scripts
"scripts": {
  "dev": "nodemon --watch src --exec node src/index.js"
}

# Run with Finch-MCP
finch-mcp run -v $(pwd):/app npm run dev
```

#### Python with watchdog

```bash
# Install watchdog
pip install watchdog

# Create a dev script
finch-mcp run -v $(pwd):/app python -m watchdog.auto_restart -d . -p "*.py" -- python server.py
```

## Environment-Specific Development

### Development Environment Variables

Create a `.env.development` file:
```env
DEBUG=true
LOG_LEVEL=debug
MCP_DEV_MODE=true
```

Run with environment file:
```bash
finch-mcp run -v $(pwd):/app -e DEBUG=true -e LOG_LEVEL=debug ./my-mcp-server
```

### Multiple Environment Configurations

```bash
# Development
finch-mcp run -e ENV=development ./my-mcp-server

# Staging
finch-mcp run -e ENV=staging ./my-mcp-server

# Production-like
finch-mcp run -e ENV=production ./my-mcp-server
```

## Debugging Techniques

### Enable Verbose Logging

```bash
# Finch-MCP verbose mode
finch-mcp run -VV ./my-mcp-server

# Pass debug flag to your server
finch-mcp run -e DEBUG=true ./my-mcp-server
```

### Interactive Debugging

#### Node.js Debugging

```bash
# Expose debugger port
finch-mcp run -v $(pwd):/app node --inspect=0.0.0.0:9229 src/index.js
```

Then connect with your IDE's debugger to `localhost:9229`.

#### Python Debugging

```bash
# Using pdb
finch-mcp run -v $(pwd):/app python -m pdb server.py

# Using debugpy for remote debugging
finch-mcp run -v $(pwd):/app -e DEBUGPY_PORT=5678 python -m debugpy --listen 0.0.0.0:5678 server.py
```

### Viewing Container Logs

```bash
# View recent build logs
finch-mcp logs show

# List all build logs
finch-mcp logs list

# Clear old logs
finch-mcp logs clear --older-than 7d
```

## Testing MCP Servers

### Using MCP Inspector

Test your containerized MCP server with the official inspector:

```bash
# Build your server first
finch-mcp run ./my-mcp-server

# Get the image name from cache
finch-mcp cache list

# Test with inspector
MCP_STDIO=1 npx @modelcontextprotocol/inspector \
  --cli finch-mcp \
  --method tools/list \
  -- run --direct mcp-local-nodejs-myserver-abc123
```

### Integration Testing

Create test scripts that run against your containerized server:

```bash
#!/bin/bash
# test-mcp-server.sh

# Start server in background
finch-mcp run ./my-mcp-server &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Run tests
npm test

# Cleanup
kill $SERVER_PID
```

### Unit Testing in Containers

Run tests inside the container environment:

```bash
# Node.js
finch-mcp run -v $(pwd):/app npm test

# Python
finch-mcp run -v $(pwd):/app pytest tests/
```

## Development Workflow

### 1. Initial Setup

```bash
# Clone or create your MCP server project
git clone https://github.com/username/my-mcp-server
cd my-mcp-server

# Test containerization works
finch-mcp run .

# Set up development mode
finch-mcp run -v $(pwd):/app npm run dev
```

### 2. Development Cycle

1. **Edit code** in your local editor
2. **Changes auto-reload** in the container
3. **Test with MCP client** or inspector
4. **Debug issues** using logs and debugger
5. **Commit changes** when ready

### 3. Pre-Production Testing

```bash
# Test without volume mount (production-like)
finch-mcp run .

# Test with production environment
finch-mcp run -e NODE_ENV=production .

# Verify image size and performance
finch images | grep mcp-local
```

## Advanced Development Features

### Custom Development Containers

Create a `Dockerfile.dev` for development-specific features:

```dockerfile
FROM node:18

# Install development tools
RUN npm install -g nodemon typescript @types/node

WORKDIR /app

# Development-specific environment
ENV NODE_ENV=development
ENV DEBUG=*

# Keep container running for exec
CMD ["tail", "-f", "/dev/null"]
```

Build and run:
```bash
finch build -f Dockerfile.dev -t my-mcp-dev .
finch-mcp run --direct -v $(pwd):/app my-mcp-dev
```

### Multi-Service Development

For MCP servers that depend on other services:

```bash
# Start database
finch run -d --name postgres postgres:15

# Run MCP server with link
finch-mcp run -e DATABASE_URL=postgresql://postgres:5432/mydb ./my-mcp-server
```

### Performance Profiling

#### Node.js Profiling

```bash
# Run with profiler
finch-mcp run -v $(pwd):/app node --prof src/index.js

# Analyze profile
finch-mcp run -v $(pwd):/app node --prof-process isolate-*.log
```

#### Python Profiling

```bash
# Using cProfile
finch-mcp run -v $(pwd):/app python -m cProfile -o profile.stats server.py

# Analyze with snakeviz
pip install snakeviz
snakeviz profile.stats
```

## Best Practices

### 1. Development vs Production

- Use volume mounts only in development
- Test without mounts before deployment
- Keep development dependencies separate
- Use multi-stage builds for production

### 2. Debugging

- Always check build logs first
- Use verbose mode for troubleshooting
- Keep logs clean with structured logging
- Test with MCP inspector regularly

### 3. Performance

- Monitor container resource usage
- Optimize Dockerfile for faster builds
- Use .dockerignore to exclude unnecessary files
- Cache dependencies properly

### 4. Security

- Don't commit secrets to repository
- Use environment variables for config
- Run as non-root user in production
- Regularly update base images

## Common Development Issues

### Issue: Changes Not Reflected

**Solution**: Ensure volume mount is correct:
```bash
# Use absolute paths
finch-mcp run -v $(pwd):/app ./server

# Check mount worked
finch-mcp run -v $(pwd):/app ls -la /app
```

### Issue: Permission Errors

**Solution**: Match container user with host:
```bash
# Run as current user
finch-mcp run -v $(pwd):/app --user $(id -u):$(id -g) ./server
```

### Issue: Slow Rebuilds

**Solution**: Use cache effectively:
```bash
# Check what's triggering rebuilds
finch-mcp run -V .

# Exclude unnecessary files
echo "node_modules/" >> .dockerignore
echo "*.log" >> .dockerignore
```

### Issue: Can't Connect Debugger

**Solution**: Expose ports properly:
```bash
# Note: Port exposure requires direct container mode
finch run -p 9229:9229 -v $(pwd):/app my-mcp-dev
```

## Next Steps

- Learn about [publishing](./publishing.md) your MCP server
- Read about [security best practices](./security.md)
- Explore [advanced commands](./commands.md)