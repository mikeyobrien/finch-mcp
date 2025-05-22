# Troubleshooting Guide

This guide helps you resolve common issues with Finch-MCP.

## Common Issues

### Finch Not Found

**Error**: `Finch is not installed or not available`

**Solution**:
1. Install Finch from [runfinch.com](https://runfinch.com/)
2. Verify installation: `finch --version`
3. Ensure Finch is in your PATH

### Finch VM Not Running

**Error**: `Finch VM is not running`

**Solution**:
```bash
# Check VM status
finch vm status

# Start VM manually
finch vm start

# Or let finch-mcp handle it
finch-mcp run uvx mcp-server-time
```

### First-Time VM Initialization Slow

**Symptom**: First run takes several minutes

**Explanation**: This is normal. Finch needs to:
- Download the VM image
- Initialize the virtual machine
- Set up container runtime

**Solution**: Be patient. Subsequent runs will be fast.

### MCP Inspector Timeout

**Error**: `Failed to connect to MCP server: Request timed out`

**Cause**: Container build time exceeds inspector timeout

**Solutions**:

1. **Pre-build the container**:
   ```bash
   # Build first
   finch-mcp run ./my-mcp-server
   
   # Get image name
   finch-mcp cache list
   
   # Use with inspector
   MCP_STDIO=1 npx @modelcontextprotocol/inspector \
     --cli finch-mcp \
     -- run --direct mcp-local-nodejs-myserver-abc123
   ```

2. **Use direct execution** (for testing):
   ```bash
   # Run directly without containerization
   MCP_STDIO=1 npx @modelcontextprotocol/inspector \
     --cli node \
     -- ./my-mcp-server/dist/index.js
   ```

### Build Failures

**Error**: `Container build failed`

**Common Causes & Solutions**:

1. **Missing dependencies**:
   ```bash
   # Check package.json or requirements.txt
   # Ensure all dependencies are listed
   ```

2. **Build script errors**:
   ```bash
   # Test build locally first
   npm run build  # or equivalent
   ```

3. **Wrong Node/Python version**:
   ```json
   // package.json
   "engines": {
     "node": ">=18.0.0"
   }
   ```

4. **View detailed logs**:
   ```bash
   finch-mcp run -V ./my-project
   finch-mcp logs show
   ```

### Cache Issues

**Symptom**: Using outdated container despite code changes

**Solutions**:

1. **Clear specific cache**:
   ```bash
   finch-mcp cache list
   finch-mcp cache clear mcp-local-nodejs-myproject-abc123
   ```

2. **Clear all cache**:
   ```bash
   finch-mcp cache clear --all
   ```

3. **Force rebuild**: Make a meaningful change to force new hash

### Permission Denied Errors

**Error**: `Permission denied` when accessing files

**Solutions**:

1. **Volume mount permissions**:
   ```bash
   # Run as current user
   finch run --user $(id -u):$(id -g) -v $(pwd):/app image
   ```

2. **File ownership**:
   ```bash
   # Fix ownership
   sudo chown -R $(whoami) ./my-project
   ```

### Network Issues

**Symptom**: Can't download packages during build

**Solutions**:

1. **Use host network** (for corporate proxies):
   ```bash
   finch-mcp run --host-network ./my-project
   ```

2. **Forward registry config**:
   ```bash
   finch-mcp run --forward-registry ./my-project
   ```

3. **Set proxy variables**:
   ```bash
   finch-mcp run \
     -e HTTP_PROXY=$HTTP_PROXY \
     -e HTTPS_PROXY=$HTTPS_PROXY \
     ./my-project
   ```

### MCP Protocol Issues

**Symptom**: Server runs but doesn't respond to MCP requests

**Checks**:

1. **Verify STDIO mode**:
   ```bash
   # Should see MCP_STDIO=true in environment
   finch-mcp run -e DEBUG=true ./my-server
   ```

2. **Test manually**:
   ```bash
   # Send test request
   echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | \
     finch-mcp run ./my-server
   ```

3. **Check server implementation**:
   - Ensure server reads from stdin
   - Ensure server writes to stdout
   - No extra output to stdout

## Debugging Techniques

### Enable Verbose Logging

```bash
# Single -V for basic debug
finch-mcp run -V ./my-project

# Double -VV for detailed debug
finch-mcp run -VV ./my-project

# Triple -VVV for trace-level
finch-mcp run -VVV ./my-project
```

### Inspect Generated Dockerfile

```bash
# View the generated Dockerfile
finch-mcp run -V ./my-project 2>&1 | grep -A 50 "Generated Dockerfile"
```

### Check Container Internals

```bash
# Run shell in container
finch run -it --entrypoint sh mcp-local-nodejs-myproject-abc123

# Check environment
env | grep MCP

# Test command directly
node dist/index.js
```

### View Build Logs

```bash
# Show recent build log
finch-mcp logs show

# List all logs
finch-mcp logs list

# Show specific log
finch-mcp logs show build-2024-01-15-10-30-45.log
```

## Platform-Specific Issues

### macOS

**Issue**: Finch VM memory limits

**Solution**:
```bash
# Edit Finch configuration
~/.finch/finch.yaml

# Increase memory
memory: 4GiB
```

### Linux

**Issue**: Docker socket conflicts

**Solution**:
```bash
# Ensure Docker is not running
sudo systemctl stop docker

# Or use different socket
export FINCH_SOCKET=/var/run/finch.sock
```

### Windows

**Issue**: Path format issues

**Solution**:
```bash
# Use forward slashes
finch-mcp run -v C:/Users/me/project:/app ./my-server

# Or use Git Bash
finch-mcp run -v $(pwd):/app ./my-server
```

## Performance Issues

### Slow Container Starts

**Causes**:
- Large images
- Many dependencies
- Inefficient Dockerfiles

**Solutions**:

1. **Use multi-stage builds** (automatically done)
2. **Optimize base images**:
   ```dockerfile
   # Use slim variants
   FROM node:18-slim  # instead of node:18
   ```
3. **Check image sizes**:
   ```bash
   finch images | grep mcp
   ```

### High Memory Usage

**Monitor usage**:
```bash
finch stats
```

**Limit resources**:
```bash
# Note: Requires direct container mode
finch run --memory 512m --cpus 1 image
```

## Getting Help

### Gather Information

Before reporting issues, collect:

1. **Version information**:
   ```bash
   finch-mcp --version
   finch --version
   ```

2. **Full error output**:
   ```bash
   finch-mcp run -VV ./my-project 2>&1 | tee error.log
   ```

3. **Build logs**:
   ```bash
   finch-mcp logs show > build.log
   ```

4. **System information**:
   ```bash
   uname -a
   echo $SHELL
   ```

### Report Issues

1. Check existing issues: [GitHub Issues](https://github.com/mikeyobrien/finch-mcp/issues)
2. Create new issue with:
   - Clear problem description
   - Steps to reproduce
   - Error messages
   - System information
   - What you've tried

### Community Support

- Discord: [Finch-MCP Community](https://discord.gg/finch-mcp)
- Stack Overflow: Tag with `finch-mcp`
- GitHub Discussions: For questions and ideas

## Quick Fixes Checklist

- [ ] Finch is installed and VM is running
- [ ] Using latest version of finch-mcp
- [ ] Project has standard structure
- [ ] Dependencies are correctly specified
- [ ] No extra output to stdout in MCP mode
- [ ] Cache is not stale
- [ ] Permissions are correct for volumes
- [ ] Network access for package downloads
- [ ] Sufficient disk space for images
- [ ] No conflicting processes on ports