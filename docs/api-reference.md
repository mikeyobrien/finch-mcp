# API Reference

This document provides a complete reference for the Finch-MCP CLI commands and options.

## Global Options

These options are available for all commands:

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--env KEY=VALUE` | `-e` | Set environment variables | None |
| `--volume HOST:CONTAINER` | `-v` | Mount volumes | None |
| `--verbose` | `-V` | Enable verbose logging (repeat for more) | Off |
| `--direct` | | Skip auto-containerization | False |
| `--host-network` | | Use host network | False |
| `--forward-registry` | | Forward registry configuration | False |

## Commands

### `finch-mcp run`

Run an MCP server in a container.

#### Synopsis

```bash
finch-mcp run [OPTIONS] <TARGET> [ARGS...]
```

#### Arguments

- `<TARGET>`: MCP server image, command, git repository URL, or local directory
- `[ARGS...]`: Additional arguments passed to the command

#### Options

All global options plus command-specific behavior.

#### Examples

```bash
# Run a command
finch-mcp run uvx mcp-server-time

# Run a local directory
finch-mcp run ./my-mcp-server

# Run a Git repository
finch-mcp run https://github.com/user/mcp-server

# Run with environment variables
finch-mcp run -e API_KEY=secret uvx my-server

# Run with volume mount
finch-mcp run -v /data:/app/data ./server

# Run existing container image
finch-mcp run --direct my-image:latest
```

#### Target Detection

The `run` command automatically detects the target type:

1. **Container Image**: Contains `:` or registry pattern
2. **Git Repository**: Starts with `http://`, `https://`, or `git@`
3. **Local Directory**: Exists on filesystem and is a directory
4. **Command**: Everything else is treated as a command to containerize

### `finch-mcp list`

List MCP-related containers and images.

#### Synopsis

```bash
finch-mcp list [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--all` | Show all containers/images (not just MCP) | False |

#### Examples

```bash
# List MCP containers and images
finch-mcp list

# List all containers and images
finch-mcp list --all
```

#### Output Format

```
CONTAINERS:
ID          IMAGE                                    STATUS
abc123      mcp-local-nodejs-server-a1b2c3          Running
def456      mcp-cmd-uvx-time-server-d4e5f6          Exited

IMAGES:
REPOSITORY                               TAG      SIZE
mcp-local-nodejs-server-a1b2c3          latest   125MB
mcp-cmd-uvx-time-server-d4e5f6          latest   89MB
```

### `finch-mcp cleanup`

Remove MCP containers and images.

#### Synopsis

```bash
finch-mcp cleanup [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--all` | Remove all MCP resources | False |
| `--containers` | Remove only containers | False |
| `--images` | Remove only images | False |
| `--force` | Force removal | False |

#### Examples

```bash
# Interactive cleanup (asks for confirmation)
finch-mcp cleanup

# Remove all MCP resources
finch-mcp cleanup --all

# Remove only containers
finch-mcp cleanup --containers

# Force remove images
finch-mcp cleanup --images --force
```

### `finch-mcp cache`

Manage the build cache.

#### Synopsis

```bash
finch-mcp cache <SUBCOMMAND>
```

#### Subcommands

##### `cache list`

List cached images with details.

```bash
finch-mcp cache list [OPTIONS]
```

Options:
- `--verbose`: Show detailed information

Output:
```
Image: mcp-local-nodejs-myserver-a1b2c3d4
  Type: Local Directory
  Created: 2024-01-15 10:30:45
  Size: 125MB
  Source: /home/user/my-server
  Hash: a1b2c3d4...
```

##### `cache stats`

Show cache statistics.

```bash
finch-mcp cache stats
```

Output:
```
Cache Statistics:
  Total Images: 15
  Total Size: 1.2GB
  Oldest: 2024-01-01
  Newest: 2024-01-15
  
By Type:
  Commands: 5 (400MB)
  Local Projects: 8 (700MB)
  Git Repos: 2 (100MB)
```

##### `cache clear`

Clear cached images.

```bash
finch-mcp cache clear [OPTIONS] [IMAGE]
```

Options:
- `--all`: Clear entire cache
- `--older-than DAYS`: Clear images older than specified days

Examples:
```bash
# Clear specific image
finch-mcp cache clear mcp-local-nodejs-server-a1b2c3

# Clear all cache
finch-mcp cache clear --all

# Clear old images
finch-mcp cache clear --older-than 30
```

### `finch-mcp logs`

Manage build logs.

#### Synopsis

```bash
finch-mcp logs <SUBCOMMAND>
```

#### Subcommands

##### `logs list`

List available build logs.

```bash
finch-mcp logs list
```

Output:
```
Build Logs:
  build-2024-01-15-10-30-45.log (125KB)
  build-2024-01-15-09-15-22.log (89KB)
  build-2024-01-14-16-45-00.log (156KB)
```

##### `logs show`

Display a build log.

```bash
finch-mcp logs show [LOG_FILE]
```

Examples:
```bash
# Show most recent log
finch-mcp logs show

# Show specific log
finch-mcp logs show build-2024-01-15-10-30-45.log
```

##### `logs clear`

Remove old build logs.

```bash
finch-mcp logs clear [OPTIONS]
```

Options:
- `--all`: Remove all logs
- `--older-than DAYS`: Remove logs older than specified days

## Environment Variables

### MCP-Specific

| Variable | Description | Set By |
|----------|-------------|--------|
| `MCP_STDIO` | Enable STDIO mode | Automatically |
| `MCP_ENABLED` | Mark as MCP server | Automatically |

### Finch-MCP Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `FINCH_MCP_CACHE_DIR` | Cache directory | `~/.cache/finch-mcp` |
| `FINCH_MCP_LOG_DIR` | Log directory | `~/.local/share/finch-mcp/logs` |
| `FINCH_MCP_NO_CACHE` | Disable caching | False |
| `FINCH_MCP_DEBUG` | Debug mode | False |

### Build Behavior

| Variable | Description | Default |
|----------|-------------|---------|
| `HTTP_PROXY` | HTTP proxy for builds | System default |
| `HTTPS_PROXY` | HTTPS proxy for builds | System default |
| `NO_PROXY` | Proxy exceptions | System default |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Finch not available |
| 4 | Build failed |
| 5 | Container failed to start |
| 130 | Interrupted (Ctrl+C) |

## Cache Structure

### Cache Directory Layout

```
~/.cache/finch-mcp/
├── metadata.json
├── images/
│   ├── mcp-local-nodejs-server-a1b2c3d4.json
│   └── mcp-cmd-uvx-time-d5e6f7g8.json
└── temp/
    └── git-clones/
```

### Metadata Format

```json
{
  "version": "1.0",
  "images": {
    "mcp-local-nodejs-server-a1b2c3d4": {
      "created": "2024-01-15T10:30:45Z",
      "source_type": "local",
      "source_path": "/home/user/my-server",
      "hash": "a1b2c3d4...",
      "size": 125000000,
      "last_used": "2024-01-15T14:22:10Z"
    }
  }
}
```

## Image Naming Convention

### Format

```
mcp-{source}-{type}-{name}-{hash}
```

### Components

- `source`: Source type (`local`, `cmd`, `git`)
- `type`: Project/command type (`nodejs`, `python`, `uvx`, `npm`)
- `name`: Sanitized name from project/command
- `hash`: First 8 characters of content hash

### Examples

- `mcp-local-nodejs-myserver-a1b2c3d4`
- `mcp-cmd-uvx-time-server-e5f6g7h8`
- `mcp-git-python-example-i9j0k1l2`

## Project Detection

### Node.js Detection

Files checked (in order):
1. `package.json`
2. `yarn.lock` (implies Yarn)
3. `pnpm-lock.yaml` (implies pnpm)
4. `package-lock.json` (implies npm)

Extracted information:
- `name`: From package.json
- `version`: From package.json
- `engines.node`: Node.js version
- `main`/`bin`: Entry point
- `workspaces`: Monorepo detection

### Python Detection

Files checked (in order):
1. `pyproject.toml` (Poetry/UV)
2. `setup.py` (Setuptools)
3. `requirements.txt` (pip)
4. `Pipfile` (pipenv)

Extracted information:
- Project name and version
- Python version requirements
- Entry points/scripts
- Dependencies

### Command Detection

Patterns recognized:
- `uvx <package>`: UV package runner
- `npx <package>`: NPM package runner
- `npm run <script>`: NPM scripts
- `python -m <module>`: Python modules
- `pip install && python`: Pip workflows

## Error Messages

### Common Errors

#### Finch Not Available

```
Error: Finch is not installed or not available
Please install Finch from https://runfinch.com/
```

#### Build Failed

```
Error: Container build failed
Check build logs: finch-mcp logs show
Common causes:
- Missing dependencies in package.json
- Build script errors
- Network issues downloading packages
```

#### Invalid Target

```
Error: Cannot determine target type: <target>
Target should be one of:
- Container image (e.g., image:tag)
- Git URL (e.g., https://github.com/user/repo)
- Local directory (e.g., ./my-server)
- Command (e.g., uvx package-name)
```

## Debugging

### Verbose Levels

- `-V`: Basic debug information
  - Mode detection logic
  - Cache hit/miss
  - Build commands
  
- `-VV`: Detailed debug information
  - Full Dockerfile content
  - Environment variables
  - Finch commands executed
  
- `-VVV`: Trace-level information
  - File hashing details
  - All shell commands
  - Complete build output