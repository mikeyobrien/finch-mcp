# Security Best Practices

This guide covers security considerations when using Finch-MCP to containerize and run MCP servers.

## Overview

Finch-MCP prioritizes security by:
- Running containers with minimal privileges
- Using secure base images
- Implementing proper signal handling
- Avoiding root user execution
- Isolating container environments

## Container Security

### Non-Root Execution

Finch-MCP containers run as non-root users by default:

```dockerfile
# Automatically added to generated Dockerfiles
RUN adduser -D -u 1000 mcp-user
USER mcp-user
```

For custom containers:
```bash
# Specify user at runtime
finch run --user 1000:1000 my-mcp-image
```

### Signal Handling

Proper signal handling prevents zombie processes:

```dockerfile
# Node.js containers use dumb-init
RUN apk add --no-cache dumb-init
ENTRYPOINT ["dumb-init", "--"]
```

This ensures:
- Clean process termination
- Proper signal propagation
- No orphaned processes

### Base Image Selection

Finch-MCP uses minimal base images:

- **Node.js**: `node:XX-slim` or `node:XX-alpine`
- **Python**: `python:XX-slim` or `python:XX-alpine`
- **General**: Alpine-based images when possible

Benefits:
- Smaller attack surface
- Fewer vulnerabilities
- Faster updates
- Reduced image size

## Secrets Management

### Environment Variables

**DO NOT** hardcode secrets:

```bash
# Bad - Secret in command
finch-mcp run -e API_KEY=sk-1234567890 ./server

# Good - Secret from environment
export API_KEY=sk-1234567890
finch-mcp run -e API_KEY ./server

# Better - Secret from file
finch-mcp run --env-file .env.production ./server
```

### Secret Files

For file-based secrets:

```bash
# Mount secret files read-only
finch-mcp run -v /path/to/secrets:/secrets:ro ./server
```

Best practices:
- Never commit secrets to Git
- Use `.gitignore` for secret files
- Rotate secrets regularly
- Use minimal permissions

### Build-Time Secrets

Avoid secrets in build process:

```dockerfile
# Bad - Secret in image
RUN echo "API_KEY=secret" > .env

# Good - Runtime injection
ENV API_KEY_PATH=/run/secrets/api_key
```

## Network Security

### Default Network Isolation

Containers run in isolated networks by default:
- No access to host network
- No access to other containers
- Only exposed ports are accessible

### Host Network Mode

Use with caution:
```bash
# Only when absolutely necessary
finch-mcp run --host-network ./server
```

Risks:
- Container can access all host ports
- Reduced network isolation
- Potential for port conflicts

### Registry Security

#### Private Registries

```bash
# Login to private registry
finch login registry.company.com

# Use private images
finch-mcp run --direct registry.company.com/mcp/server:latest
```

#### Registry Configuration

Forward secure registry configs:
```bash
finch-mcp run --forward-registry ./server
```

This safely forwards:
- npm registry settings
- pip index URLs
- Poetry sources

## File System Security

### Read-Only Containers

Make containers read-only when possible:

```bash
# Note: Requires direct container mode
finch run --read-only my-mcp-image
```

### Volume Mount Security

#### Restrict Mount Paths

```bash
# Specific directory only
finch-mcp run -v /data/mcp:/app/data:ro ./server

# Never mount system directories
# Bad: -v /:/host
# Bad: -v /etc:/config
```

#### Permission Considerations

```bash
# Match user IDs to avoid permission issues
finch-mcp run --user $(id -u):$(id -g) -v $(pwd):/app ./server
```

### Temporary Files

Use proper temp directories:

```dockerfile
# Create app-specific temp directory
RUN mkdir -p /tmp/mcp-server && \
    chown mcp-user:mcp-user /tmp/mcp-server
ENV TMPDIR=/tmp/mcp-server
```

## Supply Chain Security

### Dependency Verification

#### Node.js Projects

```json
// package-lock.json ensures reproducible builds
{
  "lockfileVersion": 2,
  "requires": true,
  "dependencies": {
    // Locked versions with integrity hashes
  }
}
```

#### Python Projects

```toml
# Use poetry.lock or requirements.txt with hashes
[[package]]
name = "package"
version = "1.0.0"
[package.hashes]
sha256 = "..."
```

### Image Scanning

Scan built images for vulnerabilities:

```bash
# Build image
finch-mcp run ./server

# Scan with Grype (example)
grype $(finch-mcp cache list | grep my-server)
```

### Update Strategy

Keep dependencies updated:

```bash
# Regular updates
npm update  # or pip-compile --upgrade

# Rebuild containers
finch-mcp cache clear --all
finch-mcp run ./server
```

## Runtime Security

### Resource Limits

Prevent resource exhaustion:

```bash
# Note: Requires direct container mode
finch run \
  --memory 512m \
  --cpus 0.5 \
  --pids-limit 100 \
  my-mcp-image
```

### Security Policies

#### Seccomp Profiles

Use default seccomp profile (automatically applied).

For custom profiles:
```bash
finch run --security-opt seccomp=profile.json image
```

#### AppArmor/SELinux

Finch respects system security modules:
- AppArmor profiles on Ubuntu/Debian
- SELinux contexts on RHEL/Fedora

### Capabilities

Drop unnecessary capabilities:

```dockerfile
# In Dockerfile
USER mcp-user
# Automatically drops most capabilities
```

## Audit and Compliance

### Logging

Enable comprehensive logging:

```bash
# Verbose mode for audit trails
finch-mcp run -VV ./server 2>&1 | tee audit.log

# Review build logs
finch-mcp logs list
finch-mcp logs show
```

### Image Provenance

Track image sources:

```bash
# List images with details
finch-mcp cache list --verbose

# Shows:
# - Build timestamp
# - Source directory/command
# - Content hash
```

### Compliance Checklist

- [ ] No hardcoded secrets
- [ ] Non-root user execution
- [ ] Minimal base images
- [ ] Updated dependencies
- [ ] Read-only file systems (where possible)
- [ ] Network isolation
- [ ] Resource limits
- [ ] Audit logging enabled

## Security Incident Response

### Detecting Issues

Monitor for:
- Unexpected network connections
- Unusual resource usage
- Modified files in containers
- Failed authentication attempts

### Response Steps

1. **Isolate affected containers**:
   ```bash
   finch stop <container>
   finch-mcp cleanup --all
   ```

2. **Investigate**:
   ```bash
   # Check running processes
   finch top <container>
   
   # Review logs
   finch logs <container>
   finch-mcp logs show
   ```

3. **Clean up**:
   ```bash
   # Remove compromised images
   finch-mcp cache clear --all
   
   # Rebuild with updated dependencies
   finch-mcp run ./server
   ```

## Security Tools Integration

### Static Analysis

Before containerizing:

```bash
# Node.js
npm audit
npx eslint --ext .js,.ts .

# Python
pip-audit
bandit -r .
```

### Container Scanning

After building:

```bash
# Get image name
IMAGE=$(finch-mcp cache list | grep my-server | awk '{print $1}')

# Scan with Trivy
finch run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image $IMAGE
```

### Runtime Protection

Consider runtime security tools:
- Falco for runtime monitoring
- Open Policy Agent for policy enforcement
- Network policies for traffic control

## Best Practices Summary

1. **Principle of Least Privilege**
   - Run as non-root
   - Drop capabilities
   - Minimal file system access

2. **Defense in Depth**
   - Multiple security layers
   - Assume breach mindset
   - Regular updates

3. **Secure by Default**
   - Safe defaults in generated Dockerfiles
   - Automatic security features
   - Clear security warnings

4. **Transparency**
   - Audit logging
   - Image provenance
   - Reproducible builds

5. **Continuous Improvement**
   - Regular security updates
   - Vulnerability scanning
   - Security training