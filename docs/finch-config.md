# .finch-mcp Configuration File

The `.finch-mcp` configuration file allows you to customize how finch-mcp builds and runs your MCP server. This is especially useful for projects that need their devDependencies during the build phase.

## File Format

The configuration file can be in YAML, JSON, or TOML format. The file should be named:
- `.finch-mcp` (any format)
- `.finch-mcp.yaml` or `.finch-mcp.yml`
- `.finch-mcp.json`
- `.finch-mcp.toml`

## Configuration Options

### dependencies

Controls how dependencies are installed.

```yaml
dependencies:
  # Install all dependencies including devDependencies (default: false)
  installAll: true
  
  # Auto-detect build dependencies from package.json scripts (default: true)
  autoDetect: true
  
  # Additional dependencies to include beyond auto-detection
  include:
    - some-build-tool
    - another-tool
  
  # Dependencies to skip (overrides auto-detection)
  skip:
    - large-unused-dependency
    - another-unused-dep
  
  # Custom install command (overrides everything above)
  installCommand: "npm ci"
  
  # Commands to run before installing dependencies
  preInstall:
    - "npm install -g typescript"
    - "npm install -g @types/node"
```

### build

Controls the build process.

```yaml
build:
  # Custom build command (overrides auto-detection)
  command: "npm run build:prod"
  
  # Skip build step entirely (default: false)
  skip: false
  
  # Additional build arguments
  args:
    - "--verbose"
    - "--production"
```

### runtime

Controls how the server runs.

```yaml
runtime:
  # Custom start command (overrides auto-detection)
  command: "node dist/server.js"
  
  # Working directory inside container
  workingDir: "/app"
  
  # Additional environment variables
  env:
    NODE_ENV: "production"
    LOG_LEVEL: "info"
```

## Examples

### TypeScript Project

For a TypeScript project that needs build dependencies:

```yaml
# .finch-mcp.yaml
dependencies:
  installAll: true

build:
  command: "npm run build"

runtime:
  command: "node dist/index.js"
```

### Monorepo Project

For a monorepo that needs specific setup:

```yaml
# .finch-mcp.yaml
dependencies:
  installCommand: "npm install --workspaces"
  
build:
  command: "npm run build -w packages/mcp-server"
  
runtime:
  workingDir: "/app/packages/mcp-server"
  command: "npm start"
```

### Python Project

For a Python project with specific requirements:

```yaml
# .finch-mcp.yaml
dependencies:
  preInstall:
    - "pip install --upgrade pip"
    - "pip install poetry"
  installCommand: "poetry install"
  
runtime:
  command: "poetry run python -m mcp_server"
```

### Skip DevDependencies But Include TypeScript

If you want to keep the image small but need TypeScript for building:

```yaml
# .finch-mcp.yaml
dependencies:
  preInstall:
    - "npm install -g typescript"
    - "npm install -g @types/node"
  installCommand: "npm install --production"

build:
  command: "tsc"
```

## Best Practices

1. **Keep it Simple**: Only add configuration when the defaults don't work
2. **Document Why**: Add comments explaining why specific settings are needed
3. **Test Locally**: Verify your configuration works before committing
4. **Version Control**: Commit the `.finch-mcp` file to your repository

## Troubleshooting

### Build Fails with Missing Dependencies

If your build fails because TypeScript or other build tools are missing:

```yaml
dependencies:
  installAll: true  # This installs devDependencies too
```

### Custom Build Process

If your project has a non-standard build process:

```yaml
build:
  command: "make build && npm run postbuild"
```

### Global Tools Required

If you need global npm packages for building:

```yaml
dependencies:
  preInstall:
    - "npm install -g node-gyp"
    - "npm install -g typescript"
```