# Architecture Overview

## Introduction

Finch-MCP is a Rust-based CLI tool that enables seamless containerization and execution of MCP (Model Context Protocol) servers using Finch. It provides multiple execution modes to accommodate different use cases, from running pre-built container images to automatically containerizing local projects.

## Core Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                           │
│  ┌─────────┐  ┌──────────┐  ┌────────┐  ┌──────────────┐ │
│  │   run   │  │   list   │  │ cleanup│  │cache/logs    │ │
│  └────┬────┘  └────┬─────┘  └───┬────┘  └──────┬───────┘ │
│       │            │             │               │         │
│  ┌────┴────────────┴─────────────┴───────────────┴──────┐ │
│  │              Command Router & Mode Detection          │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                      Core Modules                           │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐  │
│  │   Direct    │  │    Auto     │  │  Git/Local       │  │
│  │  Container  │  │Containerize │  │  Containerize    │  │
│  └──────┬──────┘  └──────┬──────┘  └────────┬─────────┘  │
│         │                 │                   │            │
│  ┌──────┴─────────────────┴───────────────────┴────────┐  │
│  │              Container Build & Cache Layer           │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                     │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐  │
│  │   Finch     │  │   Cache     │  │    Logging       │  │
│  │   Client    │  │  Manager    │  │    Manager       │  │
│  └─────────────┘  └─────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Execution Modes

### 1. Direct Container Mode
- **Purpose**: Run pre-built Docker/OCI images directly
- **Trigger**: `--direct` flag or image-like target (e.g., `myimage:latest`)
- **Flow**: CLI → Finch Client → Container execution with STDIO

### 2. Auto-Containerization Mode
- **Purpose**: Automatically containerize and run commands
- **Supported Commands**: `uvx`, `npx`, `npm`, `pip`, `python`
- **Flow**: Command detection → Dockerfile generation → Build → Execute
- **Example**: `finch-mcp run uvx mcp-server-time`

### 3. Git Repository Mode
- **Purpose**: Clone, containerize, and run Git repositories
- **Trigger**: Git URL as target
- **Flow**: Clone → Project detection → Dockerfile generation → Build → Execute

### 4. Local Directory Mode
- **Purpose**: Containerize and run local projects
- **Trigger**: Local directory path as target
- **Flow**: Project detection → Dockerfile generation → Build → Execute

## Module Breakdown

### CLI Module (`cli.rs`)
- Command-line argument parsing using `clap`
- Intelligent mode detection based on target format
- MCP client context detection for output suppression
- Environment variable and volume mount handling

### Run Module (`run.rs`)
- Manages direct container execution
- Handles STDIO piping for MCP protocol
- Implements graceful shutdown with signal handling
- Progress indication and status reporting

### Auto-Containerize Module (`core/auto_containerize.rs`)
- Command type detection and classification
- Dynamic Dockerfile generation based on command type
- Optimized multi-stage builds for smaller images
- Special handling for MCP STDIO mode

### Git Containerize Module (`core/git_containerize.rs`)
- Git repository cloning and management
- Project type detection (Python, Node.js, Rust, etc.)
- Intelligent Dockerfile generation based on project structure
- Support for monorepos and various package managers

### Finch Client (`finch/client.rs`)
- Abstraction layer over Finch CLI commands
- VM lifecycle management (init, start, status)
- Container operations (run, build, list, remove)
- Image management and cleanup

### Cache System (`cache/`)
- Content-based caching using SHA256 hashing
- Persistent cache storage with metadata
- Smart cache invalidation and cleanup
- Human-readable image naming scheme

### Logging System (`logging/`)
- Build log capture and storage
- XDG-compliant directory structure
- Log rotation and cleanup policies
- Debug information for troubleshooting

## Data Flow

### MCP Server Execution Flow

1. **Input Processing**
   - CLI parses arguments and detects execution mode
   - Environment variables and volumes are validated

2. **Mode-Specific Processing**
   - **Direct**: Skip to execution
   - **Auto**: Generate Dockerfile from command
   - **Git**: Clone and analyze repository
   - **Local**: Analyze project structure

3. **Containerization** (if needed)
   - Check cache for existing image
   - Generate optimized Dockerfile
   - Build container with Finch
   - Tag with content-based hash

4. **Execution**
   - Set up STDIO pipes
   - Configure MCP environment variables
   - Run container with proper mounts
   - Handle signals for graceful shutdown

5. **Cleanup**
   - Container termination on exit
   - Optional resource cleanup
   - Log preservation for debugging

## Key Design Decisions

### 1. Content-Based Caching
- Uses SHA256 hashing of source content
- Enables reliable cache invalidation
- Prevents unnecessary rebuilds
- Supports offline operation

### 2. MCP STDIO Mode
- Automatic detection of MCP client context
- Output suppression for clean protocol communication
- Environment variable injection for MCP compliance
- Seamless integration with Claude Desktop

### 3. Multi-Stage Builds
- Reduces final image size
- Improves security by minimizing attack surface
- Optimizes for fast startup times
- Supports various package managers

### 4. Project Detection
- Intelligent analysis of project structure
- Support for multiple languages and frameworks
- Automatic dependency detection
- Monorepo awareness

## Error Handling

- **Graceful Degradation**: Falls back to simpler modes when advanced features fail
- **Comprehensive Logging**: All operations logged for debugging
- **User-Friendly Messages**: Clear error messages with actionable suggestions
- **Recovery Mechanisms**: Automatic VM initialization, cache cleanup

## Security Considerations

- **Non-Root Execution**: Containers run as non-root users
- **Signal Handling**: Proper signal propagation with dumb-init
- **Resource Isolation**: Full container isolation via Finch
- **No Privileged Operations**: Runs without elevated permissions

## Performance Optimizations

- **Parallel Operations**: Concurrent image builds where possible
- **Lazy Initialization**: Finch VM started only when needed
- **Efficient Caching**: Content-based cache prevents rebuilds
- **Optimized Images**: Multi-stage builds for smaller containers
- **Fast Startup**: Minimal overhead for MCP server execution